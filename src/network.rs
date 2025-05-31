use rootstock_wallet::provider;

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