//! Interactive command-line interface for the Rootstock wallet

mod balance;
mod contacts;
mod history;
mod tokens;
mod transfer;
mod wallet;

use anyhow::Result;
use console::style;

pub use self::{
    balance::show_balance, contacts::manage_contacts, history::show_history, tokens::token_menu,
    transfer::send_funds, wallet::wallet_menu,
};

/// Starts the interactive CLI interface
pub async fn start() -> Result<()> {
    println!("\n{}", style("🌐 Rootstock Wallet").bold().blue());
    println!("{}", "=".repeat(30));

    loop {
        let options = vec![
            String::from("💰 Check Balance"),
            String::from("💸 Send Funds"),
            String::from("📜 Transaction History"),
            String::from("🔑 Wallet Management"),
            String::from("🪙 Token Management"),
            String::from("📇 Contact Management"),
            String::from("❌ Exit"),
        ];

        let selection = inquire::Select::new("What would you like to do?", options)
            .prompt()
            .map_err(|_| anyhow::anyhow!("Failed to get selection"))?;

        match selection.as_str() {
            "💰 Check Balance" => show_balance().await?,
            "💸 Send Funds" => send_funds().await?,
            "📜 Transaction History" => show_history().await?,
            "🔑 Wallet Management" => wallet_menu().await?,
            "🪙 Token Management" => token_menu().await?,
            "📇 Contact Management" => manage_contacts().await?,
            "❌ Exit" => {
                println!("\n👋 Goodbye!");
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
