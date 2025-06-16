// use crate::provider::get_provider;
// use colored::*;
// use dotenv::dotenv;
// use ethers::types::{Address, BlockNumber, Filter, H256, U256};
// use ethers_providers::Middleware;
// use serde::{Deserialize, Serialize, Deserializer};


// // Custom deserialization for value field
// fn deserialize_value<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     #[derive(Deserialize)]
//     #[serde(untagged)]
//     enum Value {
//         String(String),
//         Float(f64),
//     }

//     let value = Option::<Value>::deserialize(deserializer)?;
//     Ok(match value {
//         Some(Value::String(s)) => Some(s),
//         Some(Value::Float(f)) => Some(f.to_string()),
//         None => None,
//     })
// }

// #[derive(Serialize)]
// pub struct JsonRpcRequest {
//     jsonrpc: String,
//     id: u32,
//     method: String,
//     params: Vec<AssetTransferParams>,
// }

// #[derive(Serialize)]
// pub struct AssetTransferParams {
//     #[serde(rename = "fromBlock")]
//     from_block: String,
//     #[serde(rename = "fromAddress", skip_serializing_if = "Option::is_none")]
//     from_address: Option<String>,
//     #[serde(rename = "toAddress", skip_serializing_if = "Option::is_none")]
//     to_address: Option<String>,
//     category: Vec<String>,
//     #[serde(rename = "withMetadata")]
//     with_metadata: bool,
// }

// #[derive(Deserialize)]
// pub struct JsonRpcResponse {
//     result: Option<AssetTransfersResult>,
//     error: Option<JsonRpcError>,
// }

// #[derive(Deserialize)]
// pub struct JsonRpcError {
//     message: String,
// }

// #[derive(Deserialize)]
// pub struct AssetTransfersResult {
//     transfers: Vec<Transfer>,
// }

// #[derive(Deserialize)]
// pub struct Transfer {
//     from: String,
//     to: String,
//     asset: Option<String>,
//     #[serde(deserialize_with = "deserialize_value")]
//     value: Option<String>,
//     hash: String,
//     metadata: TransferMetadata,
// }

// #[derive(Deserialize)]
// pub struct TransferMetadata {
//     #[serde(rename = "blockTimestamp")]
//     block_timestamp: String,
// }

// pub async fn history_command(
//     network: &str,
//     address: Option<&str>,
//     limit: Option<u32>,
//     direction: &str,
//     wallet_address: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     dotenv().ok();
//     let limit = limit.unwrap_or(10);
    
//     // Try Alchemy API first
//     match fetch_history_alchemy(network, address, limit, direction, wallet_address).await {
//         Ok(transfers) => display_transfers(&transfers, wallet_address, network, limit).await,
//         Err(e) => {
//             log::warn!("Alchemy API failed: {}. Falling back to ethers provider.", e);
//             let transfers = fetch_history_fallback(address.unwrap_or(wallet_address), limit, network).await?;
//             display_transfers(&transfers, wallet_address, network, limit).await
//         }
//     }
// }

// async fn fetch_history_alchemy(
//     network: &str,
//     address: Option<&str>,
//     limit: u32,
//     direction: &str,
//     wallet_address: &str,
// ) -> Result<Vec<Transfer>, Box<dyn std::error::Error>> {
//     let api_key = std::env::var("ALCHEMY_API_KEY")
//         .map_err(|_| "ALCHEMY_API_KEY not set")?;
//     let base_url = std::env::var("ALCHEMY_RPC_URL")
//         .map_err(|_| "ALCHEMY_RPC_URL not set")?;

//     let url = format!("{}{}", base_url, if network == "testnet" { "/testnet" } else { "" });
//     let client = reqwest::Client::new();

//     let address = address.unwrap_or(wallet_address);
//     let params = AssetTransferParams {
//         from_block: "0x0".to_string(),
//         from_address: if direction == "outgoing" || direction == "all" {
//             Some(address.to_string())
//         } else {
//             None
//         },
//         to_address: if direction == "incoming" || direction == "all" {
//             Some(address.to_string())
//         } else {
//             None
//         },
//         category: vec!["external".to_string(), "erc20".to_string()],
//         with_metadata: true,
//     };

//     let request = JsonRpcRequest {
//         jsonrpc: "2.0".to_string(),
//         id: 1,
//         method: "alchemy_getAssetTransfers".to_string(),
//         params: vec![params],
//     };

//     let response = client
//         .post(&url)
//         .json(&request)
//         .send()
//         .await?
//         .json::<JsonRpcResponse>()
//         .await?;

//     if let Some(error) = response.error {
//         return Err(error.message.into());
//     }

//     Ok(response
//         .result
//         .map(|r| r.transfers)
//         .unwrap_or_default()
//         .into_iter()
//         .take(limit as usize)
//         .collect())
// }

// async fn fetch_history_fallback(
//     address: &str,
//     limit: u32,
//     network: &str,
// ) -> Result<Vec<Transfer>, Box<dyn std::error::Error>> {
//     let provider = get_provider(network, None);
//     let address: Address = address.parse()?;
//     let filter = Filter::new()
//         .address(address)
//         .from_block(BlockNumber::Earliest)
//         .to_block(BlockNumber::Latest);

//     let logs = provider.get_logs(&filter).await?;
//     let mut transfers = Vec::new();

//     for log in logs.into_iter().take(limit as usize) {
//         if log.topics.len() >= 3 {
//             // Assume ERC-20 Transfer event
//             let from = format!("0x{}", hex::encode(&log.topics[1][12..]));
//             let to = format!("0x{}", hex::encode(&log.topics[2][12..]));
//             let value = format!("{}", U256::from_big_endian(&log.data));
//             let asset = Some(format!("0x{}", hex::encode(log.address)));
//             let hash = format!("0x{}", hex::encode(log.transaction_hash.unwrap()));
//             let block = provider.get_block(log.block_number.unwrap()).await?;

//             transfers.push(Transfer {
//                 from,
//                 to,
//                 asset,
//                 value: Some(value),
//                 hash,
//                 metadata: TransferMetadata {
//                     block_timestamp: block.as_ref().map(|b| b.timestamp.to_string()).unwrap_or_default(),
//                 },
//             });
//         }
//     }

//     Ok(transfers)
// }

// async fn display_transfers(
//     transfers: &Vec<Transfer>,
//     wallet_address: &str,
//     network: &str,
//     limit: u32,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = get_provider(network, None);

//     println!("{}", "Transaction History".bold().underline());
//     println!("{}", "-".repeat(50).blue());

//     for transfer in transfers {
//         let direction = if transfer.from.to_lowercase() == wallet_address.to_lowercase() {
//             "OUT".red()
//         } else {
//             "IN".green()
//         };

//         let asset = transfer.asset.as_ref().unwrap_or(&"RBTC".to_string()).clone();
//         let value = match transfer.value {
//             Some(ref v) => v,
//             None => "0",
//         };

//         let tx_hash: H256 = transfer.hash.parse()?;
//         let receipt = provider.get_transaction_receipt(tx_hash).await?;

//         println!("{}", format!("Direction: {}", direction).bold());
//         println!("From: {}", transfer.from);
//         println!("To: {}", transfer.to);
//         println!("Asset: {}", asset);
//         println!("Value: {} {}", value, asset);
//         println!("Timestamp: {}", transfer.metadata.block_timestamp);
//         println!("Hash: {}", transfer.hash);
//         if let Some(receipt) = receipt {
//             println!("Block Number: {}", receipt.block_number.unwrap_or_default());
//             println!("Gas Used: {}", receipt.gas_used.unwrap_or_default());
//             if let Some(gas_price) = receipt.effective_gas_price {
//                 println!("Gas Price: {} wei", gas_price);
//             }
//         }
//         println!("{}", "-".repeat(50).blue());
//     }

//     if transfers.len() < limit as usize {
//         println!("{}", "No more transactions to show".yellow());
//     }

//     Ok(())
// }

use crate::provider::get_provider;
use colored::*;
use dotenv::dotenv;
use ethers::types::{Address, BlockNumber, Filter, H256, U256};
use serde::{Deserialize, Serialize, Deserializer};
use ethers::middleware::Middleware;

// Custom deserialization for value field
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

#[derive(Serialize)]
pub struct JsonRpcRequest {
    jsonrpc: String,
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
    network: &str,
    address: Option<&str>,
    limit: Option<u32>,
    direction: &str,
    wallet_address: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let limit = limit.unwrap_or(10);
    
    // Try Alchemy API first
    match fetch_history_alchemy(network, address, limit, direction, wallet_address).await {
        Ok(transfers) => display_transfers(&transfers, wallet_address, network, limit).await,
        Err(e) => {
            log::warn!("Alchemy API failed: {}. Falling back to ethers provider.", e);
            let transfers = fetch_history_fallback(address.unwrap_or(wallet_address), limit, network).await?;
            display_transfers(&transfers, wallet_address, network, limit).await
        }
    }
}

async fn fetch_history_alchemy(
    network: &str,
    address: Option<&str>,
    limit: u32,
    direction: &str,
    wallet_address: &str,
) -> Result<Vec<Transfer>, Box<dyn std::error::Error>> {
    let api_key = std::env::var("ALCHEMY_API_KEY")
        .map_err(|_| "ALCHEMY_API_KEY not set")?;
    let base_url = std::env::var("ALCHEMY_RPC_URL")
        .map_err(|_| "ALCHEMY_RPC_URL not set")?;

    let url = format!(
        "{}{}/{}",
        base_url,
        if network == "testnet" { "/testnet" } else { "" },
        api_key
    );
    let client = reqwest::Client::new();

    let address = address.unwrap_or(wallet_address);
    let params = AssetTransferParams {
        from_block: "0x0".to_string(),
        from_address: if direction == "outgoing" || direction == "all" {
            Some(address.to_string())
        } else {
            None
        },
        to_address: if direction == "incoming" || direction == "all" {
            Some(address.to_string())
        } else {
            None
        },
        category: vec!["external".to_string(), "erc20".to_string()],
        with_metadata: true,
    };

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
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

    Ok(response
        .result
        .map(|r| r.transfers)
        .unwrap_or_default()
        .into_iter()
        .take(limit as usize)
        .collect())
}

async fn fetch_history_fallback(
    address: &str,
    limit: u32,
    network: &str,
) -> Result<Vec<Transfer>, Box<dyn std::error::Error>> {
    let provider = get_provider(network, None);
    let address: Address = address.parse()?;
    let filter = Filter::new()
        .address(address)
        .from_block(BlockNumber::Earliest)
        .to_block(BlockNumber::Latest);

    let logs = provider.get_logs(&filter).await?;
    let mut transfers = Vec::new();

    for log in logs.into_iter().take(limit as usize) {
        if log.topics.len() >= 3 {
            // Assume ERC-20 Transfer event
            let from = format!("0x{}", hex::encode(&log.topics[1][12..]));
            let to = format!("0x{}", hex::encode(&log.topics[2][12..]));
            let value = format!("{}", U256::from_big_endian(&log.data));
            let asset = Some(format!("0x{}", hex::encode(log.address)));
            let hash = format!("0x{}", hex::encode(log.transaction_hash.unwrap()));
            let block = provider.get_block(log.block_number.unwrap()).await?;

            transfers.push(Transfer {
                from,
                to,
                asset,
                value: Some(value),
                hash,
                metadata: TransferMetadata {
                    block_timestamp: block.unwrap().timestamp.to_string(),
                },
            });
        }
    }

    Ok(transfers)
}

async fn display_transfers(
    transfers: &Vec<Transfer>,
    wallet_address: &str,
    network: &str,
    limit: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider(network, None);

    println!("{}", "Transaction History".bold().underline());
    println!("{}", "-".repeat(50).blue());

    for transfer in transfers {
        let direction = if transfer.from.to_lowercase() == wallet_address.to_lowercase() {
            "OUT".red()
        } else {
            "IN".green()
        };

        let asset = transfer.asset.as_ref().unwrap_or(&"RBTC".to_string()).clone();
        let value = match transfer.value {
            Some(ref v) => v,
            None => "0",
        };

        let tx_hash: H256 = transfer.hash.parse()?;
        let receipt = provider.get_transaction_receipt(tx_hash).await?;

        println!("{}", format!("Direction: {}", direction).bold());
        println!("From: {}", transfer.from);
        println!("To: {}", transfer.to);
        println!("Asset: {}", asset);
        println!("Value: {} {}", value, asset);
        println!("Timestamp: {}", transfer.metadata.block_timestamp);
        println!("Hash: {}", transfer.hash);
        if let Some(receipt) = receipt {
            println!("Block Number: {}", receipt.block_number.unwrap_or_default());
            println!("Gas Used: {}", receipt.gas_used.unwrap_or_default());
            if let Some(gas_price) = receipt.effective_gas_price {
                println!("Gas Price: {} wei", gas_price);
            }
        }
        println!("{}", "-".repeat(50).blue());
    }

    if transfers.len() < limit as usize {
        println!("{}", "No more transactions to show".yellow());
    }

    Ok(())
}
