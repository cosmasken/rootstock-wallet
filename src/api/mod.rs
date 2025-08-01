use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiProvider {
    Alchemy,
    Infura,
    Etherscan,
    Custom(String),  // For any other API provider
}

impl fmt::Display for ApiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiProvider::Alchemy => write!(f, "Alchemy"),
            ApiProvider::Infura => write!(f, "Infura"),
            ApiProvider::Etherscan => write!(f, "Etherscan"),
            ApiProvider::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: String,
    pub network: String,  // "mainnet", "testnet", etc.
    pub provider: ApiProvider,
    pub name: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ApiManager {
    keys: HashMap<String, ApiKey>,  // keyed by a unique identifier
}

impl ApiManager {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn add_key(&mut self, key: ApiKey) -> String {
        let id = format!("{:?}-{}", key.provider, key.network).to_lowercase();
        self.keys.insert(id.clone(), key);
        id
    }

    pub fn get_key(&self, provider: &ApiProvider, network: &str) -> Option<&ApiKey> {
        let id = format!("{:?}-{}", provider, network).to_lowercase();
        self.keys.get(&id)
    }

    pub fn remove_key(&mut self, provider: &ApiProvider, network: &str) -> Option<ApiKey> {
        let id = format!("{:?}-{}", provider, network).to_lowercase();
        self.keys.remove(&id)
    }

    pub fn list_keys(&self) -> Vec<&ApiKey> {
        self.keys.values().collect()
    }
}

// Integration with the existing config system
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    pub default_provider: Option<ApiProvider>,
    pub keys: Vec<ApiKey>,
}
