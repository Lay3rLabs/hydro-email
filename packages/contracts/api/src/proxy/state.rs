use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};

#[cw_serde]
#[derive(Default)]
pub enum ActionState {
    #[default]
    Idle,
    Forwarded,
    WithdrawReceiptTokens {
        recipient: Addr,
        coin: Coin,
    },
    WithdrawFunds {
        recipient: Addr,
        coin: Coin,
    },
}

#[cw_serde]
pub struct State {
    pub admins: Vec<Addr>,
    pub last_action: ActionState,
}
