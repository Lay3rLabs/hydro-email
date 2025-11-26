use app_utils::{
    config::{active_chain_key, load_chain_configs_from_wavs},
    path::repo_wavs_home,
};
use layer_climb::prelude::ChainConfig;
use tokio::sync::OnceCell;
use wavs_types::ChainKey;

pub(super) const PORT_WAVS_OPERATOR_BASE: u32 = 8123;
pub(super) const PORT_WAVS_AGGREGATOR: u32 = 8200;

// TODO - extend this for multiple operators
static TEST_CONFIG: OnceCell<TestConfig> = OnceCell::const_new();

#[derive(Clone)]
pub struct TestConfig {
    pub chain: ChainKey,
    pub chain_config: ChainConfig,
}

impl TestConfig {
    pub async fn get() -> Self {
        TEST_CONFIG.get_or_init(Self::instantiate).await.clone()
    }

    pub fn wavs_endpoint(operator_number: Option<u32>) -> String {
        format!(
            "http://localhost:{}",
            PORT_WAVS_OPERATOR_BASE + operator_number.unwrap_or(0)
        )
    }

    pub fn aggregator_endpoint() -> String {
        format!("http://localhost:{PORT_WAVS_AGGREGATOR}")
    }

    async fn instantiate() -> Self {
        let chain_configs = load_chain_configs_from_wavs(repo_wavs_home())
            .await
            .expect("Failed to load chain configurations");

        let chain_key = active_chain_key().await.unwrap();

        let mut chain_config = chain_configs
            .get_chain(&chain_key)
            .unwrap_or_else(|| panic!("No cosmos chain config found for {chain_key}"))
            .clone()
            .to_cosmos_config()
            .unwrap();

        chain_config.grpc_endpoint = None;

        tracing::info!("Using chain config for {chain_key}");

        Self {
            chain: chain_key,
            chain_config: chain_config.into(),
        }
    }
}
