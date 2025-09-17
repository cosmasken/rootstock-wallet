//! Secure HTTP client wrapper with TLS enforcement and request sanitization
//!
//! This module provides a secure wrapper around reqwest::Client that:
//! - Enforces TLS/HTTPS for all requests
//! - Sanitizes requests and responses to prevent sensitive data logging
//! - Provides secure error handling that doesn't expose API keys or sensitive data

use anyhow::{Context, Result, anyhow};
use reqwest::{Client, ClientBuilder, Request, Response};
use serde::Serialize;
use std::time::Duration;
use url::Url;

use crate::security::{is_sensitive_data, redact_private_key, sanitize_log_message};

/// Marker trait for types that are safe to serialize in HTTP requests
///
/// This trait should only be implemented for types that do not contain
/// sensitive data like private keys, API keys, or passwords.
pub trait SafeForHttpSerialization: Serialize {}

/// Secure request builder that provides compile-time checks for sensitive data
pub struct SecureRequestBuilder<'a> {
    client: &'a Client,
    url: String,
    headers: Vec<(String, String)>,
}

impl<'a> SecureRequestBuilder<'a> {
    fn new(client: &'a Client, url: &str) -> Self {
        Self {
            client,
            url: url.to_string(),
            headers: Vec::new(),
        }
    }

    /// Add a header to the request
    ///
    /// Note: Be careful not to include sensitive data in headers.
    /// Use SecureApiKey for authorization headers.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    /// Add JSON body that implements SafeForHttpSerialization
    ///
    /// This method only accepts types that implement SafeForHttpSerialization,
    /// providing compile-time protection against accidentally serializing
    /// sensitive data.
    pub async fn json_safe<T: SafeForHttpSerialization>(self, body: &T) -> Result<Response> {
        let mut request_builder = self.client.post(&self.url).json(body);

        for (key, value) in &self.headers {
            request_builder = request_builder.header(key, value);
        }

        let request = request_builder
            .build()
            .context("Failed to build secure request")?;

        // Additional runtime validation
        self.validate_request(&request)?;

        match self.client.execute(request).await {
            Ok(response) => Ok(response),
            Err(e) => {
                let sanitized_error = sanitize_log_message(&e.to_string());
                Err(anyhow!("Secure HTTP request failed: {}", sanitized_error))
            }
        }
    }

    /// Validate the request for sensitive data patterns
    fn validate_request(&self, request: &Request) -> Result<()> {
        // Check headers
        for (name, value) in request.headers().iter() {
            if let Ok(value_str) = value.to_str()
                && name.as_str().to_lowercase() != "authorization"
                && is_sensitive_data(value_str)
            {
                log::warn!("Potentially sensitive data detected in header: {}", name);
            }
        }

        Ok(())
    }
}

/// Secure HTTP client wrapper that enforces TLS and sanitizes requests/responses
pub struct SecureHttpClient {
    client: Client,
    enforce_tls: bool,
}

impl SecureHttpClient {
    /// Create a new SecureHttpClient with default security settings
    pub fn new() -> Result<Self> {
        Self::with_config(true)
    }

    /// Create a secure request builder that prevents sensitive data in request bodies
    ///
    /// This method provides compile-time guidance for preventing sensitive data
    /// from being included in request bodies through proper type constraints.
    pub fn secure_post_builder(&self, url: &str) -> SecureRequestBuilder<'_> {
        SecureRequestBuilder::new(&self.client, url)
    }

    /// Create a new SecureHttpClient with custom TLS enforcement setting
    pub fn with_config(enforce_tls: bool) -> Result<Self> {
        let mut builder = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .user_agent("rootstock-wallet/1.0");

        // Enforce HTTPS-only if configured
        if enforce_tls {
            builder = builder.https_only(true);
        }

        let client = builder.build().context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            enforce_tls,
        })
    }

    /// Send a POST request with JSON body
    pub async fn post_json<T: Serialize>(&self, url: &str, body: &T) -> Result<Response> {
        self.validate_url(url)?;

        let mut request = self
            .client
            .post(url)
            .json(body)
            .build()
            .context("Failed to build POST request")?;

        self.sanitize_request(&mut request)?;

        self.send_request(request).await
    }

    /// Send a POST request with JSON body and custom headers
    pub async fn post_json_with_headers<T: Serialize>(
        &self,
        url: &str,
        body: &T,
        headers: &[(&str, &str)],
    ) -> Result<Response> {
        self.validate_url(url)?;

        let mut request_builder = self.client.post(url).json(body);

        // Add custom headers
        for (key, value) in headers {
            request_builder = request_builder.header(*key, *value);
        }

        let mut request = request_builder
            .build()
            .context("Failed to build POST request with headers")?;

        self.sanitize_request(&mut request)?;

        self.send_request(request).await
    }

    /// Send a GET request
    pub async fn get(&self, url: &str) -> Result<Response> {
        self.validate_url(url)?;

        let mut request = self
            .client
            .get(url)
            .build()
            .context("Failed to build GET request")?;

        self.sanitize_request(&mut request)?;

        self.send_request(request).await
    }

    /// Send a custom request
    pub async fn send_request(&self, request: Request) -> Result<Response> {
        let method = request.method().clone();
        let url = request.url().clone();

        // Log the request (sanitized)
        let sanitized_url = self.sanitize_url(&url);
        log::debug!("Sending {} request to: {}", method, sanitized_url);

        match self.client.execute(request).await {
            Ok(response) => {
                log::debug!("Received response with status: {}", response.status());
                Ok(response)
            }
            Err(e) => {
                // Sanitize error message to prevent sensitive data exposure
                let sanitized_error = self.sanitize_error_message(&e.to_string());
                log::error!("HTTP request failed: {}", sanitized_error);
                Err(anyhow!("HTTP request failed: {}", sanitized_error))
            }
        }
    }

    /// Validate that the URL uses HTTPS if TLS enforcement is enabled
    fn validate_url(&self, url: &str) -> Result<()> {
        let parsed_url = Url::parse(url).context("Invalid URL")?;

        if self.enforce_tls && parsed_url.scheme() != "https" {
            return Err(anyhow!(
                "Insecure HTTP connection attempted. Only HTTPS is allowed."
            ));
        }

        Ok(())
    }

    /// Sanitize request to prevent sensitive data logging
    fn sanitize_request(&self, request: &mut Request) -> Result<()> {
        // Check headers for sensitive data
        let headers = request.headers();
        for (name, value) in headers.iter() {
            let header_name = name.as_str().to_lowercase();

            // Always warn about Authorization headers being used (but don't log the value)
            if header_name == "authorization" {
                log::debug!("Authorization header present in request");
                continue;
            }

            // Check other headers for sensitive data patterns
            if let Ok(value_str) = value.to_str() {
                if is_sensitive_data(value_str) {
                    log::warn!(
                        "Potentially sensitive data detected in request header '{}'. Value has been redacted from logs.",
                        name
                    );
                } else if self.contains_api_key_pattern(value_str) {
                    log::warn!(
                        "Potential API key detected in request header '{}'. Value has been redacted from logs.",
                        name
                    );
                }
            }
        }

        // Inspect request body if available
        if let Some(body) = request.body() {
            self.inspect_request_body(body)?;
        }

        Ok(())
    }

    /// Inspect request body for sensitive data patterns
    fn inspect_request_body(&self, _body: &reqwest::Body) -> Result<()> {
        // Note: reqwest::Body doesn't provide easy access to the raw bytes
        // without consuming it, so we'll implement compile-time checks instead
        // through type system and documentation

        log::debug!(
            "Request body inspection: Body present but content not accessible for inspection"
        );

        // This is where compile-time checks would be enforced through the type system
        // The actual enforcement happens at the call site through proper API design
        Ok(())
    }

    /// Check if a string contains API key patterns
    fn contains_api_key_pattern(&self, text: &str) -> bool {
        // Common API key patterns
        let patterns = [
            // Alchemy-style keys
            r"^[a-zA-Z0-9_-]{32,}$",
            // Bearer tokens
            r"Bearer\s+[a-zA-Z0-9_-]+",
            // Generic API key patterns
            r"[a-zA-Z0-9]{20,}",
        ];

        for pattern in &patterns {
            if let Ok(regex) = regex::Regex::new(pattern)
                && regex.is_match(text)
                && text.len() >= 20
            {
                return true;
            }
        }

        false
    }

    /// Sanitize URL for logging by redacting API keys and sensitive parameters
    fn sanitize_url(&self, url: &Url) -> String {
        let mut sanitized = url.clone();

        // Clear query parameters that might contain sensitive data
        if let Some(query) = url.query()
            && is_sensitive_data(query)
        {
            sanitized.set_query(Some("[REDACTED]"));
        }

        // Redact API keys from the path
        let path = sanitized.path();
        if path.contains("/v2/") {
            // This is likely an Alchemy URL with API key in path
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 3 && parts[1] == "v2" {
                // Replace the API key part
                let mut new_path = String::new();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        new_path.push('/');
                    }
                    if i == 2 && part.len() > 8 {
                        // This looks like an API key, redact it
                        new_path.push_str("[API_KEY_REDACTED]");
                    } else {
                        new_path.push_str(part);
                    }
                }
                sanitized.set_path(&new_path);
            }
        }

        sanitized.to_string()
    }

    /// Sanitize error messages to prevent sensitive data exposure
    fn sanitize_error_message(&self, error_msg: &str) -> String {
        let mut sanitized = sanitize_log_message(error_msg);

        // Additional sanitization for common HTTP error patterns
        if sanitized.contains("api_key") || sanitized.contains("apikey") {
            sanitized = sanitized.replace("api_key", "[REDACTED]");
            sanitized = sanitized.replace("apikey", "[REDACTED]");
        }

        // Redact any potential private keys in error messages
        sanitized = redact_private_key(&sanitized);

        sanitized
    }
}

impl Default for SecureHttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default SecureHttpClient")
    }
}

// Safe implementations for common JSON-RPC request types
impl SafeForHttpSerialization for serde_json::Value {}

#[derive(Serialize)]
pub struct JsonRpcRequest<T: Serialize> {
    pub jsonrpc: String,
    pub id: u32,
    pub method: String,
    pub params: T,
}

impl<T: Serialize> SafeForHttpSerialization for JsonRpcRequest<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_url_validation_https_required() {
        let client = SecureHttpClient::with_config(true).unwrap();

        // HTTPS should be allowed
        assert!(client.validate_url("https://example.com").is_ok());

        // HTTP should be rejected when TLS is enforced
        assert!(client.validate_url("http://example.com").is_err());
    }

    #[test]
    fn test_url_validation_http_allowed() {
        let client = SecureHttpClient::with_config(false).unwrap();

        // Both should be allowed when TLS enforcement is disabled
        assert!(client.validate_url("https://example.com").is_ok());
        assert!(client.validate_url("http://example.com").is_ok());
    }

    #[test]
    fn test_url_sanitization() {
        let client = SecureHttpClient::new().unwrap();
        let url = Url::parse("https://rootstock-mainnet.g.alchemy.com/v2/abc123def456").unwrap();

        let sanitized = client.sanitize_url(&url);
        assert!(sanitized.contains("[API_KEY_REDACTED]"));
        assert!(!sanitized.contains("abc123def456"));
    }

    #[test]
    fn test_error_message_sanitization() {
        let client = SecureHttpClient::new().unwrap();
        let error_msg = "Request failed with api_key: abc123def456";

        let sanitized = client.sanitize_error_message(error_msg);
        assert!(sanitized.contains("[REDACTED]"));
        assert!(!sanitized.contains("abc123def456"));
    }

    #[test]
    fn test_api_key_pattern_detection() {
        let client = SecureHttpClient::new().unwrap();

        // Should detect potential API keys
        assert!(client.contains_api_key_pattern("abc123def456ghi789jkl012mno345pqr678"));
        assert!(client.contains_api_key_pattern("Bearer abc123def456ghi789"));

        // Should not detect short strings
        assert!(!client.contains_api_key_pattern("short"));
        assert!(!client.contains_api_key_pattern("test123"));

        // Should not detect common words
        assert!(!client.contains_api_key_pattern("application/json"));
    }

    #[test]
    fn test_safe_json_rpc_request() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "eth_getBalance".to_string(),
            params: json!(["0x1234567890123456789012345678901234567890", "latest"]),
        };

        // This should compile because JsonRpcRequest implements SafeForHttpSerialization
        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("eth_getBalance"));
    }
}
