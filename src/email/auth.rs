use imap::{Client, Session};
use wstd::http::{Body, HeaderValue, Request};

use crate::{
    config::{ImapConfig, ImapCredentials},
    connection::Connection,
    error::{AppError, AppResult},
};

pub async fn auth_session(
    client: Client<Connection>,
    config: &ImapConfig,
) -> AppResult<Session<Connection>> {
    match &config.credentials {
        ImapCredentials::Plain { username, password } => client
            .login(&username, &password)
            .map_err(|(e, _)| AppError::Auth(e.into())),
        ImapCredentials::Gmail {
            client_id,
            client_secret,
            refresh_token,
        } => {
            let access_token =
                fetch_gmail_access_token(client_id, client_secret, refresh_token).await?;

            let username = fetch_gmail_email(&access_token).await?;

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

async fn fetch_gmail_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> AppResult<String> {
    let http_client = wstd::http::Client::new();

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let body = serde_urlencoded::to_string(&params)
        .map_err(|e| AppError::Auth(anyhow::anyhow!("Failed to encode OAuth2 params: {}", e)))?;

    let request = Request::post("https://oauth2.googleapis.com/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .map_err(|e| AppError::Auth(anyhow::anyhow!("Failed to build OAuth2 request: {}", e)))?;

    let response = http_client
        .send(request)
        .await
        .map_err(|e| AppError::Auth(anyhow::anyhow!("OAuth2 request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Auth(anyhow::anyhow!(
            "OAuth2 request returned error status: {}",
            response.status()
        )));
    }

    let mut body = response.into_body();
    let body = body.contents().await.map_err(AppError::Auth)?;

    let json_body: serde_json::Value = serde_json::from_slice(&body).map_err(|e| {
        AppError::Auth(anyhow::anyhow!(
            "Failed to parse OAuth2 response JSON: {}",
            e
        ))
    })?;

    let access_token = json_body
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Auth(anyhow::anyhow!("No access_token in OAuth2 response")))?;

    Ok(access_token.to_string())
}

async fn fetch_gmail_email(access_token: &str) -> AppResult<String> {
    let http_client = wstd::http::Client::new();

    let request = Request::get("https://gmail.googleapis.com/gmail/v1/users/me/profile")
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Accept", "application/json")
        .body(Body::empty())
        .map_err(|e| AppError::Auth(anyhow::anyhow!("Failed to build profile request: {}", e)))?;

    let response = http_client
        .send(request)
        .await
        .map_err(|e| AppError::Auth(anyhow::anyhow!("Profile request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Auth(anyhow::anyhow!(
            "Profile request returned error status: {}",
            response.status()
        )));
    }

    let mut body = response.into_body();
    let body = body.contents().await.map_err(AppError::Auth)?;

    let json_body: serde_json::Value = serde_json::from_slice(&body).map_err(|e| {
        AppError::Auth(anyhow::anyhow!(
            "Failed to parse Profile response JSON: {}",
            e
        ))
    })?;

    let email_address = json_body
        .get("emailAddress")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Auth(anyhow::anyhow!("No emailAddress in Profile response")))?;

    Ok(email_address.to_string())
}
