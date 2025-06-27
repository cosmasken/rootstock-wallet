use crate::commands::tokens::TokenRegistry;
use crate::commands::transfer::TransferCommand;
use crate::config::ConfigManager;
use anyhow::Result;
use colored::*;
use console::style;
use inquire::Select;
use inquire::validator::Validation;
use anyhow::anyhow;

/// Displays the fund transfer interface
pub async fn send_funds() -> Result<()> {
    println!("\n{}", style("💸 Send Funds").bold());
    println!("{}", "=".repeat(30));

    // Get the current network from config
    let config = ConfigManager::new()?.load()?;
    let network = config.default_network.to_string().to_lowercase();
    println!("Using network: {}", network);

    // Get recipient address
    let to = inquire::Text::new("Recipient address (0x...):")
        .with_help_message("Enter the Ethereum address to send to")
        .with_validator(|input: &str| {
            if input.starts_with("0x") && input.len() == 42 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(
                    "Please enter a valid Ethereum address (0x...)".into(),
                ))
            }
        })
        .prompt()?;

    // Load token registry
    let registry = TokenRegistry::load()
        .map_err(|e| {
            eprintln!("⚠️  Warning: Could not load token registry: {}", e);
            e
        })
        .unwrap_or_default();

    // Get tokens for the current network
    let mut tokens = registry.list_tokens(Some(&network));

    // Add RBTC as the first option
    tokens.insert(
        0,
        (
            "RBTC (Native)".to_string(),
            crate::commands::tokens::TokenInfo {
                address: "0x0000000000000000000000000000000000000000".to_string(),
                decimals: 18,
            },
        ),
    );
    
    if tokens.is_empty() {
        return Err(anyhow!("No tokens found for {} network", network));
    }

    // Create a vector of (display_name, token_info) pairs
    let token_choices: Vec<(String, crate::commands::tokens::TokenInfo)> = tokens
        .into_iter()
        .filter(|(_, info)| {
            // Only include tokens that match the current network or are RBTC
            info.address == "0x0000000000000000000000000000000000000000" || 
            registry.list_tokens(Some(&network))
                .iter()
                .any(|(_, token_info)| token_info.address == info.address)
        })
        .collect();

    // Get just the display names for the selection menu
    let token_display_names: Vec<String> = token_choices
        .iter()
        .map(|(name, _)| name.clone())
        .collect();

    // Let the user select which token to send
    let selection = Select::new("Select token to send:", token_display_names)
        .prompt()?;

    // Find the selected token info
    let (display_name, token_info) = token_choices
        .into_iter()
        .find(|(name, _)| name == &selection)
        .ok_or_else(|| anyhow!("Selected token not found"))?;
        
    // Extract the token symbol (remove the (Native) suffix if present)
    let token_symbol = display_name
        .split_whitespace()
        .next()
        .unwrap_or(&display_name)
        .to_string();

    let amount = inquire::Text::new(&format!("Amount of {} to send:", token_symbol))
        .with_help_message("Enter the amount to send")
        .with_validator(|input: &str| match input.parse::<f64>() {
            Ok(n) if n > 0.0 => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid("Please enter a valid positive number".into())),
        })
        .prompt()?
        .parse::<f64>()?;

    // Clone the address since we need to use it multiple times
    let token_address = token_info.address.clone();
    let _token = if token_address == "0x0000000000000000000000000000000000000000" {
        None
    } else {
        Some(token_address.clone())
    };
    
    // Show transaction summary
    println!("\n{}", style("📝 Transaction Summary").bold());
    println!("{}", "=".repeat(30));
    println!("To: {}", to);
    println!("Token: {}", token_symbol);
    println!("Amount: {} {}", amount, token_symbol);
    println!("Network: {}", network);

    // Confirm transaction
    let confirm = inquire::Confirm::new("Confirm transaction?")
        .with_default(false)
        .prompt()?;

    if !confirm {
        println!("Transaction cancelled");
        return Ok(());
    }

    // Execute the transfer command
    let cmd = TransferCommand {
        address: to,
        value: amount,
        token: if token_address == "0x0000000000000000000000000000000000000000" {
            None
        } else {
            Some(token_address)
        },
    };

    let result = cmd.execute().await?;
    
    println!(
        "\n{}: Transaction confirmed! Tx Hash: {}",
        "Success".green().bold(),
        result.tx_hash
    );

    Ok(())
}
