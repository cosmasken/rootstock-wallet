use crate::types::wallet::WalletData;
use crate::utils::constants;
use crate::utils::eth::EthClient;
use crate::utils::helper::Config;
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

    /// Network to use (mainnet/testnet)
    #[arg(long, default_value = "mainnet")]
    pub network: String,
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

        // Inject the private key into the config for EthClient
        let mut config = Config::default();
        config.network = crate::types::network::Network::from_str(&self.network)
            .unwrap_or(crate::types::network::Network::Mainnet)
            .get_config();
        config.wallet.private_key = Some(private_key.clone());
        let eth_client = EthClient::new(&config, None).await?;

        // Parse recipient address
        let to = Address::from_str(&self.address)
            .map_err(|_| anyhow!("Invalid recipient address: {}", &self.address))?;

        // Parse amount (convert f64 to wei or token units, assuming 18 decimals)
        let amount = ethers::utils::parse_units(self.value.to_string(), 18)
            .map_err(|e| anyhow!("Invalid amount: {}", e))?;

        // Parse optional token address
        let token_address = self
            .token
            .as_ref()
            .map(|t| Address::from_str(t).map_err(|_| anyhow!("Invalid token address: {}", t)))
            .transpose()?;

        // Get token info if transferring ERC-20
        let token_symbol = if let Some(token_addr) = token_address {
            let (decimals, symbol) = eth_client.get_token_info(token_addr).await?;
            if decimals != 18 {
                return Err(anyhow!(
                    "Token decimals ({}) not supported; only 18 decimals allowed",
                    decimals
                ));
            }
            Some(symbol)
        } else {
            None
        };

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
