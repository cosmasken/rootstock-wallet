// use serde::Deserialize;
// use std::collections::HashMap;

// #[derive(Deserialize)]
// pub struct TokenInfo {
//     pub address: String,
//     pub decimals: u8,
// }

// #[derive(Deserialize)]
// pub struct TokenRegistry {
//     pub mainnet: HashMap<String, TokenInfo>,
//     pub testnet: HashMap<String, TokenInfo>,
// }

// pub fn load_token_registry() -> TokenRegistry {
//     let data = std::fs::read_to_string("tokens.json").unwrap_or_default();
//     serde_json::from_str(&data).unwrap_or(TokenRegistry {
//         mainnet: HashMap::new(),
//         testnet: HashMap::new(),
//     })
// }

// // pub fn get_network_name() -> &'static str {
// //     match std::env::var("CHAIN_ID").as_deref() {
// //         Ok("30") => "mainnet",
// //         Ok("31") => "testnet",
// //         _ => "testnet", // default
// //     }
// // }
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct TokenInfo {
    pub address: String,
    pub decimals: u8,
}

#[derive(Deserialize, Debug, Default)]
pub struct TokenRegistry {
    pub mainnet: HashMap<String, TokenInfo>,
    pub testnet: HashMap<String, TokenInfo>,
}

pub fn load_token_registry() -> TokenRegistry {
    let data = fs::read_to_string("tokens.json").unwrap_or_default();
    serde_json::from_str(&data).unwrap_or(TokenRegistry {
        mainnet: HashMap::new(),
        testnet: HashMap::new(),
    })
}

pub fn get_network_name(network: &str) -> &str {
    match network.to_lowercase().as_str() {
        "mainnet" => "mainnet",
        "testnet" => "testnet",
        _ => "testnet", // Default to testnet for unknown networks
    }
}