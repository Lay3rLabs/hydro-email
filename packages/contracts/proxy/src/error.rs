use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("hydro proxy error: {0}")]
    HydroProxy(#[from] hydro_proxy::error::ContractError),

    #[error("no admins provided")]
    NoAdmins {},

    #[error("unauthorized")]
    Unauthorized {},
}
