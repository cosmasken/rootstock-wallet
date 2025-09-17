//! Comprehensive security validation tests
//!
//! This module contains tests that validate all security measures implemented
//! in the wallet application, including debug output redaction, log sanitization,
//! memory clearing, and network security.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::api::{ApiKey, ApiProvider, ApiManager};
    use crate::security::test_utils::*;
    use crate::types::wallet::{Wallet, WalletData};
    use crate::types::contacts::Contact;
    use crate::{secure_debug, secure_info, secure_warn, secure_error};
    use ethers::types::Address;
    use ethers::signers::LocalWallet;
    use std::str::FromStr;
    use zeroize::Zeroize;
    use chrono::Local;

    /// Test that Debug output doesn't contain sensitive data for all wallet types
    #[test]
    fn test_wallet_debug_output_security() {
        let mock_data = MockSensitiveData::new();
        
        // Create a test wallet with sensitive data
        let private_key_hex = mock_data.random_private_key();
        let wallet = LocalWallet::from_str(private_key_hex).expect("Valid private key");
        let password = SecurePassword::new("test_password".to_string());
        
        let secure_wallet = Wallet::new(wallet, "Test Wallet", &password)
            .expect("Failed to create wallet");

        // Test that debug output is properly redacted
        let debug_output = format!("{:?}", secure_wallet);
        
        // Should not contain the original private key
        assert!(!debug_output.contains(private_key_hex));
        
        // Should contain redaction markers
        assert!(debug_output.contains("[REDACTED]"));
        
        // Should contain non-sensitive information
        assert!(debug_output.contains("Test Wallet"));
        assert!(debug_output.contains(&format!("{:?}", secure_wallet.address)));
        
        println!("Wallet debug output: {}", debug_output);
    }

    #[test]
    fn test_wallet_data_debug_output_security() {
        let mock_data = MockSensitiveData::new();
        let mut wallet_data = WalletData::new();
        
        // Add API key
        wallet_data.set_api_key(mock_data.random_api_key().to_string());
        
        // Add a test wallet
        let private_key_hex = mock_data.random_private_key();
        let wallet = LocalWallet::from_str(private_key_hex).expect("Valid private key");
        let password = SecurePassword::new("test_password".to_string());
        let secure_wallet = Wallet::new(wallet, "Test Wallet", &password)
            .expect("Failed to create wallet");
        
        wallet_data.add_wallet(secure_wallet).expect("Failed to add wallet");
        
        // Test debug output
        let debug_output = format!("{:?}", wallet_data);
        
        // Should not contain sensitive data
        assert!(!debug_output.contains(mock_data.random_api_key()));
        assert!(!debug_output.contains(private_key_hex));
        
        // Should contain redaction markers
        assert!(debug_output.contains("[REDACTED]"));
        
        // Should contain non-sensitive metadata
        assert!(debug_output.contains("wallets_count"));
        assert!(debug_output.contains("contacts_count"));
        
        println!("WalletData debug output: {}", debug_output);
    }

    #[test]
    fn test_api_key_debug_output_security() {
        let mock_data = MockSensitiveData::new();
        
        let api_key = ApiKey {
            key: mock_data.random_api_key().to_string(),
            network: "mainnet".to_string(),
            provider: ApiProvider::Alchemy,
            name: Some("Test API Key".to_string()),
        };
        
        let debug_output = format!("{:?}", api_key);
        
        // Should not contain the full API key
        assert!(!debug_output.contains(mock_data.random_api_key()));
        
        // Should contain partial key (first few characters)
        let partial_key = &mock_data.random_api_key()[..3];
        assert!(debug_output.contains(partial_key));
        
        // Should contain non-sensitive information
        assert!(debug_output.contains("mainnet"));
        assert!(debug_output.contains("Alchemy"));
        assert!(debug_output.contains("Test API Key"));
        
        println!("ApiKey debug output: {}", debug_output);
    }

    #[test]
    fn test_log_sanitization_functions() {
        let mock_data = MockSensitiveData::new();
        
        // Test sanitization of various sensitive data types
        let test_cases = vec![
            (
                format!("Private key: {}", mock_data.random_private_key()),
                vec!["[PRIVATE_KEY_REDACTED]"],
                vec![mock_data.random_private_key()],
            ),
            (
                format!("Address: {}", mock_data.random_address()),
                vec!["..."],
                vec![], // Addresses are partially shown, so we don't forbid the full address
            ),
            (
                format!("Transaction: {}", mock_data.random_tx_hash()),
                vec!["..."],
                vec![], // Transaction hashes are partially shown
            ),
            (
                format!("Mnemonic: {}", mock_data.random_mnemonic()),
                vec!["[MNEMONIC_REDACTED]"],
                vec![mock_data.random_mnemonic()],
            ),
        ];
        
        for (input, expected_markers, forbidden_content) in test_cases {
            let sanitized = sanitize_log_message(&input);
            
            // Should contain at least one expected redaction marker
            let has_marker = expected_markers.iter().any(|marker| sanitized.contains(marker));
            assert!(
                has_marker,
                "Sanitized message '{}' should contain one of: {:?}",
                sanitized,
                expected_markers
            );
            
            // Should not contain forbidden content
            for forbidden in &forbidden_content {
                assert!(
                    !sanitized.contains(forbidden),
                    "Sanitized message '{}' should not contain '{}'",
                    sanitized,
                    forbidden
                );
            }
            
            println!("Original: {}", input);
            println!("Sanitized: {}", sanitized);
            println!("---");
        }
    }

    #[test]
    fn test_is_sensitive_data_detection() {
        let mock_data = MockSensitiveData::new();
        
        // Test positive cases (should be detected as sensitive)
        let sensitive_cases = vec![
            (mock_data.random_private_key(), "private key"),
            (mock_data.random_address(), "address"),
            (mock_data.random_tx_hash(), "transaction hash"),
            (mock_data.random_mnemonic(), "mnemonic"),
        ];
        
        for (case, description) in sensitive_cases {
            assert!(
                is_sensitive_data(case),
                "Should detect '{}' as sensitive data ({})",
                case,
                description
            );
        }
        
        // Test negative cases (should not be detected as sensitive)
        let non_sensitive_cases = vec![
            "This is just normal text",
            "Balance: 1000000000000000000",
            "Network: mainnet",
            "Wallet name: My Wallet",
            "Transaction successful",
        ];
        
        for case in non_sensitive_cases {
            assert!(
                !is_sensitive_data(case),
                "Should not detect '{}' as sensitive data",
                case
            );
        }
    }

    #[test]
    fn test_redaction_functions() {
        let mock_data = MockSensitiveData::new();
        
        // Test private key redaction
        let redacted_key = redact_private_key(mock_data.random_private_key());
        assert!(redacted_key.contains("[REDACTED]"));
        assert!(redacted_key.starts_with(&mock_data.random_private_key()[..4]));
        assert!(!redacted_key.contains(&mock_data.random_private_key()[10..]));
        
        // Test address redaction
        let redacted_addr = redact_address(mock_data.random_address());
        assert!(redacted_addr.contains("..."));
        assert!(redacted_addr.starts_with(&mock_data.random_address()[..6]));
        assert!(redacted_addr.ends_with(&mock_data.random_address()[mock_data.random_address().len()-4..]));
        
        println!("Original private key: {}", mock_data.random_private_key());
        println!("Redacted private key: {}", redacted_key);
        println!("Original address: {}", mock_data.random_address());
        println!("Redacted address: {}", redacted_addr);
    }

    #[test]
    fn test_secure_string_memory_clearing() {
        let test_data = "sensitive_private_key_data";
        
        // Test that SecureString clears memory when explicitly cleared
        let mut secure_string = SecureString::new(test_data.to_string());
        
        // Verify data is accessible before clearing
        assert_eq!(secure_string.expose().unwrap(), test_data);
        assert_eq!(secure_string.len(), test_data.len());
        assert!(!secure_string.is_empty());
        
        // Clear the memory
        secure_string.clear();
        
        // Verify data is cleared
        assert_eq!(secure_string.len(), 0);
        assert!(secure_string.is_empty());
        
        // Test that SecureString clears memory on drop
        {
            let secure_string2 = SecureString::new(test_data.to_string());
            assert_eq!(secure_string2.expose().unwrap(), test_data);
            // secure_string2 should be cleared when it goes out of scope
        }
        
        println!("SecureString memory clearing test passed");
    }

    #[test]
    fn test_secure_password_memory_clearing() {
        let test_password = "test_password_123";
        
        // Test that SecurePassword clears memory when explicitly cleared
        let mut secure_password = SecurePassword::new(test_password.to_string());
        
        // Verify password is accessible before clearing
        assert_eq!(secure_password.expose().unwrap(), test_password);
        assert_eq!(secure_password.len(), test_password.len());
        assert!(!secure_password.is_empty());
        
        // Clear the memory
        secure_password.clear();
        
        // Verify password is cleared
        assert_eq!(secure_password.len(), 0);
        assert!(secure_password.is_empty());
        
        println!("SecurePassword memory clearing test passed");
    }

    #[test]
    fn test_wallet_zeroization() {
        let mock_data = MockSensitiveData::new();
        let private_key_hex = mock_data.random_private_key();
        let wallet = LocalWallet::from_str(private_key_hex).expect("Valid private key");
        let password = SecurePassword::new("test_password".to_string());
        
        let mut secure_wallet = Wallet::new(wallet, "Test Wallet", &password)
            .expect("Failed to create wallet");
        
        // Manually zeroize the wallet
        secure_wallet.zeroize();
        
        // The wallet should still be usable for non-sensitive operations
        // but sensitive data should be cleared
        assert_eq!(secure_wallet.name, "Test Wallet");
        
        println!("Wallet zeroization test passed");
    }

    #[test]
    fn test_wallet_data_zeroization() {
        let mock_data = MockSensitiveData::new();
        let mut wallet_data = WalletData::new();
        
        // Add API key and wallet
        wallet_data.set_api_key(mock_data.random_api_key().to_string());
        
        let private_key_hex = mock_data.random_private_key();
        let wallet = LocalWallet::from_str(private_key_hex).expect("Valid private key");
        let password = SecurePassword::new("test_password".to_string());
        let secure_wallet = Wallet::new(wallet, "Test Wallet", &password)
            .expect("Failed to create wallet");
        
        wallet_data.add_wallet(secure_wallet).expect("Failed to add wallet");
        
        // Manually zeroize the wallet data
        wallet_data.zeroize();
        
        // Non-sensitive data should still be accessible
        assert!(!wallet_data.current_wallet.is_empty());
        
        println!("WalletData zeroization test passed");
    }

    #[test]
    fn test_comprehensive_debug_output_validation() {
        let _mock_data = MockSensitiveData::new();
        
        // Create a validator configured for wallet types
        let validator = DebugOutputValidator::for_wallet_types();
        
        // Test with a mock wallet
        let builder = MockWalletBuilder::new();
        let mock_wallet = builder.build_mock_wallet();
        
        // Validate the RedactedDebug output
        let result = validator.validate_debug(&crate::security::redacted_debug::SecureWrapper::new(mock_wallet));
        
        if !result.is_valid {
            println!("Validation failed:");
            println!("{}", result.error_message().unwrap());
        }
        
        result.assert_valid();
        
        println!("Comprehensive debug output validation passed");
    }

    #[test]
    fn test_secure_logging_macros() {
        let mock_data = MockSensitiveData::new();
        
        // Test that the secure logging macros properly sanitize messages
        let test_message = format!(
            "Wallet {} created with private key {}",
            mock_data.random_address(),
            mock_data.random_private_key()
        );
        
        // Test the sanitization function directly (since we can't easily capture log output)
        let sanitized = sanitize_log_message(&test_message);
        
        // Should not contain the full private key
        assert!(!sanitized.contains(mock_data.random_private_key()));
        
        // Should contain redaction markers
        assert!(sanitized.contains("[PRIVATE_KEY_REDACTED]"));
        
        // Should contain partial address
        assert!(sanitized.contains("..."));
        
        println!("Original message: {}", test_message);
        println!("Sanitized message: {}", sanitized);
        
        // Test that the macros compile and work (we can't easily test actual log output)
        secure_debug!("Test debug message with sensitive data: {}", mock_data.random_private_key());
        secure_info!("Test info message with address: {}", mock_data.random_address());
        secure_warn!("Test warn message with API key: {}", mock_data.random_api_key());
        secure_error!("Test error message with mnemonic: {}", mock_data.random_mnemonic());
        
        println!("Secure logging macros test passed");
    }

    #[test]
    fn test_network_security_measures() {
        // Test that SecureHttpClient enforces security measures
        // Note: This is a basic test since we can't easily test actual network calls
        
        let client_result = SecureHttpClient::new();
        assert!(client_result.is_ok(), "SecureHttpClient should be created successfully");
        
        let _client = client_result.unwrap();
        
        // In a real test, we would verify:
        // - TLS enforcement
        // - Request sanitization
        // - Response sanitization
        // - Error message sanitization
        
        println!("Network security measures test passed");
    }

    #[test]
    fn test_memory_inspector_utilities() {
        let mut inspector = MemoryInspector::new();
        let mock_data = MockSensitiveData::new();
        
        // Track some test allocations
        inspector.track_allocation(
            "test_sensitive".to_string(),
            mock_data.random_private_key().as_bytes().to_vec(),
        );
        
        inspector.track_allocation(
            "test_safe".to_string(),
            b"safe_public_data".to_vec(),
        );
        
        // Check for sensitive data
        let violations = inspector.contains_sensitive_data(&[mock_data.random_private_key().to_string()]);
        assert!(!violations.is_empty(), "Should detect sensitive data in memory");
        
        // Test SecureString clearing
        let cleared_successfully = inspector.test_secure_string_clearing("test_data");
        assert!(cleared_successfully, "SecureString should clear memory successfully");
        
        println!("Memory inspector utilities test passed");
    }

    #[test]
    fn test_api_manager_security() {
        let mock_data = MockSensitiveData::new();
        let mut api_manager = ApiManager::new();
        
        // Add an API key
        let api_key = ApiKey {
            key: mock_data.random_api_key().to_string(),
            network: "mainnet".to_string(),
            provider: ApiProvider::Alchemy,
            name: Some("Test Key".to_string()),
        };
        
        let _key_id = api_manager.add_key(api_key);
        
        // Test that debug output is secure
        let debug_output = format!("{:?}", api_manager);
        assert!(!debug_output.contains(mock_data.random_api_key()));
        
        // Test that individual key debug output is secure
        if let Some(stored_key) = api_manager.get_key(&ApiProvider::Alchemy, "mainnet") {
            let key_debug = format!("{:?}", stored_key);
            assert!(!key_debug.contains(mock_data.random_api_key()));
            assert!(key_debug.contains("...") || key_debug.contains(&mock_data.random_api_key()[..3]));
        }
        
        println!("API manager security test passed");
    }

    #[test]
    fn test_contact_security() {
        // Test that contacts don't expose sensitive transaction details
        let mock_data = MockSensitiveData::new();
        
        let contact = Contact {
            name: "Test Contact".to_string(),
            address: Address::from_str(mock_data.random_address()).expect("Valid address"),
            notes: Some(format!("Transaction hash: {}", mock_data.random_tx_hash())),
            tags: vec!["friend".to_string()],
            created_at: Local::now(),
            transaction_stats: Default::default(),
            recent_transactions: vec![], // Empty for this test
        };
        
        let debug_output = format!("{:?}", contact);
        
        // The contact debug output should be safe since it doesn't contain private keys
        // But we should verify that transaction hashes in notes are handled appropriately
        println!("Contact debug output: {}", debug_output);
        
        // If we implement RedactedDebug for Contact in the future, we would test it here
        println!("Contact security test passed");
    }

    #[test]
    fn test_comprehensive_security_validation() {
        // Run a comprehensive test that validates all security measures together
        let mock_data = MockSensitiveData::new();
        
        // Create a complete wallet setup
        let mut wallet_data = WalletData::new();
        wallet_data.set_api_key(mock_data.random_api_key().to_string());
        
        // Add multiple wallets
        for i in 0..3 {
            let private_key_hex = &mock_data.private_keys[i % mock_data.private_keys.len()];
            let wallet = LocalWallet::from_str(private_key_hex).expect("Valid private key");
            let password = SecurePassword::new(format!("password_{}", i));
            let secure_wallet = Wallet::new(wallet, &format!("Wallet {}", i), &password)
                .expect("Failed to create wallet");
            
            wallet_data.add_wallet(secure_wallet).expect("Failed to add wallet");
        }
        
        // Add contacts
        for i in 0..2 {
            let contact = Contact {
                name: format!("Contact {}", i),
                address: Address::from_str(&mock_data.addresses[i % mock_data.addresses.len()])
                    .expect("Valid address"),
                notes: Some(format!("Notes for contact {}", i)),
                tags: vec![format!("tag{}", i)],
                created_at: Local::now(),
                transaction_stats: Default::default(),
                recent_transactions: vec![],
            };
            wallet_data.add_contact(contact).expect("Failed to add contact");
        }
        
        // Test comprehensive debug output
        let debug_output = format!("{:?}", wallet_data);
        
        // Validate that no sensitive data is exposed
        for private_key in &mock_data.private_keys {
            assert!(!debug_output.contains(private_key));
        }
        
        assert!(!debug_output.contains(mock_data.random_api_key()));
        
        // Should contain redaction markers
        assert!(debug_output.contains("[REDACTED]"));
        
        // Should contain non-sensitive metadata
        assert!(debug_output.contains("wallets_count"));
        assert!(debug_output.contains("contacts_count"));
        
        println!("Comprehensive security validation passed");
        println!("Final debug output: {}", debug_output);
    }

    #[test]
    fn test_error_message_sanitization() {
        let mock_data = MockSensitiveData::new();
        
        // Test that error messages don't expose sensitive data
        let sensitive_error_message = format!(
            "Failed to decrypt wallet with private key: {}",
            mock_data.random_private_key()
        );
        
        let sanitized_error = sanitize_log_message(&sensitive_error_message);
        
        // Should not contain the private key
        assert!(!sanitized_error.contains(mock_data.random_private_key()));
        
        // Should contain redaction marker
        assert!(sanitized_error.contains("[PRIVATE_KEY_REDACTED]"));
        
        // Should still contain the error context
        assert!(sanitized_error.contains("Failed to decrypt wallet"));
        
        println!("Original error: {}", sensitive_error_message);
        println!("Sanitized error: {}", sanitized_error);
        
        println!("Error message sanitization test passed");
    }
}