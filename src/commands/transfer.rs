// use crate::utils::{config::Config, eth::EthClient, table::TableBuilder, shared::Shared};
// use crate::types::transaction::TransactionReceipt;
use anyhow::Result;
use clap::Parser;
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
        let (config, eth_client) = crate::utils::helper::Helper::init_eth_client(&self.network).await?;
        Ok(())
    }
}
