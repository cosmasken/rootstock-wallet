//! Security utilities for protecting sensitive data
//!
//! This module provides core security infrastructure including:
//! - SecureString wrapper for sensitive string data with automatic zeroization
//! - RedactedDebug trait for safe debug output that redacts sensitive information
//! - Secure logging utilities with sanitization functions

pub mod redacted_debug;
pub mod secure_logging;
pub mod secure_string;

pub use redacted_debug::RedactedDebug;
pub use secure_logging::{
    is_sensitive_data, redact_address, redact_private_key, sanitize_log_message,
};
pub use secure_string::SecureString;
