use app_contract_api::proxy::{
    msg::{QueryMsg, StateResponse},
    state::{ActionState, State},
};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};
use cw2::set_contract_version;
use hydro_proxy::msg::{ExecuteMsg, InstantiateMsg};

use crate::{error::ContractError, state::STATE};

const CONTRACT_NAME: &str = "crates.io:proxy";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Save our local state for action tracking first (before deps is moved)
    STATE.save(deps.storage, &State::default())?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Initialize hydro proxy's config
    hydro_proxy::contract::instantiate(deps, env, info, msg)?;

    Ok(Response::new().add_attribute("action", "instantiate_proxy"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // Update action state before calling hydro proxy
    let action_state = match &msg {
        ExecuteMsg::ForwardToInflow {} => ActionState::Forwarded,
        ExecuteMsg::WithdrawReceiptTokens { address, coin } => ActionState::WithdrawReceiptTokens {
            recipient: deps.api.addr_validate(address)?,
            coin: coin.clone(),
        },
        ExecuteMsg::WithdrawFunds { address, coin } => ActionState::WithdrawFunds {
            recipient: deps.api.addr_validate(address)?,
            coin: coin.clone(),
        },
    };

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.last_action = action_state;
        Ok(state)
    })?;

    // Execute hydro proxy
    let response = hydro_proxy::contract::execute(deps, env, info, msg)?;
    Ok(response)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => {
            let state = STATE.load(deps.storage)?;
            to_json_binary(&StateResponse { state })
        }
    }
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    // Forward reply to hydro proxy
    let response = hydro_proxy::contract::reply(deps, env, msg)?;
    Ok(response)
}
