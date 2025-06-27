use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};

// Import types from the types module
use crate::types::network::Network;

// Import Config and ConfigManager from the config module
use crate::config::{Config, ConfigManager};

// This module provides the show_config_menu function

pub async fn show_config_menu() -> Result<()> {
    let config_manager = ConfigManager::new()?;

    loop {
        // Reload config in each iteration to show current state
        let config = config_manager.load()?;
        
        clearscreen::clear().ok();
        
        println!("\n{}", style("⚙️  Configuration").bold().blue().underlined());
        println!("{}\n", "-".repeat(40));
        
        // Show current settings
        println!("  {}", style("Current Settings:").bold());
        println!("  • Network: {}", style(config.default_network).cyan());
        
        // Get the appropriate API key based on the current network
        let api_key = match config.default_network {
            Network::Mainnet | Network::AlchemyMainnet | Network::RootStockMainnet => {
                config.alchemy_mainnet_key.as_deref().unwrap_or("Not set")
            }
            Network::Testnet | Network::AlchemyTestnet | Network::RootStockTestnet | Network::Regtest => {
                config.alchemy_testnet_key.as_deref().unwrap_or("Not set")
            }
        };
        
        println!("  • API Key: {}", style(api_key).dim());
        
        // Show default wallet if set
        if let Some(wallet) = &config.default_wallet {
            println!("  • Default Wallet: {}", style(wallet).dim());
        }
        
        let options = vec![
            format!("{}  Change Network", style("🔄").bold().yellow()),
            format!("{}  Back to Main Menu", style("⬅️").bold().blue()),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("\nWhat would you like to do?")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => change_network(&config_manager).await?,
            1 => break,
            _ => {}
        }
    }
    
    Ok(())
}

async fn change_network(config_manager: &ConfigManager) -> Result<()> {
    let mut config = config_manager.load()?;
    
    // Define all available networks with their display names
    let networks = [
        Network::Mainnet,
        Network::Testnet,
        Network::Regtest,
        Network::AlchemyMainnet,
        Network::AlchemyTestnet,
        Network::RootStockMainnet,
        Network::RootStockTestnet,
    ];
    
    let network_descriptions = [
        "Mainnet (Production, real RSK)",
        "Testnet (Test network, free test tokens)",
        "Regtest (Local development)",
        "Alchemy Mainnet (Production, Alchemy RPC)",
        "Alchemy Testnet (Test network, Alchemy RPC)",
        "Rootstock Mainnet (Production, Rootstock RPC)",
        "Rootstock Testnet (Test network, Rootstock RPC)",
    ];

    let current_network = config.default_network;
    
    // Find the current network's index
    let current_index = networks.iter()
        .position(|&n| n == current_network)
        .unwrap_or(0);
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select network:")
        .items(&network_descriptions)
        .default(current_index)
        .interact()?;

    let selected_network = networks[selection];
    
    // Always update the network, even if it's the same, to ensure consistency
    config.default_network = selected_network;
    
    // Save the updated config
    config_manager.save(&config)?;
    
    println!(
        "\n{} Network changed to: {}",
        style("✓").green().bold(),
        style(selected_network).bold()
    );
    
    // Show a brief confirmation before returning to menu
    println!("\n{}", style("Press Enter to continue...").dim());
    let _ = std::io::stdin().read_line(&mut String::new());
    
    Ok(())
}
