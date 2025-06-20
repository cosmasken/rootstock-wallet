use crate::commands::contacts::ContactsCommand;
use crate::commands::wallet::WalletCommand;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Commands {
    /// Manage wallets
    Wallet(WalletCommand),
    /// Manage contacts
    Contacts(ContactsCommand),
    /// Show transaction history
    History {
        #[arg(short, long, default_value = "10")]
        limit: usize,
        #[arg(short, long)]
        address: Option<String>,
        #[arg(short, long)]
        token: Option<String>,
        #[arg(short, long)]
        status: Option<String>,
        #[arg(short, long)]
        incoming: bool,
        #[arg(short, long)]
        outgoing: bool,
    },
    /// Check balance of an address
    Balance {
        /// Address to check balance for
        #[arg(short, long)]
        address: String,
        /// Network to use (mainnet/testnet)
        #[arg(short, long, default_value = "mainnet")]
        network: String,
        /// Show token balances
        #[arg(short, long)]
        tokens: Option<bool>,
    },
      /// Transfer RBTC or tokens
    Transfer {
        /// Address to send to
        #[arg( long, required = true)]
        to: String,
        /// Amount to send (in RBTC or token units)
        #[arg(short, long, required = true)]
        amount: f64,
        /// Token address (for ERC20 transfers)
        #[arg( long)]
        token: Option<String>,
        #[arg(short, long, default_value = "default")]
        wallet: String,
    },
}
