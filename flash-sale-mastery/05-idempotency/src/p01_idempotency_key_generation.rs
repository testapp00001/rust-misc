//! # Exercise 01: Idempotency Key Generation
//!
//! ## Learning Objective
//! Understand how to generate and validate idempotency keys that uniquely
//! identify a purchase intent. A good idempotency key must be unique enough
//! to never collide across different purchases, yet deterministic enough
//! that the same user clicking "Buy" twice on the same item produces the
//! same key.
//!
//! ## Flash Sale Context
//! When a user clicks "Buy" on a flash sale item, the frontend generates an
//! idempotency key and sends it with the request. If the user clicks again
//! (or the browser retries), the same key is sent, allowing the backend to
//! recognize it as a duplicate. Two strategies exist:
//!
//! 1. **Random keys** (UUID v4): The frontend generates a unique key per
//!    purchase *attempt*. Each click gets a new key. Works when combined with
//!    client-side deduplication (disable button after first click).
//!
//! 2. **Deterministic keys**: Generated from (account_id, product_id, sale_id).
//!    Same user + same product + same sale always produces the same key,
//!    regardless of how many times they click. No client-side logic needed.
//!
//! ## Instructions
//! 1. Implement `generate_key()` to return a UUID v4 string
//! 2. Implement `generate_deterministic_key()` to hash (account_id, product_id, sale_id)
//! 3. Implement `IdempotencyKey::new()` with validation (non-empty, reasonable length)
//! 4. Implement `IdempotencyKey::as_str()` to access the inner key
//!
//! ## Hints
//! - Use `uuid::Uuid::new_v4().to_string()` for random keys
//! - Use `sha2::Sha256` for deterministic hashing
//! - A key longer than 256 characters is suspicious -- validate it

use sha2::{Digest, Sha256};

/// Generate a random idempotency key (UUID v4).
///
/// Each call produces a unique key. Use this when the frontend manages
/// deduplication (e.g., disabling the Buy button after first click).
pub fn generate_key() -> String {
    // TODO: Generate a UUID v4 and return it as a string
    todo!("Implement random key generation")
}

/// Generate a deterministic idempotency key from purchase context.
///
/// Given the same (account_id, product_id, sale_id), this always returns
/// the same key. This means a user clicking "Buy" multiple times on the
/// same flash sale item will always produce the same idempotency key,
/// even without client-side deduplication.
///
/// # Arguments
/// * `account_id` - The buyer's account identifier
/// * `product_id` - The product being purchased
/// * `sale_id` - The flash sale event identifier
///
/// # Returns
/// A hex-encoded SHA-256 hash of the concatenated inputs.
pub fn generate_deterministic_key(account_id: &str, product_id: &str, sale_id: &str) -> String {
    // TODO: Concatenate the three inputs with a separator (e.g., ":")
    // TODO: Hash the result with SHA-256
    // TODO: Return the hex-encoded hash
    todo!("Implement deterministic key generation")
}

/// A validated idempotency key.
///
/// Wraps a key string with validation to prevent empty, excessively long,
/// or malformed keys from entering the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Create a new validated idempotency key.
    ///
    /// # Errors
    /// Returns `IdempotencyKeyError` if the key is empty or exceeds 256 characters.
    pub fn new(key: String) -> Result<Self, IdempotencyKeyError> {
        // TODO: Check that the key is not empty
        // TODO: Check that the key does not exceed 256 characters
        // TODO: Return Ok(IdempotencyKey(key)) or the appropriate error
        todo!("Implement key validation")
    }

    /// Get the key as a string slice.
    pub fn as_str(&self) -> &str {
        // TODO: Return a reference to the inner string
        todo!("Implement as_str")
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
        // UUID v4 format: 8-4-4-4-12 hex chars with dashes
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
        // SHA-256 hex is always 64 characters
        assert_eq!(key.len(), 64, "SHA-256 hex should be 64 characters");
        // All characters should be hex digits
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
