use app_client::service_handler::{ServiceHandlerExecutor, ServiceHandlerQuerier};
use cosmwasm_std::Addr;
use cw_multi_test::{ContractWrapper, Executor};

use crate::client::AppClient;

#[derive(Clone)]
pub struct ServiceHandlerClient {
    pub querier: ServiceHandlerQuerier,
    pub executor: ServiceHandlerExecutor,
}

impl ServiceHandlerClient {
    pub fn new(app_client: AppClient) -> Self {
        let admin = app_client.admin();
        Self::new_with_admin(app_client, admin)
    }

    pub fn new_with_admin(app_client: AppClient, admin: Addr) -> Self {
        let contract = ContractWrapper::new(
            app_contract_service_handler::execute,
            app_contract_service_handler::instantiate,
            app_contract_service_handler::query,
        );
        let code_id = app_client.with_app_mut(|app| app.store_code(Box::new(contract)));

        let msg = app_contract_api::service_handler::msg::InstantiateMsg {
            auth: app_contract_api::service_handler::msg::Auth::Admin(admin.to_string()),
        };

        let address = app_client.with_app_mut(|app| {
            app.instantiate_contract(code_id, admin.clone(), &msg, &[], "telegram-payments", None)
                .unwrap()
        });

        let querier =
            ServiceHandlerQuerier::new(app_client.querier.clone(), address.clone().into());
        let executor = ServiceHandlerExecutor::new(app_client.executor.clone(), address.into());

        Self { querier, executor }
    }
}
