use rootstock_wallet::wallet::Wallet;
use ethers::types::{Address, U256};
use std::str::FromStr;

#[tokio::test]
async fn test_erc20_transfer() {
    // Load the wallet
    let wallet_file = "wallet.json";
    let wallet = Wallet::load_from_file(wallet_file).expect("Failed to load wallet");

    // Define the token contract address, recipient, and amount
    let token_address = Address::from_str("0xTokenContractAddress").expect("Invalid token address");
    let recipient = Address::from_str("0xRecipientAddress").expect("Invalid recipient address");
    let amount = U256::from_dec_str("1000000000000000000").expect("Invalid amount"); // 1 token in wei

    // Load the ERC-20 contract ABI
    let erc20_abi = include_str!("../abi/ERC20.json");
    let provider = rootstock_wallet::provider::get_provider();
    let erc20_contract = ethers::contract::Contract::new(token_address, erc20_abi.parse().unwrap(), provider);

    // Create the transfer transaction
    let tx = erc20_contract
        .method::<_, ()>("transfer", (recipient, amount))
        .expect("Failed to create transfer method")
        .from(wallet.address.parse::<Address>().expect("Invalid wallet address"));

    // Estimate gas
    let gas_estimate = tx.estimate_gas().await.expect("Failed to estimate gas");
    assert!(gas_estimate > U256::zero(), "Gas estimate should be greater than zero");

    println!("Gas estimate: {}", gas_estimate);
}