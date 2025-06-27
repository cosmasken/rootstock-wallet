//! Interactive command-line interface for the Rootstock wallet

mod balance;
mod bulk_transfer;
mod config;
mod contacts;
mod history;
mod system;
mod tokens;
mod transfer;
mod transfer_preview;
mod tx;
mod wallet;

use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};

// Re-export public functions
pub use self::{
    balance::show_balance,
    bulk_transfer::bulk_transfer,
    config::show_config_menu,
    contacts::manage_contacts,
    history::show_history,
    wallet::create_wallet_with_name,
    tokens::token_menu,
    transfer::send_funds,
    tx::check_transaction_status,
    wallet::wallet_menu,
    system::system_menu,
};

// Import for network status display
use crate::config::ConfigManager;

// Import Network from the types module
use crate::types::network::Network;

// Re-export the Network type for consistency
pub use crate::types::network::Network as ConfigNetwork;

// Helper function to get styled network status
fn get_network_status(network: Network) -> console::StyledObject<&'static str> {
    match network {
        Network::Mainnet => style("🔗 Mainnet").cyan(),
        Network::Testnet => style("🔗 Testnet").yellow(),
        Network::Regtest => style("🔗 Regtest").magenta(),
        Network::AlchemyMainnet => style("🔗 Alchemy Mainnet").blue(),
        Network::AlchemyTestnet => style("🔗 Alchemy Testnet").blue(),
        Network::RootStockMainnet => style("🔗 Rootstock Mainnet").green(),
        Network::RootStockTestnet => style("🔗 Rootstock Testnet").green(),
    }
}

/// Starts the interactive CLI interface
pub async fn start() -> Result<()> {
    // Clear the screen for a fresh start
    clearscreen::clear().ok();
    
    // Display welcome banner
    println!("\n{}", style("🌐 Rootstock Wallet").bold().blue().underlined());
    println!("{}", style("Your Gateway to the Rootstock Blockchain").dim());
    println!("{}\n", "-".repeat(40));
    
    // Display current status
    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;
    
    println!("  {}", style("🟢 Online").green());
    println!("  {}", get_network_status(config.default_network));
    println!("  {}\n", style("💼 1 wallet loaded").dim());

    loop {
        let options = vec![
            format!("{}  Check Balance", style("💰").bold().green()),
            format!("{}  Send Funds", style("💸").bold().yellow()),
            format!("{}  Bulk Transfer", style("📤").bold().yellow()),
            format!("{}  Check Transaction Status", style("🔍").bold().cyan()),
            format!("{}  Transaction History", style("📜").bold().cyan()),
            format!("{}  Wallet Management", style("🔑").bold().blue()),
            format!("{}  Token Management", style("🪙").bold().magenta()),
            format!("{}  Contact Management", style("📇").bold().cyan()),
            format!("{}  Configuration", style("⚙️").bold().white()),
            format!("{}  System", style("💻").bold().cyan()),
            format!("{}  Exit", style("🚪").bold().red()),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("\nWhat would you like to do?")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => show_balance().await?,
            1 => send_funds().await?,
            2 => bulk_transfer().await?,
            3 => check_transaction_status().await?,
            4 => show_history().await?,
            5 => wallet_menu().await?,
            6 => token_menu().await?,
            7 => manage_contacts().await?,
            8 => show_config_menu().await?,
            9 => system_menu().await?,
            11 => {
                println!("\n👋 Goodbye!");
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
