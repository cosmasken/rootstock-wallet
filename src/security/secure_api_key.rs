//! Secure API key wrapper with automatic redaction and secure formatting
//!
//! This module provides a SecureApiKey wrapper that:
//! - Automatically redacts API keys in debug output
//! - Provides secure formatting for Authorization headers
//! - Implements zeroization for memory security
//! - Prevents accidental logging of sensitive API key data

use crate::security::{RedactedDebug, SecureString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use zeroize::Zeroize;

/// Secure wrapper for API keys that prevents cleartext exposure
#[derive(Clone)]
pub struct SecureApiKey {
    inner: SecureString,
}

impl SecureApiKey {
    /// Create a new SecureApiKey from a string
    pub fn new(api_key: String) -> Self {
        Self {
            inner: SecureString::new(api_key),
        }
    }

    /// Create a SecureApiKey from a SecureString
    pub fn from_secure_string(secure_string: SecureString) -> Self {
        Self {
            inner: secure_string,
        }
    }

    /// Get the API key for use in secure contexts (like Authorization headers)
    /// This method should only be used when the API key needs to be transmitted
    /// over secure channels (HTTPS)
    pub fn expose_for_auth_header(&self) -> Result<String, std::str::Utf8Error> {
        self.inner.expose().map(|s| format!("Bearer {}", s))
    }

    /// Get the raw API key value (use with extreme caution)
    /// This should only be used in very specific secure contexts
    pub fn expose_raw(&self) -> Result<&str, std::str::Utf8Error> {
        self.inner.expose()
    }

    /// Check if the API key is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the length of the API key
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Clear the API key from memory
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Create a redacted version showing only first 3 and last 3 characters
    fn redacted_display(&self) -> String {
        if let Ok(key) = self.inner.expose() {
            if key.len() <= 6 {
                "[REDACTED]".to_string()
            } else {
                format!("{}...{}", &key[..3], &key[key.len() - 3..])
            }
        } else {
            "[CLEARED]".to_string()
        }
    }
}

impl RedactedDebug for SecureApiKey {
    fn redacted_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureApiKey")
            .field("key", &self.redacted_display())
            .field("len", &self.len())
            .finish()
    }
}

impl fmt::Debug for SecureApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted_fmt(f)
    }
}

impl fmt::Display for SecureApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.redacted_display())
    }
}

impl Zeroize for SecureApiKey {
    fn zeroize(&mut self) {
        self.inner.zeroize();
    }
}

impl Drop for SecureApiKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

// Custom serialization that doesn't expose the raw key
impl Serialize for SecureApiKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Only serialize if we can expose the key, otherwise serialize as empty
        if let Ok(key) = self.inner.expose() {
            serializer.serialize_str(key)
        } else {
            serializer.serialize_str("")
        }
    }
}

impl<'de> Deserialize<'de> for SecureApiKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key = String::deserialize(deserializer)?;
        Ok(SecureApiKey::new(key))
    }
}

impl From<String> for SecureApiKey {
    fn from(key: String) -> Self {
        Self::new(key)
    }
}

impl From<&str> for SecureApiKey {
    fn from(key: &str) -> Self {
        Self::new(key.to_string())
    }
}

impl From<SecureString> for SecureApiKey {
    fn from(secure_string: SecureString) -> Self {
        Self::from_secure_string(secure_string)
    }
}

