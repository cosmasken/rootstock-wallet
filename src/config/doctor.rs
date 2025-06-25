use anyhow::Result;
use console::style;

use crate::config::{Config, ConfigManager, Network};

pub fn run_doctor() -> Result<()> {
    println!("\n{}", style("🩺 Running diagnostics...").bold().cyan());
    println!("{}", "=".repeat(40));

    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;

    // Check config file
    println!("\n{}", style("🔍 Configuration:").bold());
    println!("  Config file: {}", config_manager.config_path().display());
    
    if !config_manager.config_path().exists() {
        println!("  ❌ Configuration file not found");
        println!("     Run `setup` to create a new configuration");
        return Ok(());
    }

    // Check network configuration
    println!("\n{}", style("🌐 Network Configuration:").bold());
    println!("  Default network: {}", config.default_network);

    // Check API keys
    println!("\n{}", style("🔑 API Keys:").bold());
    check_api_key(&config, Network::Mainnet);
    check_api_key(&config, Network::Testnet);

    // Check wallet configuration
    println!("\n{}", style("💼 Wallet Configuration:").bold());
    if let Some(wallet) = &config.default_wallet {
        println!("  Default wallet: {}", wallet);
        // TODO: Add wallet existence check
    } else {
        println!("  ℹ️ No default wallet set");
        println!("     Run `wallet create` to create a new wallet");
    }

    println!("\n{}", style("✅ Diagnostics complete").bold().green());
    Ok(())
}

fn check_api_key(config: &Config, network: Network) {
    let key = match network {
        Network::Mainnet => &config.alchemy_mainnet_key,
        Network::Testnet => &config.alchemy_testnet_key,
    };

    let status = match key {
        Some(_) => style("✓ Configured").green(),
        None => style("✗ Missing").red(),
    };

    println!(
        "  {} API key: {}",
        match network {
            Network::Mainnet => "Mainnet",
            Network::Testnet => "Testnet",
        },
        status
    );
}