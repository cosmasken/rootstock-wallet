use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::process;

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
    let cli = Cli::parse();
    match cli.command {
        // commands::Commands::Wallet(cmd) => cmd.execute().await?,
        // // commands::Commands::Balance { wallet, token } => {
        // //     let cmd = BalanceCommand {
        // //         address: Some(wallet),
        // //         network: String::from("mainnet"),
        // //         tokens: token.is_some()
        // //     };
        // //     cmd.execute().await?
        // // },
        commands::Commands::Contacts(cmd) => cmd.execute().await?,
    }
    Ok(())
}

fn handle_error(err: anyhow::Error) {
    eprintln!("{}: {}", "Error".red().bold(), err);
    process::exit(1);
}
