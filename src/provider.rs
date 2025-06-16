use dotenv::dotenv;
use ethers::providers::{Http, Provider};
use std::convert::TryFrom;
use std::env;

pub fn get_provider(network: &str) -> Provider<Http> {
    dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL environment variable not set");
    let api_key = env::var("RPC_API_KEY").expect("API_KEY environment variable not set");
    
    // Determine the correct network suffix
    let network_suffix = match network.to_lowercase().as_str() {
        "mainnet" => "",
        "testnet" => "/testnet",
        _ => panic!("Unsupported network: {}. Use 'mainnet' or 'testnet'", network),
    };
    
    // Ensure no trailing slash in rpc_url
    let rpc_url = rpc_url.trim_end_matches('/');
    let url = format!("{}{}{}", rpc_url, network_suffix, api_key);
    
    Provider::<Http>::try_from(url).expect("Failed to connect to provider")
}

pub fn get_chain_id(network: &str) -> u64 {
    match network.to_lowercase().as_str() {
        "mainnet" => 30, // Rootstock Mainnet chain ID
        "testnet" => 31, // Rootstock Testnet chain ID
        _ => panic!("Unsupported network: {}. Use 'mainnet' or 'testnet'", network),
    }
}
