use colored::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// Structs for JSON-RPC request
#[derive(Serialize)]
pub struct JsonRpcRequest {
    #[serde(rename = "jsonrpc")]
    json_rpc: String,
    id: u32,
    method: String,
    params: Vec<AssetTransferParams>,
}

#[derive(Serialize)]
pub struct AssetTransferParams {
    #[serde(rename = "fromBlock")]
    from_block: String,
    #[serde(rename = "fromAddress")]
    from_address: String,
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
    value: Option<String>,
    hash: String,
    metadata: TransferMetadata,
}

#[derive(Deserialize)]
pub struct TransferMetadata {
    #[serde(rename = "blockTimestamp")]
    block_timestamp: String,
}

// Fetch transaction history using Alchemy API
pub async fn history_command(
    testnet: bool,
    number: Option<String>,
    wallet_address: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    let api_key = std::env::var("ALCHEMY_API_KEY")
        .map_err(|_| "ALCHEMY_API_KEY environment variable not set")?;
    let base_url = std::env::var("ALCHEMY_RPC_URL")
        .map_err(|_| "ALCHEMY_RPC_URL environment variable not set")?;

    println!(
        "{}",
        format!(
            "🔍 Fetching transaction history on Rootstock {} for {} ...",
            if testnet { "Testnet" } else { "Mainnet" },
            wallet_address
        )
            .blue()
    );

    // Construct JSON-RPC request
    let request = JsonRpcRequest {
        json_rpc: "2.0".to_string(),
        id: 0,
        method: "alchemy_getAssetTransfers".to_string(),
        params: vec![AssetTransferParams {
            from_block: "0x0".to_string(),
            from_address: wallet_address.to_string(),
            category: vec![
                "external".to_string(),
                "erc20".to_string(),
                "erc721".to_string(),
                "erc1155".to_string(),
            ],
            with_metadata: true,
        }],
    };

    // Construct full URL
    let url = format!("{}{}", base_url, api_key);

    // Send HTTP request
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !response.status().is_success() {
        println!(
            "{}",
            format!("❌ API request failed with status: {}", response.status()).red()
        );
        return Ok(());
    }

    let result: JsonRpcResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Handle Alchemy-specific errors
    if let Some(error) = result.error {
        println!("{}", format!("❌ Error from Alchemy: {}", error.message).red());
        return Ok(());
    }

    let transfers = match result.result {
        Some(result) => result.transfers,
        None => {
            println!("{}", "⚠️ No transactions found.".yellow());
            return Ok(());
        }
    };

    // Limit the number of transfers if specified
    let transfers = if let Some(num) = number {
        let limit = usize::from_str(&num).map_err(|e| format!("Invalid number: {}", e))?;
        transfers.into_iter().take(limit).collect()
    } else {
        transfers
    };

    // Display transfers
    for transfer in transfers {
        println!("{}", "✅ Transfer:".green());
        println!("   From: {}", transfer.from);
        println!("   To: {}", transfer.to);
        println!("   Token: {}", transfer.asset.unwrap_or("N/A".to_string()));
        println!("   Value: {}", transfer.value.unwrap_or("N/A".to_string()));
        println!("   Tx Hash: {}", transfer.hash);
        println!("   Time: {}", transfer.metadata.block_timestamp);
    }

    Ok(())
}