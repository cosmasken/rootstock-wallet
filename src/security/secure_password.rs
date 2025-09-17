//! Secure password handling utilities

use std::fmt;
use zeroize::Zeroize;

/// A secure wrapper for password data that automatically clears memory on drop
#[derive(Clone)]
pub struct SecurePassword {
    data: Vec<u8>,
}

impl SecurePassword {
    /// Create a new SecurePassword from a string
    pub fn new(password: String) -> Self {
        Self {
            data: password.into_bytes(),
        }
    }

    /// Create a new SecurePassword from bytes
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Expose the password as a string slice (use with caution)
    pub fn expose(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    /// Expose the password as bytes (use with caution)
    pub fn expose_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the length of the password
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the password is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Manually clear the password from memory
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

impl From<String> for SecurePassword {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecurePassword {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl fmt::Debug for SecurePassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecurePassword([REDACTED {} bytes])", self.data.len())
    }
}

impl fmt::Display for SecurePassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl PartialEq for SecurePassword {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for SecurePassword {}

impl Zeroize for SecurePassword {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

impl Drop for SecurePassword {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Secure password input function that returns a SecurePassword
pub fn prompt_secure_password(prompt: &str) -> Result<SecurePassword, anyhow::Error> {
    let password = rpassword::prompt_password(prompt)?;
    Ok(SecurePassword::new(password))
}

/// Secure password input with confirmation
pub fn prompt_secure_password_with_confirmation(
    prompt: &str,
    confirm_prompt: &str,
) -> Result<SecurePassword, anyhow::Error> {
    loop {
        let password = prompt_secure_password(prompt)?;
        let confirm = prompt_secure_password(confirm_prompt)?;

        if password == confirm {
            return Ok(password);
        }

        eprintln!("Passwords don't match. Please try again.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_password_creation() {
        let password = SecurePassword::new("test_password".to_string());
        assert_eq!(password.expose().unwrap(), "test_password");
        assert_eq!(password.len(), 13);
        assert!(!password.is_empty());
    }

    #[test]
    fn test_secure_password_debug() {
        let password = SecurePassword::new("sensitive_password".to_string());
        let debug_output = format!("{:?}", password);
        assert!(debug_output.contains("[REDACTED"));
        assert!(!debug_output.contains("sensitive_password"));
    }

    #[test]
    fn test_secure_password_display() {
        let password = SecurePassword::new("sensitive_password".to_string());
        let display_output = format!("{}", password);
        assert_eq!(display_output, "[REDACTED]");
    }

    #[test]
    fn test_secure_password_clear() {
        let mut password = SecurePassword::new("test_password".to_string());
        password.clear();
        assert_eq!(password.len(), 0);
        assert!(password.is_empty());
    }

    #[test]
    fn test_secure_password_equality() {
        let password1 = SecurePassword::new("same_password".to_string());
        let password2 = SecurePassword::new("same_password".to_string());
        let password3 = SecurePassword::new("different_password".to_string());

        assert_eq!(password1, password2);
        assert_ne!(password1, password3);
    }
}
