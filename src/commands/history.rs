use crate::types::contacts::Contact;
use crate::types::network::Network;
use crate::types::transaction::{RskTransaction, TransactionStatus};
use crate::types::wallet::WalletData;
use crate::utils::{constants, eth::EthClient, helper::Config, table::TableBuilder};
use anyhow::Result;
use chrono::TimeZone;
use clap::Parser;
use colored::Colorize;
use ethers::types::{Address, U256};
use std::fs;
use std::str::FromStr;

/// Show the transaction history for an address or the current wallet
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

    /// Alchemy API key (if not already saved)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Network to query (mainnet | testnet). Defaults to mainnet.
    #[arg(long, default_value = "mainnet")]
    pub network: String,
}

impl HistoryCommand {
    pub async fn execute(&self) -> Result<()> {
        // ---------------------------------------------------------
        // 1. Resolve RPC endpoint (Alchemy URL)
        // ---------------------------------------------------------
        // Use Helper to initialize client and config for the selected network
        let (mut config, eth_client) =
            crate::utils::helper::Helper::init_eth_client(&self.network).await?;

        // Highest priority: CLI flag  >  wallet.json  >  ENV var
        let wallet_file = constants::wallet_file_path();
        let mut stored_api_key: Option<String> = None;

        if wallet_file.exists() {
            let json = fs::read_to_string(&wallet_file)?;
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json) {
                stored_api_key = val["alchemyApiKey"].as_str().map(|s| s.to_string());

                // Persist CLI key if supplied and not yet saved
                if stored_api_key.is_none() && self.api_key.is_some() {
                    let mut val_mut = val;
                    val_mut["alchemyApiKey"] =
                        serde_json::Value::String(self.api_key.clone().unwrap());
                    fs::write(&wallet_file, serde_json::to_string_pretty(&val_mut)?)?;
                    stored_api_key = self.api_key.clone();
                    println!("{}", "Saved Alchemy API key ✅".green());
                }
            }
        }

        let final_api_key = self
            .api_key
            .clone()
            .or(stored_api_key)
            .or(std::env::var("ALCHEMY_API_KEY").ok())
            .ok_or_else(|| anyhow::anyhow!("Alchemy API key missing – supply --api-key once"))?;

        let is_testnet = self.network.to_lowercase() == "testnet";
        if self.network.to_lowercase() != "mainnet" && !is_testnet {
            anyhow::bail!("Invalid network: use 'mainnet' or 'testnet'");
        }
        let rpc_url = if is_testnet {
            format!(
                "https://rootstock-testnet.g.alchemy.com/v2/{}",
                final_api_key
            )
        } else {
            format!(
                "https://rootstock-mainnet.g.alchemy.com/v2/{}",
                final_api_key
            )
        };
        config.network.rpc_url = rpc_url;
        // ---------------------------------------------------------

        let eth_client = EthClient::new(&config, None).await?;

        // ---------------------------------------------------------
        // 2. Determine address (CLI or current wallet)
        // ---------------------------------------------------------
        let address = if let Some(addr) = &self.address {
            Address::from_str(addr).map_err(|_| {
                anyhow::anyhow!("Invalid address format. Expected 0x-prefixed hex string")
            })?
        } else {
            let wallet_file = constants::wallet_file_path();
            if !wallet_file.exists() {
                anyhow::bail!("No wallets found. Create or import a wallet first.");
            }
            let data = fs::read_to_string(&wallet_file)?;
            let wallet_data = serde_json::from_str::<WalletData>(&data)?;
            wallet_data
                .get_current_wallet()
                .ok_or_else(|| {
                    anyhow::anyhow!("No default wallet selected. Use `wallet switch` first.")
                })?
                .address
        };
        // ---------------------------------------------------------

        // 3. Fetch & display history
        let mut txs = eth_client
            .get_transaction_history(
                &address,
                self.limit,
                self.status.as_deref(),
                self.token.as_deref(),
                self.from.as_deref(),
                self.to.as_deref(),
            )
            .await?;

        // Apply direction filters relative to the queried address
        if self.incoming && self.outgoing {
            anyhow::bail!("Cannot use both --incoming and --outgoing at the same time");
        }
        if self.incoming {
            txs.retain(|tx| tx.to == Some(address));
        } else if self.outgoing {
            txs.retain(|tx| tx.from == address);
        }

        // --- existing formatting / table code (unchanged) ---
        if txs.is_empty() {
            println!("{}", "⚠️  No transactions found.".yellow());
            return Ok(());
        }

        // Sort
        match (self.sort_by.as_str(), self.sort_order.as_str()) {
            ("timestamp", "asc") => txs.sort_by_key(|t| t.timestamp),
            ("timestamp", _) => txs.sort_by_key(|t| std::cmp::Reverse(t.timestamp)),
            ("value", "asc") => txs.sort_by_key(|t| t.value),
            ("value", _) => txs.sort_by_key(|t| std::cmp::Reverse(t.value)),
            _ => {}
        }

        let mut table = TableBuilder::new();
        if self.detailed {
            table.add_header(&[
                "TX Hash",
                "From",
                "To",
                "Value",
                "Status",
                "Timestamp",
                "Gas Used",
                "Token",
            ]);
        } else {
            table.add_header(&["TX Hash", "From", "To", "Value", "Status", "Timestamp"]);
        }

        for tx in &txs {
            let status_disp = match tx.status {
                TransactionStatus::Success => "Success".green(),
                TransactionStatus::Failed => "Failed".red(),
                TransactionStatus::Pending => "Pending".yellow(),
                TransactionStatus::Unknown => "Unknown".yellow(),
            };
            let ts = chrono::Local.timestamp(
                tx.timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                0,
            );
            table.add_row(&[
                &format!("{}{}", "0x".green(), &tx.hash.to_string()[2..]),
                &format!("{}{}", "0x".green(), &tx.from.to_string()[2..]),
                &tx.to
                    .map(|a| format!("{}{}", "0x".green(), &a.to_string()[2..]))
                    .unwrap_or_else(|| "-".into()),
                &ethers::utils::format_units(tx.value, 18)?,
                &status_disp.to_string(),
                &ts.format("%Y-%m-%d %H:%M:%S").to_string(),
                &if self.detailed {
                    tx.gas.to_string()
                } else {
                    "".into()
                },
                &tx.token_address
                    .map(|a| format!("0x{}", &a.to_string()[2..]))
                    .unwrap_or_else(|| "-".into()),
            ]);
        }
        table.print();
        Ok(())
    }
}
