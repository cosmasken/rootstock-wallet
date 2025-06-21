use serde::{Deserialize, Serialize};


// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct NetworkConfig {
//     pub name: String,
//     pub rpc_url: String,
//     pub explorer_url: String,
// }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub rpc_url: String,
    pub explorer_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Network {
    Mainnet,
    Testnet,
    AlchemyMainnet,
    AlchemyTestnet,
    RootStockMainnet,
    RootStockTestnet,
}

impl Network {
    pub fn get_config(&self) -> NetworkConfig {
        match self {
            Network::Mainnet => NetworkConfig {
                name: "RSK Mainnet".to_string(),
                rpc_url: "https://public-node.rsk.co".to_string(),
                explorer_url: "https://explorer.rsk.co".to_string(),
            },
            Network::Testnet => NetworkConfig {
                name: "RSK Testnet".to_string(),
                rpc_url: "https://public-node.testnet.rsk.co".to_string(),
                explorer_url: "https://explorer.testnet.rsk.co".to_string(),
            },
            Network::AlchemyMainnet => NetworkConfig {
                name: "RSK Mainnet".to_string(),
                rpc_url: "https://public-node.rsk.co".to_string(),
                explorer_url: "https://explorer.rsk.co".to_string(),
            },
            Network::AlchemyTestnet => NetworkConfig {
                name: "RSK Testnet".to_string(),
                rpc_url: "https://public-node.testnet.rsk.co".to_string(),
                explorer_url: "https://explorer.testnet.rsk.co".to_string(),
            },
            Network::RootStockMainnet => NetworkConfig {
                name: "RSK Mainnet".to_string(),
                rpc_url: "https://public-node.rsk.co".to_string(),
                explorer_url: "https://explorer.rsk.co".to_string(),
            },
            Network::RootStockTestnet => NetworkConfig {
                name: "RSK Testnet".to_string(),
                rpc_url: "https://public-node.testnet.rsk.co".to_string(),
                explorer_url: "https://explorer.testnet.rsk.co".to_string(),
            },
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mainnet" => Some(Network::Mainnet),
            "testnet" => Some(Network::Testnet),    
            "alchemy-mainnet" => Some(Network::AlchemyMainnet),
            "alchemy-testnet" => Some(Network::AlchemyTestnet),
            "rootstock-mainnet" => Some(Network::RootStockMainnet),
            "rootstock-testnet" => Some(Network::RootStockTestnet),
            _ => None,
        }
    }
}
