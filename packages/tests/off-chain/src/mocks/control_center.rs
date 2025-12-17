//! Mock control center contract for testing
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw_multi_test::{Contract, ContractWrapper};
use cw_storage_plus::Item;
use hydro_interface::inflow_control_center::{QueryMsg, SubvaultsResponse};

const SUBVAULTS: Item<Vec<Addr>> = Item::new("subvaults");

/// Simplified instantiate message for testing
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub subvaults: Vec<String>,
}

fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let subvaults: Vec<Addr> = msg
        .subvaults
        .iter()
        .map(|s| deps.api.addr_validate(s))
        .collect::<StdResult<_>>()?;
    SUBVAULTS.save(deps.storage, &subvaults)?;
    Ok(Response::new())
}

fn execute(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: Empty) -> StdResult<Response> {
    Ok(Response::new())
}

fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Subvaults {} => {
            let subvaults = SUBVAULTS.load(deps.storage)?;
            to_json_binary(&SubvaultsResponse { subvaults })
        }
        _ => Err(StdError::msg("not implemented")),
    }
}

pub fn contract() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new(execute, instantiate, query))
}
