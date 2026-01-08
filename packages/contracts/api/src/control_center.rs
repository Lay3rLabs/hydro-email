//! Local copy of hydro control-center messages.
//! Importing directly from hydro-control-center would bring in neutron-sdk dependencies.

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub deposit_cap: Uint128,
    pub whitelist: Vec<String>,
    pub subvaults: Vec<String>,
}
