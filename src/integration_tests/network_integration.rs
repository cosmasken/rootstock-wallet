use rootstock_wallet::provider;

#[tokio::test]
async fn test_network_info() {
    // Get the provider
    let provider = provider::get_provider();

    // Fetch the current block number
    let block_number = provider
        .get_block_number()
        .await
        .expect("Failed to fetch block number");
    assert!(block_number > 0.into(), "Block number should be greater than zero");
    println!("Current block number: {}", block_number);

    // Fetch the current gas price
    let gas_price = provider
        .get_gas_price()
        .await
        .expect("Failed to fetch gas price");
    assert!(gas_price > 0.into(), "Gas price should be greater than zero");
    println!("Current gas price: {} wei", gas_price);

    // Fetch the chain ID
    let chain_id = provider
        .get_chainid()
        .await
        .expect("Failed to fetch chain ID");
    assert!(chain_id > 0.into(), "Chain ID should be greater than zero");
    println!("Chain ID: {}", chain_id);
}