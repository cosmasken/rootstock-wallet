use anyhow::Context;
use async_trait::async_trait;
use clap::Parser;
use console::style;
use ethers::types::H256;
use serde_json::Value;

use crate::{
    commands::traits::ApiKeyCommand,
    utils::{
        api::{ApiKeys, Network},
        constants,
    },
};

/// Command to check transaction status
#[derive(Debug, Parser)]
pub struct TxCommand {
    /// Transaction hash to check
    #[arg(short, long)]
    pub tx_hash: String,

    /// Use testnet
    #[arg(long)]
    pub testnet: bool,

    /// Alchemy API key (optional, will use saved key if not provided)
    #[arg(long)]
    pub api_key: Option<String>,
}

impl TxCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        let api_keys = if let Some(key) = &self.api_key {
            let mut keys = ApiKeys::default();
            if self.testnet {
                keys.alchemy_testnet = Some(key.clone());
            } else {
                keys.alchemy_mainnet = Some(key.clone());
            }
            keys
        } else {
            ApiKeys::load()?
        };

        self.execute_with_api_key(&api_keys).await
    }
}

#[async_trait]
impl ApiKeyCommand for TxCommand {
    fn network(&self) -> Network {
        if self.testnet {
            Network::Testnet
        } else {
            Network::Mainnet
        }
    }

    async fn execute_with_api_key(&self, api_keys: &ApiKeys) -> anyhow::Result<()> {
        let client = ApiKeys::get_http_client();
        let network = self.network();
        let url = api_keys.get_alchemy_url(network)?;

        println!(
            "\n{}",
            style(format!("🔍 Checking transaction status on {}...", network))
                .bold()
                .cyan()
        );
        println!("{}", "=".repeat(60));

        // Get transaction receipt
        let receipt = self
            .get_transaction_receipt(&client, &url, &self.tx_hash)
            .await?;

        // Get transaction details
        let tx_details = self
            .get_transaction_details(&client, &url, &self.tx_hash)
            .await?;

        self.display_transaction_info(&tx_details, &receipt)?;

        Ok(())
    }
}

impl TxCommand {
    async fn get_transaction_receipt(
        &self,
        client: &reqwest::Client,
        url: &str,
        tx_hash: &str,
    ) -> anyhow::Result<Value> {
        let params = serde_json::json!([tx_hash]);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getTransactionReceipt",
            "params": params
        });

        let response = client
            .post(url)
            .json(&request)
            .send()
            .await?
            .json::<Value>()
            .await?;

        if let Some(error) = response.get("error") {
            anyhow::bail!("Alchemy API error: {}", error);
        }

        response["result"]
            .as_object()
            .cloned()
            .map(Value::Object)
            .context("Invalid transaction receipt response")
    }

    async fn get_transaction_details(
        &self,
        client: &reqwest::Client,
        url: &str,
        tx_hash: &str,
    ) -> anyhow::Result<Value> {
        let params = serde_json::json!([tx_hash]);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getTransactionByHash",
            "params": params
        });

        let response = client
            .post(url)
            .json(&request)
            .send()
            .await?
            .json::<Value>()
            .await?;

        if let Some(error) = response.get("error") {
            anyhow::bail!("Alchemy API error: {}", error);
        }

        response["result"]
            .as_object()
            .cloned()
            .map(Value::Object)
            .context("Invalid transaction details response")
    }

    fn display_transaction_info(&self, tx_details: &Value, receipt: &Value) -> anyhow::Result<()> {
        println!("\n{}", style("📄 Transaction Details").bold().cyan());
        println!(
            "{} {}",
            "  Hash:".dim(),
            style(tx_details["hash"].as_str().unwrap_or("N/A")).white()
        );
        println!(
            "{} {}",
            "  Block:".dim(),
            style(tx_details["blockNumber"].as_str().unwrap_or("N/A")).white()
        );
        println!(
            "{} {}",
            "  From:".dim(),
            style(tx_details["from"].as_str().unwrap_or("N/A")).white()
        );
        println!(
            "{} {}",
            "  To:".dim(),
            style(
                tx_details["to"]
                    .as_str()
                    .unwrap_or("Contract Creation")
            )
            .white()
        );
        println!(
            "{} {} RBTC",
            "  Value:".dim(),
            style(hex_to_rbtc(tx_details["value"].as_str().unwrap_or("0x0"))?).white()
        );
        println!(
            "{} {}",
            "  Gas Price:".dim(),
            style(hex_to_gwei(tx_details["gasPrice"].as_str().unwrap_or("0x0"))?).white()
        );
        println!(
            "{} {}",
            "  Gas Used:".dim(),
            style(hex_to_u64(&receipt["gasUsed"].to_string())?).white()
        );
        println!(
            "{} {}",
            "  Status:".dim(),
            if receipt["status"].as_str() == Some("0x1") {
                style("✅ Success").green()
            } else {
                style("❌ Failed").red()
            }
        );

        if let Some(events) = receipt["logs"].as_array() {
            if !events.is_empty() {
                println!("\n{}", style("📝 Events:").bold().cyan());
                for event in events {
                    println!(
                        "  - {}",
                        event["topics"][0].as_str().unwrap_or("Unknown")
                    );
                }
            }
        }

        let explorer_url = if self.testnet {
            format!(
                "https://explorer.testnet.rootstock.io/tx/{}",
                self.tx_hash.trim_start_matches("0x")
            )
        } else {
            format!(
                "https://explorer.rsk.co/tx/{}",
                self.tx_hash.trim_start_matches("0x")
            )
        };
        println!("\n🔗 View on Explorer: {}", style(explorer_url).blue().underlined());

        Ok(())
    }
}

fn hex_to_rbtc(hex: &str) -> anyhow::Result<f64> {
    let wei = u128::from_str_radix(hex.trim_start_matches("0x"), 16)?;
    Ok(wei as f64 / 1e18)
}

fn hex_to_gwei(hex: &str) -> anyhow::Result<f64> {
    let wei = u128::from_str_radix(hex.trim_start_matches("0x"), 16)?;
    Ok(wei as f64 / 1e9)
}

fn hex_to_u64(hex: &str) -> anyhow::Result<u64> {
    Ok(u64::from_str_radix(hex.trim_start_matches("0x"), 16)?)
}