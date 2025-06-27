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
    // Clear the screen for a fresh start
    clearscreen::clear().ok();
    
    // Display welcome banner
    println!("\n{}", style("🌐 Rootstock Wallet").bold().blue().underlined());
    println!("{}", style("Your Gateway to the Rootstock Blockchain").dim());
    println!("{}\n", "-".repeat(40));
    
    // Display quick status (you can enhance this with actual wallet status)
    println!("  {}", style("🟢 Online").green());
    println!("  {}", style("🔗 Mainnet").cyan());
    println!("  {}\n", style("💼 1 wallet loaded").dim());

    loop {
        let options = vec![
            format!("{}  Check Balance", style("💰").bold().green()),
            format!("{}  Send Funds", style("💸").bold().yellow()),
            format!("{}  Transaction History", style("📜").bold().cyan()),
            format!("{}  Wallet Management", style("🔑").bold().blue()),
            format!("{}  Token Management", style("🪙").bold().magenta()),
            format!("{}  Contact Management", style("📇").bold().cyan()),
            format!("{}  Exit", style("🚪").bold().red()),
        ];

        let selection = inquire::Select::new(
            "\nWhat would you like to do? (Use ↑↓ arrows to navigate, Enter to select)",
            options,
        )
        .with_page_size(10)
        .with_help_message("Press 'q' to quit at any time")
        .with_formatter(&|i| i.value.to_string())
        .prompt()
        .map_err(|_| anyhow::anyhow!("Operation cancelled"))?;

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
