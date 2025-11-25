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

use app_contract_api::service_handler::msg::{
    AdminResponse, CustomExecuteMsg, CustomQueryMsg, Email, EmailAddrsResponse, ExecuteMsg,
    QueryMsg,
};

#[derive(Clone)]
pub struct ServiceHandlerContract {
    pub querier: ServiceHandlerQuerier,
    pub executor: ServiceHandlerExecutor,
    pub address: AnyAddr,
}

#[derive(Clone)]
pub struct ServiceHandlerQuerier {
    pub inner: AnyQuerier,
    pub addr: AnyAddr,
}

impl ServiceHandlerQuerier {
    pub fn new(inner: AnyQuerier, addr: AnyAddr) -> Self {
        Self { inner, addr }
    }
    pub async fn query<RESP: DeserializeOwned + Send + Sync + Debug>(
        &self,
        msg: &QueryMsg,
    ) -> Result<RESP> {
        self.inner.contract_query(&self.addr, msg).await
    }

    pub async fn admin(&self) -> Result<Option<String>> {
        let resp: AdminResponse = self
            .query(&QueryMsg::Custom(CustomQueryMsg::Admin {}))
            .await?;

        Ok(resp.admin)
    }

    pub async fn all_email_addresses(&self) -> Result<Vec<String>> {
        let mut emails = Vec::new();
        let mut start_after: Option<String> = None;

        loop {
            let batch = self.email_addresses(Some(100), start_after.clone()).await?;

            if batch.is_empty() {
                break;
            }

            start_after = batch.last().cloned();
            emails.extend(batch);
        }

        Ok(emails)
    }

    pub async fn all_emails_from(
        &self,
        from: &str,
    ) -> Result<
        Vec<(
            app_contract_api::service_handler::msg::EmailMessageOnly,
            u64,
        )>,
    > {
        let mut emails = Vec::new();
        let mut start_after: Option<u64> = None;

        loop {
            let batch = self.emails_from(from, Some(100), start_after).await?;

            if batch.is_empty() {
                break;
            }

            start_after = Some(batch.last().unwrap().1);
            emails.extend(batch);
        }

        Ok(emails)
    }

    pub async fn all_emails(
        &self,
    ) -> Result<Vec<(app_contract_api::service_handler::msg::Email, u64)>> {
        let mut emails = Vec::new();
        let mut start_after: Option<u64> = None;

        loop {
            let batch = self.emails(Some(100), start_after).await?;

            if batch.is_empty() {
                break;
            }

            start_after = Some(batch.last().unwrap().1);
            emails.extend(batch);
        }

        Ok(emails)
    }

    pub async fn email_addresses(
        &self,
        limit: Option<u32>,
        start_after: Option<String>,
    ) -> Result<Vec<String>> {
        let resp: EmailAddrsResponse = self
            .query(&QueryMsg::Custom(CustomQueryMsg::EmailAddrs {
                limit,
                start_after,
            }))
            .await?;

        Ok(resp.email_addrs)
    }

    pub async fn emails_from(
        &self,
        from: &str,
        limit: Option<u32>,
        start_after: Option<u64>,
    ) -> Result<
        Vec<(
            app_contract_api::service_handler::msg::EmailMessageOnly,
            u64,
        )>,
    > {
        let resp: app_contract_api::service_handler::msg::EmailsFromResponse = self
            .query(&QueryMsg::Custom(CustomQueryMsg::EmailsFrom {
                from: from.to_string(),
                limit,
                start_after,
            }))
            .await?;

        Ok(resp.emails)
    }

    pub async fn emails(
        &self,
        limit: Option<u32>,
        start_after: Option<u64>,
    ) -> Result<Vec<(app_contract_api::service_handler::msg::Email, u64)>> {
        let resp: app_contract_api::service_handler::msg::EmailsResponse = self
            .query(&QueryMsg::Custom(CustomQueryMsg::Emails {
                limit,
                start_after,
            }))
            .await?;
        Ok(resp.emails)
    }
}

#[derive(Clone)]
pub struct ServiceHandlerExecutor {
    pub inner: AnyExecutor,
    pub addr: AnyAddr,
}

impl ServiceHandlerExecutor {
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

    pub async fn push_email(&self, email: Email) -> Result<AnyTxResponse> {
        self.exec(&ExecuteMsg::Custom(CustomExecuteMsg::Email(email)), &[])
            .await
    }
}
