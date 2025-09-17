//! Security utilities for protecting sensitive data
//!
//! This module provides core security infrastructure including:
//! - SecureString wrapper for sensitive string data with automatic zeroization
//! - SecurePassword wrapper for password data with automatic zeroization
//! - RedactedDebug trait for safe debug output that redacts sensitive information
//! - Secure logging utilities with sanitization functions

pub mod redacted_debug;
pub mod secure_api_key;
pub mod secure_http_client;
pub mod secure_logging;
pub mod secure_password;
pub mod secure_string;

#[cfg(test)]
pub mod test_utils;

// #[cfg(test)]
// mod security_validation_tests;

pub use redacted_debug::RedactedDebug;
pub use secure_api_key::SecureApiKey;
pub use secure_http_client::SecureHttpClient;
pub use secure_logging::{
    is_sensitive_data, redact_address, redact_private_key, sanitize_log_message,
};
pub use secure_password::{
    SecurePassword, prompt_secure_password, prompt_secure_password_with_confirmation,
};
pub use secure_string::SecureString;
