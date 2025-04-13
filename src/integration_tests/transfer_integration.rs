use rootstock_wallet::provider;
use rootstock_wallet::wallet::Wallet;
use ethers::types::{Address, U256};
use std::str::FromStr;

#[tokio::test]
async fn test_rbtc_transfer() {
    // Load the wallet
    let wallet_file = "wallet.json";
    let wallet = Wallet::load_from_file(wallet_file).expect("Failed to load wallet");

    // Get the provider
    let provider = provider::get_provider();

    // Define recipient and amount
    let recipient = Address::from_str("0x09aB514B6974601967E7b379478EFf4073cceD06")
        .expect("Invalid recipient address");
    let amount = U256::from_dec_str("500000000000000").expect("Invalid amount"); // 0.0005 RBTC in wei

    // Create a transaction request
    let tx = ethers::types::TransactionRequest::new()
        .to(recipient)
        .value(amount)
        .from(wallet.address.parse::<Address>().expect("Invalid wallet address"));

    // Sign the transaction
    let signed_tx = wallet
        .sign_transaction(&tx.into())
        .await
        .expect("Failed to sign transaction");

    // Send the signed transaction
    let tx_hash = provider
        .send_raw_transaction(signed_tx)
        .await
        .expect("Failed to send transaction");

    // Assert that the transaction hash is valid
    assert!(!tx_hash.as_bytes().is_empty(), "Transaction hash should not be empty");

    println!("Transaction successful with hash: {:?}", tx_hash);
}