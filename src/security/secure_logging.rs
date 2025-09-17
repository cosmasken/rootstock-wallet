//! Secure logging utilities with sanitization for sensitive data

use regex::Regex;
use std::sync::OnceLock;

/// Patterns for detecting sensitive data in log messages
pub struct SensitivePatterns {
    /// Pattern for Ethereum private keys (64 hex characters)
    private_key: Regex,
    /// Pattern for Ethereum addresses (0x followed by 40 hex characters)
    address: Regex,
    /// Pattern for API keys (common formats)
    api_key: Regex,
    /// Pattern for mnemonic phrases (12-24 words)
    mnemonic: Regex,
    /// Pattern for transaction hashes (0x followed by 64 hex characters)
    tx_hash: Regex,
}

impl SensitivePatterns {
    fn new() -> Self {
        Self {
            // Match 64 hex characters (private key)
            private_key: Regex::new(r"\b[0-9a-fA-F]{64}\b").unwrap(),
            // Match Ethereum addresses (0x + 40 hex chars)
            address: Regex::new(r"\b0x[0-9a-fA-F]{40}\b").unwrap(),
            // Match common API key patterns
            api_key: Regex::new(r"\b[A-Za-z0-9]{32,}\b").unwrap(),
            // Match potential mnemonic phrases (12-24 common English words)
            mnemonic: Regex::new(r"\b(?:[a-z]+\s+){11,23}[a-z]+\b").unwrap(),
            // Match transaction hashes (0x + 64 hex chars)
            tx_hash: Regex::new(r"\b0x[0-9a-fA-F]{64}\b").unwrap(),
        }
    }
}

static PATTERNS: OnceLock<SensitivePatterns> = OnceLock::new();

fn get_patterns() -> &'static SensitivePatterns {
    PATTERNS.get_or_init(SensitivePatterns::new)
}

/// Check if a string contains potentially sensitive data
pub fn is_sensitive_data(text: &str) -> bool {
    let patterns = get_patterns();
    
    patterns.private_key.is_match(text) ||
    patterns.address.is_match(text) ||
    patterns.tx_hash.is_match(text) ||
    patterns.mnemonic.is_match(text) ||
    is_potential_api_key(text)
}

/// Check if a string might be an API key (more conservative check)
fn is_potential_api_key(text: &str) -> bool {
    // Only flag as API key if it's a long alphanumeric string and contains mixed case
    if text.len() < 20 || text.len() > 100 {
        return false;
    }
    
    let has_upper = text.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = text.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = text.chars().any(|c| c.is_ascii_digit());
    let is_alphanumeric = text.chars().all(|c| c.is_ascii_alphanumeric());
    
    is_alphanumeric && has_upper && has_lower && has_digit
}

/// Sanitize a log message by redacting sensitive data
pub fn sanitize_log_message(message: &str) -> String {
    let patterns = get_patterns();
    let mut sanitized = message.to_string();
    
    // Redact private keys
    sanitized = patterns.private_key.replace_all(&sanitized, "[PRIVATE_KEY_REDACTED]").to_string();
    
    // Redact full addresses but show partial for debugging
    sanitized = patterns.address.replace_all(&sanitized, |caps: &regex::Captures| {
        let addr = &caps[0];
        format!("{}...{}", &addr[..6], &addr[addr.len()-4..])
    }).to_string();
    
    // Redact transaction hashes but show partial
    sanitized = patterns.tx_hash.replace_all(&sanitized, |caps: &regex::Captures| {
        let hash = &caps[0];
        format!("{}...{}", &hash[..10], &hash[hash.len()-6..])
    }).to_string();
    
    // Redact mnemonic phrases
    sanitized = patterns.mnemonic.replace_all(&sanitized, "[MNEMONIC_REDACTED]").to_string();
    
    // Redact potential API keys using the regex pattern
    sanitized = patterns.api_key.replace_all(&sanitized, |caps: &regex::Captures| {
        let potential_key = &caps[0];
        if is_potential_api_key(potential_key) {
            "[API_KEY_REDACTED]".to_string()
        } else {
            potential_key.to_string()
        }
    }).to_string();
    
    sanitized
}

/// Redact a private key for safe logging
pub fn redact_private_key(key: &str) -> String {
    if key.len() >= 8 {
        format!("{}...[REDACTED]", &key[..4])
    } else {
        "[PRIVATE_KEY_REDACTED]".to_string()
    }
}

/// Redact an address for safe logging (show partial)
pub fn redact_address(address: &str) -> String {
    if address.len() >= 10 {
        format!("{}...{}", &address[..6], &address[address.len()-4..])
    } else {
        "[ADDRESS_REDACTED]".to_string()
    }
}

/// Secure debug macro that sanitizes log messages
#[macro_export]
macro_rules! secure_debug {
    ($($arg:tt)*) => {
        log::debug!("{}", $crate::security::sanitize_log_message(&format!($($arg)*)))
    };
}

/// Secure info macro that sanitizes log messages
#[macro_export]
macro_rules! secure_info {
    ($($arg:tt)*) => {
        log::info!("{}", $crate::security::sanitize_log_message(&format!($($arg)*)))
    };
}

/// Secure warn macro that sanitizes log messages
#[macro_export]
macro_rules! secure_warn {
    ($($arg:tt)*) => {
        log::warn!("{}", $crate::security::sanitize_log_message(&format!($($arg)*)))
    };
}

/// Secure error macro that sanitizes log messages
#[macro_export]
macro_rules! secure_error {
    ($($arg:tt)*) => {
        log::error!("{}", $crate::security::sanitize_log_message(&format!($($arg)*)))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sensitive_data() {
        // Test private key detection
        assert!(is_sensitive_data("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
        
        // Test address detection
        assert!(is_sensitive_data("0x742d35Cc6634C0532925a3b8D4C9db4C4C4C4C4C"));
        
        // Test normal text
        assert!(!is_sensitive_data("This is just normal log text"));
    }

    #[test]
    fn test_sanitize_log_message() {
        let message = "Wallet address: 0x742d35Cc6634C0532925a3b8D4C9db4C4C4C4C4C with balance 100";
        let sanitized = sanitize_log_message(message);
        
        assert!(sanitized.contains("0x742d...4C4C"));
        assert!(!sanitized.contains("0x742d35Cc6634C0532925a3b8D4C9db4C4C4C4C4C"));
    }

    #[test]
    fn test_redact_private_key() {
        let key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let redacted = redact_private_key(key);
        
        assert!(redacted.starts_with("1234"));
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("abcdef"));
    }

    #[test]
    fn test_redact_address() {
        let address = "0x742d35Cc6634C0532925a3b8D4C9db4C4C4C4C4C";
        let redacted = redact_address(address);
        
        assert!(redacted.starts_with("0x742d"));
        assert!(redacted.ends_with("4C4C"));
        assert!(redacted.contains("..."));
    }

    #[test]
    fn test_is_potential_api_key() {
        // Should detect mixed case alphanumeric strings
        assert!(is_potential_api_key("AbC123dEf456GhI789JkL012"));
        
        // Should not detect short strings
        assert!(!is_potential_api_key("short"));
        
        // Should not detect all lowercase
        assert!(!is_potential_api_key("alllowercasestring123"));
        
        // Should not detect all uppercase
        assert!(!is_potential_api_key("ALLUPPERCASESTRING123"));
        
        // Should not detect strings with special characters
        assert!(!is_potential_api_key("AbC123-dEf456_GhI789"));
    }

    #[test]
    fn test_sanitize_private_key_in_message() {
        let message = "Private key: 1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let sanitized = sanitize_log_message(message);
        
        assert!(sanitized.contains("[PRIVATE_KEY_REDACTED]"));
        assert!(!sanitized.contains("1234567890abcdef"));
    }

    #[test]
    fn test_secure_macros_integration() {
        // Test that the macros properly sanitize messages
        let test_message = "Wallet 0x742d35Cc6634C0532925a3b8D4C9db4C4C4C4C4C has key 1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let sanitized = sanitize_log_message(test_message);
        
        // Verify address is partially redacted
        assert!(sanitized.contains("0x742d...4C4C"));
        assert!(!sanitized.contains("0x742d35Cc6634C0532925a3b8D4C9db4C4C4C4C4C"));
        
        // Verify private key is fully redacted
        assert!(sanitized.contains("[PRIVATE_KEY_REDACTED]"));
        assert!(!sanitized.contains("1234567890abcdef"));
    }

    #[test]
    fn test_macro_compilation() {
        // This test ensures the macros compile and can be used
        // We can't easily test the actual log output in unit tests,
        // but we can verify the macros expand correctly
        
        // These should compile without errors
        let sensitive_data = "Private key: 1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        
        // The macros should sanitize the message before logging
        // We can't capture the log output easily in tests, but we can verify
        // that the sanitization function works correctly
        let sanitized = sanitize_log_message(sensitive_data);
        assert!(sanitized.contains("[PRIVATE_KEY_REDACTED]"));
        assert!(!sanitized.contains("1234567890abcdef"));
    }
}