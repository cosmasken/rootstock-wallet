use async_trait::async_trait;
use ethers::types::H160;

use crate::utils::api::{ApiKeys, Network};

#[async_trait]
pub trait ApiKeyCommand {
    fn requires_api_key(&self) -> bool {
        true
    }

    fn network(&self) -> Network;

    async fn execute_with_api_key(&self, api_keys: &ApiKeys) -> anyhow::Result<()>;

    async fn execute(&self) -> anyhow::Result<()> {
        let api_keys = ApiKeys::load()?;
        self.execute_with_api_key(&api_keys).await
    }
}

pub fn parse_address(address: &str) -> anyhow::Result<H160> {
    let address = address.trim_start_matches("0x");
    H160::from_slice(&hex::decode(address)?).context("Invalid address format")
}

pub fn parse_tx_hash(tx_hash: &str) -> anyhow::Result<[u8; 32]> {
    let tx_hash = tx_hash.trim_start_matches("0x");
    let bytes = hex::decode(tx_hash)?;
    if bytes.len() != 32 {
        anyhow::bail!("Transaction hash must be 32 bytes");
    }
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}