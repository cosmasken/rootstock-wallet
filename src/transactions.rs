// use crate::provider::get_provider;
// use crate::wallet::Wallet;
// use ethers::abi::Abi;
// use ethers::contract::Contract;
// use ethers::providers::Middleware;
// use ethers::types::transaction::eip2718::TypedTransaction;
// use ethers::types::{Address, TransactionRequest, U256};
// use indicatif::ProgressBar;
// use log::{error, info};

// pub async fn send_transaction(
//     wallet: &Wallet,
//     recipient: &str,
//     amount: &str,
//     network: &str,
//     custom_rpc: Option<&str>,
//     nonce: Option<u64>,
// ) -> Result<String, Box<dyn std::error::Error>> {
//     let pb = ProgressBar::new_spinner();
//     pb.set_message("Sending transaction...");

//     let provider = get_provider(network, custom_rpc);
//     let recipient = recipient.parse::<Address>()?;
//     let amount = ethers::utils::parse_units(amount, "ether")?;
//     let chain_id = provider.get_chainid().await?;
//     let nonce = match nonce {
//         Some(n) => U256::from(n),
//         None => {
//             provider
//                 .get_transaction_count(wallet.address.parse::<Address>()?, None)
//                 .await?
//         }
//     };

//     let tx = TransactionRequest::new()
//         .to(recipient)
//         .value(amount)
//         .from(wallet.address.parse::<Address>()?)
//         .chain_id(chain_id.as_u64())
//         .nonce(nonce)
//         .gas(21000)
//         .gas_price(provider.get_gas_price().await?);

//     let typed_tx: TypedTransaction = tx.into();
//     let signed_tx = wallet.sign_transaction(&typed_tx).await?;
//     let pending_tx = provider
//         .send_raw_transaction(signed_tx)
//         .await
//         .map_err(|e| {
//             error!("Failed to send transaction: {}", e);
//             Box::new(e) as Box<dyn std::error::Error>
//         })?;

//     let tx_hash = format!("{:?}", pending_tx);
//     pb.finish_with_message("Transaction sent!");
//     info!("Transaction sent with hash: {}", tx_hash);
//     Ok(tx_hash)
// }

// pub async fn estimate_gas(
//     wallet: &Wallet,
//     recipient: &str,
//     amount: &str,
//     network: &str,
//     custom_rpc: Option<&str>,
// ) -> Result<U256, Box<dyn std::error::Error>> {
//     let provider = get_provider(network, custom_rpc);
//     let recipient = recipient.parse::<Address>()?;
//     let amount = ethers::utils::parse_units(amount, "ether")?;

//     let tx = TransactionRequest::new()
//         .to(recipient)
//         .value(amount)
//         .from(wallet.address.parse::<Address>()?);

//     let gas_estimate = provider.estimate_gas(&tx.into(), None).await.map_err(|e| {
//         error!("Failed to estimate gas: {}", e);
//         Box::new(e) as Box<dyn std::error::Error>
//     })?;

//     Ok(gas_estimate)
// }

// pub async fn send_token_transfer(
//     wallet: &Wallet,
//     token_address: &str,
//     recipient: &str,
//     amount: &str,
//     decimals: u8,
//     network: &str,
//     custom_rpc: Option<&str>,
// ) -> Result<String, Box<dyn std::error::Error>> {
//     let pb = ProgressBar::new_spinner();
//     pb.set_message("Processing token transfer...");

//     let provider = get_provider(network, custom_rpc);
//     let token_address = token_address.parse::<Address>()?;
//     let recipient = recipient.parse::<Address>()?;
//     let amount: U256 = ethers::utils::parse_units(amount, decimals as usize)?.into();

//     let erc20_abi: Abi = serde_json::from_str(include_str!("abi/ERC20.json"))?;
//     let erc20_contract = Contract::new(token_address, erc20_abi, provider.clone().into());

//     let tx = erc20_contract
//         .method::<(Address, U256), bool>("transfer", (recipient, amount))?
//         .from(wallet.address.parse::<Address>()?);

//     let gas_estimate = tx.estimate_gas().await.map_err(|e| {
//         error!("Failed to estimate gas for token transfer: {}", e);
//         Box::new(e) as Box<dyn std::error::Error>
//     })?;

//     info!("Estimated gas for token transfer: {}", gas_estimate);

//     let signed_tx = wallet.sign_transaction(&tx.tx).await?;
//     let pending_tx = provider
//         .send_raw_transaction(signed_tx)
//         .await
//         .map_err(|e| {
//             error!("Failed to send token transfer: {}", e);
//             Box::new(e) as Box<dyn std::error::Error>
//         })?;

//     let tx_hash = format!("{:?}", pending_tx);
//     pb.finish_with_message("Token transfer sent!");
//     info!("Token transfer sent with hash: {}", tx_hash);
//     Ok(tx_hash)
// }
