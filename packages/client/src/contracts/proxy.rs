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
    msg::{ExecuteMsg, QueryMsg, StateResponse},
    state::State,
};

#[derive(Clone)]
pub struct ProxyContract {
    pub querier: ProxyQuerier,
    pub executor: ProxyExecutor,
    pub address: AnyAddr,
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

    pub async fn state(&self) -> Result<State> {
        let resp: StateResponse = self.query(&QueryMsg::State {}).await?;

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
