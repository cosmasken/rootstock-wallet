use anyhow::Result;
use console::style;
use crate::commands::balance::BalanceCommand;
use crate::commands::tokens::TokenRegistry;
use inquire::Select;

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
    
    // Ask if user wants to check token balance instead of RBTC
    let check_token = inquire::Confirm::new("Check token balance? (No for RBTC balance)")
        .with_default(false)
        .prompt()?;
    
    let token = if check_token {
        let registry = TokenRegistry::load()
            .map_err(|e| {
                eprintln!("⚠️  Warning: Could not load token registry: {}", e);
                e
            })
            .unwrap_or_default();
            
        let mut tokens = registry.list_tokens(Some(&network));
        
        // Add RBTC as default option
        tokens.insert(0, ("RBTC".to_string(), 
            crate::commands::tokens::TokenInfo {
                address: "0x0000000000000000000000000000000000000000".to_string(),
                decimals: 18,
            })
        );
        
        // Create a vector of (symbol, address) pairs and get symbols for selection
        let token_choices: Vec<(String, String)> = tokens.into_iter()
            .map(|(symbol, info)| (symbol, info.address))
            .collect();
            
        // Get the selected token
        let selected_symbol = {
            // Create a vector of just the symbols for the selection menu
            let token_symbols: Vec<String> = token_choices.iter()
                .map(|(symbol, _)| symbol.clone())
                .collect();
                
            Select::new("Select token:", token_symbols)
                .prompt_skippable()?
                .ok_or_else(|| anyhow::anyhow!("No token selected"))?
        };
            
        // Find the selected token info by matching the symbol
        token_choices.into_iter()
            .find(|(symbol, _)| symbol == &selected_symbol)
            .map(|(_, address)| address)
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
