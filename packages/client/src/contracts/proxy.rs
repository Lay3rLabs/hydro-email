//! Contract-specific abstraction for different backends (Climb, Climb Pool, MultiTest)
//! Define helper methods here and they'll be available for all backends

use anyhow::Result;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::{
    address::AnyAddr,
    executor::{AnyExecutor, AnyTxResponse},
    querier::AnyQuerier,
};

use app_contract_api::proxy::{
    msg::{QueryMsg as LocalQueryMsg, StateResponse},
    state::State,
};
use hydro_proxy::msg::{ConfigResponse, ExecuteMsg, QueryMsg};

#[derive(Clone)]
pub struct ProxyContract {
    pub querier: ProxyQuerier,
    pub executor: ProxyExecutor,
    pub address: AnyAddr,
}

impl ProxyContract {
    pub fn new(querier: AnyQuerier, executor: AnyExecutor, address: AnyAddr) -> Self {
        Self {
            querier: ProxyQuerier::new(querier, address.clone()),
            executor: ProxyExecutor::new(executor, address.clone()),
            address,
        }
    }
}

#[derive(Clone)]
pub struct ProxyQuerier {
    pub inner: AnyQuerier,
    pub addr: AnyAddr,
}

impl ProxyQuerier {
    pub fn new(inner: AnyQuerier, addr: AnyAddr) -> Self {
        Self { inner, addr }
    }
    pub async fn query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &QueryMsg,
    ) -> Result<RESP> {
        self.inner.contract_query(&self.addr, msg).await
    }

    pub async fn config(&self) -> Result<ConfigResponse> {
        self.query(&QueryMsg::Config {}).await
    }

    pub async fn state(&self) -> Result<State> {
        let resp: StateResponse = self
            .inner
            .contract_query(&self.addr, &LocalQueryMsg::State {})
            .await?;
        Ok(resp.state)
    }
}

#[derive(Clone)]
pub struct ProxyExecutor {
    pub inner: AnyExecutor,
    pub addr: AnyAddr,
}

impl ProxyExecutor {
    pub fn new(inner: AnyExecutor, addr: AnyAddr) -> Self {
        Self { inner, addr }
    }
    pub async fn exec(
        &self,
        msg: &ExecuteMsg,
        funds: &[cosmwasm_std::Coin],
    ) -> Result<AnyTxResponse> {
        self.inner.contract_exec(&self.addr, msg, funds).await
    }
}
