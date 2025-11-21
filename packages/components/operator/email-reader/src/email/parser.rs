use std::borrow::Cow;

use anyhow::{Context, Result};
use imap::{types::Fetch, Session};
use mailparse::*;

use crate::error::AppResult;

pub struct EmailMessage {
    // 1) "Original sender" best-effort (From / Resent-From / Sender / envelope)
    pub original_sender: String,
    // 2) All DKIM-Signature header values (there can be multiple)
    pub dkim_signatures: Vec<String>,
    // 3) Subject (decoded)
    pub subject: Option<String>,
    // 4) Body (best-effort: prefer text/plain part; fall back to full text)
    pub body_text: Option<String>,
    // 5) Raw email bytes (can be parsed on demand)
    pub raw_bytes: Vec<u8>,
}

impl std::fmt::Debug for EmailMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmailMessage")
            .field("original_sender", &self.original_sender)
            .field("dkim_signatures", &self.dkim_signatures)
            .field("subject", &self.subject)
            .field(
                "body_text",
                &self
                    .body_text
                    .as_ref()
                    .map(|s| Cow::Owned(format!("{}...", &s[..s.len().min(30)])))
                    .unwrap_or(Cow::Borrowed("None")),
            )
            .field("raw_bytes_len", &self.raw_bytes.len())
            .finish()
    }
}

impl EmailMessage {
    pub fn parse(f: &Fetch) -> anyhow::Result<Self> {
        let body_bytes = f.body().context("Missing email body")?;
        let msg = parse_mail(body_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to parse email message: {:?}", e))?;

        let subject = msg
            .headers
            .iter()
            .find(|h| h.get_key().eq_ignore_ascii_case("Subject"))
            .map(|h| h.get_value());

        // best effort original sender extraction
        let original_sender = msg
            .headers
            .iter()
            .find(|h| {
                h.get_key().eq_ignore_ascii_case("Resent-From")
                    || h.get_key().eq_ignore_ascii_case("From")
                    || h.get_key().eq_ignore_ascii_case("Sender")
            })
            .map(|h| h.get_value())
            .or_else(|| {
                f.envelope()
                    .and_then(|env| {
                        env.from.as_ref().map(|addrs| {
                            addrs
                                .first()
                                .map(|a| a.mailbox.map(|s| String::from_utf8(s.to_vec()).ok()))
                        })
                    })
                    .flatten()
                    .flatten()
                    .flatten()
            })
            .ok_or_else(|| anyhow::anyhow!("Failed to extract original sender"))?;

        // there can be multiple signatures
        let dkim_signatures = msg
            .headers
            .iter()
            .filter(|h| h.get_key().eq_ignore_ascii_case("DKIM-Signature"))
            .map(|h| h.get_value())
            .collect::<Vec<_>>();

        // prefer text/plain part; fall back to html text
        let body_text = msg.get_body().ok();

        Ok(Self {
            subject: subject.map(|s| s.to_string()),
            original_sender: original_sender.to_string(),
            dkim_signatures: dkim_signatures.into_iter().map(|s| s.to_string()).collect(),
            body_text: body_text.map(|s| s.to_string()),
            raw_bytes: body_bytes.to_vec(),
        })
    }

    /// Parse the raw email bytes and return a ParsedMail
    pub fn get_parsed(&self) -> AppResult<ParsedMail> {
        Ok(parse_mail(&self.raw_bytes)?)
    }
}
