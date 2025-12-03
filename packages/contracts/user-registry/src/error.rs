use app_contract_api::user_registry::msg::UserId;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("no admins provided")]
    NoAdmins {},

    #[error("unauthorized")]
    Unauthorized {},

    #[error("user already registered: {user_id}")]
    UserAlreadyRegistered { user_id: UserId },

    #[error("no proxy address for user: {user_id}")]
    ProxyAddressNotFound { user_id: UserId },
}
