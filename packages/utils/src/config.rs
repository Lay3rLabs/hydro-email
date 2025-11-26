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
    match std::env::var("DEPLOY_TARGET").as_deref() {
        Ok("local") => Ok(DeployTarget::Local),
        Ok("testnet") => Ok(DeployTarget::Testnet),
        Ok("mainnet") => Ok(DeployTarget::Mainnet),
        _ => Err(anyhow::anyhow!(
            "DEPLOY_TARGET must be set to one of: local, testnet, or mainnet"
        )),
    }
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
