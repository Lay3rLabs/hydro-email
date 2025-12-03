#[cfg(feature = "client-pool")]
use std::sync::Arc;

use anyhow::Result;
use layer_climb::{events::CosmosTxEvents, signing::SigningClient};
use serde::Serialize;

use crate::address::AnyAddr;

cfg_if::cfg_if! {
    if #[cfg(feature = "multitest")] {
        use cw_multi_test::{App, Executor};
        use std::rc::Rc;
        use std::cell::RefCell;
        type AppWrapper = Rc<RefCell<App>>;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "client-pool")] {
        pub type SigningClientWrapper = Arc<deadpool::managed::Object<layer_climb::pool::SigningClientPoolManager>>;
        pub type SigningClientWrapperRef<'a> = &'a deadpool::managed::Object<layer_climb::pool::SigningClientPoolManager>;
    }
}

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum AnyExecutor {
    Climb(SigningClient),
    #[cfg(feature = "client-pool")]
    ClimbPool(layer_climb::pool::SigningClientPool),
    #[cfg(feature = "client-pool")]
    ClimbPoolObject(SigningClientWrapper),
    #[cfg(feature = "multitest")]
    MultiTest {
        app: AppWrapper,
        admin: cosmwasm_std::Addr,
    },
}

impl From<SigningClient> for AnyExecutor {
    fn from(client: SigningClient) -> AnyExecutor {
        AnyExecutor::Climb(client)
    }
}

#[cfg(feature = "client-pool")]
impl From<layer_climb::pool::SigningClientPool> for AnyExecutor {
    fn from(pool: layer_climb::pool::SigningClientPool) -> AnyExecutor {
        AnyExecutor::ClimbPool(pool)
    }
}

#[cfg(feature = "client-pool")]
impl From<SigningClientWrapper> for AnyExecutor {
    fn from(client: SigningClientWrapper) -> AnyExecutor {
        AnyExecutor::ClimbPoolObject(client)
    }
}

#[cfg(feature = "multitest")]
impl From<(AppWrapper, cosmwasm_std::Addr)> for AnyExecutor {
    fn from((app, admin): (AppWrapper, cosmwasm_std::Addr)) -> AnyExecutor {
        AnyExecutor::MultiTest { app, admin }
    }
}

impl AnyExecutor {
    pub async fn contract_exec<MSG: Serialize + std::fmt::Debug>(
        &self,
        address: &AnyAddr,
        msg: &MSG,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<AnyTxResponse> {
        match self {
            Self::Climb(client) => {
                let funds = funds
                    .iter()
                    .map(|c| layer_climb::prelude::Coin {
                        denom: c.denom.clone(),
                        amount: c.amount.to_string(),
                    })
                    .collect::<Vec<_>>();

                client
                    .contract_execute(&address.into(), msg, funds, None)
                    .await
                    .map(AnyTxResponse::Climb)
            }
            #[cfg(feature = "client-pool")]
            Self::ClimbPool(pool) => {
                let client = pool.get().await.map_err(|e| anyhow::anyhow!("{e:?}"))?;
                let funds = funds
                    .iter()
                    .map(|c| layer_climb::prelude::Coin {
                        denom: c.denom.clone(),
                        amount: c.amount.to_string(),
                    })
                    .collect::<Vec<_>>();

                client
                    .contract_execute(&address.into(), msg, funds, None)
                    .await
                    .map(AnyTxResponse::Climb)
            }
            #[cfg(feature = "client-pool")]
            Self::ClimbPoolObject(client) => {
                let funds = funds
                    .iter()
                    .map(|c| layer_climb::prelude::Coin {
                        denom: c.denom.clone(),
                        amount: c.amount.to_string(),
                    })
                    .collect::<Vec<_>>();

                client
                    .contract_execute(&address.into(), msg, funds, None)
                    .await
                    .map(AnyTxResponse::Climb)
            }
            #[cfg(feature = "multitest")]
            Self::MultiTest { app, admin } => Ok(app
                .borrow_mut()
                .execute_contract(admin.clone(), address.into(), msg, funds)
                .map(AnyTxResponse::MultiTest)
                .map_err(|e| anyhow::anyhow!("{e:?}"))?),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum AnyTxResponse {
    Climb(layer_climb::proto::abci::TxResponse),
    #[cfg(feature = "multitest")]
    MultiTest(cw_multi_test::AppResponse),
}

impl<'a> From<&'a AnyTxResponse> for CosmosTxEvents<'a> {
    fn from(value: &'a AnyTxResponse) -> Self {
        match value {
            AnyTxResponse::Climb(resp) => CosmosTxEvents::from(resp),
            #[cfg(feature = "multitest")]
            AnyTxResponse::MultiTest(resp) => CosmosTxEvents::from(resp.events.as_slice()),
        }
    }
}

impl AnyTxResponse {
    pub fn unchecked_into_tx_response(self) -> layer_climb::proto::abci::TxResponse {
        match self {
            Self::Climb(tx_resp) => tx_resp,
            #[allow(unreachable_patterns)]
            _ => panic!("unable to get unchecked tx response"),
        }
    }
}
