use rootstock_wallet::wallet::Wallet;
use std::fs;

#[test]
fn test_wallet_generation() {
    // Generate a new wallet
    let wallet = Wallet::generate();

    // Assert that the wallet has a valid address, public key, and private key
    assert!(!wallet.address.is_empty(), "Wallet address should not be empty");
    assert!(!wallet.public_key.is_empty(), "Wallet public key should not be empty");
    assert!(!wallet.private_key.is_empty(), "Wallet private key should not be empty");

    println!("Wallet generation test passed.");
}

#[test]
fn test_wallet_save_and_load() {
    // Generate a new wallet
    let wallet = Wallet::generate();

    // Save the wallet to a file
    let wallet_file = "test_wallet.json";
    wallet
        .save_to_file(wallet_file)
        .expect("Failed to save wallet to file");

    // Load the wallet from the file
    let loaded_wallet = Wallet::load_from_file(wallet_file)
        .expect("Failed to load wallet from file");

    // Assert that the loaded wallet matches the generated wallet
    assert_eq!(wallet.address, loaded_wallet.address, "Wallet addresses should match");
    assert_eq!(wallet.public_key, loaded_wallet.public_key, "Public keys should match");
    assert_eq!(wallet.private_key, loaded_wallet.private_key, "Private keys should match");

    // Clean up the test wallet file
    fs::remove_file(wallet_file).expect("Failed to delete test wallet file");

    println!("Wallet save and load test passed.");
}

#[test]
fn test_wallet_private_key_export() {
    // Generate a new wallet
    let wallet = Wallet::generate();

    // Export the private key to a file
    let private_key_file = "test_private.key";
    fs::write(private_key_file, &wallet.private_key).expect("Failed to export private key");

    // Read the private key from the file
    let exported_private_key = fs::read_to_string(private_key_file)
        .expect("Failed to read exported private key");

    // Assert that the exported private key matches the wallet's private key
    assert_eq!(
        wallet.private_key, exported_private_key,
        "Exported private key should match the wallet's private key"
    );

    // Clean up the test private key file
    fs::remove_file(private_key_file).expect("Failed to delete test private key file");

    println!("Wallet private key export test passed.");
}