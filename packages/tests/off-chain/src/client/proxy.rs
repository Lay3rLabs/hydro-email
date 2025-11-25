use app_client::contracts::proxy::{ProxyExecutor, ProxyQuerier};
use cosmwasm_std::Addr;
use cw_multi_test::{ContractWrapper, Executor};

use crate::client::AppClient;

#[derive(Clone)]
pub struct ProxyClient {
    pub querier: ProxyQuerier,
    pub executor: ProxyExecutor,
    pub address: Addr,
}

impl ProxyClient {
    pub fn new(app_client: AppClient, admins: Vec<Addr>) -> Self {
        let contract = ContractWrapper::new(
            app_contract_proxy::execute,
            app_contract_proxy::instantiate,
            app_contract_proxy::query,
        );
        let code_id = app_client.with_app_mut(|app| app.store_code(Box::new(contract)));

        let admins = if admins.is_empty() {
            vec![app_client.admin()]
        } else {
            admins
        };

        let msg = app_contract_api::proxy::msg::InstantiateMsg {
            admins: admins.into_iter().map(|x| x.to_string()).collect(),
        };

        let address = app_client.with_app_mut(|app| {
            app.instantiate_contract(code_id, app_client.admin(), &msg, &[], "proxy", None)
                .unwrap()
        });

        let querier = ProxyQuerier::new(app_client.querier.clone(), address.clone().into());
        let executor = ProxyExecutor::new(app_client.executor.clone(), address.clone().into());

        Self {
            querier,
            executor,
            address,
        }
    }
}
