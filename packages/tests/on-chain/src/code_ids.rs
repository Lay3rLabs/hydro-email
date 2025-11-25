use app_utils::path::repo_root;
use std::path::{Path, PathBuf};
use tracing::{debug, info, instrument};

use crate::client::TestPool;

static SERVICE_HANDLER_CODE_ID: tokio::sync::OnceCell<u64> = tokio::sync::OnceCell::const_new();
static PROXY_CODE_ID: tokio::sync::OnceCell<u64> = tokio::sync::OnceCell::const_new();

pub struct CodeId {}

impl CodeId {
    #[instrument]
    pub async fn new_service_handler() -> u64 {
        *SERVICE_HANDLER_CODE_ID
            .get_or_init(upload_service_handler)
            .await
    }

    #[instrument]
    pub async fn new_proxy() -> u64 {
        *PROXY_CODE_ID.get_or_init(upload_proxy).await
    }
}

async fn upload_service_handler() -> u64 {
    upload(wasm_path("service-handler")).await
}

async fn upload_proxy() -> u64 {
    upload(wasm_path("proxy")).await
}

#[instrument(skip(wasm_path), fields(path = %wasm_path.as_ref().display()))]
async fn upload(wasm_path: impl AsRef<Path>) -> u64 {
    let wasm_path = wasm_path.as_ref();

    info!("Reading WASM file");
    let wasm_bytes = tokio::fs::read(&wasm_path)
        .await
        .unwrap_or_else(|_| panic!("Failed to read {}", wasm_path.display()));

    debug!(size_bytes = wasm_bytes.len(), "WASM file loaded");

    let pool = TestPool::get().await;
    let client = pool.pool.get().await.unwrap();

    debug!("Uploading contract to chain");
    let code_id = client
        .contract_upload_file(wasm_bytes, None)
        .await
        .unwrap()
        .0;

    info!(code_id, "Contract uploaded successfully");

    code_id
}

fn wasm_path(contract: &str) -> PathBuf {
    let contract = contract.replace("-", "_");
    repo_root()
        .unwrap()
        .join(".builds")
        .join("contracts")
        .join(format!("app_contract_{contract}.wasm"))
}
