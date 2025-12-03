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

use app_contract_api::user_registry::msg::{ExecuteMsg, ProxyAddressResponse, QueryMsg, UserId};

#[derive(Clone)]
pub struct UserRegistryContract {
    pub querier: UserRegistryQuerier,
    pub executor: UserRegistryExecutor,
    pub address: AnyAddr,
}

impl UserRegistryContract {
    pub fn new(querier: AnyQuerier, executor: AnyExecutor, address: AnyAddr) -> Self {
        Self {
            querier: UserRegistryQuerier::new(querier, address.clone()),
            executor: UserRegistryExecutor::new(executor, address.clone()),
            address,
        }
    }
}

#[derive(Clone)]
pub struct UserRegistryQuerier {
    pub inner: AnyQuerier,
    pub addr: AnyAddr,
}

impl UserRegistryQuerier {
    pub fn new(inner: AnyQuerier, addr: AnyAddr) -> Self {
        Self { inner, addr }
    }
    pub async fn query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &QueryMsg,
    ) -> Result<RESP> {
        self.inner.contract_query(&self.addr, msg).await
    }

    pub async fn proxy_address_email(&self, email: &str) -> Result<AnyAddr> {
        let user_id = UserId::new_email_address(email);
        let resp: ProxyAddressResponse = self.query(&QueryMsg::ProxyAddress { user_id }).await?;

        Ok(AnyAddr::from(resp.address))
    }
}

#[derive(Clone)]
pub struct UserRegistryExecutor {
    pub inner: AnyExecutor,
    pub addr: AnyAddr,
}

impl UserRegistryExecutor {
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

    pub async fn register_user_email(
        &self,
        email: &str,
        proxy_address: AnyAddr,
    ) -> Result<(AnyTxResponse, UserId)> {
        let user_id = UserId::new_email_address(&email);

        let msg = ExecuteMsg::RegisterUser {
            user_id: user_id.clone(),
            proxy_address: proxy_address.to_string(),
        };

        let tx_resp = self.exec(&msg, &[]).await?;

        Ok((tx_resp, user_id))
    }
}
