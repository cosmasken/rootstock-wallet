use rootstock_wallet::qr;
use std::fs;

#[test]
fn test_generate_qr_code() {
    // Define a test wallet address
    let wallet_address = "0x7be423bf55e894f05a2b2f6a692d00ce258b203d";

    // Define the output file for the QR code
    let output_file = "test_wallet_qr.png";

    // Generate the QR code
    qr::generate_qr_code(wallet_address, output_file).expect("Failed to generate QR code");

    // Assert that the QR code file exists
    assert!(fs::metadata(output_file).is_ok(), "QR code file should exist");

    // Clean up the test QR code file
    fs::remove_file(output_file).expect("Failed to delete test QR code file");

    println!("QR code generation test passed.");
}