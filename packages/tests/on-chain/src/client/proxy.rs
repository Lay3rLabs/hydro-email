use app_client::contracts::proxy::{ProxyExecutor, ProxyQuerier};
use layer_climb::prelude::Address;

use crate::{client::AppClient, code_ids::CodeId};

#[derive(Clone)]
pub struct ProxyClient {
    pub querier: ProxyQuerier,
    pub executor: ProxyExecutor,
    pub address: Address,
}

impl ProxyClient {
    pub async fn new(app_client: AppClient, admins: Vec<Address>) -> Self {
        let pool = app_client.pool();
        let client = pool.get().await.unwrap();

        let admins = if admins.is_empty() {
            vec![client.addr.clone()]
        } else {
            admins
        };

        let msg = app_contract_api::proxy::msg::InstantiateMsg {
            admins: admins.into_iter().map(|x| x.to_string()).collect(),
        };

        let (address, _) = client
            .contract_instantiate(None, CodeId::new_proxy().await, "Proxy", &msg, vec![], None)
            .await
            .unwrap();

        let querier = ProxyQuerier::new(app_client.querier.clone(), address.clone().into());
        let executor = ProxyExecutor::new(client.clone().into(), address.clone().into());

        Self {
            querier,
            executor,
            address,
        }
    }
}
