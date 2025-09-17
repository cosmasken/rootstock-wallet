//! Rootstock Wallet CLI
//!
//! A command-line interface for managing Rootstock and ERC20 wallets.

pub mod api;
pub mod commands;
pub mod config;
pub mod interactive;
pub mod qr;
pub mod security;
pub mod types;
pub mod utils;

// Re-export secure logging macros for easy access
// Note: macros with #[macro_export] are automatically available at crate root
