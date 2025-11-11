use std::borrow::Cow;

use anyhow::{Context, Result};
use imap::{types::Fetch, Session};
use mail_parser::{HeaderValue, Message, MessageParser};

#[derive(Debug)]
pub struct EmailMessage {
    // 1) “Original sender” best-effort (From / Resent-From / Sender / envelope)
    original_sender: String,
    // 2) All DKIM-Signature header values (there can be multiple)
    dkim_signatures: Vec<String>,
    // 3) Subject (decoded)
    subject: Option<String>,
    // 4) Body (best-effort: prefer text/plain part; fall back to full text)
    body_text: Option<String>,
}

impl EmailMessage {
    pub fn parse(f: &Fetch) -> anyhow::Result<Self> {
        let msg = MessageParser::new()
            .parse(f.body().context("Missing email body")?)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse email message"))?;

        let subject = msg.subject();

        // best effort original sender extraction
        let original_sender = msg
            .header("Resent-From")
            .or_else(|| msg.header("From"))
            .or_else(|| msg.header("Sender"))
            .and_then(|h| match h {
                HeaderValue::Address(addr) => addr.first().map(|a| a.address()).flatten(),
                _ => h.as_text(),
            })
            .map(|s| s.to_string())
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
            .header_values("DKIM-Signature")
            .flat_map(|h| h.as_text())
            .collect::<Vec<_>>();

        // prefer text/plain part; fall back to html text
        let body_text = msg.body_text(0).or_else(|| msg.body_html(0));

        Ok(Self {
            subject: subject.map(|s| s.to_string()),
            original_sender,
            dkim_signatures: dkim_signatures.into_iter().map(|s| s.to_string()).collect(),
            body_text: body_text.map(|s| s.to_string()),
        })
    }
}
