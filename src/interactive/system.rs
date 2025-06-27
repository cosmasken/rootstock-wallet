use crate::config::ConfigManager;
use crate::types::network::Network;
use crate::utils::terminal::{self, show_version};
use anyhow::Result;
use console::style;
use dialoguer::{Select, theme::ColorfulTheme};
use std::io;

/// Helper function to get styled network status
fn get_network_status(network: &Network) -> String {
    match network {
        Network::Mainnet => style(format!("🌐 {}", network)).green().bold().to_string(),
        Network::Testnet => style(format!("🔧 {}", network)).yellow().bold().to_string(),
        _ => style(format!("❓ {}", network)).white().bold().to_string(),
    }
}

/// Helper function to get API key status
fn get_api_key_status(has_key: bool) -> String {
    if has_key {
        style("✓ Configured").green().to_string()
    } else {
        style("✗ Not configured").red().to_string()
    }
}

/// Display system information including network status and API key configuration
fn show_system_info() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?;
    
    println!("\n{}", style("System Information").bold().underlined());
    println!("• Version: {}", style(env!("CARGO_PKG_VERSION")).cyan());
    println!("• Network: {}", get_network_status(&config.default_network));
    
    // Show API key status
    match config.default_network {
        Network::Mainnet => {
            let has_key = config.alchemy_mainnet_key.as_ref().map_or(false, |k| !k.is_empty());
            println!("• Alchemy API Key: {}", get_api_key_status(has_key));
        }
        Network::Testnet => {
            let has_key = config.alchemy_testnet_key.as_ref().map_or(false, |k| !k.is_empty());
            println!("• Alchemy API Key: {}", get_api_key_status(has_key));
        }
        _ => {}
    }
    
    println!();
    Ok(())
}

/// System menu for various system-related commands
pub async fn system_menu() -> Result<()> {
    loop {
        let options = vec![
            format!("{}  Clear Screen", style("🧹").bold().cyan()),
            format!("{}  Show Version", style("ℹ️").bold().blue()),
            format!("{}  Network Status", style("🌐").bold().green()),
            format!("{}  Back to Main Menu", style("⬅️").bold().white()),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("\nSystem Menu")
            .items(&options)
            .default(0)
            .interact()?;

        let result = match selection {
            0 => {
                terminal::clear_screen();
                Ok(())
            }
            1 => {
                show_version();
                Ok(())
            }
            2 => show_system_info(),
            3 => break,
            _ => Ok(())
        };
        
        if let Err(e) = result {
            eprintln!("Error: {}", e);
            continue;
        }

        if selection < 3 {  // Don't pause after "Back"
            println!("\nPress Enter to continue...");
            let _ = io::stdin().read_line(&mut String::new())?;
        }
    }
    
    Ok(())
}
