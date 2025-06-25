use crate::commands::transfer::TransferCommand;
use anyhow::Result;
use console::style;
use inquire::validator::Validation;

/// Displays the fund transfer interface
pub async fn send_funds() -> Result<()> {
    println!("\n{}", style("💸 Send Funds").bold());
    println!("{}", "=".repeat(30));
    
    // Select network
    let network = inquire::Select::new(
        "Select network:",
        vec![String::from("mainnet"), String::from("testnet")],
    )
    .prompt()?
    .to_string();
    
    // Get recipient address
    let to = inquire::Text::new("Recipient address (0x...):")
        .with_help_message("Enter the Ethereum address to send to")
        .with_validator(|input: &str| {
            if input.starts_with("0x") && input.len() == 42 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Please enter a valid Ethereum address (0x...)".into()))
            }
        })
        .prompt()?;
    
    // Check if sending tokens or RBTC
    let is_token = inquire::Confirm::new("Are you sending an ERC20 token?")
        .with_default(false)
        .prompt()?;
    
    let (amount, token) = if is_token {
        // For tokens, we need the token contract address
        let token_address = inquire::Text::new("Token contract address (0x...):")
            .with_validator(|input: &str| {
                if input.starts_with("0x") && input.len() == 42 {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid("Please enter a valid token contract address (0x...)".into()))
                }
            })
            .prompt()?;
            
        let amount = inquire::Text::new("Amount to send:")
            .with_help_message("Enter the amount of tokens to send")
            .with_validator(|input: &str| {
                match input.parse::<f64>() {
                    Ok(_) => Ok(Validation::Valid),
                    Err(_) => Ok(Validation::Invalid("Please enter a valid number".into()))
                }
            })
            .prompt()?
            .parse::<f64>()?;
            
        (amount, Some(token_address))
    } else {
        // For RBTC
        let amount = inquire::Text::new("Amount to send (in RBTC):")
            .with_help_message("Enter the amount of RBTC to send")
            .with_validator(|input: &str| {
                match input.parse::<f64>() {
                    Ok(_) => Ok(Validation::Valid),
                    Err(_) => Ok(Validation::Invalid("Please enter a valid number".into()))
                }
            })
            .prompt()?
            .parse::<f64>()?;
            
        (amount, None)
    };
    
    // Show transaction summary
    println!("\n{}", style("📝 Transaction Summary").bold());
    println!("{}", "=".repeat(30));
    println!("To: {}", to);
    if let Some(token_addr) = &token {
        println!("Token: {}", token_addr);
    } else {
        println!("Asset: RBTC");
    }
    println!("Amount: {}", amount);
    println!("Network: {}", network);
    
    // Confirm transaction
    let confirm = inquire::Confirm::new("Confirm transaction?")
        .with_default(false)
        .prompt()?;
        
    if confirm {
        let cmd = TransferCommand {
            address: to,
            value: amount,
            token,
            network: network.to_string(),
        };
        
        // Execute the transfer
        cmd.execute().await?;
        println!("\n{}", style("✅ Transaction sent successfully!").green());
    } else {
        println!("\n{}", style("❌ Transaction cancelled").yellow());
    }
    
    Ok(())
}
