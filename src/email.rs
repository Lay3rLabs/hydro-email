mod auth;
mod parser;
pub mod verify;

use futures::StreamExt;
use imap::Session;
use thiserror::Error;

use crate::{
    config::{ImapConfig, DEBUG},
    connection::{Connection, ConnectionError},
    email::{auth::auth_session, parser::EmailMessage},
    error::{AppError, AppResult},
};

pub async fn read_next_email() -> AppResult<Option<EmailMessage>> {
    let config = crate::config::ImapConfig::new()?;

    let connection = Connection::new(&config).await?;
    println!("Successfully connected to {config}");

    let mut client = imap::Client::new(connection);

    let greeting = {
        let s = client.read_greeting()?;
        let s = String::from_utf8_lossy(&s);
        s.trim_end_matches(['\r', '\n']).to_string()
    };

    if DEBUG.print_greeting {
        println!("Imap greeting: {greeting}");
    }

    let mut session = auth_session(client, &config).await?;

    if DEBUG.print_capabilities {
        for capability in session.capabilities()?.iter() {
            println!("Server capability: {:?}", capability);
        }
    }

    let mailbox = session.select("INBOX")?;

    if mailbox.exists == 0 {
        return Ok(None);
    }

    let uids = session.uid_search("UNSEEN")?;
    if uids.is_empty() {
        return Ok(None);
    }

    let latest_uid = *uids.iter().max().unwrap();

    let fetches = session.uid_fetch(latest_uid.to_string(), "(ENVELOPE BODY[])")?;

    let fetch = fetches
        .iter()
        .next()
        .ok_or(AppError::FailedToFetchEmail(latest_uid))?;

    Ok(Some(
        EmailMessage::parse(&fetch).map_err(AppError::AnyMessageParse)?,
    ))
}
