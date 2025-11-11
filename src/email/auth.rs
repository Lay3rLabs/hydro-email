use imap::{Client, Session};

use crate::{config::ImapConfig, connection::Connection, error::AppResult};

pub fn auth_session(
    client: Client<Connection>,
    config: &ImapConfig,
) -> AppResult<Session<Connection>> {
    match config.credential_kind {
        crate::config::ImapCredentialKind::Plain => {
            let session = client
                .login(&config.username, &config.password)
                .map_err(|(e, _)| e)?;
            Ok(session)
        }
        crate::config::ImapCredentialKind::OAuth2 => {
            // Implement OAuth2 authentication here
            unimplemented!("OAuth2 authentication is not yet implemented");
        }
    }
}
