use crate::oauth::{fetch_gmail_access_token, fetch_gmail_email_address};
use imap::{Client, Session};
use wstd::http::{Body, HeaderValue, Request};

use crate::{
    config::{ImapConfig, ImapCredentials},
    email::imap::connection::ImapConnection,
    error::{AppError, AppResult},
};

pub async fn auth_session(
    client: Client<ImapConnection>,
    config: &ImapConfig,
) -> AppResult<Session<ImapConnection>> {
    match &config.credentials {
        ImapCredentials::Plain { username, password } => {
            println!("Getting email for {username}");

            client
                .login(&username, &password)
                .map_err(|(e, _)| AppError::Auth(e.into()))
        }
        ImapCredentials::Gmail {
            client_id,
            client_secret,
            refresh_token,
        } => {
            let access_token =
                fetch_gmail_access_token(client_id, client_secret, refresh_token).await?;

            let username = fetch_gmail_email_address(&access_token).await?;

            println!("Getting email for {username}");

            client
                .authenticate(
                    "XOAUTH2",
                    &OAuth2 {
                        username: &username,
                        access_token: &access_token,
                    },
                )
                .map_err(|(e, _)| AppError::Auth(e.into()))
        }
    }
}

struct OAuth2<'a> {
    username: &'a str,
    access_token: &'a str,
}

impl imap::Authenticator for OAuth2<'_> {
    type Response = String;
    fn process(&self, _: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.username, self.access_token
        )
    }
}
