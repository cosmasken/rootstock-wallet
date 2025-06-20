use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::str::FromStr;
// use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub chain_id: u64,
    pub rpc_url: String,
    pub explorer_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    pub fn get_config(&self) -> NetworkConfig {
        match self {
            Network::Mainnet => NetworkConfig {
                name: "RSK Mainnet".to_string(),
                chain_id: 30,
                rpc_url: "https://public-node.rsk.co".to_string(),
                explorer_url: "https://explorer.rsk.co".to_string(),
            },
            Network::Testnet => NetworkConfig {
                name: "RSK Testnet".to_string(),
                chain_id: 31,
                rpc_url: "https://public-node.testnet.rsk.co".to_string(),
                explorer_url: "https://explorer.testnet.rsk.co".to_string(),
            },
        }
    }
}
