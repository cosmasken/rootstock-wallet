use ethers::types::Address;
use ethers::types::{H256, Transaction, TxHash, U256};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RskTransaction {
    pub hash: TxHash,
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub gas_price: U256,
    pub gas: U256,
    pub nonce: U256,
    pub timestamp: SystemTime,
    pub status: TransactionStatus,
    pub token_address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub transaction_hash: TxHash,
    pub status: TransactionStatus,
    pub gas_used: U256,
    pub block_number: Option<U256>,
    pub block_hash: Option<H256>,
    pub cumulative_gas_used: U256,
}
