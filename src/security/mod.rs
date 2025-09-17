//! Security utilities for protecting sensitive data
//!
//! This module provides core security infrastructure including:
//! - SecureString wrapper for sensitive string data with automatic zeroization
//! - RedactedDebug trait for safe debug output that redacts sensitive information
//! - Secure logging utilities with sanitization functions

pub mod secure_string;
pub mod redacted_debug;
pub mod secure_logging;

pub use secure_string::SecureString;
pub use redacted_debug::RedactedDebug;
pub use secure_logging::{
    sanitize_log_message, is_sensitive_data, redact_private_key, redact_address
};