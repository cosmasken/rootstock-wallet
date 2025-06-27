use crate::commands::tokens::TokenRegistry;
use crate::commands::transfer::TransferCommand;
use crate::config::ConfigManager;
use anyhow::Result;
use console::style;
use inquire::Select;
use inquire::validator::Validation;

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

    // Check if sending tokens or RBTC
    let is_token = inquire::Confirm::new("Are you sending an ERC20 token?")
        .with_default(false)
        .prompt()?;

    let (amount, token) = if is_token {
        let registry = TokenRegistry::load()
            .map_err(|e| {
                eprintln!("⚠️  Warning: Could not load token registry: {}", e);
                e
            })
            .unwrap_or_default();

        let tokens = registry.list_tokens(Some(&network));

        if tokens.is_empty() {
            return Err(anyhow::anyhow!(
                "No tokens found. Please add tokens first using the token management menu (option 3)."
            ));
        }

        // Create a vector of (symbol, token_info) pairs
        let token_choices: Vec<(String, crate::commands::tokens::TokenInfo)> = tokens
            .into_iter()
            .map(|(symbol, info)| (symbol, info))
            .collect();

        // Create a parallel vector of just the symbols for the selection menu
        let token_symbols: Vec<&str> = token_choices
            .iter()
            .map(|(symbol, _)| symbol.as_str())
            .collect();

        // Get the selected symbol index
        let selection_idx = Select::new("Select token to send:", token_symbols)
            .prompt_skippable()?
            .and_then(|selected| {
                token_choices
                    .iter()
                    .position(|(symbol, _)| symbol == selected)
            })
            .ok_or_else(|| anyhow::anyhow!("No token selected"))?;

        // Get the selected token info by index
        let (symbol, token_info) = &token_choices[selection_idx];
        let token_info = token_info.clone();

        let amount = inquire::Text::new("Amount to send:")
            .with_help_message(&format!("Enter the amount of {} to send", symbol))
            .with_validator(|input: &str| match input.parse::<f64>() {
                Ok(_) => Ok(Validation::Valid),
                Err(_) => Ok(Validation::Invalid("Please enter a valid number".into())),
            })
            .prompt()?
            .parse::<f64>()?;

        (amount, Some(token_info.address.clone()))
    } else {
        // For RBTC
        let amount = inquire::Text::new("Amount to send (in RBTC):")
            .with_help_message("Enter the amount of RBTC to send")
            .with_validator(|input: &str| match input.parse::<f64>() {
                Ok(_) => Ok(Validation::Valid),
                Err(_) => Ok(Validation::Invalid("Please enter a valid number".into())),
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
        // Get the network from config if not provided
        let config = ConfigManager::new()?.load()?;
        
        let cmd = TransferCommand {
            address: to,
            value: amount,
            token,
        };

        // Execute the transfer
        cmd.execute().await?;
        println!("\n{}", style("✅ Transaction sent successfully!").green());
    } else {
        println!("\n{}", style("❌ Transaction cancelled").yellow());
    }

    Ok(())
}
