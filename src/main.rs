#![allow(warnings)]
use anyhow::Result;
use clap::Parser;
use clap::CommandFactory;
use dotenv::dotenv;
use crate::commands::history::HistoryCommand;
use crate::commands::balance::BalanceCommand;
use crate::commands::transfer::TransferCommand;

mod commands;
mod interactive;
mod types;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Run in interactive mode
    #[arg(short, long)]
    interactive: bool,
    
    #[command(subcommand)]
    command: Option<commands::Commands>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv().ok();

    let cli = Cli::parse();
    
    // If no command is provided and not in interactive mode, show help
    if cli.command.is_none() && !cli.interactive {
        let _ = Cli::command().print_help();
        println!("\nRun with --interactive for interactive mode");
        return Ok(());
    }
    
    // Start interactive mode if requested
    if cli.interactive {
        return interactive::start().await;
    }
    
    // Otherwise, execute the provided command
    match cli.command.unwrap() {
        commands::Commands::Contacts(cmd) => cmd.execute().await?,
        commands::Commands::History {
            limit,
            address,
            token,
            status,
            incoming,
            outgoing,
            api_key,
            network,
        } => {
            let cmd = HistoryCommand {
                address,
                contact: None,
                limit: limit as u32,
                token,
                network,
                api_key,
                detailed: false,
                status,
                from: None,
                to: None,
                sort_by: String::from("timestamp"),
                sort_order: String::from("desc"),
                incoming,
                outgoing,
            };
            cmd.execute().await?
        }
        commands::Commands::Balance {
            address,
            network,
            token,
        } => {
            let cmd = BalanceCommand {
                address,
                network,
                token,
            };
            cmd.execute().await?
        }
        commands::Commands::Transfer {
            address,
            value,
            token,
            network,
        } => {
            let cmd = TransferCommand {
                address,
                value,
                token,
                network,
            };
            cmd.execute().await?
        }
        commands::Commands::Wallet(cmd) => cmd.execute().await?,
        commands::Commands::SetApiKey(cmd) => cmd.execute().await?,
        commands::Commands::TokenAdd(cmd) => {
            if let Err(e) = commands::tokens::add_token(&cmd.network, &cmd.symbol, &cmd.address, cmd.decimals) {
                eprintln!("Error adding token: {}", e);
            }
        }
        commands::Commands::TokenRemove(cmd) => {
            if let Err(e) = commands::tokens::remove_token(&cmd.network, &cmd.symbol) {
                eprintln!("Error removing token: {}", e);
            }
        }
        commands::Commands::TokenList(cmd) => {
            if let Err(e) = commands::tokens::list_tokens(cmd.network.as_deref()) {
                eprintln!("Error listing tokens: {}", e);
            }
        }
    }
    Ok(())
}
