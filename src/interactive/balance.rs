use anyhow::Result;
use console::style;
use crate::commands::balance::BalanceCommand;

/// Displays the balance checking interface
pub async fn show_balance() -> Result<()> {
    println!("\n{}", style("💰 Check Balance").bold());
    println!("{}", "=".repeat(30));
    
    // Select network
    let network = inquire::Select::new(
        "Select network:",
        vec![String::from("mainnet"), String::from("testnet")],
    )
    .prompt()?
    .to_string();
    
    // Ask if user wants to check token balance
    let check_token = inquire::Confirm::new("Do you want to check token balance?")
        .with_default(false)
        .prompt()?;
    
    let token = if check_token {
        // In a real implementation, you would list the user's tokens here
        let token_address = inquire::Text::new("Enter token contract address (0x...):")
            .with_help_message("Leave empty to check RBTC balance")
            .prompt_skippable()?;
            
        token_address.filter(|s| !s.trim().is_empty())
    } else {
        None
    };
    
    // Execute the balance command
    let cmd = BalanceCommand {
        address: None, // Will use default wallet
        network: network.to_string(),
        token,
    };
    
    cmd.execute().await
}
