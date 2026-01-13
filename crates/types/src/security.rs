//! Security utilities for handling sensitive data.
//!
//! This module provides types and utilities for secure handling of sensitive data:
//! - `SecretString` for API keys and tokens (prevents accidental logging)
//! - Constant-time comparison for sensitive data
//! - Input validation utilities

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use subtle::ConstantTimeEq;

/// A wrapper around `SecretString` that implements Serialize/Deserialize.
///
/// This type is used for sensitive data like API keys and tokens.
/// It prevents accidental logging and serialization of secrets.
///
/// # Example
///
/// ```rust
/// use claude_agent_types::security::ApiKey;
///
/// let key = ApiKey::new("sk-secret-key-12345");
/// println!("Key: {}", key); // Prints: Key: [REDACTED]
/// ```
#[derive(Clone)]
pub struct ApiKey(SecretString);

impl ApiKey {
    /// Create a new API key from a string.
    pub fn new(key: impl Into<String>) -> Self {
        Self(SecretString::from(key.into()))
    }

    /// Expose the secret value.
    ///
    /// # Security
    ///
    /// Use this method carefully. The exposed value should not be logged.
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }

    /// Check if the key is empty.
    pub fn is_empty(&self) -> bool {
        self.0.expose_secret().is_empty()
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiKey([REDACTED])")
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl Serialize for ApiKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Never serialize the actual secret
        serializer.serialize_str("[REDACTED]")
    }
}

impl<'de> Deserialize<'de> for ApiKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(ApiKey::new(s))
    }
}

impl Default for ApiKey {
    fn default() -> Self {
        Self::new("")
    }
}

/// Constant-time comparison for sensitive data.
///
/// This function performs a constant-time comparison to prevent timing attacks.
///
/// # Example
///
/// ```rust
/// use claude_agent_types::security::constant_time_eq;
///
/// let token1 = "secret-token-123";
/// let token2 = "secret-token-123";
/// assert!(constant_time_eq(token1.as_bytes(), token2.as_bytes()));
/// ```
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Constant-time string comparison.
pub fn constant_time_str_eq(a: &str, b: &str) -> bool {
    constant_time_eq(a.as_bytes(), b.as_bytes())
}

/// Input validation result.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validate that a string is not empty.
pub fn validate_not_empty(field: &str, value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError {
            field: field.to_string(),
            message: "must not be empty".to_string(),
        })
    } else {
        Ok(())
    }
}

/// Validate that a string does not exceed a maximum length.
pub fn validate_max_length(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    if value.len() > max {
        Err(ValidationError {
            field: field.to_string(),
            message: format!("must not exceed {} characters", max),
        })
    } else {
        Ok(())
    }
}

/// Validate JSON payload size.
pub fn validate_json_size(
    field: &str,
    value: &serde_json::Value,
    max_bytes: usize,
) -> Result<(), ValidationError> {
    let size = value.to_string().len();
    if size > max_bytes {
        Err(ValidationError {
            field: field.to_string(),
            message: format!(
                "JSON payload exceeds {} bytes (got {} bytes)",
                max_bytes, size
            ),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_redaction() {
        let key = ApiKey::new("sk-secret-key-12345");
        assert_eq!(format!("{}", key), "[REDACTED]");
        assert_eq!(format!("{:?}", key), "ApiKey([REDACTED])");
    }

    #[test]
    fn test_api_key_expose() {
        let key = ApiKey::new("sk-secret-key-12345");
        assert_eq!(key.expose(), "sk-secret-key-12345");
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hello!")); // Different lengths
    }

    #[test]
    fn test_constant_time_str_eq() {
        assert!(constant_time_str_eq("password123", "password123"));
        assert!(!constant_time_str_eq("password123", "password124"));
    }

    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("name", "John").is_ok());
        assert!(validate_not_empty("name", "").is_err());
        assert!(validate_not_empty("name", "   ").is_err());
    }

    #[test]
    fn test_validate_max_length() {
        assert!(validate_max_length("name", "John", 10).is_ok());
        assert!(validate_max_length("name", "John Doe Smith", 10).is_err());
    }

    #[test]
    fn test_validate_json_size() {
        let small = serde_json::json!({"key": "value"});
        let large = serde_json::json!({"key": "x".repeat(1000)});

        assert!(validate_json_size("payload", &small, 100).is_ok());
        assert!(validate_json_size("payload", &large, 100).is_err());
    }
}
