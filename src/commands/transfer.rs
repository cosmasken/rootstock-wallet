use crate::types::wallet::WalletData;
use crate::utils::constants;
use crate::utils::eth::EthClient;
use crate::utils::helper::Config as HelperConfig;
use crate::config::ConfigManager;
use anyhow::{Result, anyhow};
use clap::Parser;
use colored::Colorize;
use ethers::signers::LocalWallet;
use ethers::types::{Address, H256, U64, U256};
use rpassword::prompt_password;
use std::fs;
use std::str::FromStr;

/// Result of a transfer operation
#[derive(Debug)]
pub struct TransferResult {
    pub tx_hash: H256,
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub gas_used: U256,
    pub gas_price: U256,
    pub status: U64,
    pub token_address: Option<Address>,
    pub token_symbol: Option<String>,
}

#[derive(Parser, Debug)]
pub struct TransferCommand {
    /// Address to send to
    #[arg(long, required = true)]
    pub address: String,

    /// Amount to send (in tokens or RBTC)
    #[arg(long, required = true)]
    pub value: f64,

    /// Token address (for ERC20 transfers)
    #[arg(long)]
    pub token: Option<String>,

}

impl TransferCommand {
    /// Execute the transfer command and return the transfer result
    pub async fn execute(&self) -> Result<TransferResult> {
        // Load wallet file and get current wallet
        let wallet_file = constants::wallet_file_path();
        if !wallet_file.exists() {
            return Err(anyhow!(
                "No wallets found. Please create or import a wallet first."
            ));
        }
        let data = fs::read_to_string(&wallet_file)?;
        let wallet_data: WalletData = serde_json::from_str(&data)?;
        let default_wallet = wallet_data.get_current_wallet().ok_or_else(|| {
            anyhow!(
                "No default wallet selected. Please use 'wallet switch' to select a default wallet."
            )
        })?;

        // Prompt for password and decrypt private key
        let password = prompt_password("Enter password for the default wallet: ")?;
        let private_key = default_wallet.decrypt_private_key(&password)?;
        let _local_wallet = LocalWallet::from_str(&private_key)
            .map_err(|e| anyhow!("Failed to create LocalWallet: {}", e))?;

        // Get the network from config
        let config = ConfigManager::new()?.load()?;
        
        // Create a new helper config with the private key
        let client_config = HelperConfig {
            network: config.default_network.get_config(),
            wallet: crate::utils::helper::WalletConfig {
                current_wallet_address: None,
                private_key: Some(private_key.clone()),
                mnemonic: None,
            },
        };
        
        let eth_client = EthClient::new(&client_config, None).await?;

        // Parse recipient address
        let to = Address::from_str(&self.address)
            .map_err(|_| anyhow!("Invalid recipient address: {}", &self.address))?;

        // Parse optional token address
        let (token_address, token_symbol) = if let Some(token_addr) = &self.token {
            // Handle RBTC case (zero address or None)
            if token_addr == "0x0000000000000000000000000000000000000000" || token_addr.is_empty() {
                (None, Some("RBTC".to_string()))
            } else {
                // Parse token address
                let addr = Address::from_str(token_addr)
                    .map_err(|_| anyhow!("Invalid token address: {}", token_addr))?;
                
                // Try to get token info, but don't fail if we can't
                let symbol = match eth_client.get_token_info(addr).await {
                    Ok((_, sym)) => sym,
                    Err(_) => format!("Token (0x{})", &token_addr[2..10]),
                };
                
                (Some(addr), Some(symbol))
            }
        } else {
            // Native RBTC transfer
            (None, Some("RBTC".to_string()))
        };

        // Parse amount (convert f64 to wei or token units)
        let decimals = if token_address.is_some() { 18 } else { 18 }; // Default to 18 for both RBTC and tokens
        let amount = ethers::utils::parse_units(self.value.to_string(), decimals)
            .map_err(|e| anyhow!("Invalid amount: {}", e))?;

        // Send transaction
        let tx_hash = eth_client
            .send_transaction(to, amount.into(), token_address)
            .await?;

        println!(
            "{}: Transaction sent: 0x{:x} for {} {}",
            "Success".green().bold(),
            tx_hash,
            self.value,
            token_symbol.clone().unwrap_or("RBTC".to_string())
        );

        // Wait for transaction receipt
        let receipt = eth_client.get_transaction_receipt(tx_hash).await?;

        Ok(TransferResult {
            tx_hash,
            from: default_wallet.address(),
            to,
            value: amount.into(),
            gas_used: receipt.gas_used.unwrap_or_default(),
            gas_price: receipt.effective_gas_price.unwrap_or_default(),
            status: receipt.status.unwrap_or_else(|| U64::from(0)),
            token_address,
            token_symbol,
        })
    }
}
