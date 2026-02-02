//! Local copy of hydro vault messages.
//! Importing directly from hydro-vault would bring in neutron-sdk dependencies.

use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub deposit_denom: String,
    pub subdenom: String,
    pub token_metadata: DenomMetadata,
    pub control_center_contract: String,
    pub token_info_provider_contract: Option<String>,
    pub whitelist: Vec<String>,
    pub max_withdrawals_per_user: u64,
}

#[cw_serde]
pub struct DenomMetadata {
    pub exponent: u32,
    pub display: String,
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub uri: Option<String>,
    pub uri_hash: Option<String>,
}
