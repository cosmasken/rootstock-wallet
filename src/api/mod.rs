use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::security::{SecureString, SecureApiKey};
use crate::security::redacted_debug::RedactedDebug;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiProvider {
    /// Alchemy API - Used for transaction history and advanced queries
    Alchemy,
    /// RSK RPC API - Primary RPC for blockchain operations (balances, transactions, etc.)
    RskRpc,
    /// Custom API provider
    Custom(String),
}

impl fmt::Display for ApiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiProvider::Alchemy => write!(f, "Alchemy"),
            ApiProvider::RskRpc => write!(f, "RSK RPC"),
            ApiProvider::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: SecureString,
    pub network: String, // "mainnet", "testnet", etc.
    pub provider: ApiProvider,
    pub name: Option<String>,
}

impl RedactedDebug for ApiKey {
    fn redacted_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiKey")
            .field("key", &self.key) // SecureString already implements RedactedDebug
            .field("network", &self.network)
            .field("provider", &self.provider)
            .field("name", &self.name)
            .finish()
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted_fmt(f)
    }
}

impl ApiKey {
    /// Create a new ApiKey with secure storage
    pub fn new(key: String, network: String, provider: ApiProvider, name: Option<String>) -> Self {
        Self {
            key: SecureString::new(key),
            network,
            provider,
            name,
        }
    }

    /// Get the API key as a SecureApiKey for secure usage
    pub fn as_secure_api_key(&self) -> Result<SecureApiKey, std::str::Utf8Error> {
        let key_str = self.key.expose()?.to_string();
        Ok(SecureApiKey::new(key_str))
    }

    /// Get the raw key value (use with caution, prefer as_secure_api_key)
    pub fn expose_key(&self) -> Result<&str, std::str::Utf8Error> {
        self.key.expose()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ApiManager {
    keys: HashMap<String, ApiKey>, // keyed by a unique identifier
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
