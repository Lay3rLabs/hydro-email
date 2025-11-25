mod command;
mod config;
mod context;
mod ipfs;
mod output;

use std::process::exit;

use app_utils::{faucet, tracing::tracing_init};
use cosmwasm_std::Uint256;
use layer_climb::prelude::EvmAddr;
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize};
use wavs_types::{
    ComponentSource, GetSignerRequest, Service, ServiceManager, SignatureKind, SignerResponse,
    Submit, Trigger, Workflow,
};

use crate::{
    command::{AuthKind, CliCommand, ContractKind},
    config::path_deployments,
    context::CliContext,
    ipfs::IpfsFile,
    output::{
        OutputComponentUpload, OutputContractInstantiate, OutputContractUpload,
        OutputOperatorSetSigningKey, OutputServiceUpload,
    },
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Install rustls crypto provider before any TLS operations
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    tracing_init();

    let ctx = CliContext::new().await;

    match ctx.command.clone() {
        CliCommand::OperatorSetSigningKey {
            service_manager_address,
            evm_operator_address,
            stake_registry_address,
            weight,
            wavs_url,
            args,
        } => {
            let service_manager_address =
                ctx.parse_address(&service_manager_address).await.unwrap();
            let stake_registry_address = ctx.parse_address(&stake_registry_address).await.unwrap();

            let service_manager = ServiceManager::Cosmos {
                chain: ctx.chain_key(),
                address: service_manager_address.clone().try_into().unwrap(),
            };

            let body = serde_json::to_string(&GetSignerRequest { service_manager }).unwrap();

            let url = wavs_url.join("services/signer").unwrap();
            let SignerResponse::Secp256k1 {
                evm_address: evm_signing_key_address,
                hd_index: _,
            } = reqwest::Client::new()
                .post(url.clone())
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            let client = ctx.signing_client().await.unwrap();

            // Parse EVM addresses
            let evm_operator_address: EvmAddr = evm_operator_address.parse().unwrap_or_else(|_| {
                panic!("Invalid operator EVM address '{}'", evm_operator_address)
            });

            let evm_signing_key_address: EvmAddr =
                evm_signing_key_address.parse().unwrap_or_else(|_| {
                    panic!(
                        "Invalid signing key EVM address '{}'",
                        evm_signing_key_address
                    )
                });

            // Parse weight as Uint256
            let weight_uint: Uint256 = weight
                .parse()
                .unwrap_or_else(|e| panic!("Invalid weight '{}': {}", weight, e));

            // Create the SetSigningKey message
            // TODO: move this to middleware docker cli
            let set_signing_key_msg = serde_json::json!({
                "set_signing_key": {
                    "operator": evm_operator_address.to_string(),
                    "signing_key": evm_signing_key_address.to_string(),
                    "weight": weight_uint.to_string()
                }
            });

            let tx_resp = client
                .contract_execute(&service_manager_address, &set_signing_key_msg, vec![], None)
                .await
                .unwrap();

            let service_manager_tx_hash = tx_resp.txhash;

            // Create the SetOperatorDetails message
            // TODO: move this to middleware docker cli
            let set_operator_details_msg = serde_json::json!({
                "set_operator_details": {
                    "operator": evm_operator_address.to_string(),
                    "signing_key": evm_signing_key_address.to_string(),
                    "weight": weight_uint.to_string()
                }
            });

            let tx_resp = client
                .contract_execute(
                    &stake_registry_address,
                    &set_operator_details_msg,
                    vec![],
                    None,
                )
                .await
                .unwrap();

            let stake_registry_tx_hash = tx_resp.txhash;

            args.output()
                .write(OutputOperatorSetSigningKey {
                    service_manager_tx_hash,
                    stake_registry_tx_hash,
                    evm_operator_address,
                    evm_signing_key_address,
                })
                .await
                .unwrap();
        }
        CliCommand::AssertAccountExists { addr, args: _ } => {
            let client = ctx.query_client().await.unwrap();
            let addr = match addr {
                Some(addr) => ctx.parse_address(&addr).await.unwrap(),
                None => ctx.wallet_addr().await.unwrap(),
            };
            let balance = client
                .balance(addr.clone(), None)
                .await
                .unwrap()
                .unwrap_or_default();

            if balance == 0 {
                eprintln!(
                    "{} has zero balance. Please fund the wallet before proceeding.",
                    addr
                );
                exit(1);
            }
        }
        CliCommand::UploadContract { kind, args } => {
            let client = ctx.signing_client().await.unwrap();

            let (code_id, tx_resp) = client
                .contract_upload_file(kind.wasm_bytes().await, None)
                .await
                .unwrap();

            println!("Uploaded {kind} contract with code ID: {code_id}");

            args.output()
                .write(OutputContractUpload {
                    kind,
                    code_id,
                    tx_hash: tx_resp.txhash,
                })
                .await
                .unwrap();
        }
        CliCommand::InstantiateServiceHandler {
            auth_address,
            auth_kind,
            args,
            code_id,
            proxy_code_id,
            proxy_salt,
        } => {
            let client = ctx.signing_client().await.unwrap();

            let auth = match auth_kind {
                AuthKind::ServiceManager => {
                    app_contract_api::service_handler::msg::Auth::ServiceManager(match auth_address
                    {
                        Some(addr) => ctx.parse_address(&addr).await.unwrap().to_string(),
                        None => {
                            panic!("Service manager auth requires an address to be provided")
                        }
                    })
                }
                AuthKind::User => {
                    app_contract_api::service_handler::msg::Auth::Admin(match auth_address {
                        Some(addr) => ctx.parse_address(&addr).await.unwrap().to_string(),
                        None => ctx.wallet_addr().await.unwrap().to_string(),
                    })
                }
            };

            let proxy_address = client
                .querier
                .contract_predict_address(proxy_code_id, &client.addr, proxy_salt.as_ref())
                .await
                .unwrap();

            let instantiate_msg = app_contract_api::service_handler::msg::InstantiateMsg {
                auth,
                proxy_address: proxy_address.to_string(),
            };

            let (contract_addr, tx_resp) = client
                .contract_instantiate(
                    None,
                    code_id,
                    "Service Handler",
                    &instantiate_msg,
                    vec![],
                    None,
                )
                .await
                .unwrap();

            println!("Instantiated Service Handler contract at address: {contract_addr}");

            args.output()
                .write(OutputContractInstantiate {
                    kind: ContractKind::ServiceHandler,
                    address: contract_addr.to_string(),
                    tx_hash: tx_resp.txhash,
                })
                .await
                .unwrap();
        }
        CliCommand::InstantiateProxy {
            admins,
            args,
            code_id,
            salt,
        } => {
            let client = ctx.signing_client().await.unwrap();

            // borrowing salt for sanity check
            let predicted_addr = client
                .querier
                .contract_predict_address(code_id, &client.addr, salt.as_ref())
                .await
                .unwrap();

            let instantiate_msg = app_contract_api::proxy::msg::InstantiateMsg { admins };

            let (contract_addr, tx_resp) = client
                .contract_instantiate2(
                    None,
                    code_id,
                    "Proxy",
                    &instantiate_msg,
                    vec![],
                    salt.into_inner(),
                    false,
                    None,
                )
                .await
                .unwrap();

            // sanity check
            if predicted_addr != contract_addr {
                panic!("Predicted address does not match instantiated address (predicted: {predicted_addr}, received: {contract_addr})");
            }

            println!("Instantiated Proxy contract at address: {contract_addr}");

            args.output()
                .write(OutputContractInstantiate {
                    kind: ContractKind::Proxy,
                    address: contract_addr.to_string(),
                    tx_hash: tx_resp.txhash,
                })
                .await
                .unwrap();
        }
        CliCommand::FaucetTap {
            addr,
            amount,
            denom,
            args: _,
        } => {
            let client = ctx.query_client().await.unwrap();
            let addr = match addr {
                Some(addr) => ctx.parse_address(&addr).await.unwrap(),
                None => ctx.wallet_addr().await.unwrap(),
            };
            let balance_before = client
                .balance(addr.clone(), None)
                .await
                .unwrap()
                .unwrap_or_default();
            faucet::tap(&addr, amount, denom.as_deref()).await.unwrap();
            let balance_after = client
                .balance(addr.clone(), None)
                .await
                .unwrap()
                .unwrap_or_default();

            println!(
                "Tapped faucet for {addr} - balance before: {balance_before} balance after: {balance_after}"
            );
        }
        CliCommand::UploadComponent {
            kind,
            component,
            args,
            ipfs_api_url,
            ipfs_gateway_url,
        } => {
            let bytes = kind.wasm_bytes(&component).await;

            let digest = wavs_types::ComponentDigest::hash(&bytes);

            let resp = IpfsFile::upload(
                bytes,
                &format!("{kind}-{component}.wasm"),
                ipfs_api_url.as_ref(),
                ipfs_gateway_url.as_ref(),
                true,
            )
            .await
            .unwrap();

            let IpfsFile {
                cid,
                uri,
                gateway_url,
            } = resp;

            args.output()
                .write(OutputComponentUpload {
                    kind,
                    component,
                    digest,
                    cid,
                    uri,
                    gateway_url,
                })
                .await
                .unwrap();
        }
        CliCommand::UploadService {
            args,
            ipfs_api_url,
            ipfs_gateway_url,
            contract_service_handler_instantiation_file,
            component_operator_email_reader_cid_file,
            component_aggregator_submitter_cid_file,
            trigger_cron_schedule,
            middleware_instantiation_file,
            aggregator_url,
            activate,
        } => {
            let output_directory = path_deployments();

            let contract_service_handler_instantiation_file =
                output_directory.join(contract_service_handler_instantiation_file);
            let component_operator_email_reader_cid_file =
                output_directory.join(component_operator_email_reader_cid_file);
            let component_aggregator_submitter_cid_file =
                output_directory.join(component_aggregator_submitter_cid_file);
            let middleware_instantiation_file =
                output_directory.join(middleware_instantiation_file);

            fn strip_trailing_slash(url: &Url) -> String {
                let s = url.as_str();
                match s.strip_suffix('/') {
                    Some(stripped) => stripped.to_string(),
                    None => s.to_string(),
                }
            }

            let ipfs_api_url = strip_trailing_slash(&ipfs_api_url);
            let ipfs_gateway_url = strip_trailing_slash(&ipfs_gateway_url);
            let aggregator_url = strip_trailing_slash(&aggregator_url);

            async fn read_and_decode<T: DeserializeOwned>(path: std::path::PathBuf) -> T {
                match tokio::fs::read_to_string(&path).await {
                    Err(e) => {
                        panic!("Failed to read file {}: {}", path.display(), e);
                    }
                    Ok(content) => match serde_json::from_str(&content) {
                        Err(e) => {
                            panic!("Failed to decode JSON from file {}: {}", path.display(), e);
                        }
                        Ok(data) => data,
                    },
                }
            }

            let contract_service_handler: OutputContractInstantiate =
                read_and_decode(contract_service_handler_instantiation_file).await;

            let component_operator_email_reader: OutputComponentUpload =
                read_and_decode(component_operator_email_reader_cid_file).await;

            let component_aggregator_submitter: OutputComponentUpload =
                read_and_decode(component_aggregator_submitter_cid_file).await;

            #[derive(Debug, Deserialize)]
            struct MiddlewareInstantiation {
                #[serde(rename = "registry_address")]
                pub _registry_address: String,
                pub service_manager_address: String,
            }

            let middleware_instantiation: MiddlewareInstantiation =
                read_and_decode(middleware_instantiation_file).await;

            let trigger = Trigger::Cron {
                schedule: trigger_cron_schedule,
                start_time: None,
                end_time: None,
            };

            let operator_email_reader_component = wavs_types::Component {
                source: ComponentSource::Download {
                    //uri: component_operator.uri.parse().unwrap(),
                    uri: component_operator_email_reader.gateway_url.parse().unwrap(),
                    digest: component_operator_email_reader.digest,
                },
                permissions: wavs_types::Permissions {
                    allowed_http_hosts: wavs_types::AllowedHostPermission::All,
                    file_system: false,
                    raw_sockets: true,
                    dns_resolution: true,
                },
                fuel_limit: None,
                time_limit_seconds: None,
                config: Default::default(),
                env_keys: [
                    "WAVS_ENV_IMAP_DEBUG_CAPABILITIES",
                    "WAVS_ENV_IMAP_PORT",
                    "WAVS_ENV_IMAP_HOST",
                    "WAVS_ENV_IMAP_TLS",
                    "WAVS_ENV_IMAP_CREDENTIAL_KIND",
                    "WAVS_ENV_IMAP_USERNAME",
                    "WAVS_ENV_IMAP_PASSWORD",
                    "WAVS_ENV_GMAIL_CLIENT_ID",
                    "WAVS_ENV_GMAIL_CLIENT_SECRET",
                    "WAVS_ENV_GMAIL_TOKEN",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            };

            let aggregator_submitter_component = wavs_types::Component {
                source: ComponentSource::Download {
                    //uri: component_aggregator.uri.parse().unwrap(),
                    uri: component_aggregator_submitter.gateway_url.parse().unwrap(),
                    digest: component_aggregator_submitter.digest,
                },
                permissions: wavs_types::Permissions {
                    allowed_http_hosts: wavs_types::AllowedHostPermission::All,
                    file_system: false,
                    raw_sockets: false,
                    dns_resolution: false,
                },
                fuel_limit: None,
                time_limit_seconds: None,
                config: [
                    (
                        "SERVICE_HANDLER_CONTRACT_ADDRESS".to_string(),
                        contract_service_handler.address.clone(),
                    ),
                    ("CHAIN".to_string(), ctx.chain_key().to_string()),
                ]
                .into_iter()
                .collect(),
                env_keys: Default::default(),
            };

            let submit_chain = Submit::Aggregator {
                url: aggregator_url.clone(),
                component: Box::new(aggregator_submitter_component),
                signature_kind: SignatureKind::evm_default(),
            };

            let workflow = Workflow {
                trigger,
                component: operator_email_reader_component,
                submit: submit_chain,
            };

            let service = Service {
                name: "Hydro Email".to_string(),
                workflows: [("workflow-1".parse().unwrap(), workflow)]
                    .into_iter()
                    .collect(),
                status: if activate {
                    wavs_types::ServiceStatus::Active
                } else {
                    wavs_types::ServiceStatus::Paused
                },
                manager: ServiceManager::Cosmos {
                    chain: ctx.chain_key(),
                    address: middleware_instantiation
                        .service_manager_address
                        .parse()
                        .unwrap(),
                },
            };

            let bytes = serde_json::to_vec_pretty(&service).unwrap();

            let digest = wavs_types::ServiceDigest::hash(&bytes);

            let resp = IpfsFile::upload(
                bytes,
                "service.json",
                ipfs_api_url.as_ref(),
                ipfs_gateway_url.as_ref(),
                true,
            )
            .await
            .unwrap();

            let IpfsFile {
                cid,
                uri,
                gateway_url,
            } = resp;

            args.output()
                .write(OutputServiceUpload {
                    service,
                    digest,
                    cid,
                    uri: uri.clone(),
                    gateway_url: gateway_url.clone(),
                })
                .await
                .unwrap();

            println!("\nService URI: {}", uri);
            println!("Service Gateway URL: {}\n", gateway_url);
        }
        CliCommand::AggregatorRegisterService {
            args: _,
            service_manager_address,
            aggregator_url,
        } => {
            let req = wavs_types::aggregator::RegisterServiceRequest {
                service_manager: ServiceManager::Cosmos {
                    chain: ctx.chain_key(),
                    address: service_manager_address.parse().unwrap(),
                },
            };

            reqwest::Client::new()
                .post(aggregator_url.join("services").unwrap())
                .json(&req)
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap();
        }

        CliCommand::OperatorAddService {
            args: _,
            service_manager_address,
            wavs_url,
        } => {
            let req = wavs_types::AddServiceRequest {
                service_manager: ServiceManager::Cosmos {
                    chain: ctx.chain_key(),
                    address: service_manager_address.parse().unwrap(),
                },
            };

            reqwest::Client::new()
                .post(wavs_url.join("services").unwrap())
                .json(&req)
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap();
        }

        CliCommand::OperatorDeleteService {
            args: _,
            service_manager_address,
            wavs_url,
        } => {
            let req = wavs_types::DeleteServicesRequest {
                service_managers: vec![ServiceManager::Cosmos {
                    chain: ctx.chain_key(),
                    address: service_manager_address.parse().unwrap(),
                }],
            };

            reqwest::Client::new()
                .delete(wavs_url.join("services").unwrap())
                .json(&req)
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap();
        }
        CliCommand::QueryServiceHandlerEmails {
            address,
            args: _,
            limit,
            start_after,
        } => {
            let address = ctx.parse_address(&address).await.unwrap();
            let client = ctx.service_handler_querier(address).await.unwrap();

            let emails = client.emails(limit, start_after).await.unwrap();

            println!("{:#?}\n", emails);

            println!("{} emails", emails.len());
        }
        CliCommand::QueryProxyState { address, args: _ } => {
            let address = ctx.parse_address(&address).await.unwrap();
            let client = ctx.proxy_querier(address).await.unwrap();

            let state = client.state().await.unwrap();

            println!("{:#?}\n", state);
        }
    }
}
