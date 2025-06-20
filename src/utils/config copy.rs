use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub provider_url: String,
}

impl Config {
    pub fn load() -> Self {
        let content = fs::read_to_string("config.toml").expect("Failed to read config.toml");
        toml::from_str(&content).expect("Failed to parse config.toml")
    }

    pub fn network(&self) -> &'static str {
        if self.provider_url.contains("testnet") {
            "testnet"
        } else {
            "mainnet"
        }
    }
}