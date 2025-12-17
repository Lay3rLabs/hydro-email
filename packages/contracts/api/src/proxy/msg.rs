use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

use super::state::State;

#[cw_serde]
pub enum ExecuteMsg {
    ForwardToInflow {},
    WithdrawReceiptTokens { address: String, coin: Coin },
    WithdrawFunds { address: String, coin: Coin },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
    pub control_centers: Vec<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StateResponse)]
    State {},
}

#[cw_serde]
pub struct StateResponse {
    pub state: State,
}
