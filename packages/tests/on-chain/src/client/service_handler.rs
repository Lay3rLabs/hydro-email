use app_client::contracts::service_handler::{
    ServiceHandlerContract, ServiceHandlerExecutor, ServiceHandlerQuerier,
};
use layer_climb::prelude::Address;

use crate::{client::AppClient, code_ids::CodeId};

#[derive(Clone)]
pub struct ServiceHandlerClient {
    pub querier: ServiceHandlerQuerier,
    pub executor: ServiceHandlerExecutor,
    pub address: Address,
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
    pub async fn new(
        app_client: AppClient,
        proxy_address: Address,
        admin: Option<Address>,
    ) -> Self {
        let pool = app_client.pool();
        let client = pool.get().await.unwrap();

        let admin = admin.unwrap_or_else(|| client.addr.clone());

        let msg = app_contract_api::service_handler::msg::InstantiateMsg {
            auth: app_contract_api::service_handler::msg::Auth::Admin(admin.to_string()),
            proxy_address: proxy_address.to_string(),
        };

        let (address, _) = client
            .contract_instantiate(
                None,
                CodeId::new_service_handler().await,
                "Service Handler",
                &msg,
                vec![],
                None,
            )
            .await
            .unwrap();

        let querier =
            ServiceHandlerQuerier::new(app_client.querier.clone(), address.clone().into());
        let executor = ServiceHandlerExecutor::new(client.clone().into(), address.clone().into());

        Self {
            querier,
            executor,
            address,
        }
    }
}
