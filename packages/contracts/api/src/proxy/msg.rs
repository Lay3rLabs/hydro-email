use cosmwasm_schema::{cw_serde, QueryResponses};

use super::state::State;

pub use hydro_inflow_proxy::msg::ExecuteMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
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
