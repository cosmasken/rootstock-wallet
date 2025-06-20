// use crate::utils::{config::Config, eth::EthClient};
// use crate::types::network::Network;
use crate::utils::config::Config;
use anyhow::Result;
use colored::Colorize;
use ethers::types::Address;
use std::str::FromStr;

use crate::types::network::Network;
use crate::utils::eth::EthClient;
pub struct Helper;

impl Helper {
    /// Initialize the Ethereum client with the specified network       
    pub async fn init_eth_client(network: &str) -> Result<(Config, EthClient)> {
        let mut config = Config::load()?;

        // Update network config based on command line argument
        let network = match network.to_lowercase().as_str() {
            "mainnet" => Network::Mainnet,
            "testnet" => Network::Testnet,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid network specified. Use 'mainnet' or 'testnet'"
                ));
            }
        };
        config.network.rpc_url = network.get_config().rpc_url;

        let eth_client = EthClient::new(&config).await?;
        Ok((config, eth_client))
    }

    /// Validate and parse an Ethereum address
    pub fn _validate_address(address: &str) -> Result<Address> {
        Address::from_str(address)
            .map_err(|_| anyhow::anyhow!("Invalid address format. Expected 0x-prefixed hex string"))
    }
     /// Format a network name with colored output
    pub fn format_network(network: &str) -> String {
        match network.to_lowercase().as_str() {
            "mainnet" => format!("{}", "Mainnet".yellow().bold()),
            "testnet" => format!("{}", "Testnet".blue().bold()),
            _ => network.to_string(),
        }
    }

    /// Format an address with colored output
    pub fn format_address(address: &Address) -> String {
        format!("{}{}", "0x".green(), address.to_string()[2..].green())
    }
    /// Format a balance in either wei or tokens
    pub fn format_balance(balance: u128, as_tokens: bool) -> Result<String> {
        if as_tokens {
            Ok(format!("{} RBTC", ethers::utils::format_units(balance, 18)?))
        } else {
            Ok(format!("{} wei", balance))
        }
    }

    /// Format a transaction status
    pub fn format_tx_status(status: Option<u64>) -> String {
        match status {
            Some(1) => format!("{}", "Success".green().bold()),
            Some(0) => format!("{}", "Failed".red().bold()),
            None => format!("{}", "Pending".yellow().bold()),
            _ => format!("{}", "Unknown".yellow().bold()),
        }
    }
}
