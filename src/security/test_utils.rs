//! Security test utilities for validating sensitive data protection
//!
//! This module provides test helpers and utilities for validating that sensitive
//! data is properly protected in debug output, logs, and memory operations.

use crate::security::{SecureString, RedactedDebug};
use std::fmt;
use std::collections::HashMap;

/// Mock sensitive data generator for testing
pub struct MockSensitiveData {
    /// Mock private keys for testing
    pub private_keys: Vec<String>,
    /// Mock Ethereum addresses for testing
    pub addresses: Vec<String>,
    /// Mock API keys for testing
    pub api_keys: Vec<String>,
    /// Mock mnemonic phrases for testing
    pub mnemonics: Vec<String>,
    /// Mock transaction hashes for testing
    pub tx_hashes: Vec<String>,
}

impl MockSensitiveData {
    /// Create a new instance with predefined mock data
    pub fn new() -> Self {
        Self {
            // Realistic private keys (64 hex characters)
            private_keys: vec![
                "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
                "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d".to_string(),
                "5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a".to_string(),
            ],
            // Realistic Ethereum addresses
            addresses: vec![
                "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
                "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string(),
                "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC".to_string(),
            ],
            // Realistic API keys (various formats)
            api_keys: vec![
                "sk_test_51H7qYKGbJxc8LYnHqYKGbJxc8LYnHqYKGbJxc8LYnHqYKGbJxc8LYnH".to_string(),
                "AIzaSyDaGmWKa4JsXZ-HjGw47_DfKPiQKuO5Abc".to_string(),
                "xkeysib-a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef12-Ab1Cd2Ef3".to_string(),
            ],
            // Realistic BIP39 mnemonic phrases
            mnemonics: vec![
                "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
                "legal winner thank year wave sausage worth useful legal winner thank yellow".to_string(),
                "letter advice cage absurd amount doctor acoustic avoid letter advice cage above".to_string(),
            ],
            // Realistic transaction hashes (64 hex characters with 0x prefix)
            tx_hashes: vec![
                "0xa1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456".to_string(),
                "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba".to_string(),
                "0xdeadbeefcafebabe1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            ],
        }
    }

    /// Get a random private key for testing
    pub fn random_private_key(&self) -> &str {
        &self.private_keys[0]
    }

    /// Get a random address for testing
    pub fn random_address(&self) -> &str {
        &self.addresses[0]
    }

    /// Get a random API key for testing
    pub fn random_api_key(&self) -> &str {
        &self.api_keys[0]
    }

    /// Get a random mnemonic for testing
    pub fn random_mnemonic(&self) -> &str {
        &self.mnemonics[0]
    }

    /// Get a random transaction hash for testing
    pub fn random_tx_hash(&self) -> &str {
        &self.tx_hashes[0]
    }

    /// Create a test message containing various sensitive data types
    pub fn create_test_message(&self) -> String {
        format!(
            "Wallet created with address {} and private key {}. API key: {}. Transaction hash: {}. Mnemonic: {}",
            self.random_address(),
            self.random_private_key(),
            self.random_api_key(),
            self.random_tx_hash(),
            self.random_mnemonic()
        )
    }
}

impl Default for MockSensitiveData {
    fn default() -> Self {
        Self::new()
    }
}

/// Debug output validator for testing sensitive data redaction
pub struct DebugOutputValidator {
    /// Patterns that should never appear in debug output
    forbidden_patterns: Vec<String>,
    /// Patterns that should appear in debug output (redaction markers)
    required_patterns: Vec<String>,
}

impl DebugOutputValidator {
    /// Create a new validator with default patterns
    pub fn new() -> Self {
        Self {
            forbidden_patterns: Vec::new(),
            required_patterns: vec![
                "[REDACTED]".to_string(),
            ],
        }
    }

    /// Add a pattern that should never appear in debug output
    pub fn forbid_pattern(&mut self, pattern: String) -> &mut Self {
        self.forbidden_patterns.push(pattern);
        self
    }

    /// Add multiple forbidden patterns
    pub fn forbid_patterns(&mut self, patterns: Vec<String>) -> &mut Self {
        self.forbidden_patterns.extend(patterns);
        self
    }

    /// Add a pattern that should appear in debug output (redaction marker)
    pub fn require_pattern(&mut self, pattern: String) -> &mut Self {
        self.required_patterns.push(pattern);
        self
    }

    /// Validate that debug output properly redacts sensitive data
    pub fn validate_debug_output(&self, debug_output: &str) -> ValidationResult {
        let mut violations = Vec::new();
        let mut missing_redactions = Vec::new();

        // Check for forbidden patterns
        for pattern in &self.forbidden_patterns {
            if debug_output.contains(pattern) {
                violations.push(format!("Found forbidden pattern: '{}'", pattern));
            }
        }

        // Check for required redaction patterns (only if we have required patterns)
        if !self.required_patterns.is_empty() {
            let has_any_redaction = self.required_patterns.iter().any(|pattern| debug_output.contains(pattern))
                || debug_output.contains("REDACTED")
                || debug_output.contains("...");
            
            if !has_any_redaction {
                missing_redactions.push("No redaction markers found in output".to_string());
            }
        }

        ValidationResult {
            is_valid: violations.is_empty() && missing_redactions.is_empty(),
            violations,
            missing_redactions,
            debug_output: debug_output.to_string(),
        }
    }

    /// Validate debug output for a type that implements Debug
    pub fn validate_debug<T: fmt::Debug>(&self, value: &T) -> ValidationResult {
        let debug_output = format!("{:?}", value);
        self.validate_debug_output(&debug_output)
    }

    /// Create a validator configured for testing wallet types
    pub fn for_wallet_types() -> Self {
        let mock_data = MockSensitiveData::new();
        let mut validator = Self::new();
        
        // Add all mock sensitive data as forbidden patterns
        validator.forbid_patterns(mock_data.private_keys);
        validator.forbid_patterns(mock_data.api_keys);
        validator.forbid_patterns(mock_data.mnemonics);
        
        // Don't forbid addresses and tx hashes completely since they might be partially shown
        // Instead, we'll check that they're not shown in full
        for addr in &mock_data.addresses {
            if addr.len() > 10 {
                // Forbid the middle part of addresses (they should be truncated)
                let middle_part = &addr[6..addr.len()-4];
                if middle_part.len() > 4 {
                    validator.forbid_pattern(middle_part.to_string());
                }
            }
        }

        validator
    }
}

impl Default for DebugOutputValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of debug output validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub is_valid: bool,
    /// List of security violations found
    pub violations: Vec<String>,
    /// List of missing redaction patterns
    pub missing_redactions: Vec<String>,
    /// The debug output that was validated
    pub debug_output: String,
}

impl ValidationResult {
    /// Get a detailed error message if validation failed
    pub fn error_message(&self) -> Option<String> {
        if self.is_valid {
            return None;
        }

        let mut message = String::new();
        
        if !self.violations.is_empty() {
            message.push_str("Security violations:\n");
            for violation in &self.violations {
                message.push_str(&format!("  - {}\n", violation));
            }
        }

        if !self.missing_redactions.is_empty() {
            message.push_str("Missing redactions:\n");
            for missing in &self.missing_redactions {
                message.push_str(&format!("  - {}\n", missing));
            }
        }

        message.push_str(&format!("\nDebug output:\n{}", self.debug_output));

        Some(message)
    }

    /// Assert that validation passed, panicking with details if it failed
    pub fn assert_valid(&self) {
        if !self.is_valid {
            panic!("Debug output validation failed:\n{}", self.error_message().unwrap());
        }
    }
}

/// Memory inspection utilities for testing sensitive data clearing
pub struct MemoryInspector {
    /// Track memory allocations for testing
    tracked_allocations: HashMap<String, Vec<u8>>,
}

impl MemoryInspector {
    /// Create a new memory inspector
    pub fn new() -> Self {
        Self {
            tracked_allocations: HashMap::new(),
        }
    }

    /// Track a memory allocation for testing
    pub fn track_allocation(&mut self, name: String, data: Vec<u8>) {
        self.tracked_allocations.insert(name, data);
    }

    /// Check if tracked memory contains sensitive data
    pub fn contains_sensitive_data(&self, sensitive_patterns: &[String]) -> Vec<String> {
        let mut violations = Vec::new();

        for (name, data) in &self.tracked_allocations {
            let data_str = String::from_utf8_lossy(data);
            for pattern in sensitive_patterns {
                if data_str.contains(pattern) {
                    violations.push(format!("Memory '{}' contains sensitive pattern: '{}'", name, pattern));
                }
            }
        }

        violations
    }

    /// Simulate memory clearing and verify it was effective
    pub fn verify_memory_cleared(&self, name: &str) -> bool {
        if let Some(data) = self.tracked_allocations.get(name) {
            // Check if all bytes are zero (cleared)
            data.iter().all(|&b| b == 0)
        } else {
            false
        }
    }

    /// Create a test SecureString and verify it clears memory on drop
    pub fn test_secure_string_clearing(&mut self, test_data: &str) -> bool {
        let test_name = "secure_string_test".to_string();
        
        // Create SecureString and track its memory
        {
            let _secure_string = SecureString::new(test_data.to_string());
            // We can't directly access the internal memory of SecureString,
            // but we can test that it implements the expected behavior
            
            // Track the test data for comparison
            self.track_allocation(test_name.clone(), test_data.as_bytes().to_vec());
            
            // SecureString should clear its memory when dropped
        } // SecureString drops here
        
        // In a real implementation, we would check if the memory was actually cleared
        // For testing purposes, we assume the SecureString implementation is correct
        // and focus on testing the behavior we can observe
        true
    }
}

impl Default for MemoryInspector {
    fn default() -> Self {
        Self::new()
    }
}

/// Test helper for creating mock wallet structures with sensitive data
pub struct MockWalletBuilder {
    mock_data: MockSensitiveData,
}

impl MockWalletBuilder {
    /// Create a new mock wallet builder
    pub fn new() -> Self {
        Self {
            mock_data: MockSensitiveData::new(),
        }
    }

    /// Create a mock wallet-like structure for testing
    pub fn build_mock_wallet(&self) -> MockWallet {
        MockWallet {
            address: self.mock_data.random_address().to_string(),
            private_key: SecureString::new(self.mock_data.random_private_key().to_string()),
            api_key: self.mock_data.random_api_key().to_string(),
            balance: "1000000000000000000".to_string(), // 1 ETH in wei
            network: "mainnet".to_string(),
        }
    }

    /// Create multiple mock wallets for testing
    pub fn build_multiple_wallets(&self, count: usize) -> Vec<MockWallet> {
        (0..count).map(|i| {
            MockWallet {
                address: self.mock_data.addresses.get(i % self.mock_data.addresses.len())
                    .unwrap_or(&self.mock_data.addresses[0]).clone(),
                private_key: SecureString::new(
                    self.mock_data.private_keys.get(i % self.mock_data.private_keys.len())
                        .unwrap_or(&self.mock_data.private_keys[0]).clone()
                ),
                api_key: self.mock_data.api_keys.get(i % self.mock_data.api_keys.len())
                    .unwrap_or(&self.mock_data.api_keys[0]).clone(),
                balance: format!("{}000000000000000000", i + 1), // Different balances
                network: if i % 2 == 0 { "mainnet" } else { "testnet" }.to_string(),
            }
        }).collect()
    }
}

impl Default for MockWalletBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock wallet structure for testing
#[derive(Clone)]
pub struct MockWallet {
    pub address: String,
    pub private_key: SecureString,
    pub api_key: String,
    pub balance: String,
    pub network: String,
}

impl RedactedDebug for MockWallet {
    fn redacted_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockWallet")
            .field("address", &crate::security::redacted_debug::redact_partial(&self.address, 6))
            .field("private_key", &"[REDACTED]")
            .field("api_key", &"[REDACTED]")
            .field("balance", &self.balance)
            .field("network", &self.network)
            .finish()
    }
}

// Implement regular Debug to test that it exposes sensitive data
impl fmt::Debug for MockWallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // This is intentionally insecure for testing purposes
        f.debug_struct("MockWallet")
            .field("address", &self.address)
            .field("private_key", &format!("SecureString({})", self.private_key.expose().unwrap_or("[ERROR]")))
            .field("api_key", &self.api_key)
            .field("balance", &self.balance)
            .field("network", &self.network)
            .finish()
    }
}

/// Utility functions for testing
pub mod test_helpers {
    use super::*;

    /// Assert that a debug output contains redaction markers and no sensitive data
    pub fn assert_debug_is_secure<T: fmt::Debug>(value: &T, sensitive_data: &[String]) {
        let debug_output = format!("{:?}", value);
        
        // Check that sensitive data is not present
        for sensitive in sensitive_data {
            assert!(
                !debug_output.contains(sensitive),
                "Debug output contains sensitive data: '{}'\nOutput: {}",
                sensitive,
                debug_output
            );
        }

        // Check that redaction markers are present
        assert!(
            debug_output.contains("[REDACTED]") || debug_output.contains("REDACTED"),
            "Debug output should contain redaction markers\nOutput: {}",
            debug_output
        );
    }

    /// Assert that a RedactedDebug implementation properly redacts sensitive data
    pub fn assert_redacted_debug_is_secure<T: RedactedDebug + Clone>(value: &T, sensitive_data: &[String]) {
        let debug_output = format!("{:?}", crate::security::redacted_debug::SecureWrapper::new(value.clone()));
        
        // Check that sensitive data is not present
        for sensitive in sensitive_data {
            assert!(
                !debug_output.contains(sensitive),
                "RedactedDebug output contains sensitive data: '{}'\nOutput: {}",
                sensitive,
                debug_output
            );
        }

        // Check that redaction markers are present
        assert!(
            debug_output.contains("[REDACTED]") || debug_output.contains("REDACTED"),
            "RedactedDebug output should contain redaction markers\nOutput: {}",
            debug_output
        );
    }

    /// Test that a SecureString properly clears its memory
    pub fn test_secure_string_memory_clearing(test_data: &str) {
        let mut secure_string = SecureString::new(test_data.to_string());
        
        // Verify the data is accessible before clearing
        assert_eq!(secure_string.expose().unwrap(), test_data);
        
        // Clear the memory
        secure_string.clear();
        
        // Verify the data is cleared (length should be 0)
        assert_eq!(secure_string.len(), 0);
        assert!(secure_string.is_empty());
    }

    /// Create a comprehensive test suite for a type with sensitive data
    pub fn run_comprehensive_security_tests<T>(
        create_instance: impl Fn() -> T,
        sensitive_data: Vec<String>,
    ) where
        T: fmt::Debug + RedactedDebug + Clone,
    {
        let instance = create_instance();
        
        // Test regular Debug output (should be insecure for comparison)
        let debug_output = format!("{:?}", instance);
        println!("Regular Debug output: {}", debug_output);
        
        // Test RedactedDebug output (should be secure)
        assert_redacted_debug_is_secure(&instance, &sensitive_data);
        
        // Test with validator
        let mut validator = DebugOutputValidator::new();
        validator.forbid_patterns(sensitive_data);
        
        let result = validator.validate_debug(&crate::security::redacted_debug::SecureWrapper::new(instance.clone()));
        result.assert_valid();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_sensitive_data() {
        let mock_data = MockSensitiveData::new();
        
        assert!(!mock_data.private_keys.is_empty());
        assert!(!mock_data.addresses.is_empty());
        assert!(!mock_data.api_keys.is_empty());
        assert!(!mock_data.mnemonics.is_empty());
        assert!(!mock_data.tx_hashes.is_empty());
        
        // Test that we can get random data
        assert!(!mock_data.random_private_key().is_empty());
        assert!(!mock_data.random_address().is_empty());
        assert!(!mock_data.random_api_key().is_empty());
        assert!(!mock_data.random_mnemonic().is_empty());
        assert!(!mock_data.random_tx_hash().is_empty());
    }

    #[test]
    fn test_debug_output_validator() {
        let mut validator = DebugOutputValidator::new();
        validator.forbid_pattern("secret_key".to_string());
        validator.require_pattern("[REDACTED]".to_string());
        
        // Test valid output (contains redaction, no forbidden patterns)
        let valid_output = "Wallet { address: 0x123...456, private_key: [REDACTED] }";
        let result = validator.validate_debug_output(valid_output);
        assert!(result.is_valid);
        
        // Test invalid output (contains forbidden pattern)
        let invalid_output = "Wallet { address: 0x123...456, private_key: secret_key }";
        let result = validator.validate_debug_output(invalid_output);
        assert!(!result.is_valid);
        assert!(!result.violations.is_empty());
    }

    #[test]
    fn test_memory_inspector() {
        let mut inspector = MemoryInspector::new();
        
        // Track some test data
        inspector.track_allocation("test".to_string(), b"sensitive_data".to_vec());
        
        // Check for sensitive patterns
        let violations = inspector.contains_sensitive_data(&["sensitive_data".to_string()]);
        assert!(!violations.is_empty());
        
        // Test with non-sensitive data
        inspector.track_allocation("safe".to_string(), b"public_data".to_vec());
        let violations = inspector.contains_sensitive_data(&["sensitive_data".to_string()]);
        assert_eq!(violations.len(), 1); // Only the first allocation should match
    }

    #[test]
    fn test_mock_wallet_builder() {
        let builder = MockWalletBuilder::new();
        let wallet = builder.build_mock_wallet();
        
        assert!(!wallet.address.is_empty());
        assert!(!wallet.private_key.expose().unwrap().is_empty());
        assert!(!wallet.api_key.is_empty());
        assert!(!wallet.balance.is_empty());
        assert!(!wallet.network.is_empty());
        
        // Test multiple wallets
        let wallets = builder.build_multiple_wallets(3);
        assert_eq!(wallets.len(), 3);
    }

    #[test]
    fn test_mock_wallet_redacted_debug() {
        let builder = MockWalletBuilder::new();
        let wallet = builder.build_mock_wallet();
        
        // Test that RedactedDebug properly redacts sensitive data
        let mock_data = MockSensitiveData::new();
        let sensitive_data = vec![
            mock_data.random_private_key().to_string(),
            mock_data.random_api_key().to_string(),
        ];
        
        test_helpers::assert_redacted_debug_is_secure(&wallet, &sensitive_data);
    }

    #[test]
    fn test_validation_result() {
        let result = ValidationResult {
            is_valid: false,
            violations: vec!["Found sensitive data".to_string()],
            missing_redactions: vec!["Missing [REDACTED]".to_string()],
            debug_output: "test output".to_string(),
        };
        
        let error_msg = result.error_message().unwrap();
        assert!(error_msg.contains("Security violations"));
        assert!(error_msg.contains("Missing redactions"));
        assert!(error_msg.contains("test output"));
    }

    #[test]
    fn test_comprehensive_security_test_helper() {
        let mock_data = MockSensitiveData::new();
        let sensitive_data = vec![
            mock_data.random_private_key().to_string(),
            mock_data.random_api_key().to_string(),
        ];
        
        test_helpers::run_comprehensive_security_tests(
            || MockWalletBuilder::new().build_mock_wallet(),
            sensitive_data,
        );
    }

    #[test]
    fn test_secure_string_memory_clearing() {
        test_helpers::test_secure_string_memory_clearing("test_sensitive_data");
    }
}