use app_client::contracts::service_handler::{
    ServiceHandlerContract, ServiceHandlerExecutor, ServiceHandlerQuerier,
};
use cosmwasm_std::Addr;
use cw_multi_test::{ContractWrapper, Executor};

use crate::client::AppClient;

#[derive(Clone)]
pub struct ServiceHandlerClient {
    pub querier: ServiceHandlerQuerier,
    pub executor: ServiceHandlerExecutor,
    pub address: Addr,
}

impl From<ServiceHandlerClient> for ServiceHandlerContract {
    fn from(client: ServiceHandlerClient) -> Self {
        ServiceHandlerContract {
            querier: client.querier,
            executor: client.executor,
            address: client.address.into(),
        }
    }
}

impl ServiceHandlerClient {
    pub fn new(app_client: AppClient, user_registry: Addr) -> Self {
        let admin = app_client.admin();
        Self::new_with_admin(app_client, user_registry, admin)
    }

    pub fn new_with_admin(app_client: AppClient, user_registry: Addr, admin: Addr) -> Self {
        let contract = ContractWrapper::new(
            app_contract_service_handler::execute,
            app_contract_service_handler::instantiate,
            app_contract_service_handler::query,
        );
        let code_id = app_client.with_app_mut(|app| app.store_code(Box::new(contract)));

        let msg = app_contract_api::service_handler::msg::InstantiateMsg {
            auth: app_contract_api::service_handler::msg::Auth::Admin(admin.to_string()),
            user_registry: user_registry.to_string(),
        };

        let address = app_client.with_app_mut(|app| {
            app.instantiate_contract(
                code_id,
                app_client.admin(),
                &msg,
                &[],
                "service handler",
                None,
            )
            .unwrap()
        });

        let querier =
            ServiceHandlerQuerier::new(app_client.querier.clone(), address.clone().into());
        let executor =
            ServiceHandlerExecutor::new(app_client.executor.clone(), address.clone().into());

        Self {
            querier,
            executor,
            address,
        }
    }
}
