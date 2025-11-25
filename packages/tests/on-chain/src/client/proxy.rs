use std::sync::Arc;

use app_client::contracts::proxy::{ProxyContract, ProxyExecutor, ProxyQuerier};
use layer_climb::prelude::*;

use crate::code_ids::CodeId;

#[derive(Clone)]
pub struct ProxyClient {
    pub querier: ProxyQuerier,
    pub executor: ProxyExecutor,
    pub address: Address,
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

// For stable addresses with instantiate2, we need to use the same client
// and so it can't benefit from the pool which is totally fine, ProxyClient will just have that specific wallet throughout
impl ProxyClient {
    pub async fn predict_address(
        client: &deadpool::managed::Object<SigningClientPoolManager>,
    ) -> Address {
        let code_id = CodeId::new_proxy().await;

        client
            .querier
            .contract_predict_address(code_id, &client.addr, b"hello world")
            .await
            .unwrap()
    }
    pub async fn new(
        creator_client: deadpool::managed::Object<SigningClientPoolManager>,
        admins: Vec<Address>,
    ) -> Self {
        let admins = if admins.is_empty() {
            vec![creator_client.addr.clone()]
        } else {
            admins
        };

        let msg = app_contract_api::proxy::msg::InstantiateMsg {
            admins: admins.into_iter().map(|x| x.to_string()).collect(),
        };

        let (address, _) = creator_client
            .contract_instantiate2(
                None,
                CodeId::new_proxy().await,
                "Proxy",
                &msg,
                vec![],
                b"hello world".to_vec(),
                false,
                None,
            )
            .await
            .unwrap();

        // sanity check
        if address != Self::predict_address(&creator_client).await {
            panic!("Predicted address does not match instantiated address (predicted: {} received: {})", Self::predict_address(&creator_client).await, address);
        }

        let querier = ProxyQuerier::new(
            creator_client.querier.clone().into(),
            address.clone().into(),
        );
        let executor = ProxyExecutor::new(Arc::new(creator_client).into(), address.clone().into());

        Self {
            querier,
            executor,
            address,
        }
    }
}
