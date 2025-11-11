#![allow(warnings)]
mod config;
mod connection;
mod email;
mod error;

use anyhow::bail;

use crate::wavs::operator::input::TriggerData;

// this is needed just to make the ide/compiler happy... we're _always_ compiling to wasm32-wasi
wit_bindgen::generate!({
    path: "wit-definitions/operator/wit",
    world: "wavs-world",
    generate_all,
    with: {
        "wasi:io/poll@0.2.0": wasip2::io::poll
    },
    features: ["tls"]
});

struct Component;

impl Guest for Component {
    fn run(trigger_action: TriggerAction) -> Result<Option<WasmResponse>, String> {
        wstd::runtime::block_on(async move {
            match inner(trigger_action).await {
                Ok(resp) => Ok(resp),
                Err(e) => {
                    eprintln!("{:?}", e);
                    Err(e.to_string())
                }
            }
        })
    }
}

async fn inner(trigger_action: TriggerAction) -> anyhow::Result<Option<WasmResponse>> {
    match trigger_action.data {
        TriggerData::Raw(data) => {
            let data = std::str::from_utf8(&data)?;
            match data {
                "read-mail" => {
                    let email = email::read_next_email().await?;
                    println!("{:#?}", email);
                }
                _ => {
                    bail!("Unknown command: {}", data);
                }
            }
        }
        _ => {
            bail!("Unsupported TriggerData variant");
        }
    }
    Ok(None)
}

export!(Component);
