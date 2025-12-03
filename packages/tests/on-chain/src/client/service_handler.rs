use app_client::{
    contracts::service_handler::{
        ServiceHandlerContract, ServiceHandlerExecutor, ServiceHandlerQuerier,
    },
    executor::SigningClientWrapper,
};
use layer_climb::prelude::Address;

use crate::code_ids::CodeId;

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

// For executing transactions with a stable admin address, we need to use the same client
// so it can't benefit from the pool which is totally fine, it will just have that specific wallet throughout
impl ServiceHandlerClient {
    pub async fn new(
        client: SigningClientWrapper,
        user_registry: Address,
        admin: Option<Address>,
    ) -> Self {
        let admin = admin.unwrap_or_else(|| client.addr.clone());

        let msg = app_contract_api::service_handler::msg::InstantiateMsg {
            auth: app_contract_api::service_handler::msg::Auth::Admin(admin.to_string()),
            user_registry: user_registry.to_string(),
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
            ServiceHandlerQuerier::new(client.querier.clone().into(), address.clone().into());
        let executor = ServiceHandlerExecutor::new(client.into(), address.clone().into());

        Self {
            querier,
            executor,
            address,
        }
    }
}
