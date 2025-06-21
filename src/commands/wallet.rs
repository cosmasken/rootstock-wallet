use crate::types::wallet::{Wallet, WalletData};
use crate::utils::{config::Config, constants, eth::EthClient, table::TableBuilder};
use anyhow::{Result, anyhow};
use chrono::Utc;
use clap::Parser;
use colored::Colorize;
use ethers::signers::LocalWallet;
use rand::thread_rng;
use rpassword::prompt_password;
use std::fs;
use std::path::PathBuf;
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
        #[arg(short, long, help = "Name of the wallet to backup (e.g., MyWallet)")]
        name: String,
        #[arg(short, long, help = "Backup Filename (e.g., backup.json)")]
        path: PathBuf,
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
            WalletAction::Import { private_key, name } => {
                self.import_wallet(&config, private_key, name).await?
            }
            WalletAction::List => self.list_wallets(&config)?,
            WalletAction::Switch { name } => self.switch_wallet(&mut config, name)?,
            WalletAction::Rename { old_name, new_name } => {
                self.rename_wallet(&config, old_name, new_name)?
            }
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

        // Check if wallet with this name already exists
        let wallet_file = constants::wallet_file_path();
        if wallet_file.exists() {
            let data = fs::read_to_string(&wallet_file)?;
            let wallet_data = serde_json::from_str::<WalletData>(&data)?;
            if wallet_data.get_wallet_by_name(name).is_some() {
                return Err(anyhow!("Wallet with name '{}' already exists", name));
            }
        }

        let wallet = LocalWallet::new(&mut thread_rng());
        let wallet = Wallet::new(wallet, name, &password)?;

        let mut wallet_data = if wallet_file.exists() {
            let data = fs::read_to_string(&wallet_file)?;
            serde_json::from_str::<WalletData>(&data)?
        } else {
            WalletData::new()
        };

        wallet_data.add_wallet(wallet.clone());

        // Save the updated wallet data
        fs::write(
            &wallet_file,
            serde_json::to_string_pretty(&wallet_data)?.as_bytes(),
        )?;

        println!("{}", "🎉 Wallet created successfully".green());
        println!("Address: {:?}", wallet.address());
        println!("Wallet saved at: {}", wallet_file.display());

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

        let wallet_file = constants::wallet_file_path();
        let mut wallet_data = if wallet_file.exists() {
            let data = fs::read_to_string(&wallet_file)?;
            serde_json::from_str::<WalletData>(&data)?
        } else {
            WalletData::new()
        };

        wallet_data.add_wallet(wallet);

        // Save the updated wallet data
        fs::write(
            &wallet_file,
            serde_json::to_string_pretty(&wallet_data)?.as_bytes(),
        )?;

        println!("{}", "✅ Wallet imported successfully".green());
        // println!("Address: {}", format!("0x{:x}", wallet.address));
        println!("Wallet saved at: {}", wallet_file.display());

        Ok(())
    }

    fn list_wallets(&self, config: &Config) -> Result<()> {
        let wallet_file = constants::wallet_file_path();
        if !wallet_file.exists() {
            println!("No wallets found");
            return Ok(());
        }

        let data = fs::read_to_string(&wallet_file)?;
        let wallet_data = serde_json::from_str::<WalletData>(&data)?;
        let wallets = wallet_data.list_wallets();

        let mut table = TableBuilder::new();
        table.add_row(&["Name", "Address", "Created At", "Current"]);

        for wallet in wallets {
            let is_current = if let Some(current) = wallet_data.get_current_wallet() {
                current.address == wallet.address
            } else {
                false
            };

            table.add_row(&[
                &wallet.name,
                &format!("0x{:x}", wallet.address),
                &wallet.created_at,
                if is_current { "✓" } else { "" },
            ]);
        }

        table.print();

        Ok(())
    }

    fn rename_wallet(&self, config: &Config, old_name: &str, new_name: &str) -> Result<()> {
        let wallet_file = constants::wallet_file_path();
        if !wallet_file.exists() {
            return Err(anyhow!("No wallets found"));
        }

        // Validate new name
        if new_name.is_empty() {
            return Err(anyhow!("New wallet name cannot be empty"));
        }

        let data = fs::read_to_string(&wallet_file)?;
        let mut wallet_data = serde_json::from_str::<WalletData>(&data)?;

        // Check if old_name exists
        let wallet = wallet_data
            .get_wallet_by_name(old_name)
            .ok_or_else(|| anyhow!("Wallet '{}' not found", old_name))?;

        // Check if new_name is already in use
        if wallet_data.get_wallet_by_name(new_name).is_some() {
            return Err(anyhow!("Wallet with name '{}' already exists", new_name));
        }

        // Update wallet name
        let address = format!("0x{:x}", wallet.address);
        if let Some(wallet) = wallet_data.wallets.get_mut(&address) {
            wallet.name = new_name.to_string();
        } else {
            return Err(anyhow!("Failed to rename wallet '{}'", old_name));
        }

        // Save the updated wallet data
        fs::write(&wallet_file, serde_json::to_string_pretty(&wallet_data)?)?;

        println!(
            "{}",
            format!("✅ Wallet renamed from '{}' to '{}'", old_name, new_name).green()
        );
        println!("Address: {}", address);

        Ok(())
    }

    fn switch_wallet(&self, config: &mut Config, name: &str) -> Result<()> {
        let wallet_file = constants::wallet_file_path();
        let data = fs::read_to_string(&wallet_file)?;
        let mut wallet_data = serde_json::from_str::<WalletData>(&data)?;

        // Get the wallet address first to avoid borrowing conflicts
        let wallet_address = wallet_data
            .get_wallet_by_name(name)
            .ok_or_else(|| anyhow!("Wallet '{}' not found", name))?
            .address;

        wallet_data.switch_wallet(&format!("0x{:x}", wallet_address));

        // Save the updated wallet data
        fs::write(&wallet_file, serde_json::to_string_pretty(&wallet_data)?)?;

        println!("{}", format!("✅ Switched to wallet: {}", name).green());
        println!("Address: {}", format!("0x{:x}", wallet_address));

        Ok(())
    }

    fn backup_wallet(&self, config: &Config, name: &str, path: &PathBuf) -> Result<()> {
        let wallet_file = constants::wallet_file_path();
        if !wallet_file.exists() {
            return Err(anyhow!("No wallets found"));
        }

        let data = fs::read_to_string(&wallet_file)?;
        let wallet_data = serde_json::from_str::<WalletData>(&data)?;

        // Check if name looks like a file path
        if name.ends_with(".json") {
            return Err(anyhow!(
                "Invalid wallet name '{}'. Use --name for the wallet name and --path for the filename. Example: --name MyWallet --path backup.json",
                name
            ));
        }

        let wallet = wallet_data
            .get_wallet_by_name(name)
            .ok_or_else(|| anyhow!("Wallet '{}' not found", name))?;

        // Extract filename from path and prepend "./"
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| anyhow!("Invalid filename in path: {}", path.display()))?;
        let backup_path = PathBuf::from(format!("./{}", filename));

        // Write the wallet data to backup location
        fs::write(&backup_path, serde_json::to_string_pretty(&wallet)?)
            .map_err(|e| anyhow!("Failed to write backup file: {}", e))?;

        // Verify the file was created
        if !backup_path.exists() {
            return Err(anyhow!(
                "Backup file was not created at: {}",
                backup_path.display()
            ));
        }

        println!("{}", "✅ Backup created successfully".green());
        println!("Backup saved at: {}", backup_path.display());

        Ok(())
    }

    fn delete_wallet(&self, config: &Config, name: &str) -> Result<()> {
        let wallet_file = constants::wallet_file_path();
        let data = fs::read_to_string(&wallet_file)?;
        let mut wallet_data = serde_json::from_str::<WalletData>(&data)?;

        let wallet = wallet_data
            .get_wallet_by_name(name)
            .ok_or_else(|| anyhow!("Wallet '{}' not found", name))?;

        let address = format!("0x{:x}", wallet.address);
        if wallet_data.current_wallet == address {
            return Err(anyhow!(
                "Cannot delete currently selected wallet. Please switch to a different wallet first."
            ));
        }

        wallet_data.remove_wallet(&address);

        // Save the updated wallet data
        fs::write(&wallet_file, serde_json::to_string_pretty(&wallet_data)?)?;

        println!("{}", format!("✅ Deleted wallet: {}", name).green());
        println!("Address: {}", address);

        Ok(())
    }
}
