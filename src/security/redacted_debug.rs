//! RedactedDebug trait for safe debug output of sensitive data structures

use std::fmt;

/// Trait for types that contain sensitive data and need custom debug formatting
/// 
/// This trait allows types to implement safe debug output that redacts sensitive
/// information while still providing useful debugging information for non-sensitive fields.
pub trait RedactedDebug {
    /// Format the type for debug output with sensitive fields redacted
    fn redacted_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

/// A wrapper type that implements Debug using RedactedDebug
/// 
/// This can be used to wrap existing types and provide secure debug output
#[derive(Clone, PartialEq, Eq)]
pub struct SecureWrapper<T>(pub T);

impl<T> SecureWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn inner(&self) -> &T {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: RedactedDebug> fmt::Debug for SecureWrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.redacted_fmt(f)
    }
}

impl<T> std::ops::Deref for SecureWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for SecureWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for SecureWrapper<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

/// Helper function to redact a string value for debug output
pub fn redact_string(value: &str, show_length: bool) -> String {
    if show_length {
        format!("[REDACTED {} chars]", value.len())
    } else {
        "[REDACTED]".to_string()
    }
}

/// Helper function to redact a byte array for debug output
pub fn redact_bytes(value: &[u8], show_length: bool) -> String {
    if show_length {
        format!("[REDACTED {} bytes]", value.len())
    } else {
        "[REDACTED]".to_string()
    }
}

/// Helper function to show only first and last few characters of a string
pub fn redact_partial(value: &str, show_chars: usize) -> String {
    if value.len() <= show_chars * 2 {
        "[REDACTED]".to_string()
    } else {
        format!(
            "{}...{}",
            &value[..show_chars],
            &value[value.len() - show_chars..]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStruct {
        public_field: String,
        private_field: String,
    }

    impl RedactedDebug for TestStruct {
        fn redacted_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("TestStruct")
                .field("public_field", &self.public_field)
                .field("private_field", &redact_string(&self.private_field, true))
                .finish()
        }
    }

    #[test]
    fn test_redacted_debug() {
        let test_struct = TestStruct {
            public_field: "visible".to_string(),
            private_field: "secret".to_string(),
        };

        let wrapper = SecureWrapper::new(test_struct);
        let debug_output = format!("{:?}", wrapper);
        
        assert!(debug_output.contains("visible"));
        assert!(debug_output.contains("[REDACTED"));
        assert!(!debug_output.contains("secret"));
    }

    #[test]
    fn test_redact_string() {
        assert_eq!(redact_string("secret", false), "[REDACTED]");
        assert_eq!(redact_string("secret", true), "[REDACTED 6 chars]");
    }

    #[test]
    fn test_redact_bytes() {
        let bytes = b"secret_bytes";
        assert_eq!(redact_bytes(bytes, false), "[REDACTED]");
        assert_eq!(redact_bytes(bytes, true), "[REDACTED 12 bytes]");
    }

    #[test]
    fn test_redact_partial() {
        assert_eq!(redact_partial("short", 2), "sh...rt");
        assert_eq!(redact_partial("very_long_secret_key", 3), "ver...key");
    }

    #[test]
    fn test_secure_wrapper_deref() {
        let test_struct = TestStruct {
            public_field: "visible".to_string(),
            private_field: "secret".to_string(),
        };

        let wrapper = SecureWrapper::new(test_struct);
        assert_eq!(wrapper.public_field, "visible");
        assert_eq!(wrapper.private_field, "secret");
    }
}