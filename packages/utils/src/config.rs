use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use wavs_types::{ChainConfigs, ChainKey};

use crate::path::{repo_root, repo_wavs_home};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployTarget {
    Local,
    Testnet,
    Mainnet,
}

pub fn deploy_target() -> Result<DeployTarget> {
    match std::env::var("DEPLOY_CHAIN_TARGET").as_deref() {
        Ok("local") => Ok(DeployTarget::Local),
        Ok("testnet") => Ok(DeployTarget::Testnet),
        Ok("mainnet") => Ok(DeployTarget::Mainnet),
        _ => Err(anyhow::anyhow!(
            "DEPLOY_CHAIN_TARGET must be set to one of: local, testnet, or mainnet"
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalPortKind {
    WavsOperatorBase,
    WavsAggregator,
    IpfsApi,
    IpfsGateway,
    JaegerUi,
    PrometheusUi,
}

pub async fn local_port(kind: LocalPortKind) -> Result<u32> {
    let contents = tokio::fs::read_to_string(
        repo_root()
            .ok_or(anyhow::anyhow!("could not get repo root"))?
            .join("taskfile")
            .join("config.yml"),
    )
    .await
    .map_err(|e| anyhow::anyhow!("could not read taskfile config: {e:?}"))?;

    #[derive(Deserialize, Debug)]
    struct TaskfileConfig {
        vars: TaskfileConfigVars,
    }
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    struct TaskfileConfigVars {
        local_port_wavs_operator_base: u32,
        local_port_wavs_aggregator: u32,
        local_port_ipfs_api: u32,
        local_port_ipfs_gateway: u32,
        local_port_jaeger_ui: u32,
        local_port_prometheus_ui: u32,
    }

    let taskfile_config: TaskfileConfig = serde_yml::from_str(&contents)?;

    let local_port = match kind {
        LocalPortKind::WavsOperatorBase => taskfile_config.vars.local_port_wavs_operator_base,
        LocalPortKind::WavsAggregator => taskfile_config.vars.local_port_wavs_aggregator,
        LocalPortKind::IpfsApi => taskfile_config.vars.local_port_ipfs_api,
        LocalPortKind::IpfsGateway => taskfile_config.vars.local_port_ipfs_gateway,
        LocalPortKind::JaegerUi => taskfile_config.vars.local_port_jaeger_ui,
        LocalPortKind::PrometheusUi => taskfile_config.vars.local_port_prometheus_ui,
    };

    Ok(local_port)
}
pub async fn active_chain_key() -> Result<ChainKey> {
    let contents = tokio::fs::read_to_string(
        repo_root()
            .ok_or(anyhow::anyhow!("could not get repo root"))?
            .join("taskfile")
            .join("config.yml"),
    )
    .await
    .map_err(|e| anyhow::anyhow!("could not read taskfile config: {e:?}"))?;

    #[derive(Deserialize, Debug)]
    struct TaskfileConfig {
        vars: TaskfileConfigVars,
    }
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    struct TaskfileConfigVars {
        testnet_chain_key: ChainKey,
        mainnet_chain_key: ChainKey,
        local_chain_key: ChainKey,
    }

    let taskfile_config: TaskfileConfig = serde_yml::from_str(&contents)?;

    let chain_key = match deploy_target()? {
        DeployTarget::Local => taskfile_config.vars.local_chain_key.clone(),
        DeployTarget::Testnet => taskfile_config.vars.testnet_chain_key.clone(),
        DeployTarget::Mainnet => taskfile_config.vars.mainnet_chain_key.clone(),
    };

    Ok(chain_key)
}

pub async fn load_chain_configs_from_wavs(
    wavs_home: Option<impl AsRef<Path>>,
) -> Result<ChainConfigs> {
    #[derive(Deserialize)]
    struct ConfigFile {
        default: ConfigDefault,
    }

    #[derive(Deserialize)]
    struct ConfigDefault {
        chains: ChainConfigs,
    }

    let wavs_home = match wavs_home {
        Some(path) => path.as_ref().to_path_buf(),
        None => repo_wavs_home()
            .ok_or_else(|| anyhow::anyhow!("Failed to determine WAVS home directory"))?,
    };

    let contents = tokio::fs::read_to_string(wavs_home.join("wavs.toml")).await?;
    let config: ConfigFile = toml::from_str(&contents)?;

    Ok(config.default.chains)
}
