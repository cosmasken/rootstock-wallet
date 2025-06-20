use crate::utils::{config::Config, eth::EthClient, table::TableBuilder};
use crate::types::wallet::Wallet;
use clap::Parser;
use anyhow::{Result, anyhow};
use colored::Colorize;
use ethers::signers::LocalWallet;
use rand::thread_rng;
use std::fs;
use std::path::PathBuf;
use rpassword::prompt_password;
use chrono::Utc;
use std::str::FromStr;

#[derive(Parser, Debug)]
pub struct WalletCommand {
    /// Manage wallets
    #[command(subcommand)]
    pub action: WalletAction,
}

#[derive(Parser, Debug)]
enum WalletAction {
    /// Create a new wallet
    Create {
        #[arg(short, long, help = "Name for the new wallet")]
        name: String,
    },
    /// Import an existing wallet
    Import {
        #[arg(short, long, help = "Private key of the wallet")]
        private_key: String,
        #[arg(short, long, help = "Name for the imported wallet")]
        name: String,
    },
    /// List all saved wallets
    List,
    /// Switch to a different wallet
    Switch {
        #[arg(short, long, help = "Name of the wallet to switch to")]
        name: String,
    },
    /// Update wallet name
    Rename {
        #[arg(short, long, help = "Current wallet name")]
        old_name: String,
        #[arg(short, long, help = "New wallet name")]
        new_name: String,
    },
    /// Backup wallet file
    Backup {
        #[arg(short, long, help = "Name of the wallet to backup")]
        name: String,
        #[arg(short, long, help = "Path to save backup")]
        path: Option<PathBuf>,
    },
    /// Delete a wallet
    Delete {
        #[arg(short, long, help = "Name of the wallet to delete")]
        name: String,
    },
}

impl WalletCommand {
    pub async fn execute(&self) -> Result<()> {
        let mut config = Config::load()?;
        
        match &self.action {
            WalletAction::Create { name } => self.create_wallet(&config, name).await?,
            WalletAction::Import { private_key, name } => self.import_wallet(&config, private_key, name).await?,
            WalletAction::List => self.list_wallets(&config)?,
            WalletAction::Switch { name } => self.switch_wallet(&mut config, name)?,
            WalletAction::Rename { old_name, new_name } => self.rename_wallet(&config, old_name, new_name)?,
            WalletAction::Backup { name, path } => self.backup_wallet(&config, name, path)?,
            WalletAction::Delete { name } => self.delete_wallet(&config, name)?,
        }
        
        Ok(())
    }

    async fn create_wallet(&self, config: &Config, name: &str) -> Result<()> {
        let password = prompt_password("Enter password to encrypt wallet: ")?;
        let confirm_password = prompt_password("Confirm password: ")?;
        
        if password != confirm_password {
            return Err(anyhow!("Passwords do not match"));
        }
        
        let wallet = LocalWallet::new(&mut thread_rng());
        let wallet = Wallet::new(wallet, name, &password)?;
        
        let wallet_dir = config.wallet_dir()?;
        fs::create_dir_all(&wallet_dir)?;
        
        let wallet_path = wallet_dir.join(format!("{}.json", name));
        let wallet_json = serde_json::to_string_pretty(&wallet)?;
        
        fs::write(&wallet_path, wallet_json)?;
        
        println!("{}", "🎉 Wallet created successfully".green());
        println!("Address: {}", wallet.address());
        // println!("Private key: {}", self.decrypt_private_key(&password)?);
        println!("Wallet saved at: {}", wallet_path.display());
        
        Ok(())
    }

    async fn import_wallet(&self, config: &Config, private_key: &str, name: &str) -> Result<()> {
        let password = prompt_password("Enter password to encrypt wallet: ")?;
        let confirm_password = prompt_password("Confirm password: ")?;
        
        if password != confirm_password {
            return Err(anyhow!("Passwords do not match"));
        }
        
        let wallet = LocalWallet::from_str(private_key)?;
        let wallet = Wallet::new(wallet, name, &password)?;
        
        let wallet_dir = config.wallet_dir()?;
        fs::create_dir_all(&wallet_dir)?;
        
        let wallet_path = wallet_dir.join(format!("{}.json", name));
        let wallet_json = serde_json::to_string_pretty(&wallet)?;
        
        fs::write(&wallet_path, wallet_json)?;
        
        println!("{}", "✅ Wallet imported successfully".green());
        println!("Address: {}", wallet.address());
        println!("Wallet saved at: {}", wallet_path.display());
        
        Ok(())
    }

    fn list_wallets(&self, config: &Config) -> Result<()> {
        let wallet_dir = config.wallet_dir()?;
        let mut wallets = Vec::new();
        
        for entry in fs::read_dir(wallet_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "json") {
                let wallet: Wallet = serde_json::from_slice(&fs::read(entry.path())?)?;
                wallets.push(wallet);
            }
        }
        
         let mut table = TableBuilder::new();
        table.add_row(&["Name", "Address", "Created At"]);
        
        for wallet in wallets {
            table.add_row(&[
                &wallet.name,
                &wallet.address().to_string(),
                &wallet.created_at,
            ]);
        }
        
        table.print();
        
        Ok(())
    }

    fn switch_wallet(&self, config:&mut Config, name: &str) -> Result<()> {
        let wallet_dir = config.wallet_dir()?;
        let wallet_path = wallet_dir.join(format!("{}.json", name));
        
        if !wallet_path.exists() {
            return Err(anyhow!("Wallet '{}' not found", name));
        }
        
        let wallet: Wallet = serde_json::from_slice(&fs::read(wallet_path)?)?;
        
        config.set_current_wallet(&wallet.address().to_string())?;
        
        println!("{}", format!("✅ Switched to wallet: {}", name).green());
        println!("Address: {}", wallet.address());
        
        Ok(())
    }

    fn rename_wallet(&self, config: &Config, old_name: &str, new_name: &str) -> Result<()> {
        let wallet_dir = config.wallet_dir()?;
        let old_path = wallet_dir.join(format!("{}.json", old_name));
        
        if !old_path.exists() {
            return Err(anyhow!("Wallet '{}' not found", old_name));
        }
        
        let wallet: Wallet = serde_json::from_slice(&fs::read(&old_path)?)?;
        let new_path = wallet_dir.join(format!("{}.json", new_name));
        
        fs::write(new_path, serde_json::to_string_pretty(&wallet)?)?;
        fs::remove_file(&old_path)?;
        
        println!("{}", format!("✅ Renamed wallet from '{}' to '{}'", old_name, new_name).green());
        
        Ok(())
    }

    fn backup_wallet(&self, config: &Config, name: &str, backup_path: &Option<PathBuf>) -> Result<()> {
        let wallet_dir = config.wallet_dir()?;
        let wallet_path = wallet_dir.join(format!("{}.json", name));
        
        if !wallet_path.exists() {
            return Err(anyhow!("Wallet '{}' not found", name));
        }
        
        let backup_dir = match backup_path {
            Some(path) => path.clone(),
            None => config.backup_dir()?,
        };
        
        fs::create_dir_all(&backup_dir)?;
        
        let wallet: Wallet = serde_json::from_slice(&fs::read(wallet_path)?)?;
        let timestamp = Utc::now().timestamp();
        let backup_file = backup_dir.join(format!("backup_{}_{}.json", name, timestamp));
        
        fs::write(backup_file.clone(), serde_json::to_string_pretty(&wallet)?)?;
        
        println!("{}", "✅ Backup created successfully".green());
        println!("Backup saved at: {}", backup_file.display());
        
        Ok(())
    }

    fn delete_wallet(&self, config: &Config, name: &str) -> Result<()> {
        let wallet_dir = config.wallet_dir()?;
        let wallet_path = wallet_dir.join(format!("{}.json", name));
        
        if !wallet_path.exists() {
            return Err(anyhow!("Wallet '{}' not found", name));
        }
        
        fs::remove_file(wallet_path)?;
        
        println!("{}", format!("🗑️ Deleted wallet: {}", name).yellow());
        
        Ok(())
    }
}