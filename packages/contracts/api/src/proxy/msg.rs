use cosmwasm_schema::{cw_serde, QueryResponses};

use super::state::State;

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
