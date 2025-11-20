use anyhow::Result;
use layer_climb::querier::QueryClient;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::address::AnyAddr;

cfg_if::cfg_if! {
    if #[cfg(feature = "multitest")] {
        use cw_multi_test::App;
        use std::rc::Rc;
        use std::cell::RefCell;
        type AppWrapper = Rc<RefCell<App>>;
    }
}

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum AnyQuerier {
    Climb(QueryClient),
    #[cfg(feature = "client-pool")]
    ClimbPool(layer_climb::pool::SigningClientPool),
    #[cfg(feature = "multitest")]
    MultiTest(AppWrapper),
}

impl From<QueryClient> for AnyQuerier {
    fn from(client: QueryClient) -> AnyQuerier {
        AnyQuerier::Climb(client)
    }
}

#[cfg(feature = "client-pool")]
impl From<layer_climb::pool::SigningClientPool> for AnyQuerier {
    fn from(pool: layer_climb::pool::SigningClientPool) -> AnyQuerier {
        AnyQuerier::ClimbPool(pool)
    }
}

#[cfg(feature = "multitest")]
impl From<AppWrapper> for AnyQuerier {
    fn from(app: AppWrapper) -> AnyQuerier {
        AnyQuerier::MultiTest(app)
    }
}

impl AnyQuerier {
    pub async fn contract_query<
        RESP: DeserializeOwned + Send + Sync + Debug,
        MSG: Serialize + Debug,
    >(
        &self,
        address: &AnyAddr,
        msg: &MSG,
    ) -> Result<RESP> {
        match self {
            Self::Climb(client) => client.contract_smart(&address.into(), msg).await,
            #[cfg(feature = "client-pool")]
            Self::ClimbPool(pool) => {
                let client = pool.get().await.map_err(|e| anyhow::anyhow!("{e:?}"))?;
                client.querier.contract_smart(&address.into(), msg).await
            }
            #[cfg(feature = "multitest")]
            Self::MultiTest(app) => Ok(app
                .borrow()
                .wrap()
                .query_wasm_smart(address.to_string(), msg)
                .map_err(|e| anyhow::anyhow!("{e:?}"))?),
        }
    }
}
