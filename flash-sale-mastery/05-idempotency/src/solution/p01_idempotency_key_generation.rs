//! # Solution 01: Idempotency Key Generation
//!
//! Complete implementation of idempotency key generation strategies.

use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Generate a random idempotency key (UUID v4).
///
/// Each call produces a unique key. Use this when the frontend manages
/// deduplication (e.g., disabling the Buy button after first click).
pub fn generate_key() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a deterministic idempotency key from purchase context.
///
/// Given the same (account_id, product_id, sale_id), this always returns
/// the same key. This means a user clicking "Buy" multiple times on the
/// same flash sale item will always produce the same idempotency key.
pub fn generate_deterministic_key(account_id: &str, product_id: &str, sale_id: &str) -> String {
    let input = format!("{account_id}:{product_id}:{sale_id}");
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(hash)
}

/// A validated idempotency key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Create a new validated idempotency key.
    pub fn new(key: String) -> Result<Self, IdempotencyKeyError> {
        if key.is_empty() {
            return Err(IdempotencyKeyError::Empty);
        }
        if key.len() > 256 {
            return Err(IdempotencyKeyError::TooLong(key.len()));
        }
        Ok(IdempotencyKey(key))
    }

    /// Get the key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Errors that can occur when creating an `IdempotencyKey`.
#[derive(Debug, thiserror::Error)]
pub enum IdempotencyKeyError {
    #[error("Idempotency key must not be empty")]
    Empty,

    #[error("Idempotency key exceeds maximum length of 256 characters (got {0})")]
    TooLong(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_uniqueness() {
        let key1 = generate_key();
        let key2 = generate_key();
        assert_ne!(key1, key2, "Two random keys must be different");
        assert!(!key1.is_empty(), "Key must not be empty");
    }

    #[test]
    fn test_generate_key_format() {
        let key = generate_key();
        assert_eq!(key.len(), 36, "UUID v4 string should be 36 characters");
        assert_eq!(key.chars().nth(8).unwrap(), '-');
        assert_eq!(key.chars().nth(13).unwrap(), '-');
        assert_eq!(key.chars().nth(18).unwrap(), '-');
        assert_eq!(key.chars().nth(23).unwrap(), '-');
    }

    #[test]
    fn test_deterministic_key_same_inputs() {
        let key1 = generate_deterministic_key("user1", "prod1", "sale1");
        let key2 = generate_deterministic_key("user1", "prod1", "sale1");
        assert_eq!(
            key1, key2,
            "Same inputs must always produce the same key"
        );
    }

    #[test]
    fn test_deterministic_key_different_inputs() {
        let key1 = generate_deterministic_key("user1", "prod1", "sale1");
        let key2 = generate_deterministic_key("user2", "prod1", "sale1");
        let key3 = generate_deterministic_key("user1", "prod2", "sale1");
        let key4 = generate_deterministic_key("user1", "prod1", "sale2");

        assert_ne!(key1, key2, "Different user should produce different key");
        assert_ne!(key1, key3, "Different product should produce different key");
        assert_ne!(key1, key4, "Different sale should produce different key");
    }

    #[test]
    fn test_deterministic_key_format() {
        let key = generate_deterministic_key("user1", "prod1", "sale1");
        assert_eq!(key.len(), 64, "SHA-256 hex should be 64 characters");
        assert!(
            key.chars().all(|c| c.is_ascii_hexdigit()),
            "Key should only contain hex digits"
        );
    }

    #[test]
    fn test_idempotency_key_valid() {
        let key = IdempotencyKey::new("abc-123-def".to_string()).unwrap();
        assert_eq!(key.as_str(), "abc-123-def");
    }

    #[test]
    fn test_idempotency_key_empty() {
        let result = IdempotencyKey::new("".to_string());
        assert!(result.is_err(), "Empty key should be rejected");
        match result.unwrap_err() {
            IdempotencyKeyError::Empty => {} // expected
            other => panic!("Expected Empty error, got: {other}"),
        }
    }

    #[test]
    fn test_idempotency_key_too_long() {
        let long_key = "x".repeat(257);
        let result = IdempotencyKey::new(long_key);
        assert!(result.is_err(), "Key over 256 chars should be rejected");
        match result.unwrap_err() {
            IdempotencyKeyError::TooLong(len) => assert_eq!(len, 257),
            other => panic!("Expected TooLong error, got: {other}"),
        }
    }

    #[test]
    fn test_idempotency_key_max_length_accepted() {
        let max_key = "x".repeat(256);
        let result = IdempotencyKey::new(max_key.clone());
        assert!(result.is_ok(), "256-char key should be accepted");
        assert_eq!(result.unwrap().as_str(), &max_key);
    }
}
