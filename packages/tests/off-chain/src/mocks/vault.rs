//! Mock inflow vault contract for testing
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult,
};
use cw_multi_test::{Contract, ContractWrapper};
use cw_storage_plus::Item;
use hydro_interface::inflow::{Config, ConfigResponse, ExecuteMsg, QueryMsg};

const CONFIG: Item<Config> = Item::new("config");

/// Simplified instantiate message for testing
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub deposit_denom: String,
    pub vault_shares_denom: String,
    pub control_center_contract: String,
}

fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        deposit_denom: msg.deposit_denom,
        vault_shares_denom: msg.vault_shares_denom,
        control_center_contract: deps.api.addr_validate(&msg.control_center_contract)?,
        token_info_provider_contract: None,
        max_withdrawals_per_user: 10,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

fn execute(_deps: DepsMut, _env: Env, _info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Deposit { .. } => Ok(Response::new().add_attribute("action", "deposit")),
        _ => Err(StdError::msg("not implemented")),
    }
}

fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config = CONFIG.load(deps.storage)?;
            to_json_binary(&ConfigResponse { config })
        }
        _ => Err(StdError::msg("not implemented")),
    }
}

pub fn contract() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new(execute, instantiate, query))
}
