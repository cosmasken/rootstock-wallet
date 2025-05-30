use dotenv::dotenv;
use ethers::providers::{Http, Provider};
use std::convert::TryFrom;
use std::env;

pub fn get_provider() -> Provider<Http> {
    dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL environment variable not set");
    let api_key = env::var("API_KEY").expect("API_KEY environment variable not set");
    // Ensure no trailing slash in rpc_url
    let rpc_url = rpc_url.trim_end_matches('/');
    let url = format!("{}/{}", rpc_url, api_key);
    Provider::<Http>::try_from(url).expect("Failed to connect to provider")
}
