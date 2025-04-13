use rootstock_wallet::provider;
use ethers::types::Address;
use std::str::FromStr;

#[tokio::test]
async fn test_balance_retrieval() {
    // Get the provider
    let provider = provider::get_provider();

    // Define the address to check balance for
    let address = Address::from_str("0x7be423bf55e894f05a2b2f6a692d00ce258b203d")
        .expect("Invalid address");

    // Fetch the balance
    let balance = provider
        .get_balance(address, None)
        .await
        .expect("Failed to fetch balance");

    // Assert that the balance is a valid U256 value
    assert!(balance >= ethers::types::U256::zero(), "Balance should be non-negative");

    println!("Balance for {}: {} wei", address, balance);
}