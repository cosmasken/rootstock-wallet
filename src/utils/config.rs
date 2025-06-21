use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub wallet: WalletConfig,
}

// Use NetworkConfig from types::network
use crate::types::network::NetworkConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletConfig {
    pub current_wallet_address: Option<String>,
    pub private_key: Option<String>,
    pub mnemonic: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        let config_path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
            // .join("rsk-wallet")
            .join("config.toml");

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(config_path)?;
        toml::from_str(&content).map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))
    }
    pub fn set_current_wallet(&mut self, address: &str) -> Result<(), anyhow::Error> {
        self.wallet.current_wallet_address = Some(address.to_string());
        self.save()?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let config_path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
            // .join("rsk-wallet")
            .join("config.toml");

        std::fs::create_dir_all(config_path.parent().unwrap())?;
        let content = toml::to_string(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                name: "Mainnet".to_string(),
                rpc_url: "https://public-node.rsk.co".to_string(),
                explorer_url: "https://explorer.rsk.co".to_string(),
            },
            wallet: WalletConfig {
                current_wallet_address: None,
                private_key: None,
                mnemonic: None,
            },
        }
    }
}
