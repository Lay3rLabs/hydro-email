use thiserror::Error;

use crate::connection::ConnectionError;

pub type AppResult<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0:?}")]
    Connection(#[from] ConnectionError),

    #[error("{0:?}")]
    Client(#[from] imap::Error),

    #[error("Missing env var for {key}")]
    MissingEnv { key: String },

    #[error("Invalid env var for {key}: {reason}")]
    InvalidEnv {
        key: &'static str,
        reason: &'static str,
    },

    #[error("Failed to fetch email, uid: {0}")]
    FailedToFetchEmail(u32),

    #[error("{0:?}")]
    MessageParse(anyhow::Error),
}
