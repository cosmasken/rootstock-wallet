use dotenv::dotenv;
use ethers::providers::{Http, Middleware, Provider};
use std::convert::TryFrom;
use std::env;

/// Creates a new provider for the specified network.
/// If `custom_rpc` is provided, it overrides the default RPC URL.
pub fn get_provider(network: &str, custom_rpc: Option<&str>) -> Provider<Http> {
    dotenv().ok();
    let url = match custom_rpc {
        Some(url) => url.to_string(),
        None => {
            let rpc_url = env::var("RPC_URL").expect("RPC_URL not set");
            let api_key = env::var("RPC_API_KEY").expect("API_KEY not set");
            let network_suffix = match network.to_lowercase().as_str() {
                "mainnet" => "",
                "testnet" => "/testnet",
                _ => panic!(
                    "Unsupported network: {}. Use 'mainnet' or 'testnet'",
                    network
                ),
            };
            format!(
                "{}{}{}",
                rpc_url.trim_end_matches('/'),
                network_suffix,
                api_key
            )
        }
    };
    Provider::<Http>::try_from(url).expect("Failed to connect to provider")
}

/// Validates network connectivity by fetching the chain ID.
pub async fn validate_network(provider: &Provider<Http>) -> Result<(), Box<dyn std::error::Error>> {
    provider.get_chainid().await?;
    Ok(())
}

pub fn get_chain_id(network: &str) -> u64 {
    match network.to_lowercase().as_str() {
        "mainnet" => 30, // Rootstock Mainnet chain ID
        "testnet" => 31, // Rootstock Testnet chain ID
        _ => panic!(
            "Unsupported network: {}. Use 'mainnet' or 'testnet'",
            network
        ),
    }
}
