#![allow(warnings)]
use crate::commands::balance::BalanceCommand;
use crate::commands::history::HistoryCommand;
use crate::commands::transfer::TransferCommand;
use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
// use colored::Colorize;
// use std::process;

mod commands;
mod types;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: commands::Commands,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv().ok();

    let cli = Cli::parse();
    match cli.command {
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
