// use ethers::prelude::*;
// use ethers::providers::Provider;
// use std::sync::Arc;

// pub struct EthClient {
//     provider: Arc<Provider<Http>>,
//     wallet: Option<Wallet<LocalWallet>>,
// }

// impl EthClient {
//     // pub async fn new(config: &Config) -> Result<Self, anyhow::Error> {
//     //     let provider = Provider::<Http>::try_from(&config.network.rpc_url)
//     //         .map_err(|e| anyhow::anyhow!("Failed to connect to RPC: {}", e))?;

//     //     Ok(Self {
//     //         provider: Arc::new(provider),
//     //         wallet: config.wallet.private_key.as_ref().map(|key| {
//     //             key.parse::<LocalWallet>()
//     //                 .expect("Invalid private key")
//     //         }),
//     //     })
//     // }
//     pub async fn new(

//     pub async fn get_balance(&self, address: &Address) -> Result<U256, anyhow::Error> {
//         self.provider.get_balance(*address, None).await.map_err(|e| {
//             anyhow::anyhow!("Failed to get balance: {}", e)
//         })
//     }

//     pub async fn send_transaction(
//         &self,
//         to: Address,
//         amount: U256,
//         token_address: Option<Address>,
//     ) -> Result<H256, anyhow::Error> {
//         let tx = if let Some(token_address) = token_address {
//             // ERC20 transfer
//             // Implementation needed
//             todo!()
//         } else {
//             // Native token transfer
//             TransactionRequest::new()
//                 .to(to)
//                 .value(amount)
//         };

//         // Implementation needed for signing and sending
//         todo!()
//     }
//     pub async fn get_transaction_history(
//         &self,
//         address: &Address,
//         limit: u32,
//         status: Option<&str>,
//         token: Option<&str>,
//         from_date: Option<&str>,
//         to_date: Option<&str>,
//     ) -> Result<Vec<TransactionReceipt>, anyhow::Error> {
//         let mut filter = ethers::types::FilterBuilder::default()
//             .address(std::vec![*address])
//             .limit(limit)
//             .build();

//         // Add timestamp filters if provided
//         if let Some(from_date) = from_date {
//             let from_timestamp = chrono::DateTime::parse_from_str(from_date, "%Y-%m-%d")?
//                 .timestamp() as u64;
//             filter = filter.from_block(from_timestamp);
//         }

//         if let Some(to_date) = to_date {
//             let to_timestamp = chrono::DateTime::parse_from_str(to_date, "%Y-%m-%d")?
//                 .timestamp() as u64;
//             filter = filter.to_block(to_timestamp);
//         }

//         let logs = self.provider.get_logs(&filter).await?;
//         let mut receipts = Vec::new();

//         for log in logs {
//             if let Some(tx_hash) = log.transaction_hash {
//                 if let Some(receipt) = self.provider.get_transaction_receipt(tx_hash).await? {
//                     let transaction = self.provider.get_transaction(tx_hash).await?
//                         .ok_or_else(|| anyhow::anyhow!("Transaction not found"))?;

//                     // Filter by status if provided
//                     let status = match receipt.status {
//                         Some(1) => TransactionStatus::Success,
//                         Some(0) => TransactionStatus::Failed,
//                         None => TransactionStatus::Pending,
//                         _ => TransactionStatus::Unknown,
//                     };

//                     if let Some(status_filter) = status {
//                         if status.to_string().to_lowercase() != status_filter.to_lowercase() {
//                             continue;
//                         }
//                     }

//                     // Filter by token if provided
//                     let token_address = token.map(|token| {
//                         Address::from_str(token).map_err(|_| {
//                             anyhow::anyhow!("Invalid token address format")
//                         })
//                     }).transpose()?;

//                     let is_token_transfer = transaction.input.len() >= 10 &&
//                         &transaction.input[0..10] == hex::decode("a9059cbb").unwrap();

//                     if let Some(token_address) = token_address {
//                         if !is_token_transfer || transaction.to != Some(token_address) {
//                             continue;
//                         }
//                     }

//                     receipts.push(TransactionReceipt {
//                         transaction_hash: tx_hash,
//                         status,
//                         gas_used: receipt.gas_used.unwrap_or_default(),
//                         block_number: receipt.block_number,
//                         block_hash: receipt.block_hash,
//                         cumulative_gas_used: receipt.cumulative_gas_used.unwrap_or_default(),
//                         from: transaction.from,
//                         to: transaction.to,
//                         value: transaction.value,
//                         timestamp: receipt.timestamp,
//                         token_address,
//                     });
//                 }
//             }
//         }

//         Ok(receipts)
//     }
// }

use ethers::prelude::*;
use ethers::providers::Provider;
use std::sync::Arc;
use crate::utils::config::Config;

pub struct EthClient {
    provider: Arc<Provider<Http>>,
    wallet: Option<LocalWallet>,
}
impl EthClient {
    pub async fn new(config: &Config) -> Result<Self, anyhow::Error> {
        let provider = Provider::<Http>::try_from(&config.network.rpc_url)
            .map_err(|e| anyhow::anyhow!("Failed to connect to RPC: {}", e))?;

        Ok(Self {
            provider: Arc::new(provider),
            wallet: config.wallet.private_key.as_ref().map(|key| {
                key.parse::<LocalWallet>()
                    .expect("Invalid private key")
            }),
        })
    }

    pub async fn get_balance(&self, address: &Address) -> Result<U256, anyhow::Error> {
        self.provider
            .get_balance(*address, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get balance: {}", e))
    }

    pub async fn _send_transaction(
        &self,
        to: Address,
        amount: U256,
        token_address: Option<Address>,
    ) -> Result<H256, anyhow::Error> {
        let tx = if let Some(token_address) = token_address {
            // ERC20 transfer
            // Implementation needed
            todo!()
        } else {
            // Native token transfer
            TransactionRequest::new()
                .to(to)
                .value(amount)
        };

        // Implementation needed for signing and sending
        todo!()
    }
}
