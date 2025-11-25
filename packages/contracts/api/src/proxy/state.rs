use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
#[derive(Default)]
pub enum ActionState {
    #[default]
    Idle,
    Forwarded,
    WithdrawReceiptTokens {
        recipient: Addr,
        amount: Uint128,
    },
    WithdrawFunds {
        recipient: Addr,
        amount: Uint128,
    },
}

#[cw_serde]
pub struct State {
    pub admins: Vec<Addr>,
    pub last_action: ActionState,
}
