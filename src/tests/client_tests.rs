use rootstock_wallet::client::Client;
use ethers::types::Address;
use std::str::FromStr;

#[tokio::test]
async fn test_client_initialization() {
    // Initialize the client with a valid provider URL
    let provider_url = "https://public-node.rsk.co";
    let client = Client::new(provider_url).expect("Failed to initialize client");

    // Assert that the client is initialized correctly
    assert!(client.provider().is_some(), "Client provider should be initialized");
    println!("Client initialization test passed.");
}

#[tokio::test]
async fn test_client_get_balance() {
    // Initialize the client
    let provider_url = "https://public-node.rsk.co";
    let client = Client::new(provider_url).expect("Failed to initialize client");

    // Define a test address
    let address = Address::from_str("0x7be423bf55e894f05a2b2f6a692d00ce258b203d")
        .expect("Invalid address");

    // Fetch the balance
    let balance = client
        .get_balance(address)
        .await
        .expect("Failed to fetch balance");

    // Assert that the balance is a valid U256 value
    assert!(balance >= ethers::types::U256::zero(), "Balance should be non-negative");
    println!("Client get balance test passed.");
}

#[tokio::test]
async fn test_client_send_transaction() {
    // Initialize the client
    let provider_url = "https://public-node.rsk.co";
    let client = Client::new(provider_url).expect("Failed to initialize client");

    // Define a test transaction
    let recipient = Address::from_str("0x09aB514B6974601967E7b379478EFf4073cceD06")
        .expect("Invalid recipient address");
    let amount = ethers::types::U256::from_dec_str("1000000000000000").expect("Invalid amount"); // 0.001 RBTC in wei

    // Attempt to send the transaction (this will likely fail due to insufficient funds)
    let result = client
        .send_transaction(recipient, amount)
        .await;

    // Assert that the result is an error (since this is a test environment)
    assert!(result.is_err(), "Transaction should fail in test environment");
    println!("Client send transaction test passed.");
}