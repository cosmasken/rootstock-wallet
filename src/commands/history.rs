// use crate::utils::{config::Config, contacts::Contact, eth::EthClient, table::TableBuilder};
use anyhow::{Ok, Result};
use clap::Parser;
use colored::Colorize;
// use ethers::types::Address;
// use ethers::types::U256;
// use std::str::FromStr;

#[derive(Parser, Debug)]
pub struct HistoryCommand {
    /// Address to check transaction history for
    #[arg(short, long)]
    pub address: Option<String>,

    /// Contact name to check transaction history for
    #[arg(short, long)]
    pub contact: Option<String>,

    /// Number of transactions to show
    #[arg(short, long, default_value = "10")]
    pub limit: u32,

    /// Network to use (mainnet/testnet)
    // #[arg(short, long, default_value = "mainnet")]
    // pub network: String,

    /// Show detailed transaction information
    #[arg(short, long)]
    pub detailed: bool,

    /// Filter by transaction status (pending/success/failed)
    #[arg(short, long)]
    pub status: Option<String>,

    /// Filter by token address
    #[arg(short, long)]
    pub token: Option<String>,

    /// Start date for filtering (YYYY-MM-DD)
    #[arg(short, long)]
    pub from: Option<String>,

    /// End date for filtering (YYYY-MM-DD)
    #[arg(short, long)]
    pub to: Option<String>,

    /// Sort by field (timestamp, value, gas)
    #[arg(short, long, default_value = "timestamp")]
    pub sort_by: String,

    /// Sort order (asc/desc)
    #[arg(short, long, default_value = "desc")]
    pub sort_order: String,

    /// Show only incoming transactions
    #[arg(short, long)]
    pub incoming: bool,

    /// Show only outgoing transactions
    #[arg(short, long)]
    pub outgoing: bool,
}

impl HistoryCommand {
    pub async fn execute(&self) -> Result<()> {
        println!(
            "{}: Fetching transaction history...",
            "History".bold().green()
        );
        Ok(())
    }
}
