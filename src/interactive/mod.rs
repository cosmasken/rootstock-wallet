//! Interactive command-line interface for the Rootstock wallet

mod wallet;
mod balance;
mod transfer;
mod tokens;

use anyhow::Result;
use console::style;

pub use self::{
    wallet::wallet_menu,
    balance::show_balance,
    transfer::send_funds,
    tokens::token_menu,
};

/// Starts the interactive CLI interface
pub async fn start() -> Result<()> {
    println!("\n{}", style("🌐 Rootstock Wallet").bold().blue());
    println!("{}", "=".repeat(30));

    loop {
        let options = vec![
            String::from("💰 Check Balance"),
            String::from("💸 Send Funds"),
            String::from("🔑 Wallet Management"),
            String::from("🪙 Token Management"),
            String::from("❌ Exit"),
        ];

        let selection = inquire::Select::new("What would you like to do?", options)
            .prompt()
            .map_err(|_| anyhow::anyhow!("Failed to get selection"))?;

        match selection.as_str() {
            "💰 Check Balance" => show_balance().await?,
            "💸 Send Funds" => send_funds().await?,
            "🔑 Wallet Management" => wallet_menu().await?,
            "🪙 Token Management" => token_menu().await?,
            "❌ Exit" => {
                println!("\n👋 Goodbye!");
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
