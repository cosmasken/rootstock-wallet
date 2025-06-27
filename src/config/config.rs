use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

// Import Network from the types module
use crate::types::network::Network;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub default_network: Network,
    pub alchemy_mainnet_key: Option<String>,
    pub alchemy_testnet_key: Option<String>,
    pub default_wallet: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_network: Network::Testnet,
            alchemy_mainnet_key: None,
            alchemy_testnet_key: None,
            default_wallet: None,
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Could not find config directory")?
            .join("rootstock-wallet");
        
        std::fs::create_dir_all(&config_dir)?;
        
        Ok(Self {
            config_path: config_dir.join("config.json"),
        })
    }

    pub fn load(&self) -> Result<Config> {
        if !self.config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&self.config_path)
            .context("Failed to read config file")?;
        
        serde_json::from_str(&content)
            .context("Failed to parse config file")
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        let content = serde_json::to_string_pretty(config)
            .context("Failed to serialize config")?;
        
        fs::write(&self.config_path, content)
            .context("Failed to write config file")
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn ensure_configured(&self) -> Result<()> {
        let config = self.load()?;
        
        match config.default_network {
            Network::Mainnet if config.alchemy_mainnet_key.is_none() => {
                anyhow::bail!(
                    "Mainnet API key not configured. Please run `setup` or `config set alchemy-mainnet-key <key>`"
                );
            }
            Network::Testnet if config.alchemy_testnet_key.is_none() => {
                anyhow::bail!(
                    "Testnet API key not configured. Please run `setup` or `config set alchemy-testnet-key <key>`"
                );
            }
            _ => Ok(())
        }
    }
}