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
            if let Ok(value_str) = value.to_str()
                && is_sensitive_data(value_str)
            {
                log::warn!(
                    "Potentially sensitive data detected in request header: {}",
                    name
                );
            }
        }

        // Note: We can't easily modify the request body here without reconstructing it,
        // but we can log warnings if sensitive patterns are detected
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
