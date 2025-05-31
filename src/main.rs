use clap::{Parser, Subcommand};
use dotenv::dotenv;
use ethers::providers::Middleware;
use ethers::types::{Address, TransactionRequest};
use rootstock_wallet::contacts::{Contact, ContactsBook};
use rootstock_wallet::provider;
use rootstock_wallet::qr::generate_qr_code;
use rootstock_wallet::registry::{get_network_name, load_token_registry};
use rootstock_wallet::wallet::Wallet;
use rootstock_wallet::history;
use std::str::FromStr;
use std::collections::HashMap;
use serde_json::Value;

#[derive(Parser)]
#[command(name = "Rootstock Wallet")]
#[command(version = "1.0")]
#[command(about = "CLI for Rootstock RBTC operations", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
     AddToken {
        #[arg(short, long)]
        symbol: String,
        #[arg(short, long)]
        address: String,
        #[arg(short, long)]
        decimals: u8,
        #[arg(short, long, default_value = "testnet")]
        network: String, // "mainnet" or "testnet"
    },
    TransferToContact {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        amount: String,
    },
    AddContact {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        address: String,
    },
    ListContacts,
    ShowContact {
        #[arg(short, long)]
        name: String,
    },
    Transfer {
        #[arg(short, long)]
        recipient: String,

        /// Amount in RBTC (e.g. 1.5)
        #[arg(short, long)]
        amount: String,
    },
    GetBalance {
        /// Address to fetch the balance for
        #[arg(short, long)]
        address: String,
    },
    ShowWallet,
    GenerateQr {
        #[arg(short, long, default_value = "wallet_qr.png")]
        output: String,
    },
    EstimateGas {
        recipient: String,
        amount: String,
    },
    ExportPrivateKey {
        #[arg(short, long, default_value = "private.key")]
        output: String,
    },
    NetworkInfo,
    ImportWallet {
        #[arg(short, long)]
        mnemonic: Option<String>, // Optional mnemonic phrase
        #[arg(short, long)]
        private_key: Option<String>, // Optional private key
    },
    ExportKeystore {
        #[arg(short, long)]
        password: String, // Password to encrypt the keystore
        #[arg(short, long, default_value = "keystore.json")]
        output: String, // Output file
    },
    CreateMultisig {
        #[arg(short, long)]
        owners: Vec<String>, // List of owner addresses
        #[arg(short, long)]
        required: u64, // Number of required approvals
    },
    ProposeTransaction {
        #[arg(short, long)]
        multisig: String, // Multi-signature wallet address
        #[arg(short, long)]
        to: String, // Recipient address
        #[arg(short, long)]
        value: String, // Amount in RBTC
        #[arg(short, long)]
        data: Option<String>, // Optional data payload
    },
    ApproveTransaction {
        #[arg(short, long)]
        multisig: String, // Multi-signature wallet address
        #[arg(short, long)]
        tx_id: u64, // Transaction ID
    },
    TransferToken {
        #[arg(short, long)]
        token_address: String, // ERC-20 token contract address
        #[arg(short, long)]
        recipient: String, // Recipient address
        #[arg(short, long)]
        amount: String, // Amount to transfer
    },
    History {
        #[arg(short, long, default_value = "false")]
        testnet: bool,
        #[arg(short, long)]
        number: Option<String>,
    },
}

// async fn handle_transfer_token(
//     token_address: &str,
//     recipient: &str,
//     amount: &str,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider();

//     // Parse the token contract address and recipient address
//     let token_address = token_address.parse::<Address>()?;
//     let recipient = recipient.parse::<Address>()?;

//     // Parse the amount (convert to the token's smallest unit)
//     let amount: ethers::types::U256 = ethers::utils::parse_units(amount, 18)?.into(); // Adjust decimals as needed

//     // Load the ERC-20 contract ABI
//     let erc20_abi = include_str!("abi/ERC20.json");
//     let erc20_abi: ethers::abi::Abi = serde_json::from_str(erc20_abi)?;
//     let erc20_contract =
//         ethers::contract::Contract::new(token_address, erc20_abi, provider.clone().into());

//     // Create the transfer transaction
//     let tx = erc20_contract
//         .method::<(Address, ethers::types::U256), ()>("transfer", (recipient, amount))?
//         .from(wallet.address.parse::<Address>()?);

//     // Estimate gas for the transaction
//     let gas_estimate = tx.estimate_gas().await?;
//     println!("Estimated gas: {}", gas_estimate);

//     // Sign and send the transaction
//     let signed_tx = wallet.sign_transaction(&tx.tx).await?;
//     let tx_hash = provider.send_raw_transaction(signed_tx).await?;

//     println!("Transaction successful with hash: {:?}", tx_hash);

//     Ok(())
// }

async fn handle_add_token(
    symbol: &str,
    address: &str,
    decimals: u8,
    network: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "tokens.json";
    let mut data: Value = if std::path::Path::new(file_path).exists() {
        serde_json::from_str(&std::fs::read_to_string(file_path)?)?
    } else {
        serde_json::json!({ "mainnet": {}, "testnet": {} })
    };

    let network_obj = data
        .get_mut(network)
        .and_then(|v| v.as_object_mut())
        .ok_or("Invalid network (must be 'mainnet' or 'testnet')")?;

    network_obj.insert(
        symbol.to_uppercase(),
        serde_json::json!({
            "address": address,
            "decimals": decimals
        }),
    );

    std::fs::write(file_path, serde_json::to_string_pretty(&data)?)?;
    println!(
        "Token {} added to {} registry in {}.",
        symbol, network, file_path
    );
    Ok(())
}

async fn handle_transfer_token(
    token: &str, // symbol or address
    recipient: &str,
    amount: &str,
    wallet: &Wallet,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = provider::get_provider();
    let registry = load_token_registry();
    let network = get_network_name();

    // Try to resolve symbol to address/decimals
    let (token_address, decimals) = match network {
        "mainnet" => registry.mainnet.get(&token.to_uppercase()),
        "testnet" => registry.testnet.get(&token.to_uppercase()),
        _ => None,
    }
    .map(|info| (info.address.parse::<Address>().unwrap(), info.decimals))
    .unwrap_or_else(|| (token.parse::<Address>().unwrap(), 18)); // fallback: treat as address

    let recipient = recipient.parse::<Address>()?;
    let amount: ethers::types::U256 = ethers::utils::parse_units(amount, decimals as usize)?.into();

    let erc20_abi = include_str!("abi/ERC20.json");
    let erc20_abi: ethers::abi::Abi = serde_json::from_str(erc20_abi)?;
    let erc20_contract =
        ethers::contract::Contract::new(token_address, erc20_abi, provider.clone().into());

    let tx = erc20_contract
        .method::<(Address, ethers::types::U256), ()>("transfer", (recipient, amount))?
        .from(wallet.address.parse::<Address>()?);

    let gas_estimate = tx.estimate_gas().await?;
    println!("Estimated gas: {}", gas_estimate);

    let signed_tx = wallet.sign_transaction(&tx.tx).await?;
    let tx_hash = provider.send_raw_transaction(signed_tx).await?;

    println!("Transaction successful with hash: {:?}", tx_hash);

    Ok(())
}

async fn handle_transfer_to_contact(
    name: &str,
    amount: &str,
    wallet: &Wallet,
    contacts_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!(
        "Attempting transfer to contact '{}' for amount {}",
        name,
        amount
    );
    let book = ContactsBook::load(contacts_file);
    match book.get_contact(name) {
        Some(contact) => {
            log::info!("Resolved contact '{}' to address {}", name, contact.address);
            match handle_transfer(&contact.address, amount, wallet).await {
                Ok(_) => {
                    log::info!(
                        "Transfer to contact '{}' ({}) succeeded.",
                        name,
                        contact.address
                    );
                    println!(
                        "Transfer to contact '{}' ({}) succeeded.",
                        name, contact.address
                    );
                    Ok(())
                }
                Err(e) => {
                    log::error!(
                        "Transfer to contact '{}' ({}) failed: {}",
                        name,
                        contact.address,
                        e
                    );
                    println!(
                        "Transfer to contact '{}' ({}) failed: {}",
                        name, contact.address, e
                    );
                    if e.to_string().contains("nonce too low") {
                        println!(
                            "Hint: The transaction nonce is too low. You may have pending transactions or need to increment the nonce."
                        );
                    }
                    Err(e)
                }
            }
        }
        None => {
            log::error!("Contact '{}' not found.", name);
            println!("Contact '{}' not found.", name);
            Err("Contact not found".into())
        }
    }
}
// ...existing code...
// async fn handle_approve_transaction(
//     multisig: &str,
//     tx_id: u64,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider();
//     let multisig_abi = include_str!("../abi/MultisigWallet.json");
//     let multisig_address = multisig.parse::<Address>()?;
//     let multisig_contract = ethers::contract::Contract::new(multisig_address, multisig_abi.parse()?, provider);

//     let tx = multisig_contract
//         .method::<_, ()>("confirmTransaction", tx_id)?
//         .from(wallet.address.parse::<Address>()?)
//         .send()
//         .await?;

//     println!("Transaction approved with hash: {:?}", tx);
//     Ok(())
// }
// async fn handle_create_multisig(
//     owners: Vec<String>,
//     required: u64,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider();
//     let factory_abi = include_str!("../abi/MultisigFactory.json");
//     let factory_address = "0xYourFactoryAddressHere".parse::<Address>()?;
//     let factory = ethers::contract::Contract::new(factory_address, factory_abi.parse()?, provider);

//     let tx = factory
//         .method::<_, Address>("createMultisig", (owners, required))?
//         .from(wallet.address.parse::<Address>()?)
//         .send()
//         .await?;

//     println!("Multi-signature wallet deployed at: {:?}", tx);
//     Ok(())
// }
async fn handle_network_info() -> Result<(), Box<dyn std::error::Error>> {
    let provider = provider::get_provider();
    let block = provider.get_block_number().await?;
    let gas_price = provider.get_gas_price().await?;
    let chain_id = provider.get_chainid().await?;

    println!("Network Status:");
    println!("- Chain ID: {}", chain_id);
    println!("- Current Block: {}", block);
    println!("- Gas Price: {} wei", gas_price);
    Ok(())
}

// async fn handle_propose_transaction(
//     multisig: &str,
//     to: &str,
//     value: &str,
//     data: Option<String>,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider();
//     let multisig_abi = include_str!("../abi/MultisigWallet.json");
//     let multisig_address = multisig.parse::<Address>()?;
//     let multisig_contract = ethers::contract::Contract::new(multisig_address, multisig_abi.parse()?, provider);

//     let tx = multisig_contract
//         .method::<_, ()>("submitTransaction", (to.parse::<Address>()?, value.parse::<U256>()?, data.unwrap_or_default()))?
//         .from(wallet.address.parse::<Address>()?)
//         .send()
//         .await?;

//     println!("Transaction proposed with hash: {:?}", tx);
//     Ok(())
// }

async fn handle_export_key(
    wallet: &Wallet,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(output, &wallet.private_key)?;
    println!("Private key saved to {}", output);
    Ok(())
}

async fn handle_generate_qr(
    wallet: &Wallet,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    generate_qr_code(&wallet.address, output)?;
    println!("QR code saved to {}", output);
    Ok(())
}

async fn handle_show_wallet(wallet: &Wallet) {
    println!("Address: {}", wallet.address);
    println!("Public Key: {}", wallet.public_key);
    // Private key only shown in debug mode
    if cfg!(debug_assertions) {
        println!("Private Key: {}", wallet.private_key);
    }
}

async fn handle_import_wallet(
    mnemonic: Option<String>,
    private_key: Option<String>,
    wallet_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = if let Some(mnemonic) = mnemonic {
        // Import wallet from mnemonic
        Wallet::from_mnemonic(&mnemonic)?
    } else if let Some(private_key) = private_key {
        // Import wallet from private key
        Wallet::from_private_key(&private_key)?
    } else {
        return Err("Either mnemonic or private key must be provided".into());
    };

    // Save the imported wallet to a file
    wallet.save_to_file(wallet_file)?;
    println!("Wallet imported and saved to {}", wallet_file);
    Ok(())
}

// async fn handle_export_keystore(
//     wallet: &Wallet,
//     password: &str,
//     output: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let keystore = Wallet::to_keystore(&wallet.private_key, password)?;
//     std::fs::write(output, keystore)?;
//     println!("Keystore exported to {}", output);
//     Ok(())
// }

async fn handle_transfer(
    recipient: &str,
    amount: &str,
    wallet: &Wallet,
) -> Result<(), Box<dyn std::error::Error>> {
    
    let provider = provider::get_provider();

    let amount = ethers::utils::parse_units(amount, "ether")
        .map_err(|e| format!("Failed to parse amount: {}", e))?;
    let recipient = recipient
        .parse::<Address>()
        .map_err(|e| format!("Invalid recipient address: {}", e))?;
    let chain_id: u64 = std::env::var("CHAIN_ID")
        .expect("CHAIN_ID environment variable not set")
        .parse()
        .expect("Invalid CHAIN_ID value");

    let tx = ethers::types::TransactionRequest::new()
        .to(recipient)
        .value(amount)
        .from(wallet.address.parse::<Address>()?)
        .chain_id(chain_id)
        .gas(21000)
        .gas_price(
            provider
                .get_gas_price()
                .await
                .map_err(|e| format!("Failed to fetch gas price: {}", e))?,
        );

    let signed_tx = wallet
        .sign_transaction(&tx.into())
        .await
        .map_err(|e| format!("Failed to sign transaction: {}", e))?;

    let tx_hash = provider
        .send_raw_transaction(signed_tx)
        .await
        .map_err(|e| format!("Failed to send transaction: {}", e))?;
    log::info!("Transaction successful with hash: {:?}", tx_hash);

    Ok(())
}

async fn handle_get_balance(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let provider = provider::get_provider();
    let address = Address::from_str(address)?;
    let balance = provider.get_balance(address, None).await?;
    let balance_in_rbtc = ethers::utils::format_units(balance, 18)?;
    println!("Balance for {}: {} RBTC", address, balance_in_rbtc);
    Ok(())
}

async fn handle_estimate_gas(
    recipient: &str,
    amount: &str,
    wallet: &Wallet,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = provider::get_provider();
    let tx = TransactionRequest::new()
        .to(recipient.parse::<Address>()?)
        .value(ethers::utils::parse_units(amount, "ether")?)
        .from(wallet.address.parse::<Address>()?);

    let gas_estimate = provider.estimate_gas(&tx.into(), None).await?;
    println!("Estimated gas: {}", gas_estimate);
    Ok(())
}
async fn handle_add_contact(name: &str, address: &str, contacts_file: &str) {
    let mut book = ContactsBook::load(contacts_file);
    book.add_contact(name.to_string(), address.to_string());
    book.save(contacts_file);
    println!("Contact '{}' added.", name);
}

async fn handle_list_contacts(contacts_file: &str) {
    let book = ContactsBook::load(contacts_file);
    for contact in book.list_contacts() {
        println!("{}: {}", contact.name, contact.address);
    }
}

async fn handle_show_contact(name: &str, contacts_file: &str) {
    let book = ContactsBook::load(contacts_file);
    if let Some(contact) = book.get_contact(name) {
        println!("{}: {}", contact.name, contact.address);
    } else {
        println!("Contact '{}' not found.", name);
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init(); // Initialize the logger
    dotenv().ok();

    let wallet_file = "wallet.json";

    let cli = Cli::parse();

    let wallet = Wallet::load_from_file(wallet_file).unwrap_or_else(|| {
        log::info!("No wallet found. Generating new one...");
        let new_wallet = Wallet::generate();
        new_wallet
            .save_to_file(wallet_file)
            .expect("Failed to save wallet");
        log::info!("New wallet generated and saved to {}", wallet_file);
        new_wallet
    });
    let contacts_file = "contacts.json";

    match cli.command {
        Commands::AddToken {
            symbol,
            address,
            decimals,
            network,
        } => {
            handle_add_token(&symbol, &address, decimals, &network).await?
        }
        Commands::TransferToContact { name, amount } => {
            log::info!("Transferring to contact: {}", name);
            handle_transfer_to_contact(&name, &amount, &wallet, contacts_file).await?
        }
        Commands::AddContact { name, address } => {
            handle_add_contact(&name, &address, contacts_file).await
        }
        Commands::ListContacts => handle_list_contacts(contacts_file).await,
        Commands::ShowContact { name } => handle_show_contact(&name, contacts_file).await,
        Commands::Transfer { recipient, amount } => {
            log::info!("Executing transfer command...");
            handle_transfer(&recipient, &amount, &wallet).await?
        }
        Commands::GetBalance { address } => {
            log::info!("Fetching balance for address: {}", address);
            handle_get_balance(&address).await?
        }
        Commands::ShowWallet => {
            log::info!("Displaying wallet information...");
            handle_show_wallet(&wallet).await
        }
        Commands::GenerateQr { output } => {
            log::info!("Generating QR code for wallet address...");
            handle_generate_qr(&wallet, &output).await?
        }
        Commands::EstimateGas { recipient, amount } => {
            log::info!(
                "Estimating gas for recipient: {}, amount: {}",
                recipient,
                amount
            );
            handle_estimate_gas(&recipient, &amount, &wallet).await?
        }
        Commands::ExportPrivateKey { output } => {
            log::info!("Exporting private key to file: {}", output);
            handle_export_key(&wallet, &output).await?
        }
        Commands::NetworkInfo => {
            log::info!("Fetching network information...");
            handle_network_info().await?
        }
        Commands::ImportWallet {
            mnemonic,
            private_key,
        } => {
            log::info!("Importing wallet...");
            handle_import_wallet(mnemonic, private_key, wallet_file).await?
        }
        Commands::ExportKeystore { password, output } => {
            log::info!("Exporting keystore...");
            // handle_export_keystore(&wallet, &password, &output).await?
        }
        Commands::CreateMultisig { owners, required } => {
            log::info!("Creating multi-signature wallet...");
            // handle_create_multisig(owners, required, &wallet).await?
        }
        Commands::ProposeTransaction {
            multisig,
            to,
            value,
            data,
        } => {
            log::info!("Proposing transaction...");
            // handle_propose_transaction(&multisig, &to, &value, data, &wallet).await?
        }
        Commands::ApproveTransaction { multisig, tx_id } => {
            log::info!("Approving transaction...");
            // handle_approve_transaction(&multisig, tx_id, &wallet).await?
        }

        Commands::TransferToken {
            token_address,
            recipient,
            amount,
        } => {
            log::info!("Transferring ERC-20 tokens...");
            handle_transfer_token(&token_address, &recipient, &amount, &wallet).await?
        }

        Commands::History { testnet, number } => {
            log::info!("Fetching transaction history...");
            history::history_command(testnet, number, &wallet.address).await?
        }
    }

    Ok(())
}
