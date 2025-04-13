use rootstock_wallet::provider;
use rootstock_wallet::wallet::Wallet;
use ethers::types::{Address, U256};
use std::str::FromStr;

#[tokio::test]
async fn test_gas_estimation() {
    // Load the wallet
    let wallet_file = "wallet.json";
    let wallet = Wallet::load_from_file(wallet_file).expect("Failed to load wallet");

    // Get the provider
    let provider = provider::get_provider();

    // Define recipient and amount
    let recipient = Address::from_str("0x09aB514B6974601967E7b379478EFf4073cceD06")
        .expect("Invalid recipient address");
    let amount = U256::from_dec_str("1000000000000000000").expect("Invalid amount"); // 1 RBTC in wei

    // Create a transaction request
    let tx = ethers::types::TransactionRequest::new()
        .to(recipient)
        .value(amount)
        .from(wallet.address.parse::<Address>().expect("Invalid wallet address"));

    // Estimate gas
    let gas_estimate = provider
        .estimate_gas(&tx.into(), None)
        .await
        .expect("Failed to estimate gas");

    // Assert that the gas estimate is greater than zero
    assert!(gas_estimate > U256::zero(), "Gas estimate should be greater than zero");

    println!("Gas estimate: {}", gas_estimate);
}