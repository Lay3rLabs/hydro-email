use app_client::executor::SigningClientWrapper;
use hydro_interface::inflow::{ConfigResponse, QueryMsg};
use layer_climb::prelude::*;

use crate::code_ids::CodeId;

// from hydro/contracts/inflow/vault/src/msg.rs
// importing directly would bring in neutron-sdk dependencies
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub deposit_denom: String,
    pub subdenom: String,
    pub token_metadata: DenomMetadata,
    pub control_center_contract: String,
    pub token_info_provider_contract: Option<String>,
    pub whitelist: Vec<String>,
    pub max_withdrawals_per_user: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct DenomMetadata {
    pub exponent: u32,
    pub display: String,
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub uri: Option<String>,
    pub uri_hash: Option<String>,
}

#[derive(Clone)]
pub struct VaultClient {
    pub address: Address,
    pub querier: QueryClient,
}

impl VaultClient {
    pub async fn new(
        client: SigningClientWrapper,
        control_center: Address,
        deposit_denom: String,
        whitelist: Vec<Address>,
    ) -> Self {
        let subdenom = format!("hydro_inflow_{}", deposit_denom.replace("/", "_"));

        let msg = InstantiateMsg {
            deposit_denom: deposit_denom.clone(),
            subdenom: subdenom.clone(),
            token_metadata: DenomMetadata {
                exponent: 6,
                display: subdenom.clone(),
                name: format!("Hydro Inflow {}", deposit_denom),
                description: format!("Vault shares for {} deposits", deposit_denom),
                symbol: subdenom.to_uppercase(),
                uri: None,
                uri_hash: None,
            },
            control_center_contract: control_center.to_string(),
            token_info_provider_contract: None,
            whitelist: whitelist.iter().map(|a| a.to_string()).collect(),
            max_withdrawals_per_user: 10,
        };

        let querier = client.querier.clone();

        let (address, _) = client
            .contract_instantiate(None, CodeId::new_vault().await, "Vault", &msg, vec![], None)
            .await
            .unwrap();

        Self { address, querier }
    }

    pub async fn config(&self) -> ConfigResponse {
        self.querier
            .contract_smart(&self.address, &QueryMsg::Config {})
            .await
            .unwrap()
    }

    pub async fn shares_denom(&self) -> String {
        self.config().await.config.vault_shares_denom
    }
}
