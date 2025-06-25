use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::config::{Config, ConfigManager, Network, ALCH_MAINNET_URL, ALCH_TESTNET_URL, DOCS_URL};

pub fn run_setup_wizard() -> Result<()> {
    println!("\n{}", style("🌟 Welcome to Rootstock Wallet CLI!").bold().cyan());
    println!("{}", "=".repeat(40));
    println!("\nLet's get you set up with the basic configuration.\n");

    let config_manager = ConfigManager::new()?;
    let mut config = config_manager.load()?;

    // Network selection
    let networks = &["Testnet (recommended for testing)", "Mainnet (for real funds)"];
    let network_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select your default network")
        .default(0)
        .items(networks)
        .interact()?;

    let network = if network_idx == 0 {
        Network::Testnet
    } else {
        Network::Mainnet
    };

    config.default_network = network;

    // API Key setup
    setup_api_keys(&mut config, network)?;

    // Save configuration
    config_manager.save(&config)?;

    println!("\n{}", style("✅ Setup complete!").bold().green());
    println!("\nYou can now use the wallet. For more information, visit:");
    println!("{}", style(DOCS_URL).blue().underlined());
    println!("\nRun `rootstock-wallet --help` to see available commands.");

    Ok(())
}

fn setup_api_keys(config: &mut Config, network: Network) -> Result<()> {
    println!("\n{}", style("🔑 API Key Setup").bold().cyan());
    println!("{}", "=".repeat(40));

    let key_type = match network {
        Network::Mainnet => "mainnet",
        Network::Testnet => "testnet",
    };

    println!(
        "\nYou'll need an Alchemy API key for {}.",
        style(key_type).bold()
    );
    println!("\nIf you don't have one, get it from:");
    let url = match network {
        Network::Mainnet => ALCH_MAINNET_URL,
        Network::Testnet => ALCH_TESTNET_URL,
    };
    println!("{}", style(url).blue().underlined());

    let key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Enter your Alchemy {} API key", key_type))
        .interact_text()?;

    match network {
        Network::Mainnet => config.alchemy_mainnet_key = Some(key),
        Network::Testnet => config.alchemy_testnet_key = Some(key),
    }

    // Ask if they want to set up the other network too
    let other_network = match network {
        Network::Mainnet => {
            println!("\nWould you like to set up a testnet API key as well?");
            Network::Testnet
        }
        Network::Testnet => {
            println!("\nWould you like to set up a mainnet API key as well?");
            Network::Mainnet
        }
    };

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Set up {} API key now?",
            match other_network {
                Network::Mainnet => "mainnet",
                Network::Testnet => "testnet",
            }
        ))
        .interact()?
    {
        setup_api_keys(config, other_network)?;
    }

    Ok(())
}