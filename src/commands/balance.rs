use crate::types::wallet::{Wallet, WalletData};
use crate::utils::constants;
use crate::utils::helper::{Config, Helper};
use crate::utils::table::TableBuilder;
use anyhow::{Result, anyhow};
use clap::Parser;
use ethers::types::Address;
use std::fs;
use std::str::FromStr;

#[derive(Parser, Debug)]
pub struct BalanceCommand {
    /// Address to check balance for
    #[arg(long)]
    pub address: Option<String>,

    /// Network to use (mainnet/testnet)
    #[arg(long, default_value = "mainnet")]
    pub network: String,

    /// Optional Token to get Balance for
    #[arg(long)]
    pub token: Option<String>,
}

impl BalanceCommand {
    pub async fn execute(&self) -> Result<()> {
        let (_config, eth_client) = Helper::init_eth_client(&self.network).await?;

        // Get address - use default wallet if none provided
        let address = if let Some(addr) = &self.address {
            Address::from_str(addr).map_err(|_| anyhow!("Invalid address format: {}", addr))?
        } else {
            // Load wallet data to get default wallet
            let wallet_file = constants::wallet_file_path();
            if !wallet_file.exists() {
                return Err(anyhow!(
                    "No wallets found. Please create or import a wallet first."
                ));
            }

            let data = fs::read_to_string(&wallet_file)?;
            let wallet_data = serde_json::from_str::<WalletData>(&data)?;
            let default_wallet = wallet_data.get_current_wallet()
                .ok_or_else(|| anyhow!("No default wallet selected. Please use 'wallet switch' to select a default wallet."))?;

            default_wallet.address
        };

        let token_address_opt = if let Some(token) = &self.token {
            Some(
                Address::from_str(token)
                    .map_err(|_| anyhow!("Invalid token address format: {}", token))?,
            )
        } else {
            None
        };

        let balance = eth_client.get_balance(&address, &token_address_opt).await?;
        let balance_str = ethers::utils::format_units(balance, 18)
            .map_err(|e| anyhow!("Failed to format balance: {}", e))?;

        let mut table = TableBuilder::new();
        table.add_header(&["Address", "Network", "Balance"]);
        table.add_row(&[
            &Helper::format_address(&address),
            &Helper::format_network(&self.network),
            &balance_str,
        ]);

        table.print();
        Ok(())
    }
}
