use app_contract_api::proxy::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse},
    state::{ActionState, State},
};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};
use cw2::set_contract_version;

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
    let hydro_msg = hydro_proxy::msg::InstantiateMsg {
        admins: msg.admins,
        control_centers: msg.control_centers,
    };
    hydro_proxy::contract::instantiate(deps, env, info, hydro_msg)?;

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

    // Convert to hydro proxy message and execute
    let hydro_msg = match msg {
        ExecuteMsg::ForwardToInflow {} => hydro_proxy::msg::ExecuteMsg::ForwardToInflow {},
        ExecuteMsg::WithdrawReceiptTokens { address, coin } => {
            hydro_proxy::msg::ExecuteMsg::WithdrawReceiptTokens { address, coin }
        }
        ExecuteMsg::WithdrawFunds { address, coin } => {
            hydro_proxy::msg::ExecuteMsg::WithdrawFunds { address, coin }
        }
    };

    let response = hydro_proxy::contract::execute(deps, env, info, hydro_msg)?;
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

// TODO: Unit tests commented out - hydro proxy requires control_centers and real contract queries
// See packages/tests/off-chain for integration tests
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::{
//         testing::{mock_dependencies, mock_env},
//         Addr, MessageInfo, Coin,
//     };
//
//     const CREATOR: &str = "cosmwasm1wtqa75mkgwgncx8v4dep5aygmnq7gspaufggc5ev3u68et43qxmsqy5haw";
//     const ADMIN1: &str = "cosmwasm1g807u64s6uvk3daw4k4h778h850put0qdny3llp3xn43y5dar0hqfdcpt4";
//     const ADMIN2: &str = "cosmwasm195ay4pn6v07zenrafuhm5mnkklsj7kxa7gaz9djc9gjmkp0ehayszlp362";
//
//     fn message(sender: &Addr) -> MessageInfo {
//         MessageInfo {
//             sender: sender.clone(),
//             funds: vec![],
//         }
//     }
//
//     fn instantiate_contract(deps: DepsMut) -> (Addr, Addr) {
//         let creator = Addr::unchecked(CREATOR);
//         let admin1 = Addr::unchecked(ADMIN1);
//         let admin2 = Addr::unchecked(ADMIN2);
//         let msg = InstantiateMsg {
//             admins: vec![admin1.to_string(), admin2.to_string()],
//             control_centers: vec![],
//         };
//         instantiate(deps, mock_env(), message(&creator), msg).unwrap();
//         (admin1, admin2)
//     }
//
//     #[test]
//     fn cannot_instantiate_without_admins() {
//         let mut deps = mock_dependencies();
//         let creator = Addr::unchecked(CREATOR);
//         let err = instantiate(
//             deps.as_mut(),
//             mock_env(),
//             message(&creator),
//             InstantiateMsg { admins: vec![], control_centers: vec![] },
//         )
//         .unwrap_err();
//
//         assert!(matches!(err, ContractError::NoAdmins {}));
//     }
//
//     #[test]
//     fn forward_updates_state() {
//         let mut deps = mock_dependencies();
//         instantiate_contract(deps.as_mut());
//
//         let actor = Addr::unchecked(CREATOR);
//         execute(
//             deps.as_mut(),
//             mock_env(),
//             message(&actor),
//             ExecuteMsg::ForwardToInflow {},
//         )
//         .unwrap();
//
//         let state = STATE.load(&deps.storage).unwrap();
//         assert!(matches!(state.last_action, ActionState::Forwarded));
//     }
//
//     #[test]
//     fn withdraw_requires_admin() {
//         let mut deps = mock_dependencies();
//         let (admin1, _) = instantiate_contract(deps.as_mut());
//
//         let not_admin = Addr::unchecked(CREATOR);
//         let err = execute(
//             deps.as_mut(),
//             mock_env(),
//             message(&not_admin),
//             ExecuteMsg::WithdrawFunds {
//                 address: ADMIN1.to_string(),
//                 coin: Coin::new(10u128, "uatom"),
//             },
//         )
//         .unwrap_err();
//         assert!(matches!(err, ContractError::Unauthorized {}));
//
//         execute(
//             deps.as_mut(),
//             mock_env(),
//             message(&admin1),
//             ExecuteMsg::WithdrawReceiptTokens {
//                 address: ADMIN2.to_string(),
//                 coin: Coin::new(20u128, "uatom"),
//             },
//         )
//         .unwrap();
//
//         let state = STATE.load(&deps.storage).unwrap();
//         match state.last_action {
//             ActionState::WithdrawReceiptTokens { coin, .. } => {
//                 assert_eq!(coin.amount.u128(), 20);
//             }
//             _ => panic!("unexpected action stored"),
//         }
//     }
// }
