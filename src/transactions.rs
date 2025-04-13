use ethers::{
    core::types::{TransactionRequest, U256},
    prelude::*,
    providers::{Middleware, ProviderError},
    signers::WalletError,
};
use thiserror::Error;
use zeroize::Zeroizing;

#[derive(Error, Debug)]
pub enum TransferError {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    #[error("Wallet error: {0}")]
    Wallet(#[from] WalletError),
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Invalid recipient address")]
    InvalidAddress,
    #[error("Transaction timeout")]
    Timeout,
}

pub struct TransactionService {
    provider: Provider<Http>,
    wallet: LocalWallet,
}

impl TransactionService {
    pub fn new(
        provider: Provider<Http>,
        private_key: Zeroizing<String>,
    ) -> Result<Self, TransferError> {
        let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(31u64); // Chain ID 31 for Rootstock Testnet
        Ok(Self { provider, wallet })
    }

    pub async fn send_rbtc(
        &self,
        to: Address,
        amount: U256,
    ) -> Result<TransactionReceipt, TransferError> {
        // 1. Validate recipient address
        if to == Address::zero() {
            return Err(TransferError::InvalidAddress);
        }

        // 2. Check balance
        let balance = self
            .provider
            .get_balance(self.wallet.address(), None)
            .await?;
        if balance < amount {
            return Err(TransferError::InsufficientBalance);
        }

        // 3. Build transaction (EIP-1559)
        let tx = TransactionRequest::new()
            .to(to)
            .value(amount)
            .from(self.wallet.address())
            .chain_id(self.wallet.chain_id());

        // 4. Send and await receipt (timeout after 5 blocks)
        let pending_tx = self
            .provider
            .send_transaction(tx, None)
            .await?
            .interval(std::time::Duration::from_secs(1))
            .retries(5);

        pending_tx.await?.ok_or(TransferError::Timeout)
    }
}
