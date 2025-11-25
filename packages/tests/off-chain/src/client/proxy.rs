use app_client::contracts::proxy::{ProxyContract, ProxyExecutor, ProxyQuerier};
use cosmwasm_std::{instantiate2_address, Addr, Api, CodeInfoResponse};
use cw_multi_test::{ContractWrapper, Executor};

use crate::client::AppClient;

#[derive(Clone)]
pub struct ProxyClient {
    pub querier: ProxyQuerier,
    pub executor: ProxyExecutor,
    pub address: Addr,
}

impl From<ProxyClient> for ProxyContract {
    fn from(client: ProxyClient) -> Self {
        ProxyContract {
            querier: client.querier,
            executor: client.executor,
            address: client.address.into(),
        }
    }
}

impl ProxyClient {
    pub fn code_id(app_client: &AppClient) -> u64 {
        let contract = ContractWrapper::new(
            app_contract_proxy::execute,
            app_contract_proxy::instantiate,
            app_contract_proxy::query,
        );
        app_client.with_app_mut(|app| app.store_code(Box::new(contract)))
    }

    pub fn predict_address(app_client: &AppClient, code_id: u64) -> Addr {
        let info: CodeInfoResponse = app_client
            .with_app(|app| {
                app.wrap().query(&cosmwasm_std::QueryRequest::Wasm(
                    cosmwasm_std::WasmQuery::CodeInfo { code_id },
                ))
            })
            .unwrap();

        let creator = app_client.admin_canonical();

        let salt = b"hello world";

        let canonical_addr =
            instantiate2_address(info.checksum.as_slice(), &creator, salt).unwrap();

        app_client.with_app(|app| app.api().addr_humanize(&canonical_addr).unwrap())
    }

    pub fn new(app_client: AppClient, code_id: u64, admins: Vec<Addr>) -> Self {
        let admins = if admins.is_empty() {
            vec![app_client.admin()]
        } else {
            admins
        };

        let msg = app_contract_api::proxy::msg::InstantiateMsg {
            admins: admins.into_iter().map(|x| x.to_string()).collect(),
        };

        let address = app_client.with_app_mut(|app| {
            app.instantiate2_contract(
                code_id,
                app_client.admin(),
                &msg,
                &[],
                "proxy",
                None,
                b"hello world",
            )
            .unwrap()
        });

        // sanity check
        if address != Self::predict_address(&app_client, code_id) {
            panic!("Predicted address does not match instantiated address");
        }

        let querier = ProxyQuerier::new(app_client.querier.clone(), address.clone().into());
        let executor = ProxyExecutor::new(app_client.executor.clone(), address.clone().into());

        Self {
            querier,
            executor,
            address,
        }
    }
}
