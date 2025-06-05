use rootstock_wallet::transactions::{TransactionService, TransferError};
use rootstock_wallet::provider;
use ethers::types::{Address, U256};
use std::str::FromStr;
use rootstock_wallet::wallet::Wallet;
use dotenv::dotenv;

#[tokio::test]
async fn test_transaction_service_initialization() {
    // Initialize the provider
    let provider = provider::get_provider();

    // Create a dummy private key for testing
    let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    // Initialize the transaction service
    let tx_service = TransactionService::new(provider, private_key.into())
        .expect("Failed to initialize transaction service");

    // Assert that the transaction service is initialized correctly
    assert!(tx_service.provider.is_some(), "Transaction service provider should be initialized");
    println!("Transaction service initialization test passed.");
}

#[tokio::test]
async fn test_transaction_insufficient_balance() {
    // Initialize the provider
    let provider = provider::get_provider();

    // Create a dummy private key for testing
    let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    // Initialize the transaction service
    let tx_service = TransactionService::new(provider, private_key.into())
        .expect("Failed to initialize transaction service");

    // Define recipient and amount
    let recipient = Address::from_str("0x09aB514B6974601967E7b379478EFf4073cceD06")
        .expect("Invalid recipient address");
    let amount = U256::from_dec_str("1000000000000000000").expect("Invalid amount"); // 1 RBTC in wei

    // Attempt to send the transaction
    let result = tx_service
        .send_rbtc(recipient, amount)
        .await;

    // Assert that the transaction fails due to insufficient balance
    assert!(
        matches!(result, Err(TransferError::InsufficientBalance)),
        "Transaction should fail with InsufficientBalance error"
    );
    println!("Transaction insufficient balance test passed.");
}

#[tokio::test]
async fn test_transaction_invalid_address() {
    // Initialize the provider
    let provider = provider::get_provider();

    // Create a dummy private key for testing
    let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    // Initialize the transaction service
    let tx_service = TransactionService::new(provider, private_key.into())
        .expect("Failed to initialize transaction service");

    // Define an invalid recipient address
    let recipient = Address::zero(); // Invalid address
    let amount = U256::from_dec_str("1000000000000000000").expect("Invalid amount"); // 1 RBTC in wei

    // Attempt to send the transaction
    let result = tx_service
        .send_rbtc(recipient, amount)
        .await;

    // Assert that the transaction fails due to an invalid address
    assert!(
        matches!(result, Err(TransferError::InvalidAddress)),
        "Transaction should fail with InvalidAddress error"
    );
    println!("Transaction invalid address test passed.");
}

#[tokio::test]
async fn test_token_balance() {
    dotenv::dotenv().ok();
    let wallet = Wallet::new();
    
    // Test with valid token
    let result = handle_token_balance(
        &wallet.address,
        "RIF",
        &wallet,
    ).await;
    assert!(result.is_ok());
    
    // Test with invalid token
    let result = handle_token_balance(
        &wallet.address,
        "INVALID_TOKEN",
        &wallet,
    ).await;
    assert!(result.is_err());
    
    // Test with invalid address
    let result = handle_token_balance(
        "invalid_address",
        "RIF",
        &wallet,
    ).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_transaction_history() {
    dotenv::dotenv().ok();
    let wallet = Wallet::new();
    
    // Test with valid address and limit
    let result = history_command(
        false,  // testnet
        &wallet.address,
        Some(5),
        &wallet.address,
    ).await;
    assert!(result.is_ok());
    
    // Test with invalid address
    let result = history_command(
        false,
        "invalid_address",
        Some(5),
        &wallet.address,
    ).await;
    assert!(result.is_err());
    
    // Test with large limit
    let result = history_command(
        false,
        &wallet.address,
        Some(1000),
        &wallet.address,
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_network_selection() {
    dotenv::dotenv().ok();
    let wallet = Wallet::new();
    
    // Test mainnet
    let result = handle_token_balance(
        &wallet.address,
        "RIF",
        &wallet,
    ).await;
    assert!(result.is_ok());
    
    // Test testnet
    let result = handle_token_balance(
        &wallet.address,
        "RIF",
        &wallet,
    ).await;
    assert!(result.is_ok());
    
    // Test invalid network
    let result = handle_token_balance(
        &wallet.address,
        "RIF",
        &wallet,
    ).await;
    assert!(result.is_err());
}