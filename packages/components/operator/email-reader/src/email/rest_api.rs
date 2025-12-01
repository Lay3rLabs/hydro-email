use crate::{
    config::GmailRestApiConfig,
    email::parser::EmailMessage,
    error::{AppError, AppResult},
    oauth::{fetch_gmail_access_token, fetch_gmail_email_address},
};
use anyhow::anyhow;
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use wstd::http::{Body, Request};

pub async fn read_next_email_rest_api(
    config: GmailRestApiConfig,
) -> AppResult<Option<EmailMessage>> {
    let access_token = fetch_gmail_access_token(
        &config.client_id,
        &config.client_secret,
        &config.refresh_token,
    )
    .await?;
    let username = fetch_gmail_email_address(&access_token).await?;

    let message_ids = fetch_unread_message_ids(&access_token, 1).await?;

    let message_id = match message_ids.first() {
        Some(message_id) => message_id,
        None => return Ok(None),
    };

    let message = fetch_message_by_id(&access_token, message_id).await?;

    mark_message_as_read(&access_token, message_id).await?;

    Ok(Some(message))
}

// List unread message IDs
// https://developers.google.com/workspace/gmail/api/reference/rest/v1/users.messages/list
pub async fn fetch_unread_message_ids(
    access_token: &str,
    max_results: u32,
) -> AppResult<Vec<String>> {
    let http_client = wstd::http::Client::new();

    #[derive(Serialize)]
    struct MessageListQuery {
        q: String,
        #[serde(rename = "maxResults")]
        max_results: u32,
    }

    let query_params = MessageListQuery {
        q: "is:unread".to_string(),
        max_results,
    };

    let query_string = serde_urlencoded::to_string(&query_params)
        .map_err(|e| AppError::Auth(anyhow!("Failed to serialize query parameters: {}", e)))?;

    let url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages?{}",
        query_string
    );

    let request = Request::get(url)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Accept", "application/json")
        .body(Body::empty())
        .map_err(|e| AppError::Auth(anyhow!("Failed to build messages list request: {}", e)))?;

    let response = http_client
        .send(request)
        .await
        .map_err(|e| AppError::Auth(anyhow!("Messages list request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Auth(anyhow!(
            "Messages list request returned error status: {}",
            response.status()
        )));
    }

    let mut body = response.into_body();
    let body = body.contents().await.map_err(AppError::Auth)?;

    #[derive(Debug, Deserialize)]
    struct ListResponse {
        messages: Vec<ListMessage>,
        #[serde(rename = "nextPageToken")]
        next_page_token: String,
        #[serde(rename = "resultSizeEstimate")]
        result_size_estimate: u32,
    }

    #[derive(Debug, Deserialize)]
    pub struct ListMessage {
        pub id: String,
    }

    let response: ListResponse = serde_json::from_slice(&body).map_err(|e| {
        AppError::Auth(anyhow!(
            "Failed to parse messages list response JSON: {}",
            e
        ))
    })?;

    Ok(response
        .messages
        .into_iter()
        .map(|m| m.id)
        .collect::<Vec<String>>())
}

// Fetch a message by ID
// https://developers.google.com/workspace/gmail/api/reference/rest/v1/users.messages/get
pub async fn fetch_message_by_id(access_token: &str, message_id: &str) -> AppResult<EmailMessage> {
    let http_client = wstd::http::Client::new();

    let url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}?format=raw",
        message_id
    );

    let request = Request::get(url)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Accept", "application/json")
        .body(Body::empty())
        .map_err(|e| AppError::Auth(anyhow!("Failed to build message fetch request: {}", e)))?;

    let response = http_client
        .send(request)
        .await
        .map_err(|e| AppError::Auth(anyhow!("Message fetch request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Auth(anyhow!(
            "Message fetch request returned error status: {}",
            response.status()
        )));
    }

    let mut body = response.into_body();
    let body = body.contents().await.map_err(AppError::Auth)?;

    #[derive(Debug, Deserialize)]
    struct MessageResponse {
        id: String,
        threadId: String,
        labelIds: Vec<String>,
        snippet: String,
        sizeEstimate: u32,
        raw: String,
    }

    let response: MessageResponse = serde_json::from_slice(&body).map_err(|e| {
        AppError::Auth(anyhow!(
            "Failed to parse message fetch response JSON: {}",
            e
        ))
    })?;

    let raw_bytes = BASE64_URL_SAFE
        .decode(&response.raw)
        .map_err(|e| AppError::Auth(anyhow!("Failed to decode raw message body: {}", e)))?;

    let email_message =
        EmailMessage::parse_rest_api(&raw_bytes).map_err(AppError::AnyMessageParse)?;

    Ok(email_message)
}

// Mark message as read
// https://developers.google.com/workspace/gmail/api/reference/rest/v1/users.messages/modify
pub async fn mark_message_as_read(access_token: &str, message_id: &str) -> AppResult<()> {
    let http_client = wstd::http::Client::new();

    let url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}/modify",
        message_id
    );

    let request = Request::post(url)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{"removeLabelIds": ["UNREAD"]}"#.as_bytes().to_vec(),
        ))
        .map_err(|e| {
            AppError::Auth(anyhow!(
                "Failed to build message mark-as-read request: {}",
                e
            ))
        })?;

    let response = http_client
        .send(request)
        .await
        .map_err(|e| AppError::Auth(anyhow!("Message mark-as-read request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Auth(anyhow!(
            "Message mark-as-read request returned error status: {}",
            response.status()
        )));
    }

    let mut body = response.into_body();
    let body = body.contents().await.map_err(AppError::Auth)?;

    #[derive(Debug, Deserialize)]
    struct MarkAsReadResponse {
        id: String,
    }

    let response: MarkAsReadResponse = serde_json::from_slice(&body).map_err(|e| {
        AppError::Auth(anyhow!(
            "Failed to parse message mark-as-read response JSON: {}",
            e
        ))
    })?;

    if response.id != message_id {
        return Err(AppError::Auth(anyhow!(
            "Marked message ID does not match: expected {}, got {}",
            message_id,
            response.id
        )));
    }

    Ok(())
}
