use colored::*;
use serde::{Deserialize, Serialize, Deserializer};
use std::collections::HashSet;
use std::str::FromStr;

// Custom deserialization for value field (handles both String and f64)
fn deserialize_value<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        String(String),
        Float(f64),
    }

    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(Value::String(s)) => Some(s),
        Some(Value::Float(f)) => Some(f.to_string()),
        None => None,
    })
}

// Structs for JSON-RPC request
#[derive(Serialize)]
pub struct JsonRpcRequest {
    json_rpc: String,
    id: u32,
    method: String,
    params: Vec<AssetTransferParams>,
}

#[derive(Serialize)]
pub struct AssetTransferParams {
    #[serde(rename = "fromBlock")]
    from_block: String,
    #[serde(rename = "fromAddress", skip_serializing_if = "Option::is_none")]
    from_address: Option<String>,
    #[serde(rename = "toAddress", skip_serializing_if = "Option::is_none")]
    to_address: Option<String>,
    category: Vec<String>,
    #[serde(rename = "withMetadata")]
    with_metadata: bool,
}

// Structs for JSON-RPC response
#[derive(Deserialize)]
pub struct JsonRpcResponse {
    result: Option<AssetTransfersResult>,
    error: Option<JsonRpcError>,
}

#[derive(Deserialize)]
pub struct JsonRpcError {
    message: String,
}

#[derive(Deserialize)]
pub struct AssetTransfersResult {
    transfers: Vec<Transfer>,
}

#[derive(Deserialize)]
pub struct Transfer {
    from: String,
    to: String,
    asset: Option<String>,
    #[serde(deserialize_with = "deserialize_value")]
    value: Option<String>,
    hash: String,
    metadata: TransferMetadata,
}

#[derive(Deserialize)]
pub struct TransferMetadata {
    #[serde(rename = "blockTimestamp")]
    block_timestamp: String,
}

pub async fn history_command(
    testnet: bool,
    address: &str,
    limit: Option<u32>,
    wallet_address: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let api_key = std::env::var("ALCHEMY_API_KEY")
        .map_err(|_| "ALCHEMY_API_KEY environment variable not set")?;
    let base_url = std::env::var("ALCHEMY_RPC_URL")
        .map_err(|_| "ALCHEMY_RPC_URL environment variable not set")?;

    let url = format!(
        "{}{}",
        base_url,
        if testnet { "/testnet" } else { "" }
    );

    let client = reqwest::Client::new();
    let params = AssetTransferParams {
        from_block: "0x0".to_string(),
        from_address: Some(address.to_string()),
        to_address: None,
        category: vec!["external".to_string(), "erc20".to_string()],
        with_metadata: true,
    };

    let request = JsonRpcRequest {
        json_rpc: "2.0".to_string(),
        id: 1,
        method: "alchemy_getAssetTransfers".to_string(),
        params: vec![params],
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?
        .json::<JsonRpcResponse>()
        .await?;

    if let Some(error) = response.error {
        return Err(error.message.into());
    }

    if let Some(result) = response.result {
        let transfers = result.transfers;
        let limit = limit.unwrap_or(10);
        let transfers = &transfers[0..transfers.len().min(limit as usize)];

        println!("{}", "Transaction History".bold().underline());
        println!("{}", "-".repeat(50).blue());

        for transfer in transfers {
            let direction = if transfer.from == wallet_address {
                "OUT".red()
            } else {
                "IN".green()
            };

            let asset = transfer.asset.as_ref().unwrap_or(&"RBTC".to_string()).clone();
            let value = match transfer.value {
                Some(ref v) => v,
                None => "0",
            };

            let timestamp = transfer.metadata.block_timestamp.clone();

            println!("{}", format!("Direction: {}", direction).bold());
            println!("From: {}", transfer.from);
            println!("To: {}", transfer.to);
            println!("Asset: {}", asset);
            println!("Value: {} {}", value, asset);
            println!("Timestamp: {}", timestamp);
            println!("Hash: {}", transfer.hash);
            println!("{}", "-".repeat(50).blue());
        }

        if transfers.len() < limit as usize {
            println!("{}", "No more transactions to show".yellow());
        }
    } else {
        println!("{}", "No transactions found".yellow());
    }

    Ok(())
}