use app_contract_api::service_handler::{
    event::EmailEvent,
    msg::{
        AdminResponse, Auth, CustomExecuteMsg, CustomQueryMsg, EmailAddrsResponse,
        EmailsFromResponse, EmailsResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    },
};
use cosmwasm_std::{
    ensure, entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, Storage, WasmMsg,
};
use cw2::set_contract_version;
use wavs_types::contracts::cosmwasm::{
    service_handler::{ServiceHandlerExecuteMessages, ServiceHandlerQueryMessages},
    service_manager::{ServiceManagerQueryMessages, WavsValidateResult},
};

use crate::{
    error::ContractError,
    state::{ADMIN, SERVICE_MANAGER},
};

mod error;
mod state;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    state::initialize(&mut deps, &msg)?;

    // Set admin or service manager for later validation
    match msg.auth {
        Auth::ServiceManager(service_manager) => {
            let service_manager_addr = deps.api.addr_validate(&service_manager)?;
            SERVICE_MANAGER.save(deps.storage, &service_manager_addr)?;
        }
        Auth::Admin(admin) => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            ADMIN.save(deps.storage, &admin_addr)?;
        }
    }

    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    fn handle_custom_message(
        store: &mut dyn Storage,
        msg: CustomExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            CustomExecuteMsg::Email(email) => {
                let pagination_id = state::push_email(store, &email)?;

                let proxy_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: state::proxy_address(store)?.to_string(),
                    msg: to_json_binary(
                        &app_contract_api::proxy::msg::ExecuteMsg::ForwardToInflow {},
                    )?,
                    funds: vec![],
                });

                Ok(Response::new()
                    .add_message(proxy_msg)
                    .add_event(EmailEvent {
                        email,
                        pagination_id,
                    }))
            }
        }
    }

    match msg {
        ExecuteMsg::Custom(msg) => {
            let admin = ADMIN.load(deps.storage)?;
            ensure!(info.sender == admin, ContractError::Unauthorized);
            handle_custom_message(deps.storage, msg)
        }
        ExecuteMsg::Wavs(msg) => match msg {
            ServiceHandlerExecuteMessages::WavsHandleSignedEnvelope {
                envelope,
                signature_data,
            } => {
                let service_manager = SERVICE_MANAGER.load(deps.storage)?;

                // TODO: only allow eventId to be used once

                deps.querier.query_wasm_smart::<WavsValidateResult>(
                    service_manager,
                    &ServiceManagerQueryMessages::WavsValidate {
                        envelope: envelope.clone(),
                        signature_data: signature_data.clone(),
                    },
                )?;

                let envelope = envelope
                    .decode()
                    .map_err(|e| ContractError::AbiDecode(e.to_string()))?;

                let msg = CustomExecuteMsg::decode(&envelope.payload)
                    .map_err(|e| ContractError::PayloadDecode(e.to_string()))?;

                handle_custom_message(deps.storage, msg)
            }
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Custom(msg) => match msg {
            CustomQueryMsg::EmailAddrs { limit, start_after } => {
                let email_addrs =
                    state::list_email_addresses(deps.storage, start_after.as_deref(), limit)?;
                to_json_binary(&EmailAddrsResponse { email_addrs })
            }
            CustomQueryMsg::EmailsFrom {
                from,
                limit,
                start_after,
            } => {
                let emails = state::list_emails_from(deps.storage, &from, start_after, limit)?;
                to_json_binary(&EmailsFromResponse { emails })
            }
            CustomQueryMsg::Emails { limit, start_after } => {
                let emails = state::list_emails(deps.storage, start_after, limit)?;
                to_json_binary(&EmailsResponse { emails })
            }
            CustomQueryMsg::Admin {} => {
                let admin = ADMIN.may_load(deps.storage)?.map(Into::into);
                to_json_binary(&AdminResponse { admin })
            }
        },
        QueryMsg::Wavs(msg) => match msg {
            ServiceHandlerQueryMessages::WavsServiceManager {} => {
                let service_manager = SERVICE_MANAGER.load(deps.storage)?;
                to_json_binary(&service_manager)
            }
        },
    }
}

#[entry_point]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    Err(ContractError::UnknownReplyId { id: msg.id })
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
