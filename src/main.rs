use crate::commands::balance::BalanceCommand;
use crate::commands::history::HistoryCommand;
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
        } => {
            let cmd = HistoryCommand {
                address,
                contact: None,
                limit: limit as u32,
                token,
                // network: String::from("mainnet"),
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
        },
        commands::Commands::Balance { address, network, tokens } => {
            let cmd = BalanceCommand { address, network, tokens };
            cmd.execute().await?
        },
        commands::Commands::Transfer { to, amount, token, wallet } =>{
            todo!()
        }
         commands::Commands::Wallet(cmd) => cmd.execute().await?,
       
    }
    Ok(())
}
