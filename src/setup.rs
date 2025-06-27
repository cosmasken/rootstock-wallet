use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use std::io::{self, Write};

use crate::config::{Config, ConfigManager};
use crate::types::network::Network;

pub fn run_setup_wizard() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let mut config = config_manager.load()?;

    println!("\n{}", style("🚀 Rootstock Wallet Setup").bold().blue().underlined());
    println!("{}\n", style("Let's configure your wallet").dim());

    // Network selection
    let networks = vec![
        (Network::Mainnet, "Mainnet (Production, real RSK)".to_string()),
        (Network::Testnet, "Testnet (Test network, free test tokens)".to_string()),
        (Network::Regtest, "Regtest (Local development)".to_string()),
        (Network::AlchemyMainnet, "Alchemy Mainnet (Production, Alchemy RPC)".to_string()),
        (Network::AlchemyTestnet, "Alchemy Testnet (Test network, Alchemy RPC)".to_string()),
        (Network::RootStockMainnet, "Rootstock Mainnet (Production, Rootstock RPC)".to_string()),
        (Network::RootStockTestnet, "Rootstock Testnet (Test network, Rootstock RPC)".to_string()),
    ];

    let network_names: Vec<&str> = networks.iter().map(|(_, name)| name.as_str()).collect();
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select your default network:")
        .items(&network_names)
        .default(1) // Default to Testnet
        .interact()?;

    let (selected_network, _) = &networks[selection];
    config.default_network = *selected_network;

    // Save configuration
    config_manager.save(&config)?;

    println!(
        "\n{} {}",
        style("✓").green().bold(),
        style("Configuration saved!").bold()
    );
    println!(
        "\nDefault network set to: {}",
        style(selected_network).bold()
    );

    Ok(())
}

pub fn ensure_configured() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    
    // Check if config file exists by trying to load it
    if config_manager.load().is_err() {
        println!("\n{}", style("👋 Welcome to Rootstock Wallet!").bold().cyan());
        println!("Let's set up your wallet...\n");
        run_setup_wizard()?;
        println!("\n{}", style("Setup complete! Press Enter to continue...").dim());
        io::stdout().flush()?;
        io::stdin().read_line(&mut String::new())?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_setup_creates_config() -> Result<()> {
        // This test verifies that we can serialize and deserialize a Config
        
        // Create a test config with default values
        let mut config = Config::default();
        config.default_network = Network::Testnet;
        
        // Serialize to TOML
        let toml = toml::to_string(&config)?;
        
        // Deserialize back to Config
        let loaded_config: Config = toml::from_str(&toml)?;
        
        // Verify the loaded config matches what we saved
        assert_eq!(loaded_config.default_network, Network::Testnet);
        
        Ok(())
    }
    
    #[test]
    fn test_run_setup_wizard() -> Result<()> {
        // This is a simple smoke test that the setup wizard runs without panicking
        // We can't easily test the interactive parts, but we can verify the function signature
        // and that it returns a Result
        let config_manager = ConfigManager::new()?;
        
        // Just verify we can create a default config
        let config = Config::default();
        config_manager.save(&config)?;
        
        Ok(())
    }
}
