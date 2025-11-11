use std::sync::LazyLock;

use anyhow::Result;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{AppError, AppResult};

#[derive(Default)]
pub struct DebugSettings {
    pub print_greeting: bool,
    pub print_capabilities: bool,
}

pub static DEBUG: LazyLock<DebugSettings> = LazyLock::new(|| {
    let mut settings = DebugSettings::default();

    if let Ok(val) = get_env_var_bool("WAVS_ENV_IMAP_DEBUG_GREETING") {
        settings.print_greeting = val;
    }
    if let Ok(val) = get_env_var_bool("WAVS_ENV_IMAP_DEBUG_CAPABILITIES") {
        settings.print_capabilities = val;
    }

    settings
});

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub username: String,
    pub password: String,
    pub credential_kind: ImapCredentialKind,
}

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub enum ImapCredentialKind {
    Plain,
    OAuth2,
}

impl std::fmt::Display for ImapConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} (TLS: {})", self.host, self.port, self.tls)
    }
}

impl ImapConfig {
    pub fn new() -> AppResult<Self> {
        let host = get_env_var("WAVS_ENV_IMAP_HOST")?;
        let port = get_env_var("WAVS_ENV_IMAP_PORT")?;
        let tls = get_env_var("WAVS_ENV_IMAP_TLS")?;
        let username = get_env_var("WAVS_ENV_IMAP_USERNAME")?;
        let password = get_env_var("WAVS_ENV_IMAP_PASSWORD")?;
        let credential_kind = get_env_var("WAVS_ENV_IMAP_CREDENTIAL_KIND")?;

        let port: u16 = port.parse().map_err(|_| AppError::InvalidEnv {
            key: "WAVS_ENV_IMAP_PORT",
            reason: "Not a valid u16",
        })?;

        let tls = match tls.to_lowercase().as_str() {
            "true" => true,
            "false" => false,
            _ => {
                return Err(AppError::InvalidEnv {
                    key: "WAVS_ENV_IMAP_TLS",
                    reason: "Not a valid boolean",
                })
            }
        };

        let credential_kind = match credential_kind.to_lowercase().as_str() {
            "plain" => ImapCredentialKind::Plain,
            "oauth2" => ImapCredentialKind::OAuth2,
            _ => {
                return Err(AppError::InvalidEnv {
                    key: "WAVS_ENV_IMAP_CREDENTIAL_KIND",
                    reason: "Not a valid credential kind (expected 'plain' or 'oauth2')",
                })
            }
        };

        Ok(ImapConfig {
            host: host.to_string(),
            port,
            tls,
            username: username.to_string(),
            password: password.to_string(),
            credential_kind,
        })
    }
}

fn get_env_var(key: &str) -> AppResult<String> {
    let value = std::env::var(key).unwrap_or_default();

    // clean up extra quotes from env vars if present (docker env var weirdness)
    let value = value.trim_matches('"');

    if value.is_empty() {
        return Err(AppError::MissingEnv {
            key: key.to_string(),
        });
    }

    Ok(value.to_string())
}

fn get_env_var_bool(key: &str) -> AppResult<bool> {
    get_env_var(key).map(|x| x.to_lowercase() == "true" || x == "1")
}
