// use clap::{Parser, Subcommand};
// use dotenv::dotenv;
// use ethers::providers::Middleware;
// use ethers::types::{Address, TransactionRequest};
// use rootstock_wallet::contacts::{Contact, ContactsBook};
// use rootstock_wallet::provider;
// use rootstock_wallet::qr::generate_qr_code;
// use rootstock_wallet::registry::{get_network_name, load_token_registry};
// use rootstock_wallet::wallet::Wallet;
// use rootstock_wallet::history;
// use std::str::FromStr;
// use std::collections::HashMap;
// use serde_json::{to_string, Value};
// use std::fs::{File, OpenOptions};
// use std::io::{BufReader, BufWriter};
// use serde::{Serialize, Deserialize};
// use colored::*;
// use colored::Colorize;
// use log::{info, error};
// use reqwest;
// use serde_json::json;

// // Define JsonRpcRequest and JsonRpcResponse structs
// #[derive(Debug, Serialize, Deserialize)]
// struct JsonRpcRequest {
//     json_rpc: String,
//     id: u64,
//     method: String,
//     params: Vec<serde_json::Value>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct JsonRpcResponse {
//     json_rpc: String,
//     id: u64,
//     result: Option<String>,
//     error: Option<JsonRpcError>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct JsonRpcError {
//     code: i64,
//     message: String,
// }

// #[derive(Parser)]
// #[command(name = "Rootstock Wallet")]
// #[command(version = "1.0")]
// #[command(about = "CLI for Rootstock RBTC operations", long_about = None)]
// struct Cli {
//     #[command(subcommand)]
//     command: Commands,
    
//     /// Network to use (mainnet or testnet)
//     #[arg(short, long, default_value = "testnet")]
//     network: String,
// }

// #[derive(Subcommand)]
// enum Commands {
//      AddToken {
//         #[arg(short, long)]
//         symbol: String,
//         #[arg(short, long)]
//         address: String,
//         #[arg(short, long)]
//         decimals: u8,
//         #[arg(short, long, default_value = "testnet")]
//         network: String, // "mainnet" or "testnet"
//     },
//     TransferToContact {
//         #[arg(short, long)]
//         name: String,
//         #[arg(short, long)]
//         amount: String,
//     },
//     AddContact {
//         #[arg(short, long)]
//         name: String,
//         #[arg(short, long)]
//         address: String,
//     },
//     ListContacts,
//     ShowContact {
//         #[arg(short, long)]
//         name: String,
//     },
//     Transfer {
//         #[arg(short, long)]
//         recipient: String,

//         /// Amount in RBTC (e.g. 1.5)
//         #[arg(short, long)]
//         amount: String,
//     },
//     GetBalance {
//         /// Address to fetch the balance for
//         #[arg(short, long)]
//         address: String,
//     },
//     ShowWallet,
//     GenerateQr {
//         #[arg(short, long, default_value = "wallet_qr.png")]
//         output: String,
//     },
//     EstimateGas {
//         recipient: String,
//         amount: String,
//     },
//     ExportPrivateKey {
//         #[arg(short, long, default_value = "private.key")]
//         output: String,
//     },
//     NetworkInfo,
//     ImportWallet {
//         #[arg(short, long)]
//         mnemonic: Option<String>, // Optional mnemonic phrase
//         #[arg(short, long)]
//         private_key: Option<String>, // Optional private key
//     },
//     ExportKeystore {
//         #[arg(short, long)]
//         password: String, // Password to encrypt the keystore
//         #[arg(short, long, default_value = "keystore.json")]
//         output: String, // Output file
//     },
//     CreateMultisig {
//         #[arg(short, long)]
//         owners: Vec<String>, // List of owner addresses
//         #[arg(short, long)]
//         required: u64, // Number of required approvals
//     },
//     ProposeTransaction {
//         #[arg(short, long)]
//         multisig: String, // Multi-signature wallet address
//         #[arg(short, long)]
//         to: String, // Recipient address
//         #[arg(short, long)]
//         value: String, // Amount in RBTC
//         #[arg(short, long)]
//         data: Option<String>, // Optional data payload
//     },
//     ApproveTransaction {
//         #[arg(short, long)]
//         multisig: String, // Multi-signature wallet address
//         #[arg(short, long)]
//         tx_id: u64, // Transaction ID
//     },
//     TransferToken {
//         #[arg(short, long)]
//         token_address: String, // ERC-20 token contract address
//         #[arg(short, long)]
//         recipient: String, // Recipient address
//         #[arg(short, long)]
//         amount: String, // Amount to transfer
//     },
//     History {
//         /// Address to fetch history for
//         #[arg(short, long)]
//         address: String,
        
//         /// Number of transactions to show
//         #[arg(short, long, default_value = "10")]
//         limit: u32,
//     },
//     TokenBalance {
//         /// Address to check balance for
//         #[arg(short, long)]
//         address: String,
        
//         /// Token symbol (e.g. RIF, USDRIF)
//         #[arg(short, long)]
//         token: String,
//     },
//     Contact {
//         /// Action to perform (add, list, show, delete)
//         #[arg(short, long)]
//         action: String,
        
//         /// Name of the contact
//         #[arg(short, long)]
//         name: Option<String>,
        
//         /// Address of the contact
//         #[arg(short, long)]
//         address: Option<String>,
//     },
//     ListPendingTxs {
//         #[arg(short, long)]
//         multisig: String,
//     },
// }

// async fn handle_list_pending_txs(
//     multisig: &str,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider(&cli.network);
//     let abi: Abi = serde_json::from_str(include_str!("abi/MultiSigWallet.json"))?;
//     let contract = Contract::new(multisig.parse::<Address>()?, abi, provider.clone());
    
//     let tx_ids: Vec<U256> = contract
//         .method::<(U256, U256, bool), Vec<U256>>(
//             "getTransactionIds",
//             (U256::zero(), U256::from(u64::MAX), true),
//         )?
//         .call()
//         .await?;
    
//     for tx_id in tx_ids {
//         let tx: (Address, U256, Bytes, bool, U256) = contract
//             .method::<U256, (Address, U256, Bytes, bool, U256)>("transactions", tx_id)?
//             .call()
//             .await?;
//         println!("Transaction ID: {}", tx_id);
//         println!("To: {:?}", tx.0);
//         println!("Value: {} RBTC", ethers::utils::format_units(tx.1, "ether")?);
//         println!("Executed: {}", tx.3);
//         println!("Confirmations: {}", tx.4);
//         println!("{}", "-".repeat(50).blue());
//     }
    
//     Ok(())
// }

// fn handle_contacts_file(contacts_file: &str, action: &str, name: Option<&str>, address: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
//     let mut contacts: Vec<Contact> = match File::open(contacts_file) {
//         Ok(file) => serde_json::from_reader(BufReader::new(file))?,
//         Err(_) => Vec::new(), // Create new if file doesn't exist
//     };

//     match action {
//         "add" => {
//             if let (Some(name), Some(address)) = (name, address) {
//                 contacts.push(Contact {
//                     name: name.to_string(),
//                     address: address.to_string(),
//                 });
                
//                 let file = OpenOptions::new()
//                     .write(true)
//                     .create(true)
//                     .open(contacts_file)?;
//                 serde_json::to_writer_pretty(&mut BufWriter::new(file), &contacts)?;
//                 println!("Contact '{}' added successfully!", name);
//             } else {
//                 return Err("Both name and address are required to add a contact".into());
//             }
//         }
//         "list" => {
//             if contacts.is_empty() {
//                 println!("No contacts found.");
//             } else {
//                 println!("{}", "Contacts".bold());
//                 println!("{}", "-".repeat(50).blue());
//                 for contact in &contacts {
//                     println!("Name: {}", contact.name);
//                     println!("Address: {}", contact.address);
//                     println!("{}", "-".repeat(50).blue());
//                 }
//             }
//         }
//         "show" => {
//             if let Some(name) = name {
//                 if let Some(contact) = contacts.iter().find(|c| c.name == name) {
//                     println!("{}", format!("Contact: {}", name).bold());
//                     println!("Address: {}", contact.address);
//                     println!("{}", "-".repeat(50).blue());
//                 } else {
//                     println!("Contact '{}' not found", name);
//                 }
//             } else {
//                 return Err("Name is required to show a contact".into());
//             }
//         }
//         "delete" => {
//             if let Some(name) = name {
//                 contacts.retain(|c| c.name != name);
//                 let file = OpenOptions::new()
//                     .write(true)
//                     .create(true)
//                     .open(contacts_file)?;
//                 serde_json::to_writer_pretty(&mut BufWriter::new(file), &contacts)?;
//                 println!("Contact '{}' deleted successfully!", name);
//             } else {
//                 return Err("Name is required to delete a contact".into());
//             }
//         }
//         _ => return Err(format!("Unknown action: {}", action).into()),
//     }

//     Ok(())
// }

// async fn handle_add_token(
//     symbol: &str,
//     address: &str,
//     decimals: u8,
//     network: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let file_path = "tokens.json";
//     let mut data: Value = if std::path::Path::new(file_path).exists() {
//         serde_json::from_str(&std::fs::read_to_string(file_path)?)?
//     } else {
//         serde_json::json!({ "mainnet": {}, "testnet": {} })
//     };

//     let network_obj = data
//         .get_mut(network)
//         .and_then(|v| v.as_object_mut())
//         .ok_or("Invalid network (must be 'mainnet' or 'testnet')")?;

//     network_obj.insert(
//         symbol.to_uppercase(),
//         serde_json::json!({
//             "address": address,
//             "decimals": decimals
//         }),
//     );

//     std::fs::write(file_path, serde_json::to_string_pretty(&data)?)?;
//     println!(
//         "Token {} added to {} registry in {}.",
//         symbol, network, file_path
//     );
//     Ok(())
// }

// async fn handle_transfer_token(
//     token: &str, // symbol or address
//     recipient: &str,
//     amount: &str,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider();
//     let registry = load_token_registry();
//     // let network = get_network_name();
    

//     // Try to resolve symbol to address/decimals
//     let (token_address, decimals) = match network {
//         "mainnet" => registry.mainnet.get(&token.to_uppercase()),
//         "testnet" => registry.testnet.get(&token.to_uppercase()),
//         _ => None,
//     }
//     .map(|info| (info.address.parse::<Address>().unwrap(), info.decimals))
//     .unwrap_or_else(|| (token.parse::<Address>().unwrap(), 18)); // fallback: treat as address

//     let recipient = recipient.parse::<Address>()?;
//     let amount: ethers::types::U256 = ethers::utils::parse_units(amount, decimals as usize)?.into();

//     let erc20_abi = include_str!("abi/ERC20.json");
//     let erc20_abi: ethers::abi::Abi = serde_json::from_str(erc20_abi)?;
//     let erc20_contract =
//         ethers::contract::Contract::new(token_address, erc20_abi, provider.clone().into());

//     let tx = erc20_contract
//         .method::<(Address, ethers::types::U256), ()>("transfer", (recipient, amount))?
//         .from(wallet.address.parse::<Address>()?);

//     let gas_estimate = tx.estimate_gas().await?;
//     println!("Estimated gas: {}", gas_estimate);

//     let signed_tx = wallet.sign_transaction(&tx.tx).await?;
//     let tx_hash = provider.send_raw_transaction(signed_tx).await?;

//     println!("Transaction successful with hash: {:?}", tx_hash);

//     Ok(())
// }

// async fn handle_network_info() -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider();
//     let block = provider.get_block_number().await?;
//     let gas_price = provider.get_gas_price().await?;
//     let chain_id = provider.get_chainid().await?;

//     println!("Network Status:");
//     println!("- Chain ID: {}", chain_id);
//     println!("- Current Block: {}", block);
//     println!("- Gas Price: {} wei", gas_price);
//     Ok(())
// }

// async fn handle_export_key(
//     wallet: &Wallet,
//     output: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     std::fs::write(output, &wallet.private_key)?;
//     println!("Private key saved to {}", output);
//     Ok(())
// }

// async fn handle_generate_qr(
//     wallet: &Wallet,
//     output: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     generate_qr_code(&wallet.address, output)?;
//     println!("QR code saved to {}", output);
//     Ok(())
// }

// async fn handle_show_wallet(wallet: &Wallet) {
//     println!("Address: {}", wallet.address);
//     println!("Public Key: {}", wallet.public_key);
//     // Private key only shown in debug mode
//     if cfg!(debug_assertions) {
//         println!("Private Key: {}", wallet.private_key);
//     }
// }

// async fn handle_import_wallet(
//     mnemonic: Option<String>,
//     private_key: Option<String>,
//     wallet_file: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let wallet = if let Some(mnemonic) = mnemonic {
//         // Import wallet from mnemonic
//         Wallet::from_mnemonic(&mnemonic)?
//     } else if let Some(private_key) = private_key {
//         // Import wallet from private key
//         Wallet::from_private_key(&private_key)?
//     } else {
//         return Err("Either mnemonic or private key must be provided".into());
//     };

//     // Save the imported wallet to a file
//     wallet.save_to_file(wallet_file)?;
//     println!("Wallet imported and saved to {}", wallet_file);
//     Ok(())
// }

//  async fn handle_transfer_to_contact(
//     name: &str,
//     amount: &str,
//     wallet: &Wallet,
//     contacts_file: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     log::info!(
//         "Attempting transfer to contact '{}' for amount {}",
//         name,
//         amount
//     );
//     let book = ContactsBook::load(contacts_file);
//     match book.get_contact(name) {
//         Some(contact) => {
//             log::info!("Resolved contact '{}' to address {}", name, contact.address);
//             match handle_transfer(&contact.address, amount, wallet).await {
//                 Ok(_) => {
//                     log::info!(
//                         "Transfer to contact '{}' ({}) succeeded.",
//                         name,
//                         contact.address
//                     );
//                     println!(
//                         "Transfer to contact '{}' ({}) succeeded.",
//                         name, contact.address
//                     );
//                     Ok(())
//                 }
//                 Err(e) => {
//                     log::error!(
//                         "Transfer to contact '{}' ({}) failed: {}",
//                         name,
//                         contact.address,
//                         e
//                     );
//                     println!(
//                         "Transfer to contact '{}' ({}) failed: {}",
//                         name, contact.address, e
//                     );
//                     if e.to_string().contains("nonce too low") {
//                         println!(
//                             "Hint: The transaction nonce is too low. You may have pending transactions or need to increment the nonce."
//                         );
//                     }
//                     Err(e)
//                 }
//             }
//         }
//         None => {
//             log::error!("Contact '{}' not found.", name);
//             println!("Contact '{}' not found.", name);
//             Err("Contact not found".into())
//         }
//     }
// }

// async fn handle_export_keystore(
//     wallet: &Wallet,
//     password: &str,
//     output: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let keystore = Wallet::encrypt(wallet, password)?;
//     std::fs::write(output, keystore)?;
//     println!("Keystore exported to {}", output);
//     Ok(())
// }

// async fn handle_transfer(
//     recipient: &str,
//     amount: &str,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
    
//     let provider = provider::get_provider();

//     let amount = ethers::utils::parse_units(amount, "ether")
//         .map_err(|e| format!("Failed to parse amount: {}", e))?;
//     let recipient = recipient
//         .parse::<Address>()
//         .map_err(|e| format!("Invalid recipient address: {}", e))?;
//      let chain_id = provider.get_chainid().await.map_err(|e| format!("Failed to fetch chain ID: {}", e))?;

//     let tx = ethers::types::TransactionRequest::new()
//         .to(recipient)
//         .value(amount)
//         .from(wallet.address.parse::<Address>()?)
//         .chain_id(chain_id)
//         .gas(21000)
//         .gas_price(
//             provider
//                 .get_gas_price()
//                 .await
//                 .map_err(|e| format!("Failed to fetch gas price: {}", e))?,
//         );

//     let signed_tx = wallet
//         .sign_transaction(&tx.into())
//         .await
//         .map_err(|e| format!("Failed to sign transaction: {}", e))?;

//     let tx_hash = provider
//         .send_raw_transaction(signed_tx)
//         .await
//         .map_err(|e| format!("Failed to send transaction: {}", e))?;
//     log::info!("Transaction successful with hash: {:?}", tx_hash);

//     Ok(())
// }

// async fn handle_get_balance(address: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider();
//     let address = Address::from_str(address)?;
//     let balance = provider.get_balance(address, None).await?;
//     let balance_in_rbtc = ethers::utils::format_units(balance, 18)?;
//     println!("Balance for {}: {} RBTC", address, balance_in_rbtc);
//     Ok(())
// }

// async fn handle_estimate_gas(
//     recipient: &str,
//     amount: &str,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider();
//     let tx = TransactionRequest::new()
//         .to(recipient.parse::<Address>()?)
//         .value(ethers::utils::parse_units(amount, "ether")?)
//         .from(wallet.address.parse::<Address>()?);

//     let gas_estimate = provider.estimate_gas(&tx.into(), None).await?;
//     println!("Estimated gas: {}", gas_estimate);
//     Ok(())
// }

// async fn handle_token_balance(
//     address: &str,
//     token: &str,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     dotenv::dotenv().ok();
//     let api_key = std::env::var("ALCHEMY_API_KEY")
//         .map_err(|_| "ALCHEMY_API_KEY environment variable not set")?;
//     let base_url = std::env::var("ALCHEMY_RPC_URL")
//         .map_err(|_| "ALCHEMY_RPC_URL environment not set")?;

//     // Get token registry
//     let token_registry = load_token_registry().await?;
//     let token_address = token_registry
//         .get(token)
//         .ok_or_else(|| format!("Token {} not found in registry", token))?;

//     // Create JSON-RPC request
//     let client = reqwest::Client::new();
//     let request = JsonRpcRequest {
//         json_rpc: "2.0".to_string(),
//         id: 1,
//         method: "eth_call".to_string(),
//         params: vec![
//             json!({
//                 "to": token_address,
//                 "data": format!(
//                     "0x70a08231000000000000000000000000{}",
//                     address[2..].to_lowercase()
//                 )
//             }),
//             serde_json::Value::String("latest".to_owned())
//         ],
//     };

//     // Send request
//     let response = client
//         .post(&base_url)
//         .json(&request)
//         .send()
//         .await?
//         .json::<JsonRpcResponse>()
//         .await?;

//     if let Some(error) = response.error {
//         return Err(error.message.into());
//     }

//     if let Some(result) = response.result {
//         let balance = result
//             .as_str()
//             .ok_or_else(|| "Invalid response format".to_string())?
//             .to_string();
//         // Convert hex balance to decimal
//         let balance = i128::from_str_radix(&balance[2..], 16)?;
        
//         // Get token decimals from registry
//         let decimals = token_registry
//             .get_decimals(token)
//             .ok_or_else(|| format!("Token {} decimals not found", token))?;
        
//         // Format balance with decimals
//         let formatted_balance = format!("{}", balance as f64 / 10f64.powi(decimals as i32));
        
//         println!("{}", format!("Balance for {}:", token).bold());
//         println!("{} {}", formatted_balance, token);
//         println!("{}", "-".repeat(50).blue());
//     }

//     Ok(())
// }

// async fn handle_add_contact(name: &str, address: &str, contacts_file: &str) {
//     let mut book = ContactsBook::load(contacts_file);
//     book.add_contact(name.to_string(), address.to_string());
//     book.save(contacts_file);
//     println!("Contact '{}' added.", name);
// }

// async fn handle_list_contacts(contacts_file: &str) {
//     let book = ContactsBook::load(contacts_file);
//     for contact in book.list_contacts() {
//         println!("{}: {}", contact.name, contact.address);
//     }
// }

// async fn handle_show_contact(name: &str, contacts_file: &str) {
//     let book = ContactsBook::load(contacts_file);
//     if let Some(contact) = book.get_contact(name) {
//         println!("{}: {}", contact.name, contact.address);
//     } else {
//         println!("Contact '{}' not found.", name);
//     }
// }

// use ethers::abi::Abi;
// use ethers::contract::Contract;
// use ethers::types::{Address, U256};

// async fn handle_create_multisig(
//     owners: Vec<String>,
//     required: u64,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider(&cli.network);
//     let abi: Abi = serde_json::from_str(include_str!("abi/MultiSigWallet.json"))?;
//     let bytecode = include_bytes!("bytecode/MultiSigWallet.bin");
    
//     let owners: Vec<Address> = owners.iter().map(|o| o.parse::<Address>().unwrap()).collect();
//     let factory = Contract::new(Address::zero(), abi, provider.clone());
    
//     let deploy_tx = factory
//         .deploy((owners, U256::from(required)))?
//         .from(wallet.address.parse::<Address>()?);
    
//     let receipt = deploy_tx.send().await?;
//     let contract_address = receipt.contract_address.unwrap();
    
//     println!("Multi-sig wallet deployed at: {:?}", contract_address);
//     Ok(())
// }
// async fn handle_propose_transaction(
//     multisig: &str,
//     to: &str,
//     value: &str,
//     data: Option<String>,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider(&cli.network);
//     let abi: Abi = serde_json::from_str(include_str!("abi/MultiSigWallet.json"))?;
//     let contract = Contract::new(multisig.parse::<Address>()?, abi, provider.clone());
    
//     let value = ethers::utils::parse_units(value, "ether")?;
//     let data = data.unwrap_or_default().parse::<Bytes>()?;
    
//     let tx = contract
//         .method::<(Address, U256, Bytes), U256>("submitTransaction", (
//             to.parse::<Address>()?,
//             value,
//             data,
//         ))?
//         .from(wallet.address.parse::<Address>()?);
    
//     let receipt = tx.send().await?;
//     println!("Transaction proposed with ID: {:?}", receipt.transaction_index);
//     Ok(())
// }
// async fn handle_approve_transaction(
//     multisig: &str,
//     tx_id: u64,
//     wallet: &Wallet,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let provider = provider::get_provider(&cli.network);
//     let abi: Abi = serde_json::from_str(include_str!("abi/MultiSigWallet.json"))?;
//     let contract = Contract::new(multisig.parse::<Address>()?, abi, provider.clone());
    
//     let tx = contract
//         .method::<U256, ()>("confirmTransaction", U256::from(tx_id))?
//         .from(wallet.address.parse::<Address>()?);
    
//     let receipt = tx.send().await?;
//     println!("Transaction {} approved: {:?}", tx_id, receipt.transaction_hash);
//     Ok(())
// }
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     env_logger::init(); // Initialize the logger
//     dotenv().ok();

//     let wallet_file = "wallet.json";

//     let cli = Cli::parse();

//     let wallet = Wallet::load_from_file(wallet_file).unwrap_or_else(|| {
//         log::info!("No wallet found. Generating new one...");
//         let new_wallet = Wallet::generate();
//         new_wallet
//             .save_to_file(wallet_file)
//             .expect("Failed to save wallet");
//         log::info!("New wallet generated and saved to {}", wallet_file);
//         new_wallet
//     });
//     let contacts_file = "contacts.json";

//     match cli.command {
//         Commands::AddToken {
//             symbol,
//             address,
//             decimals,
//             network,
//         } => {
//             handle_add_token(&symbol, &address, decimals, &network).await?
//         }
//         Commands::TransferToContact { name, amount } => {
//             log::info!("Transferring to contact: {}", name);
//             handle_transfer_to_contact(&name, &amount, &wallet, contacts_file).await?
//         }
//         Commands::AddContact { name, address } => {
//             handle_add_contact(&name, &address, contacts_file).await
//         }
//         Commands::ListContacts => handle_list_contacts(contacts_file).await,
//         Commands::ShowContact { name } => handle_show_contact(&name, contacts_file).await,
//         Commands::Transfer { recipient, amount } => {
//             log::info!("Executing transfer command...");
//             handle_transfer(&recipient, &amount, &wallet).await?
//         }
//         Commands::GetBalance { address } => {
//             log::info!("Fetching balance for address: {}", address);
//             handle_get_balance(&address).await?
//         }
//         Commands::ShowWallet => {
//             log::info!("Displaying wallet information...");
//             handle_show_wallet(&wallet).await
//         }
//         Commands::GenerateQr { output } => {
//             log::info!("Generating QR code for wallet address...");
//             handle_generate_qr(&wallet, &output).await?
//         }
//         Commands::EstimateGas { recipient, amount } => {
//             log::info!(
//                 "Estimating gas for recipient: {}, amount: {}",
//                 recipient,
//                 amount
//             );
//             handle_estimate_gas(&recipient, &amount, &wallet).await?
//         }
//         Commands::ExportPrivateKey { output } => {
//             log::info!("Exporting private key to file: {}", output);
//             handle_export_key(&wallet, &output).await?
//         }
//         Commands::NetworkInfo => {
//             log::info!("Fetching network information...");
//             handle_network_info().await?
//         }
//         Commands::ImportWallet {
//             mnemonic,
//             private_key,
//         } => {
//             log::info!("Importing wallet...");
//             handle_import_wallet(mnemonic, private_key, wallet_file).await?
//         }
//         Commands::ExportKeystore { password, output } => {
//             log::info!("Exporting keystore...");
//             handle_export_keystore(&wallet, &password, &output).await?
//         }
//         Commands::CreateMultisig { owners, required } => {
//             log::info!("Creating multi-signature wallet...");
//             // handle_create_multisig(owners, required, &wallet).await?
//         }
//         Commands::ProposeTransaction {
//             multisig,
//             to,
//             value,
//             data,
//         } => {
//             log::info!("Proposing transaction...");
//             // handle_propose_transaction(&multisig, &to, &value, data, &wallet).await?
//         }
//         Commands::ApproveTransaction { multisig, tx_id } => {
//             log::info!("Approving transaction...");
//             // handle_approve_transaction(&multisig, tx_id, &wallet).await?
//         }

//         Commands::TransferToken {
//             token_address,
//             recipient,
//             amount,
//         } => {
//             log::info!("Transferring ERC-20 tokens...");
//             handle_transfer_token(&token_address, &recipient, &amount, &wallet).await?
//         }

//         Commands::History { address, limit } => {
//             log::info!("Fetching transaction history...");
//             history::history_command(false, Some(&address), Some(limit), &wallet.address).await?
//         }
//         Commands::TokenBalance { address, token } => {
//             log::info!("Fetching token balance...");
//             handle_token_balance(&address, &token, &wallet).await?
//         }
//         Commands::Contact { action, name, address } => {
//             handle_contacts_file("contacts.json", &action, name.as_deref(), address.as_deref())?
//         }
//     }

//     Ok(())
// }
use clap::{Parser, Subcommand};
use colored::*;
use dotenv::dotenv;
use ethers::abi::Abi;
use ethers::contract::Contract;
use ethers::providers::Middleware;
use ethers::types::{Address, Bytes, TransactionRequest, U256};
use indicatif::ProgressBar;
use log::{error, info};
use reqwest;
use rootstock_wallet::contacts::{Contact, ContactsBook};
use rootstock_wallet::history;
use rootstock_wallet::provider::{get_provider, validate_network};
use rootstock_wallet::qr::generate_qr_code;
use rootstock_wallet::registry::{get_network_name, load_token_registry};
use rootstock_wallet::wallet::Wallet;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, Value};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    result: Option<String>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[derive(Parser)]
#[command(name = "Rootstock Wallet")]
#[command(version = "1.0")]
#[command(about = "CLI for Rootstock RBTC operations", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Network to use (mainnet or testnet)
    #[arg(short, long, default_value = "testnet")]
    network: String,
    /// Custom RPC URL (overrides RPC_URL env var)
    #[arg(long)]
    rpc_url: Option<String>,
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
        network: String,
    },
    RemoveToken {
        #[arg(short, long)]
        symbol: String,
        #[arg(short, long, default_value = "testnet")]
        network: String,
    },
    ListTokens {
        #[arg(short, long, default_value = "testnet")]
        network: String,
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
        #[arg(short, long)]
        amount: String,
        #[arg(long)]
        nonce: Option<u64>,
    },
    BatchTransfer {
        #[arg(short, long)]
        recipients: Vec<String>,
        #[arg(short, long)]
        amounts: Vec<String>,
    },
    GetBalance {
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
    ExportMnemonic {
        #[arg(short, long)]
        password: String,
        #[arg(short, long, default_value = "mnemonic.txt")]
        output: String,
    },
    NetworkInfo,
    GetGasPrice,
    ImportWallet {
        #[arg(short, long)]
        mnemonic: Option<String>,
        #[arg(short, long)]
        private_key: Option<String>,
        #[arg(long)]
        derivation_path: Option<String>,
    },
    ExportKeystore {
        #[arg(short, long)]
        password: String,
        #[arg(short, long, default_value = "keystore.json")]
        output: String,
    },
    ListWallets,
    SwitchWallet {
        #[arg(short, long)]
        address: String,
    },
    // CreateMultisig {
    //     #[arg(short, long)]
    //     owners: Vec<String>,
    //     #[arg(short, long)]
    //     required: u64,
    // },
    ProposeTransaction {
        #[arg(short, long)]
        multisig: String,
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        value: String,
        #[arg(short, long)]
        data: Option<String>,
    },
    ApproveTransaction {
        #[arg(short, long)]
        multisig: String,
        #[arg(short, long)]
        tx_id: u64,
    },
    ListPendingTxs {
        #[arg(short, long)]
        multisig: String,
    },
    TransferToken {
        #[arg(short, long)]
        token_address: String,
        #[arg(short, long)]
        recipient: String,
        #[arg(short, long)]
        amount: String,
    },
    History {
        #[arg(short, long)]
        address: String,
        #[arg(short, long, default_value = "10")]
        limit: u32,
        #[arg(short, long, default_value = "all")]
        direction: String,
    },
    TokenBalance {
        #[arg(short, long)]
        address: String,
        #[arg(short, long)]
        token: String,
    },
    Contact {
        #[arg(short, long)]
        action: String,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        address: Option<String>,
    },
    CallContract {
        #[arg(short, long)]
        address: String,
        #[arg(short, long)]
        method: String,
        #[arg(short, long)]
        params: Vec<String>,
    },
}

fn map_error(e: impl std::error::Error) -> String {
    match e.to_string().as_str() {
        s if s.contains("nonce too low") => "Transaction failed: Nonce too low. Try incrementing the nonce.".to_string(),
        s if s.contains("insufficient funds") => "Transaction failed: Insufficient funds in wallet.".to_string(),
        _ => format!("An error occurred: {}", e),
    }
}

async fn with_retry<T, F, Fut>(mut f: F, retries: u32) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    let mut last_error = None;
    for _ in 0..retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }
    Err(last_error.unwrap_or_else(|| "Max retries reached".into()))
}

fn handle_contacts_file(
    contacts_file: &str,
    action: &str,
    name: Option<&str>,
    address: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut contacts: Vec<Contact> = match File::open(contacts_file) {
        Ok(file) => serde_json::from_reader(BufReader::new(file))?,
        Err(_) => Vec::new(),
    };

    match action {
        "add" => {
            if let (Some(name), Some(address)) = (name, address) {
                contacts.push(Contact {
                    name: name.to_string(),
                    address: address.to_string(),
                });
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(contacts_file)?;
                serde_json::to_writer_pretty(&mut BufWriter::new(file), &contacts)?;
                println!("Contact '{}' added successfully!", name);
            } else {
                return Err("Both name and address are required to add a contact".into());
            }
        }
        "list" => {
            if contacts.is_empty() {
                println!("No contacts found.");
            } else {
                println!("{}", "Contacts".bold());
                println!("{}", "-".repeat(50).blue());
                for contact in &contacts {
                    println!("Name: {}", contact.name);
                    println!("Address: {}", contact.address);
                    println!("{}", "-".repeat(50).blue());
                }
            }
        }
        "show" => {
            if let Some(name) = name {
                if let Some(contact) = contacts.iter().find(|c| c.name == name) {
                    println!("{}", format!("Contact: {}", name).bold());
                    println!("Address: {}", contact.address);
                    println!("{}", "-".repeat(50).blue());
                } else {
                    println!("Contact '{}' not found", name);
                }
            } else {
                return Err("Name is required to show a contact".into());
            }
        }
        "delete" => {
            if let Some(name) = name {
                contacts.retain(|c| c.name != name);
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(contacts_file)?;
                serde_json::to_writer_pretty(&mut BufWriter::new(file), &contacts)?;
                println!("Contact '{}' deleted successfully!", name);
            } else {
                return Err("Name is required to delete a contact".into());
            }
        }
        _ => return Err(format!("Unknown action: {}", action).into()),
    }

    Ok(())
}

async fn validate_erc20_contract(
    address: &str,
    provider: &ethers::providers::Provider<ethers::providers::Http>,
) -> Result<(), Box<dyn std::error::Error>> {
    let abi: Abi = serde_json::from_str(include_str!("abi/ERC20.json"))?;
    let contract = Contract::new(address.parse::<Address>()?, abi, provider.clone().into());
    contract.method::<(), u8>("decimals", ())?.call().await?;
    Ok(())
}

async fn handle_add_token(
    symbol: &str,
    address: &str,
    decimals: u8,
    network: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider(network, None);
    validate_erc20_contract(address, &provider).await?;

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
    println!("Token {} added to {} registry in {}.", symbol, network, file_path);
    Ok(())
}

async fn handle_remove_token(
    symbol: &str,
    network: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "tokens.json";
    let mut data: Value = serde_json::from_str(&std::fs::read_to_string(file_path)?)?;

    let network_obj = data
        .get_mut(network)
        .and_then(|v| v.as_object_mut())
        .ok_or("Invalid network")?;

    network_obj.remove(&symbol.to_uppercase());

    std::fs::write(file_path, serde_json::to_string_pretty(&data)?)?;
    println!("Token {} removed from {} registry.", symbol, network);
    Ok(())
}

async fn handle_list_tokens(network: &str) -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_token_registry();
    let tokens = match network {
        "mainnet" => &registry.mainnet,
        "testnet" => &registry.testnet,
        _ => return Err("Invalid network".into()),
    };

    println!("{}", "Token Registry".bold());
    println!("{}", "-".repeat(50).blue());
    for (symbol, info) in tokens {
        println!("Symbol: {}", symbol);
        println!("Address: {}", info.address);
        println!("Decimals: {}", info.decimals);
        println!("{}", "-".repeat(50).blue());
    }

    Ok(())
}

async fn handle_transfer_token(
    token: &str,
    recipient: &str,
    amount: &str,
    wallet: &Wallet,
    network: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_message("Processing token transfer...");

    let provider = get_provider(network, None);
    let registry = load_token_registry();

    let (token_address, decimals) = match registry.testnet.get(&token.to_uppercase()) {
        Some(info) => (info.address.parse::<Address>()?, info.decimals),
        None => (token.parse::<Address>()?, 18),
    };

    let recipient = recipient.parse::<Address>()?;
    let amount: U256 = ethers::utils::parse_units(amount, decimals as usize)?.into();

    let erc20_abi = include_str!("abi/ERC20.json");
    let erc20_abi: Abi = serde_json::from_str(erc20_abi)?;
    let erc20_contract = Contract::new(token_address, erc20_abi, provider.clone().into());

    let tx = erc20_contract
        .method::<(Address, U256), ()>("transfer", (recipient, amount))?
        .from(wallet.address.parse::<Address>()?);

    let gas_estimate = tx.estimate_gas().await?;
    println!("Estimated gas: {}", gas_estimate);

    let signed_tx = wallet.sign_transaction(&tx.tx).await?;
    let tx_hash = provider.send_raw_transaction(signed_tx).await?;

    pb.finish_with_message("Transaction sent!");
    println!("Transaction successful with hash: {:?}", tx_hash);

    Ok(())
}

async fn handle_network_info(network: &str, custom_rpc: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider(network, custom_rpc);
    validate_network(&provider).await?;

    let block = provider.get_block_number().await?;
    let gas_price = provider.get_gas_price().await?;
    let chain_id = provider.get_chainid().await?;
    let syncing = provider.syncing().await?;

    println!("Network Status:");
    println!("- Chain ID: {}", chain_id);
    println!("- Current Block: {}", block);
    println!("- Gas Price: {} wei", gas_price);
    println!("- Syncing: {:?}", syncing);
    Ok(())
}

async fn handle_get_gas_price(network: &str, custom_rpc: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider(network, custom_rpc);
    let gas_price = provider.get_gas_price().await?;
    println!("Current Gas Price: {} wei", gas_price);
    Ok(())
}

async fn handle_export_key(wallet: &Wallet, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(output, &wallet.private_key)?;
    println!("Private key saved to {}", output);
    Ok(())
}

async fn handle_export_mnemonic(
    _wallet: &Wallet,
    password: &str,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Note: This assumes mnemonic is stored or derivable, which requires changes to Wallet
    return Err("Mnemonic export not supported yet".into());
}

async fn handle_generate_qr(wallet: &Wallet, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    generate_qr_code(&wallet.address, output)?;
    println!("QR code saved to {}", output);
    Ok(())
}

async fn handle_show_wallet(wallet: &Wallet) {
    println!("Address: {}", wallet.address);
    println!("Public Key: {}", wallet.public_key);
    if cfg!(debug_assertions) {
        println!("Private Key: {}", wallet.private_key);
    }
}

async fn handle_import_wallet(
    mnemonic: Option<String>,
    private_key: Option<String>,
    derivation_path: Option<String>,
    wallet_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = if let Some(mnemonic) = mnemonic {
        let path = derivation_path.unwrap_or("m/44'/137'/0'/0/0".to_string());
        Wallet::from_mnemonic(&mnemonic, &path)?
    } else if let Some(private_key) = private_key {
        Wallet::from_private_key(&private_key)?
    } else {
        return Err("Either mnemonic or private key must be provided".into());
    };

    wallet.save_to_file(wallet_file)?;
    println!("Wallet imported and saved to {}", wallet_file);
    Ok(())
}

async fn handle_transfer_to_contact(
    name: &str,
    amount: &str,
    wallet: &Wallet,
    contacts_file: &str,
    network: &str,
    custom_rpc: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Attempting transfer to contact '{}' for amount {}", name, amount);
    let book = ContactsBook::load(contacts_file);
    match book.get_contact(name) {
        Some(contact) => {
            info!("Resolved contact '{}' to address {}", name, contact.address);
            match handle_transfer(&contact.address, amount, wallet, None, network, custom_rpc).await {
                Ok(_) => {
                    info!("Transfer to contact '{}' ({}) succeeded.", name, contact.address);
                    println!("Transfer to contact '{}' ({}) succeeded.", name, contact.address);
                    Ok(())
                }
                Err(e) => {
                    error!("Transfer to contact '{}' ({}) failed: {}", name, contact.address, e);
                    // println!("Transfer to contact '{}' ({}) failed: {}", name, contact.address, map_error(e));
                    Err("Transfer failed".into())
                }
            }
        }
        None => {
            error!("Contact '{}' not found.", name);
            println!("Contact '{}' not found.", name);
            Err("Contact not found".into())
        }
    }
}

async fn handle_export_keystore(
    wallet: &Wallet,
    password: &str,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let keystore = Wallet::encrypt(wallet, password)?;
    std::fs::write(output, keystore)?;
    println!("Keystore exported to {}", output);
    Ok(())
}

async fn handle_transfer(
    recipient: &str,
    amount: &str,
    wallet: &Wallet,
    nonce: Option<u64>,
    network: &str,
    custom_rpc: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_message("Sending transaction...");

    let provider = get_provider(network, custom_rpc);
    let amount = ethers::utils::parse_units(amount, "ether")?;
    let recipient = recipient.parse::<Address>()?;
    let chain_id = provider.get_chainid().await?;
    let nonce = match nonce {
        Some(n) => U256::from(n),
        None => provider.get_transaction_count(wallet.address.parse::<Address>()?, None).await?,
    };

    let tx = TransactionRequest::new()
        .to(recipient)
        .value(amount)
        .from(wallet.address.parse::<Address>()?)
        .chain_id(chain_id.as_u64())
        .nonce(nonce)
        .gas(21000)
        .gas_price(provider.get_gas_price().await?);

    let signed_tx = wallet.sign_transaction(&tx.into()).await?;
    let tx_hash = with_retry(|| async {
        provider.send_raw_transaction(signed_tx.clone()).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }, 3).await?;

    pb.finish_with_message("Transaction sent!");
    info!("Transaction successful with hash: {:?}", tx_hash);
    println!("Transaction successful with hash: {:?}", tx_hash);

    Ok(())
}

async fn handle_batch_transfer(
    recipients: Vec<String>,
    amounts: Vec<String>,
    wallet: &Wallet,
    network: &str,
    custom_rpc: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if recipients.len() != amounts.len() {
        return Err("Number of recipients must match number of amounts".into());
    }

    let pb = ProgressBar::new(recipients.len() as u64);
    pb.set_message("Processing batch transfer...");

    for (recipient, amount) in recipients.iter().zip(amounts.iter()) {
        handle_transfer(recipient, amount, wallet, None, network, custom_rpc).await?;
        pb.inc(1);
    }

    pb.finish_with_message("Batch transfer completed!");
    println!("Batch transfer completed.");
    Ok(())
}

async fn handle_get_balance(address: &str, network: &str, custom_rpc: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider(network, custom_rpc);
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
    network: &str,
    custom_rpc: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider(network, custom_rpc);
    let tx = TransactionRequest::new()
        .to(recipient.parse::<Address>()?)
        .value(ethers::utils::parse_units(amount, "ether")?)
        .from(wallet.address.parse::<Address>()?);

    let gas_estimate = provider.estimate_gas(&tx.into(), None).await?;
    println!("Estimated gas: {}", gas_estimate);
    Ok(())
}

async fn handle_token_balance(
    address: &str,
    token: &str,
    wallet: &Wallet,
    network: &str,
    custom_rpc: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = std::env::var("ALCHEMY_API_KEY")
        .map_err(|_| "ALCHEMY_API_KEY not set")?;
    let base_url = std::env::var("ALCHEMY_RPC_URL")
        .map_err(|_| "ALCHEMY_RPC_URL not set")?;

    let token_registry = load_token_registry();
    let token_address = token_registry
        .testnet
        .get(token)
        .ok_or_else(|| format!("Token {} not found in registry", token))?
        .address
        .parse::<Address>()?;

    let client = reqwest::Client::new();
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "eth_call".to_string(),
        params: vec![
            json!({
                "to": format!("{:?}", token_address),
                "data": format!("0x70a08231000000000000000000000000{}", &address[2..].to_lowercase())
            }),
            serde_json::Value::String("latest".to_owned()),
        ],
    };

    let response = client
        .post(&base_url)
        .json(&request)
        .send()
        .await?
        .json::<JsonRpcResponse>()
        .await?;

    if let Some(error) = response.error {
        return Err(error.message.into());
    }

    if let Some(result) = response.result {
        let balance = i128::from_str_radix(&result[2..], 16)?;
        let decimals = token_registry
            .testnet
            .get(token)
            .ok_or_else(|| format!("Token {} decimals not found", token))?
            .decimals;

        let formatted_balance = format!("{}", balance as f64 / 10f64.powi(decimals as i32));
        println!("{}", format!("Balance for {}:", token).bold());
        println!("{} {}", formatted_balance, token);
        println!("{}", "-".repeat(50).blue());
    }

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

// async fn handle_create_multisig(
//     owners: Vec<String>,
//     required: u64,
//     wallet: &Wallet,
//     network: &str,
//     custom_rpc: Option<&str>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let pb = ProgressBar::new_spinner();
//     pb.set_message("Deploying multi-sig wallet...");

//     let provider = get_provider(network, custom_rpc);
//     let abi: Abi = serde_json::from_str(include_str!("abi/MultiSigWallet.json"))?;
//     let bytecode = include_bytes!("bytecode/MultiSigWallet.bin");

//     let owners: Vec<Address> = owners.iter().map(|o| o.parse::<Address>().unwrap()).collect();
//     let factory = Contract::new(Address::zero(), abi.clone(), provider.clone().into());

//     let deploy_tx = factory
//         .deploy((owners, U256::from(required)))?
//         .from(wallet.address.parse::<Address>()?);

//     let receipt = with_retry(|| async {
//         deploy_tx.clone().send().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
//     }, 3).await?;

//     let contract_address = receipt.contract_address.unwrap();
//     pb.finish_with_message("Multi-sig wallet deployed!");
//     println!("Multi-sig wallet deployed at: {:?}", contract_address);
//     Ok(())
// }

async fn handle_propose_transaction(
    multisig: &str,
    to: &str,
    value: &str,
    data: Option<String>,
    wallet: &Wallet,
    network: &str,
    custom_rpc: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_message("Proposing transaction...");

    let provider = get_provider(network, custom_rpc);
    let abi: Abi = serde_json::from_str(include_str!("abi/MultiSigWallet.json"))?;
    let contract = Contract::new(multisig.parse::<Address>()?, abi, provider.clone().into());

    let value = ethers::utils::parse_units(value, "ether")?;
    let data = data.unwrap_or_default().parse::<Bytes>()?;

    let tx = contract
        .method::<(Address, U256, Bytes), U256>("submitTransaction", (
            to.parse::<Address>()?,
            value.into(),
            data,
        ))?
        .from(wallet.address.parse::<Address>()?);

    // let receipt = with_retry(|| async {
    //     tx.clone().send().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    // }, 3).await?;

    pb.finish_with_message("Transaction proposed!");
    // println!("Transaction proposed with ID: {:?}", receipt.transaction_index);
    Ok(())
}

async fn handle_approve_transaction(
    multisig: &str,
    tx_id: u64,
    wallet: &Wallet,
    network: &str,
    custom_rpc: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_message("Approving transaction...");

    let provider = get_provider(network, custom_rpc);
    let abi: Abi = serde_json::from_str(include_str!("abi/MultiSigWallet.json"))?;
    let contract = Contract::new(multisig.parse::<Address>()?, abi, provider.clone().into());

    let tx = contract
        .method::<U256, ()>("confirmTransaction", U256::from(tx_id))?
        .from(wallet.address.parse::<Address>()?);

    // let receipt = with_retry(|| async {
    //     tx.clone().send().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    // }, 3).await?;

    pb.finish_with_message("Transaction approved!");
    // println!("Transaction {} approved: {:?}", tx_id, receipt.transaction_hash);
    Ok(())
}

async fn handle_list_pending_txs(
    multisig: &str,
    wallet: &Wallet,
    network: &str,
    custom_rpc: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider(network, custom_rpc);
    let abi: Abi = serde_json::from_str(include_str!("abi/MultiSigWallet.json"))?;
    let contract = Contract::new(multisig.parse::<Address>()?, abi, provider.clone().into());

    let tx_ids: Vec<U256> = contract
        .method::<(U256, U256, bool), Vec<U256>>(
            "getTransactionIds",
            (U256::zero(), U256::from(u64::MAX), true),
        )?
        .call()
        .await?;

    println!("{}", "Pending Transactions".bold());
    println!("{}", "-".repeat(50).blue());

    for tx_id in tx_ids {
        let tx: (Address, U256, Bytes, bool, U256) = contract
            .method::<U256, (Address, U256, Bytes, bool, U256)>("transactions", tx_id)?
            .call()
            .await?;
        println!("Transaction ID: {}", tx_id);
        println!("To: {:?}", tx.0);
        println!("Value: {} RBTC", ethers::utils::format_units(tx.1, "ether")?);
        println!("Data: {:?}", tx.2);
        println!("Executed: {}", tx.3);
        println!("Confirmations: {}", tx.4);
        println!("{}", "-".repeat(50).blue());
    }

    // if tx_ids.is_empty() {
    //     println!("{}", "No pending transactions".yellow());
    // }

    Ok(())
}

async fn handle_call_contract(
    address: &str,
    _method: &str,
    params: Vec<String>,
    _wallet: &Wallet,
    network: &str,
    custom_rpc: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = get_provider(network, custom_rpc);
    let abi: Abi = serde_json::from_str(include_str!("abi/GenericContract.json"))?;
    let contract = Contract::new(address.parse::<Address>()?, abi, provider.clone().into());

    // Note: This is a simplified implementation; real-world use requires parsing params to ABI types
    let result = contract
        .method::<Vec<String>, Bytes>("call", params)?
        .call()
        .await?;

    println!("Result: {:?}", result);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv().ok();

    let wallet_file = "wallet.json";
    let contacts_file = "contacts.json";
    let cli = Cli::parse();

    let wallet = Wallet::load_from_file(wallet_file).unwrap_or_else(|| {
        info!("No wallet found. Generating new one...");
        let new_wallet = Wallet::generate();
        new_wallet.save_to_file(wallet_file).expect("Failed to save wallet");
        info!("New wallet generated and saved to {}", wallet_file);
        new_wallet
    });

    match cli.command {
        Commands::AddToken {
            symbol,
            address,
            decimals,
            network,
        } => handle_add_token(&symbol, &address, decimals, &network).await?,
        Commands::RemoveToken { symbol, network } => handle_remove_token(&symbol, &network).await?,
        Commands::ListTokens { network } => handle_list_tokens(&network).await?,
        Commands::TransferToContact { name, amount } => {
            handle_transfer_to_contact(&name, &amount, &wallet, contacts_file, &cli.network, cli.rpc_url.as_deref()).await?
        }
        Commands::AddContact { name, address } => handle_add_contact(&name, &address, contacts_file).await,
        Commands::ListContacts => handle_list_contacts(contacts_file).await,
        Commands::ShowContact { name } => handle_show_contact(&name, contacts_file).await,
        Commands::Transfer {
            recipient,
            amount,
            nonce,
        } => handle_transfer(&recipient, &amount, &wallet, nonce, &cli.network, cli.rpc_url.as_deref()).await?,
        Commands::BatchTransfer { recipients, amounts } => {
            handle_batch_transfer(recipients, amounts, &wallet, &cli.network, cli.rpc_url.as_deref()).await?
        }
        Commands::GetBalance { address } => handle_get_balance(&address, &cli.network, cli.rpc_url.as_deref()).await?,
        Commands::ShowWallet => handle_show_wallet(&wallet).await,
        Commands::GenerateQr { output } => handle_generate_qr(&wallet, &output).await?,
        Commands::EstimateGas { recipient, amount } => {
            handle_estimate_gas(&recipient, &amount, &wallet, &cli.network, cli.rpc_url.as_deref()).await?
        }
        Commands::ExportPrivateKey { output } => handle_export_key(&wallet, &output).await?,
        Commands::ExportMnemonic { password, output } => {
            handle_export_mnemonic(&wallet, &password, &output).await?
        }
        Commands::NetworkInfo => handle_network_info(&cli.network, cli.rpc_url.as_deref()).await?,
        Commands::GetGasPrice => handle_get_gas_price(&cli.network, cli.rpc_url.as_deref()).await?,
        Commands::ImportWallet {
            mnemonic,
            private_key,
            derivation_path,
        } => handle_import_wallet(mnemonic, private_key, derivation_path, wallet_file).await?,
        Commands::ExportKeystore { password, output } => handle_export_keystore(&wallet, &password, &output).await?,
        Commands::ListWallets => handle_list_wallets().await?,
        Commands::SwitchWallet { address } => handle_switch_wallet(&address).await?,
        // Commands::CreateMultisig { owners, required } => {
        //     handle_create_multisig(owners, required, &wallet, &cli.network, cli.rpc_url.as_deref()).await?
        // }
        Commands::ProposeTransaction {
            multisig,
            to,
            value,
            data,
        } => handle_propose_transaction(&multisig, &to, &value, data, &wallet, &cli.network, cli.rpc_url.as_deref()).await?,
        Commands::ApproveTransaction { multisig, tx_id } => {
            handle_approve_transaction(&multisig, tx_id, &wallet, &cli.network, cli.rpc_url.as_deref()).await?
        }
        Commands::ListPendingTxs { multisig } => {
            handle_list_pending_txs(&multisig, &wallet, &cli.network, cli.rpc_url.as_deref()).await?
        }
        Commands::TransferToken {
            token_address,
            recipient,
            amount,
        } => handle_transfer_token(&token_address, &recipient, &amount, &wallet, &cli.network).await?,
        Commands::History {
            address,
            limit,
            direction,
        } => history::history_command(&cli.network, Some(&address), Some(limit), &direction, &wallet.address).await?,
        Commands::TokenBalance { address, token } => {
            handle_token_balance(&address, &token, &wallet, &cli.network, cli.rpc_url.as_deref()).await?
        }
        Commands::Contact { action, name, address } => {
            handle_contacts_file(contacts_file, &action, name.as_deref(), address.as_deref())?
        }
        Commands::CallContract {
            address,
            method,
            params,
        } => handle_call_contract(&address, &method, params, &wallet, &cli.network, cli.rpc_url.as_deref()).await?,
    }

    Ok(())
}

async fn handle_list_wallets() -> Result<(), Box<dyn std::error::Error>> {
    let manager = rootstock_wallet::wallet::WalletManager::load_from_file("wallets.json")?;
    for (address, wallet) in manager.list_wallets() {
        println!("Address: {}", address);
        println!("Public Key: {}", wallet.public_key);
        println!("{}", "-".repeat(50).blue());
    }
    Ok(())
}

async fn handle_switch_wallet(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = rootstock_wallet::wallet::WalletManager::load_from_file("wallets.json")?;
    manager.set_current_wallet(address)?;
    manager.save_to_file("wallets.json")?;
    println!("Switched to wallet: {}", address);
    Ok(())
}