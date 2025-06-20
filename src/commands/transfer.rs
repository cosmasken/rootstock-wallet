// use crate::utils::{config::Config, eth::EthClient, table::TableBuilder, shared::Shared};
// use crate::types::transaction::TransactionReceipt;
use clap::Parser;
use anyhow::Result;
use colored::Colorize;
use ethers::types::{Address, U256};
use std::str::FromStr;

use crate::utils::helper::Helper;

#[derive(Parser, Debug)]
pub struct TransferCommand {
    /// Address to send to
    #[arg(short, long, required = true)]
    pub address: String,

    /// Amount to send (in RBTC)
    #[arg(short, long, required = true)]
    pub value: f64,

    /// Token address (for ERC20 transfers)
    #[arg(short, long)]
    pub token: Option<String>,

    /// Network to use (mainnet/testnet)
    #[arg(short, long, default_value = "mainnet")]
    pub network: String,
}

impl TransferCommand {
    pub async fn execute(&self) -> Result<()> {
        let (config, eth_client) = Helper::init_eth_client(&self.network).await?;

        // Validate addresses
        // let to_address = Helper::validate_address(&self.address)?;

        // let token_address = self.token.as_ref().map(|token| {
        //     Helper::validate_address(token)
        // }).transpose()?;

        // // Convert value to wei
        // let value = U256::from(self.value as u64) * U256::from(10).pow(18.into());

        // // Get current balance
        // let current_balance = eth_client.get_balance(&config.wallet.private_key.as_ref().unwrap().parse()?).await?;
        // if current_balance < value {
        //     return Err(anyhow::anyhow!("Insufficient funds"));
        // }

        // println!(
        //     "{}: Sending {} RBTC to {}",
        //     "Info".yellow().bold(),
        //     self.value,
        //     Helper::format_address(&to_address)
        // );
        // // Send transaction
       // let tx_hash = eth_client.send_transaction(to_address, value, token_address).await?;

        // println!(
        //     "{}: Transaction sent successfully",
        //     "Success".green().bold()
        // );
        // println!(
        //     "{}: {}",
        //     "Transaction Hash".green().bold(),
        //     Helper::format_address(&tx_hash)
        // );

        // // Wait for transaction confirmation
        // if let Some(receipt) = eth_client.provider.get_transaction_receipt(tx_hash).await? {
        //     let status = Helper::format_tx_status(receipt.status);
        //     println!(
        //         "{}: Transaction confirmed with status: {}",
        //         "Info".yellow().bold(),
        //         status
        //     );
        // }

        Ok(())
    }
}