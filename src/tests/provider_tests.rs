use rootstock_wallet::provider;
use ethers::types::U256;

#[tokio::test]
async fn test_provider_initialization() {
    // Initialize the provider
    let provider = provider::get_provider();

    // Assert that the provider is initialized correctly
    assert!(provider.url().is_some(), "Provider URL should be initialized");
    println!("Provider initialization test passed.");
}

#[tokio::test]
async fn test_provider_get_block_number() {
    // Initialize the provider
    let provider = provider::get_provider();

    // Fetch the current block number
    let block_number = provider
        .get_block_number()
        .await
        .expect("Failed to fetch block number");

    // Assert that the block number is greater than zero
    assert!(block_number > 0.into(), "Block number should be greater than zero");
    println!("Current block number: {}", block_number);
}

#[tokio::test]
async fn test_provider_get_gas_price() {
    // Initialize the provider
    let provider = provider::get_provider();

    // Fetch the current gas price
    let gas_price = provider
        .get_gas_price()
        .await
        .expect("Failed to fetch gas price");

    // Assert that the gas price is greater than zero
    assert!(gas_price > U256::zero(), "Gas price should be greater than zero");
    println!("Current gas price: {} wei", gas_price);
}

#[tokio::test]
async fn test_provider_get_chain_id() {
    // Initialize the provider
    let provider = provider::get_provider();

    // Fetch the chain ID
    let chain_id = provider
        .get_chainid()
        .await
        .expect("Failed to fetch chain ID");

    // Assert that the chain ID is greater than zero
    assert!(chain_id > U256::zero(), "Chain ID should be greater than zero");
    println!("Chain ID: {}", chain_id);
}