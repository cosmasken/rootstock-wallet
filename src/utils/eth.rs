use crate::types::network::NetworkConfig;
use crate::types::transaction::{RskTransaction, TransactionStatus};
use crate::types::wallet::WalletData;
use crate::utils::constants;
use crate::utils::helper::{Config, WalletConfig};
use anyhow::anyhow;
use ethers::types::{H256, U256};
use ethers::{
    contract::abigen,
    prelude::*,
    providers::Provider,
    signers::LocalWallet,
    types::{BlockNumber, TransactionReceipt, transaction::eip2718::TypedTransaction},
};
use indicatif::ProgressBar;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

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
    api_key: Option<String>,
}

impl EthClient {
    pub async fn new(config: &Config, cli_api_key: Option<String>) -> Result<Self, anyhow::Error> {
        // Load or update API key
        let wallet_file = constants::wallet_file_path();
        let mut wallet_data = if wallet_file.exists() {
            let data = fs::read_to_string(&wallet_file)?;
            serde_json::from_str::<WalletData>(&data)?
        } else {
            WalletData::new()
        };

        let api_key = if let Some(key) = cli_api_key {
            wallet_data.api_key = Some(key.clone());
            fs::write(&wallet_file, serde_json::to_string_pretty(&wallet_data)?)?;
            Some(key)
        } else {
            wallet_data.api_key.clone()
        };

        let provider = Provider::<Http>::try_from(&config.network.rpc_url)
            .map_err(|e| anyhow!("Failed to connect to RPC: {}", e))?;
        let wallet = config
            .wallet
            .private_key
            .as_ref()
            .map(|key| {
                key.parse::<LocalWallet>()
                    .map_err(|e| anyhow!("Invalid private key: {}", e))
            })
            .transpose()?;
        Ok(Self {
            provider: Arc::new(provider),
            wallet,
            network: config.network.clone(),
            api_key,
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
                    .map_err(|e| anyhow!("Failed to get token balance: {}", e))
            }
            None => self
                .provider
                .get_balance(*address, None)
                .await
                .map_err(|e| anyhow!("Failed to get RBTC balance: {}", e)),
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
            .ok_or_else(|| anyhow!("No wallet configured"))?;
        let nonce = self
            .provider
            .get_transaction_count(wallet.address(), None)
            .await
            .map_err(|e| anyhow!("Failed to get nonce: {}", e))?;
        let gas_price = self
            .provider
            .get_gas_price()
            .await
            .map_err(|e| anyhow!("Failed to get gas price: {}", e))?;
        let rbtc_balance = self
            .provider
            .get_balance(wallet.address(), None)
            .await
            .map_err(|e| anyhow!("Failed to get RBTC balance: {}", e))?;
        let estimated_gas_cost = gas_price * U256::from(100_000);
        if rbtc_balance < estimated_gas_cost {
            return Err(anyhow!("Insufficient RBTC for gas fees"));
        }
        let chain_id = self.provider.get_chainid().await?.as_u64();

        match token_address {
            Some(token_addr) => {
                let contract = IERC20::new(token_addr, Arc::clone(&self.provider));
                let token_balance = contract
                    .balance_of(wallet.address())
                    .call()
                    .await
                    .map_err(|e| anyhow!("Failed to get token balance: {}", e))?;
                if token_balance < amount {
                    return Err(anyhow!("Insufficient token balance"));
                }
                let data = contract
                    .transfer(to, amount)
                    .calldata()
                    .ok_or_else(|| anyhow!("Failed to encode transfer calldata"))?;
                let mut tx = TypedTransaction::Legacy(TransactionRequest {
                    to: Some(token_addr.into()),
                    from: Some(wallet.address()),
                    nonce: Some(nonce),
                    gas_price: Some(gas_price),
                    gas: None,
                    value: Some(U256::zero()),
                    data: Some(data),
                    chain_id: Some(chain_id.into()),
                    ..Default::default()
                });
                let gas_estimate = self
                    .provider
                    .estimate_gas(&tx, None)
                    .await
                    .map_err(|e| anyhow!("Failed to estimate gas for token transfer: {}", e))?;
                tx.set_gas(gas_estimate);
                let signature = wallet
                    .sign_transaction(&tx)
                    .await
                    .map_err(|e| anyhow!("Failed to sign transaction: {}", e))?;
                let raw_tx = tx.rlp_signed(&signature);
                let pending_tx = self
                    .provider
                    .send_raw_transaction(raw_tx)
                    .await
                    .map_err(|e| anyhow!("Failed to send token transaction: {}", e))?;
                Ok(pending_tx.tx_hash())
            }
            None => {
                if rbtc_balance < amount + estimated_gas_cost {
                    return Err(anyhow!("Insufficient RBTC for transfer and gas"));
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
                    .map_err(|e| anyhow!("Failed to estimate gas for RBTC transfer: {}", e))?;
                let typed_tx: TypedTransaction = tx.gas(gas_estimate).into();
                let signature = wallet
                    .sign_transaction(&typed_tx)
                    .await
                    .map_err(|e| anyhow!("Failed to sign transaction: {}", e))?;
                let raw_tx = typed_tx.rlp_signed(&signature);
                let pending_tx = self
                    .provider
                    .send_raw_transaction(raw_tx)
                    .await
                    .map_err(|e| anyhow!("Failed to send RBTC transaction: {}", e))?;
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
                tx.estimate_gas()
                    .await
                    .map_err(|e| anyhow!("Failed to estimate gas for token transfer: {}", e))
            }
            None => {
                let tx = TransactionRequest::new().to(to).value(amount);
                self.provider
                    .estimate_gas(&tx.into(), None)
                    .await
                    .map_err(|e| anyhow!("Failed to estimate gas for RBTC transfer: {}", e))
            }
        }
    }

    pub async fn get_transaction_history(
        &self,
        address: &Address,
        limit: u32,
        status: Option<&str>,
        token: Option<&str>,
        from_date: Option<&str>,
        to_date: Option<&str>,
    ) -> Result<Vec<RskTransaction>, anyhow::Error> {
        let mut transactions = Vec::new();

        // Try Alchemy API first
        let params = serde_json::json!([{
            "fromBlock": "0x0",
            "toBlock": "latest",
            "fromAddress": format!("{:#x}", address),
            "toAddress": format!("{:#x}", address),
            "category": ["external", "erc20"],
            "withMetadata": true,
            "excludeZeroValue": false,
            "maxCount": format!("0x{:x}", limit),
        }]);
        if let Ok(response) = self
            .provider
            .request::<_, Value>("alchemy_getAssetTransfers", params)
            .await
        {
            if let Some(transfers) = response
                .get("result")
                .and_then(|r| r.get("transfers"))
                .and_then(|t| t.as_array())
            {
                for tr in transfers {
                    let tx_hash = H256::from_str(tr["hash"].as_str().unwrap_or_default())?;
                    let receipt = self.provider.get_transaction_receipt(tx_hash).await?;
                    let tx_status = receipt
                        .as_ref()
                        .map(|r| {
                            if r.status.map_or(false, |s| s.as_u64() == 1) {
                                TransactionStatus::Success
                            } else {
                                TransactionStatus::Failed
                            }
                        })
                        .unwrap_or(TransactionStatus::Pending);

                    // Apply status filter
                    if let Some(status_filter) = status {
                        let status_str = match tx_status {
                            TransactionStatus::Success => "success",
                            TransactionStatus::Failed => "failed",
                            TransactionStatus::Pending => "pending",
                            _ => "unknown",
                        };
                        if status_str != status_filter.to_lowercase() {
                            continue;
                        }
                    }

                    let token_addr_opt = tr["rawContract"]["address"].as_str();
                    // Apply token filter
                    if let Some(token_filter) = token {
                        if token_addr_opt.map_or(true, |addr| {
                            addr.to_lowercase() != token_filter.to_lowercase()
                        }) {
                            continue;
                        }
                    }

                    let block_num = u64::from_str_radix(
                        tr["blockNum"]
                            .as_str()
                            .unwrap_or("0x0")
                            .trim_start_matches("0x"),
                        16,
                    )?;
                    let block = self.provider.get_block(block_num).await?;
                    let timestamp_secs = block.as_ref().map(|b| b.timestamp.as_u64()).unwrap_or(0);
                    let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp_secs);

                    // Apply date filters
                    if let Some(from) = from_date {
                        let from_time = chrono::DateTime::parse_from_rfc3339(from)
                            .map_err(|e| anyhow!("Invalid from_date: {}", e))?;
                        if timestamp
                            < SystemTime::UNIX_EPOCH
                                + Duration::from_secs(from_time.timestamp() as u64)
                        {
                            continue;
                        }
                    }
                    if let Some(to) = to_date {
                        let to_time = chrono::DateTime::parse_from_rfc3339(to)
                            .map_err(|e| anyhow!("Invalid to_date: {}", e))?;
                        if timestamp
                            > SystemTime::UNIX_EPOCH
                                + Duration::from_secs(to_time.timestamp() as u64)
                        {
                            continue;
                        }
                    }

                    transactions.push(RskTransaction {
                        hash: tx_hash,
                        from: Address::from_str(tr["from"].as_str().unwrap_or_default())?,
                        to: tr["to"].as_str().and_then(|s| Address::from_str(s).ok()),
                        value: U256::from_str_radix(
                            tr["rawContract"]["value"]
                                .as_str()
                                .unwrap_or("0")
                                .trim_start_matches("0x"),
                            16,
                        )?,
                        gas_price: receipt
                            .as_ref()
                            .and_then(|r| r.effective_gas_price)
                            .unwrap_or(U256::zero()),
                        gas: receipt
                            .as_ref()
                            .and_then(|r| r.gas_used)
                            .unwrap_or(U256::zero()),
                        nonce: receipt
                            .as_ref()
                            .map(|r| U256::from(r.transaction_index.as_u64()))
                            .unwrap_or(U256::zero()),
                        timestamp,
                        status: tx_status,
                        token_address: token_addr_opt.and_then(|s| Address::from_str(s).ok()),
                    });
                }
            }
        }

        // Fallback to eth_getLogs if Alchemy fails or returns no results
        if transactions.is_empty() {
            let latest_block = self.provider.get_block_number().await?;
            let scan_range: u64 = 100_000;
            let from_block_num = latest_block.as_u64().saturating_sub(scan_range);
            let mut logs = Vec::new();
            let mut start = from_block_num;
            let end = latest_block.as_u64();
            let chunk_size = 500;
            let total_chunks = ((end - start) / chunk_size + 1) as u64;
            let pb = ProgressBar::new(total_chunks);
            while start <= end {
                let chunk_end = std::cmp::min(start + chunk_size - 1, end);
                pb.set_message(format!("Fetching blocks {}–{}", start, chunk_end));
                let filter = Filter::new()
                    .address(*address)
                    .from_block(BlockNumber::Number(start.into()))
                    .to_block(BlockNumber::Number(chunk_end.into()))
                    .event("Transfer(address,address,uint256)");
                let mut chunk_logs = self.provider.get_logs(&filter).await?;
                logs.append(&mut chunk_logs);
                pb.inc(1);
                start = chunk_end + 1;
                pb.finish_with_message("Done fetching logs.");
            }

            for log in logs.into_iter().take(limit as usize) {
                if log.topics.len() >= 3 {
                    let tx_hash = log.transaction_hash.unwrap();
                    let receipt = self.provider.get_transaction_receipt(tx_hash).await?;
                    let tx_status = receipt
                        .as_ref()
                        .map(|r| {
                            if r.status.map_or(false, |s| s.as_u64() == 1) {
                                TransactionStatus::Success
                            } else {
                                TransactionStatus::Failed
                            }
                        })
                        .unwrap_or(TransactionStatus::Pending);

                    // Apply status filter
                    if let Some(status_filter) = status {
                        let status_str = match tx_status {
                            TransactionStatus::Success => "success",
                            TransactionStatus::Failed => "failed",
                            TransactionStatus::Pending => "pending",
                            _ => "unknown",
                        };
                        if status_str != status_filter.to_lowercase() {
                            continue;
                        }
                    }

                    // Apply token filter
                    if let Some(token_filter) = token {
                        if log.address.to_string().to_lowercase() != token_filter.to_lowercase() {
                            continue;
                        }
                    }

                    let block = self.provider.get_block(log.block_number.unwrap()).await?;
                    let timestamp = block
                        .ok_or_else(|| anyhow!("Block not found"))?
                        .timestamp
                        .as_u64();
                    let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);

                    // Apply date filters
                    if let Some(from) = from_date {
                        let from_time = chrono::DateTime::parse_from_rfc3339(from)
                            .map_err(|e| anyhow!("Invalid from_date: {}", e))?;
                        if timestamp
                            < SystemTime::UNIX_EPOCH
                                + Duration::from_secs(from_time.timestamp() as u64)
                        {
                            continue;
                        }
                    }
                    if let Some(to) = to_date {
                        let to_time = chrono::DateTime::parse_from_rfc3339(to)
                            .map_err(|e| anyhow!("Invalid to_date: {}", e))?;
                        if timestamp
                            > SystemTime::UNIX_EPOCH
                                + Duration::from_secs(to_time.timestamp() as u64)
                        {
                            continue;
                        }
                    }

                    transactions.push(RskTransaction {
                        hash: tx_hash,
                        from: Address::from_slice(&log.topics[1][12..32]),
                        to: Some(Address::from_slice(&log.topics[2][12..32])),
                        value: U256::from_big_endian(&log.data),
                        gas_price: receipt
                            .as_ref()
                            .and_then(|r| r.effective_gas_price)
                            .unwrap_or(U256::zero()),
                        gas: receipt
                            .as_ref()
                            .and_then(|r| r.gas_used)
                            .unwrap_or(U256::zero()),
                        nonce: receipt
                            .as_ref()
                            .map(|r| U256::from(r.transaction_index.as_u64()))
                            .unwrap_or(U256::zero()),
                        timestamp,
                        status: tx_status,
                        token_address: Some(log.address),
                    });
                }
            }

            // Fetch RBTC transactions via eth_getBlockByNumber
            let mut block_num = from_block_num;
            while block_num <= latest_block.as_u64() && transactions.len() < limit as usize {
                let block = self.provider.get_block_with_txs(block_num).await?;
                if let Some(block) = block {
                    for tx in block.transactions {
                        if tx.from == *address || tx.to == Some(*address) {
                            let receipt = self.provider.get_transaction_receipt(tx.hash).await?;
                            let tx_status = receipt
                                .as_ref()
                                .map(|r| {
                                    if r.status.map_or(false, |s| s.as_u64() == 1) {
                                        TransactionStatus::Success
                                    } else {
                                        TransactionStatus::Failed
                                    }
                                })
                                .unwrap_or(TransactionStatus::Pending);

                            // Apply status filter
                            if let Some(status_filter) = status {
                                let status_str = match tx_status {
                                    TransactionStatus::Success => "success",
                                    TransactionStatus::Failed => "failed",
                                    TransactionStatus::Pending => "pending",
                                    _ => "unknown",
                                };
                                if status_str != status_filter.to_lowercase() {
                                    continue;
                                }
                            }

                            // Apply token filter (skip for RBTC)
                            if token.is_some() {
                                continue;
                            }

                            let timestamp = block.timestamp.as_u64();
                            let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);

                            // Apply date filters
                            if let Some(from) = from_date {
                                let from_time = chrono::DateTime::parse_from_rfc3339(from)
                                    .map_err(|e| anyhow!("Invalid from_date: {}", e))?;
                                if timestamp
                                    < SystemTime::UNIX_EPOCH
                                        + Duration::from_secs(from_time.timestamp() as u64)
                                {
                                    continue;
                                }
                            }
                            if let Some(to) = to_date {
                                let to_time = chrono::DateTime::parse_from_rfc3339(to)
                                    .map_err(|e| anyhow!("Invalid to_date: {}", e))?;
                                if timestamp
                                    > SystemTime::UNIX_EPOCH
                                        + Duration::from_secs(to_time.timestamp() as u64)
                                {
                                    continue;
                                }
                            }

                            transactions.push(RskTransaction {
                                hash: tx.hash,
                                from: tx.from,
                                to: tx.to,
                                value: tx.value,
                                gas_price: tx.gas_price.unwrap_or(U256::zero()),
                                gas: tx.gas,
                                nonce: tx.nonce,
                                timestamp,
                                status: tx_status,
                                token_address: None,
                            });
                        }
                    }
                }
                block_num += 1;
            }
        }

        // Deduplicate transactions by hash
        let mut seen_hashes = HashSet::new();
        transactions.retain(|tx| seen_hashes.insert(tx.hash));

        // Sort by timestamp (newest first) and limit
        transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        transactions.truncate(limit as usize);

        Ok(transactions)
    }
}
