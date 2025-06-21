use crate::types::transaction::RskTransaction;
use crate::types::transaction::TransactionStatus;
use crate::utils::config::Config;
use ethers::types::H256;
use ethers::{
    contract::abigen,
    prelude::*,
    providers::Provider,
    signers::LocalWallet,
    types::{BlockNumber, Bytes, TransactionReceipt, transaction::eip2718::TypedTransaction},
};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use crate::types::network::NetworkConfig;

abigen!(
    IERC20,
    r#"[
        function balanceOf(address account) external view returns (uint256)
        function transfer(address recipient, uint256 amount) external returns (bool)
        function decimals() external view returns (uint8)
        function symbol() external view returns (string)
    ]"#,
);

pub struct EthClient {
    provider: Arc<Provider<Http>>,
    wallet: Option<LocalWallet>,
    network: NetworkConfig,
}

impl EthClient {
    pub async fn new(config: &Config) -> Result<Self, anyhow::Error> {
        let provider = ethers::providers::Provider::<ethers::providers::Http>::try_from(&config.network.rpc_url)
            .map_err(|e| anyhow::anyhow!("Failed to connect to RPC: {}", e))?;

        let wallet = config
            .wallet
            .private_key
            .as_ref()
            .map(|key| {
                key.parse::<LocalWallet>()
                    .map(|w| w)
            })
            .transpose()?;

        Ok(Self {
            provider: Arc::new(provider),
            wallet,
            network: config.network.clone(),
        })
    }

    pub async fn get_balance(
        &self,
        address: &Address,
        token_address: &Option<Address>,
    ) -> Result<U256, anyhow::Error> {
        match token_address {
            Some(token_addr) => {
                let contract = IERC20::new(*token_addr, Arc::clone(&self.provider));
                contract
                    .balance_of(*address)
                    .call()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get token balance: {}", e))
            }
            None => self
                .provider
                .get_balance(*address, None)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get RBTC balance: {}", e)),
        }
    }

    pub async fn send_transaction(
        &self,
        to: Address,
        amount: U256,
        token_address: Option<Address>,
    ) -> Result<H256, anyhow::Error> {
        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No wallet configured"))?;

        let nonce = self
            .provider
            .get_transaction_count(wallet.address(), None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get nonce: {}", e))?;

        let gas_price = self
            .provider
            .get_gas_price()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get gas price: {}", e))?;

        // Check RBTC balance for gas fees
        let rbtc_balance = self
            .provider
            .get_balance(wallet.address(), None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get RBTC balance: {}", e))?;
        let estimated_gas_cost = gas_price * U256::from(100_000); // Conservative estimate
        if rbtc_balance < estimated_gas_cost {
            return Err(anyhow::anyhow!("Insufficient RBTC for gas fees"));
        }
        let chain_id = self.provider.get_chainid().await?.as_u64();

        match token_address {
            Some(token_addr) => {
                // Check token balance
                let contract = IERC20::new(token_addr, Arc::clone(&self.provider));
                let token_balance = contract
                    .balance_of(wallet.address())
                    .call()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get token balance: {}", e))?;
                if token_balance < amount {
                    return Err(anyhow::anyhow!("Insufficient token balance"));
                }

                // Encode the transfer function call data
                let data = contract
                    .transfer(to, amount)
                    .calldata()
                    .ok_or_else(|| anyhow::anyhow!("Failed to encode transfer calldata"))?;

                

                // Build the transaction manually
                let mut tx = TypedTransaction::Legacy(TransactionRequest {
                    to: Some(token_addr.into()),
                    from: Some(wallet.address()),
                    nonce: Some(nonce),
                    gas_price: Some(gas_price),
                    gas: None, // will set after estimation
                    value: Some(U256::zero()),
                    data: Some(data),
                    chain_id: Some(chain_id.into()),
                    ..Default::default()
                });

                // Estimate gas for the transaction
                let gas_estimate = self.provider.estimate_gas(&tx, None).await.map_err(|e| {
                    anyhow::anyhow!("Failed to estimate gas for token transfer: {}", e)
                })?;

                // Set the estimated gas
                tx.set_gas(gas_estimate);

                // Sign and send the transaction
                let signature = wallet
                    .sign_transaction(&tx)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to sign transaction: {}", e))?;

                let raw_tx = tx.rlp_signed(&signature);
                let pending_tx = self
                    .provider
                    .send_raw_transaction(raw_tx)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to send token transaction: {}", e))?;

                Ok(pending_tx.tx_hash())
            }
            None => {
                // let chain_id = self.provider.get_chainid().await?.as_u64();
                // Check RBTC balance for transfer
                if rbtc_balance < amount + estimated_gas_cost {
                    return Err(anyhow::anyhow!("Insufficient RBTC for transfer and gas"));
                }

                let tx = TransactionRequest::new()
                    .to(to)
                    .value(amount)
                    .from(wallet.address())
                    .nonce(nonce)
                    .gas_price(gas_price)
                    .chain_id(chain_id);

                let gas_estimate = self
                    .provider
                    .estimate_gas(&tx.clone().into(), None)
                    .await
                    .map_err(|e| {
                        anyhow::anyhow!("Failed to estimate gas for RBTC transfer: {}", e)
                    })?;

                let typed_tx: TypedTransaction = tx.gas(gas_estimate).into();
                let signature = wallet
                    .sign_transaction(&typed_tx)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to sign transaction: {}", e))?;

                let raw_tx = typed_tx.rlp_signed(&signature);
                let pending_tx = self
                    .provider
                    .send_raw_transaction(raw_tx)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to send RBTC transaction: {}", e))?;

                Ok(pending_tx.tx_hash())
            }
        }
    }
    pub async fn get_token_info(
        &self,
        token_address: Address,
    ) -> Result<(u8, String), anyhow::Error> {
        let contract = IERC20::new(token_address, Arc::clone(&self.provider));

        let decimals = contract.decimals().call().await?;
        let symbol = contract.symbol().call().await?;

        Ok((decimals, symbol))
    }

    pub async fn estimate_gas(
        &self,
        to: Address,
        amount: U256,
        token_address: Option<Address>,
    ) -> Result<U256, anyhow::Error> {
        match token_address {
            Some(token_addr) => {
                let contract = IERC20::new(token_addr, Arc::clone(&self.provider));
                let tx = contract.transfer(to, amount);
                tx.estimate_gas().await.map_err(|e| e.into())
            }
            None => {
                let tx = TransactionRequest::new().to(to).value(amount);
                self.provider
                    .estimate_gas(&tx.into(), None)
                    .await
                    .map_err(|e| e.into())
            }
        }
    }

    pub async fn get_transaction_history(
        &self,
        address: &Address,
        limit: u32,
        _status: Option<&str>,
        _token: Option<&str>,
        _from_date: Option<&str>,
        _to_date: Option<&str>,
    ) -> Result<Vec<RskTransaction>, anyhow::Error> {
        // Use Alchemy's enhanced "alchemy_getAssetTransfers" to retrieve transfers.
        let params = serde_json::json!([{
            "fromBlock": "0x0",
            "toBlock": "latest",
            "fromAddress": format!("{:#x}", address),
            "category": ["external", "erc20"],
            "withMetadata": true,
            "excludeZeroValue": false,
            "maxCount": format!("0x{:x}", limit),
        }]);

        // Try Alchemy-specific method first; if not supported, fall back to generic eth_getLogs approach.
        let response_res: Result<Value, ethers::providers::ProviderError> = self
            .provider
            .request("alchemy_getAssetTransfers", params.clone())
            .await;

        if let Ok(response) = response_res {
            let transfers_val = response
                .get("result")
                .and_then(|r| r.get("transfers"))
                .or_else(|| response.get("transfers"));
            let transfers = transfers_val
                .and_then(|t| t.as_array())
                .ok_or_else(|| anyhow::anyhow!("Unexpected response structure from Alchemy: {}", response))?;

            let mut transactions = Vec::new();
            for tr in transfers {
                let hash_str = tr["hash"].as_str().unwrap_or_default();
                let from_str = tr["from"].as_str().unwrap_or_default();
                let to_opt = tr["to"].as_str();
                let value_str = tr["rawContract"]["value"].as_str().unwrap_or("0");
                let block_num_hex = tr["blockNum"].as_str().unwrap_or("0x0");
                let block_num = u64::from_str_radix(block_num_hex.trim_start_matches("0x"), 16)?;

                let block = self.provider.get_block(block_num).await?;
                let ts_secs = block.as_ref().map(|b| b.timestamp.as_u64()).unwrap_or(0);

                let token_addr_opt = tr["rawContract"]["address"].as_str();

                transactions.push(RskTransaction {
                    hash: H256::from_str(hash_str)?,
                    from: Address::from_str(from_str)?,
                    to: to_opt.and_then(|s| Address::from_str(s).ok()),
                    value: if value_str.starts_with("0x") {
                        U256::from_str_radix(value_str.trim_start_matches("0x"), 16).unwrap_or_else(|_| U256::zero())
                    } else {
                        U256::from_dec_str(value_str).unwrap_or_else(|_| U256::zero())
                    },
                    gas_price: U256::zero(),
                    gas: U256::zero(),
                    nonce: U256::zero(),
                    timestamp: SystemTime::UNIX_EPOCH + Duration::from_secs(ts_secs),
                    status: TransactionStatus::Unknown,
                    token_address: token_addr_opt.and_then(|s| Address::from_str(s).ok()),
                });
            }

            return Ok(transactions);
        }

        // ---- Fallback ----
        // fallback to generic getLogs similar to previous implementation
        let latest_block = self.provider.get_block_number().await?;
        let scan_range: u64 = 10_000;
        let from_block_num = latest_block.as_u64().saturating_sub(scan_range);

        let filter = Filter::new()
            .address(*address)
            .from_block(BlockNumber::Number(from_block_num.into()))
            .to_block(BlockNumber::Number(latest_block));

        let logs = self.provider.get_logs(&filter).await?;
        let mut transactions = Vec::new();
        for log in logs.into_iter().take(limit as usize) {
            if log.topics.len() >= 3 {
                let from = Address::from_slice(&log.topics[1][12..32]);
                let to = Address::from_slice(&log.topics[2][12..32]);
                let value = U256::from_big_endian(&log.data);
                let token_address = Some(log.address);
                let block = self.provider.get_block(log.block_number.unwrap()).await?;
                let timestamp = block
                    .ok_or_else(|| anyhow::anyhow!("Block not found"))?
                    .timestamp;

                transactions.push(RskTransaction {
                    hash: log.transaction_hash.unwrap(),
                    from,
                    to: Some(to),
                    value,
                    gas_price: U256::zero(),
                    gas: U256::zero(),
                    nonce: U256::zero(),
                    timestamp: SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp.as_u64()),
                    status: TransactionStatus::Unknown,
                    token_address,
                });
            }
        }

        Ok(transactions)
    }
}
