mod config;
mod setup;
mod doctor;

pub use config::{Config, ConfigManager, Network};
pub use setup::run_setup_wizard;
pub use doctor::run_doctor;

pub const ALCH_MAINNET_URL: &str = "https://dashboard.alchemy.com/apps/create?referrer=/apps";
pub const ALCH_TESTNET_URL: &str = "https://dashboard.alchemy.com/apps/create?referrer=/apps&chain=rsk-testnet";
pub const DOCS_URL: &str = "https://github.com/cosmasken/rootstock-wallet/wiki";