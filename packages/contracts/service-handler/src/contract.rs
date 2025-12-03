use app_contract_api::{
    service_handler::{
        event::EmailEvent,
        msg::{
            AdminResponse, CustomExecuteMsg, CustomQueryMsg, EmailAddrsResponse,
            EmailsFromResponse, EmailsResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
            UserRegistryResponse,
        },
    },
    user_registry::msg::UserId,
};
use cosmwasm_std::{
    ensure, entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, WasmMsg,
};
use wavs_types::contracts::cosmwasm::{
    service_handler::{ServiceHandlerExecuteMessages, ServiceHandlerQueryMessages},
    service_manager::{ServiceManagerQueryMessages, WavsValidateResult},
};

use crate::{
    error::ContractError,
    state::{self, ADMIN, SERVICE_MANAGER},
};

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    state::initialize(&mut deps, msg)?;

    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Custom(msg) => {
            let admin = ADMIN.load(deps.storage)?;
            ensure!(info.sender == admin, ContractError::Unauthorized);
            handle_custom_message(&mut deps, msg)
        }
        ExecuteMsg::Wavs(msg) => match msg {
            ServiceHandlerExecuteMessages::WavsHandleSignedEnvelope {
                envelope,
                signature_data,
            } => {
                let service_manager = SERVICE_MANAGER.load(deps.storage)?;

                // TODO: only allow eventId to be used once

                deps.querier.query_wasm_smart::<WavsValidateResult>(
                    service_manager,
                    &ServiceManagerQueryMessages::WavsValidate {
                        envelope: envelope.clone(),
                        signature_data: signature_data.clone(),
                    },
                )?;

                let envelope = envelope
                    .decode()
                    .map_err(|e| ContractError::AbiDecode(e.to_string()))?;

                let msg = CustomExecuteMsg::decode(&envelope.payload)
                    .map_err(|e| ContractError::PayloadDecode(e.to_string()))?;

                handle_custom_message(&mut deps, msg)
            }
        },
    }
}

fn handle_custom_message(
    deps: &mut DepsMut,
    msg: CustomExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        CustomExecuteMsg::Email(email) => {
            let pagination_id = state::push_email(deps.storage, &email)?;
            let user_id = UserId::new_email_address(&email.from);

            let proxy_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: state::proxy_address(deps.as_ref(), user_id)?.to_string(),
                msg: to_json_binary(&app_contract_api::proxy::msg::ExecuteMsg::ForwardToInflow {})?,
                funds: vec![],
            });

            Ok(Response::new()
                .add_message(proxy_msg)
                .add_event(EmailEvent {
                    email,
                    pagination_id,
                }))
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Custom(msg) => match msg {
            CustomQueryMsg::EmailAddrs { limit, start_after } => {
                let email_addrs =
                    state::list_email_addresses(deps.storage, start_after.as_deref(), limit)?;
                to_json_binary(&EmailAddrsResponse { email_addrs })
            }
            CustomQueryMsg::EmailsFrom {
                from,
                limit,
                start_after,
            } => {
                let emails = state::list_emails_from(deps.storage, &from, start_after, limit)?;
                to_json_binary(&EmailsFromResponse { emails })
            }
            CustomQueryMsg::Emails { limit, start_after } => {
                let emails = state::list_emails(deps.storage, start_after, limit)?;
                to_json_binary(&EmailsResponse { emails })
            }
            CustomQueryMsg::Admin {} => {
                let admin = ADMIN.may_load(deps.storage)?.map(Into::into);
                to_json_binary(&AdminResponse { admin })
            }
            CustomQueryMsg::UserRegistry {} => {
                let address = state::user_registry_address(deps.storage)?;
                to_json_binary(&UserRegistryResponse { address })
            }
        },
        QueryMsg::Wavs(msg) => match msg {
            ServiceHandlerQueryMessages::WavsServiceManager {} => {
                let service_manager = SERVICE_MANAGER.load(deps.storage)?;
                to_json_binary(&service_manager)
            }
        },
    }
}

#[entry_point]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    Err(ContractError::UnknownReplyId { id: msg.id })
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    state::migrate(deps.storage)?;
    Ok(Response::default())
}
