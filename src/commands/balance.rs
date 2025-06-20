use anyhow::Result;
use clap::Parser;
// use colored::Colorize;
use crate::utils::helper::Helper;
use ethers::types::Address;
use std::str::FromStr;
use crate::utils::table::TableBuilder;


#[derive(Parser, Debug)]
pub struct BalanceCommand {
    /// Address to check balance for
    #[arg(short, long)]
    pub address: String,

    /// Network to use (mainnet/testnet)
    #[arg(short, long, default_value = "mainnet")]
    pub network: String,

    /// Token to get Balance
    #[arg(short, long)]
    pub tokens: Option<bool>,
}

impl BalanceCommand {
    pub async fn execute(&self) -> Result<()> {
        let (_config, eth_client) = Helper::init_eth_client(&self.network).await?;
        let address = Address::from_str(&self.address)
            .map_err(|_| anyhow::anyhow!("Invalid address format: {}", self.address))?;

        // Get balance
        let balance = eth_client.get_balance(&address).await?;
       // Format and display
        let mut table = TableBuilder::new();
        table.add_header(&["Address", "Network", "Balance"]);
        
        // let balance_str = Helper::format_balance(balance.as_u128(), self.tokens)?;
        let balance_str = Helper::format_balance(balance.as_u128(), self.tokens.unwrap_or(true))?;


        table.add_row(&[
            &Helper::format_address(&address),
            &Helper::format_network(&self.network),
            &balance_str,
        ]);

        table.print();
        Ok(())
    }
}
