//! SecureString implementation for protecting sensitive string data in memory

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use zeroize::Zeroize;

/// A secure wrapper for sensitive string data that automatically clears memory on drop
#[derive(Clone)]
pub struct SecureString {
    data: Vec<u8>,
}

impl SecureString {
    /// Create a new SecureString from a regular string
    pub fn new(data: String) -> Self {
        Self {
            data: data.into_bytes(),
        }
    }

    /// Create a new SecureString from bytes
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Expose the underlying string data (use with caution)
    pub fn expose(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    /// Get the length of the underlying data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the SecureString is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Manually clear the sensitive data from memory
    pub fn clear(&mut self) {
        self.data.zeroize();
    }

    /// Convert to a regular String (consumes self and clears memory)
    pub fn into_string(mut self) -> Result<String, std::string::FromUtf8Error> {
        let result = String::from_utf8(self.data.clone());
        self.data.zeroize();
        result
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureString([REDACTED {} bytes])", self.data.len())
    }
}

impl fmt::Display for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl PartialEq for SecureString {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for SecureString {}

// Custom serialization that redacts the actual data
impl Serialize for SecureString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("[REDACTED]")
    }
}

// Custom deserialization - this should only be used for loading from secure storage
impl<'de> Deserialize<'de> for SecureString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SecureString::new(s))
    }
}

impl Zeroize for SecureString {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_string_creation() {
        let secure = SecureString::new("test_data".to_string());
        assert_eq!(secure.expose().unwrap(), "test_data");
        assert_eq!(secure.len(), 9);
        assert!(!secure.is_empty());
    }

    #[test]
    fn test_secure_string_debug() {
        let secure = SecureString::new("sensitive_data".to_string());
        let debug_output = format!("{:?}", secure);
        assert!(debug_output.contains("[REDACTED"));
        assert!(!debug_output.contains("sensitive_data"));
    }

    #[test]
    fn test_secure_string_display() {
        let secure = SecureString::new("sensitive_data".to_string());
        let display_output = format!("{}", secure);
        assert_eq!(display_output, "[REDACTED]");
    }

    #[test]
    fn test_secure_string_clear() {
        let mut secure = SecureString::new("test_data".to_string());
        secure.clear();
        assert_eq!(secure.len(), 0); // Length should be 0 after clearing
        assert!(secure.is_empty());
    }

    #[test]
    fn test_secure_string_serialization() {
        let secure = SecureString::new("sensitive_data".to_string());
        let serialized = serde_json::to_string(&secure).unwrap();
        assert_eq!(serialized, "\"[REDACTED]\"");
    }
}
