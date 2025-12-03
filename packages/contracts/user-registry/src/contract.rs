use app_contract_api::user_registry::{
    event::UserRegisteredEvent,
    msg::{ExecuteMsg, InstantiateMsg, ProxyAddressResponse, QueryMsg},
};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::{error::ContractError, state};

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    state::init(&mut deps, &msg)?;

    Ok(Response::new().add_attribute("action", "instantiate_user_registry"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterUser {
            user_id,
            proxy_address,
        } => {
            state::ensure_admin(deps.storage, &info.sender)?;

            let proxy_address = deps.api.addr_validate(&proxy_address)?;
            state::register_user(deps, user_id.clone(), proxy_address.clone())?;

            Ok(Response::new().add_event(UserRegisteredEvent {
                user_id,
                proxy_address,
            }))
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ProxyAddress { user_id } => {
            let address = state::get_proxy_address(deps.storage, user_id)?;
            to_json_binary(&ProxyAddressResponse { address })
        }
    }
}
