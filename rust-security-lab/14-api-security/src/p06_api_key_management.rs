//! # Lesson 06: API Key Management
//!
//! ## Why API Keys?
//!
//! API keys identify and authenticate applications (not users). They're simpler
//! than OAuth tokens but come with their own security challenges.
//!
//! ## How NOT to Store API Keys
//!
//! ```rust
//! // DANGEROUS — storing plaintext keys in a database
//! db.insert("user123", "sk_live_abc123def456");
//!
//! // If the database leaks, ALL keys are compromised instantly.
//! ```
//!
//! ## How TO Store API Keys
//!
//! Like passwords, API keys should be hashed before storage:
//!
//! ```text
//! Generation:
//!   key = random_bytes(32) → "sk_live_a1b2c3d4..."
//!   key_prefix = key[0..8] → "sk_live_a1"  (for lookup)
//!   key_hash = SHA256(key)  → stored in database
//!
//! Verification:
//!   User provides key → hash it → look up by prefix + hash
//! ```
//!
//! The prefix allows fast lookup; the hash prevents key recovery from database leaks.
//!
//! ## API Key Formats
//!
//! Common formats include:
//! - Stripe-style: `sk_live_` prefix + base62 random
//! - GitHub-style: `ghp_` prefix + base64 random
//! - Generic: base64-encoded random bytes
//!
//! A good API key should:
//! - Be long enough (at least 32 bytes of randomness)
//! - Have a recognizable prefix (for scanning/revocation)
//! - Include a checksum or be identifiable by format
//! - Be generated using cryptographic randomness
//!
//! ## Attack: Key Enumeration
//!
//! If keys are short or predictable, an attacker can brute-force them.
//! With 32 bytes of randomness (256 bits), there are 2^256 possible keys —
//! infeasible to brute-force.
//!
//! ## Defense
//!
//! 1. Generate keys with cryptographic randomness (OsRng)
//! 2. Hash keys before storage (SHA-256 minimum)
//! 3. Use key prefixes for fast lookup and identification
//! 4. Support key rotation — allow multiple active keys per user
//! 5. Log key usage for audit trails
//! 6. Rate limit key validation to prevent brute force

use sha2::{Sha256, Digest};
use rand::Rng;
use rand::rngs::OsRng;

/// Exercise 1: Generate a random API key.
///
/// Generate an API key with the given prefix and the specified number of
/// random bytes. Return it as a hex-encoded string.
///
/// Format: "{prefix}{hex(random_bytes)}"
///
/// Example: "sk_live_" + 32 random bytes → "sk_live_a1b2c3d4..."
///
/// Hints:
/// - Use `OsRng` for cryptographic randomness
/// - Generate `num_random_bytes` random bytes
/// - Encode with `hex::encode`
/// - Prepend the prefix
pub fn generate_api_key(prefix: &str, num_random_bytes: usize) -> String {
    todo!("Generate a random API key with a prefix")
}

/// Exercise 2: Hash an API key for storage.
///
/// Compute the SHA-256 hash of the API key and return it as a hex string.
/// This is what you store in the database — never the raw key.
///
/// Hints:
/// - Use `sha2::Sha256`
/// - `Sha256::digest(key.as_bytes())` → hash bytes
/// - `hex::encode(hash)` → hex string
pub fn hash_api_key(key: &str) -> String {
    todo!("Hash an API key using SHA-256")
}

/// Exercise 3: Extract a key prefix for lookup.
///
/// Given an API key, extract the first `prefix_len` characters.
/// This prefix is used for fast database lookup (indexed).
///
/// Returns the prefix string, or `None` if the key is shorter than `prefix_len`.
///
/// Hints:
/// - Check `key.len() >= prefix_len`
/// - Use `&key[..prefix_len]` for byte slicing (ASCII-safe)
pub fn extract_key_prefix(key: &str, prefix_len: usize) -> Option<String> {
    todo!("Extract a fixed-length prefix from an API key")
}

/// Exercise 4: Validate an API key against stored data.
///
/// Given a provided key, the stored hash, and the expected prefix, validate:
/// 1. The key starts with the expected prefix
/// 2. The SHA-256 hash of the key matches the stored hash
///
/// Returns:
/// - `Ok(())` if valid
/// - `Err("invalid_prefix")` if the key doesn't have the expected prefix
/// - `Err("invalid_key")` if the hash doesn't match
///
/// Hints:
/// - Check `key.starts_with(expected_prefix)`
/// - Hash the provided key and compare with stored hash
/// - Use constant-time comparison for the hash comparison
pub fn validate_api_key(
    provided_key: &str,
    stored_hash: &str,
    expected_prefix: &str,
) -> Result<(), &'static str> {
    todo!("Validate an API key against stored prefix and hash")
}

/// Exercise 5: Generate a key with a checksum.
///
/// Some API key formats include a checksum to detect typos. Generate a key
/// where the last 4 hex characters are a checksum of the key body.
///
/// Format: "{prefix}{random_hex}{checksum}"
///
/// The checksum is the first 2 bytes (4 hex chars) of SHA-256(random_hex).
///
/// Hints:
/// - Generate random bytes and encode as hex
/// - Compute SHA-256 of the random hex string
/// - Take the first 4 hex characters as the checksum
/// - Concatenate: prefix + random_hex + checksum
pub fn generate_key_with_checksum(prefix: &str, num_random_bytes: usize) -> String {
    todo!("Generate an API key with a checksum suffix")
}

/// Exercise 6: Verify a key with checksum validation.
///
/// Verify that a key's checksum is correct before doing the expensive
/// hash lookup. This is a fast pre-check.
///
/// Rules:
/// 1. Key must be at least 4 characters longer than the prefix
/// 2. Split key into body and last 4 chars (checksum)
/// 3. Compute SHA-256 of body, take first 4 hex chars
/// 4. Compare with the provided checksum
///
/// Returns `true` if the checksum is valid.
///
/// Hints:
/// - Remove the prefix: `&key[prefix.len()..]`
/// - Split into body and checksum: `body = &no_prefix[..len-4]`, `checksum = &no_prefix[len-4..]`
/// - Hash the body, compare first 4 hex chars of hash with checksum
pub fn verify_key_checksum(key: &str, prefix: &str) -> bool {
    todo!("Verify the checksum of an API key")
}

/// Exercise 7: Create an API key store entry.
///
/// Given a newly generated API key and a user ID, create a `KeyEntry` with:
/// - The key prefix (first 12 characters)
/// - The key hash (SHA-256 of the full key)
/// - The user ID
/// - A creation timestamp
///
/// This simulates what you'd store in a database.
pub struct KeyEntry {
    pub key_prefix: String,
    pub key_hash: String,
    pub user_id: String,
    pub created_at: u64,
}

pub fn create_key_entry(key: &str, user_id: &str, created_at: u64) -> KeyEntry {
    todo!("Create a KeyEntry from a generated API key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_has_prefix() {
        let key = generate_api_key("sk_live_", 32);
        assert!(key.starts_with("sk_live_"));
    }

    #[test]
    fn test_generate_key_length() {
        let key = generate_api_key("sk_", 32);
        // prefix (3) + 32 bytes * 2 hex chars = 67
        assert_eq!(key.len(), 3 + 64);
    }

    #[test]
    fn test_generate_keys_unique() {
        let k1 = generate_api_key("sk_", 32);
        let k2 = generate_api_key("sk_", 32);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_hash_deterministic() {
        let h1 = hash_api_key("sk_live_abc123");
        let h2 = hash_api_key("sk_live_abc123");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_is_hex() {
        let h = hash_api_key("test_key");
        assert!(hex::decode(&h).is_ok());
        assert_eq!(h.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }

    #[test]
    fn test_extract_prefix() {
        assert_eq!(extract_key_prefix("sk_live_abc123def", 8).unwrap(), "sk_live_");
    }

    #[test]
    fn test_extract_prefix_too_short() {
        assert!(extract_key_prefix("short", 8).is_none());
    }

    #[test]
    fn test_validate_api_key_valid() {
        let key = generate_api_key("sk_live_", 32);
        let hash = hash_api_key(&key);
        assert!(validate_api_key(&key, &hash, "sk_live_").is_ok());
    }

    #[test]
    fn test_validate_api_key_wrong_prefix() {
        let key = generate_api_key("sk_live_", 32);
        let hash = hash_api_key(&key);
        assert_eq!(validate_api_key(&key, &hash, "pk_").unwrap_err(), "invalid_prefix");
    }

    #[test]
    fn test_validate_api_key_wrong_hash() {
        let key = generate_api_key("sk_live_", 32);
        assert_eq!(validate_api_key(&key, "wrong_hash", "sk_live_").unwrap_err(), "invalid_key");
    }

    #[test]
    fn test_key_with_checksum_format() {
        let key = generate_key_with_checksum("sk_", 16);
        assert!(key.starts_with("sk_"));
        // prefix (3) + 16*2 hex (32) + 4 checksum = 39
        assert_eq!(key.len(), 3 + 32 + 4);
    }

    #[test]
    fn test_verify_checksum_valid() {
        let key = generate_key_with_checksum("sk_", 16);
        assert!(verify_key_checksum(&key, "sk_"));
    }

    #[test]
    fn test_verify_checksum_invalid() {
        let mut key = generate_key_with_checksum("sk_", 16);
        // Tamper with the last character
        let last = key.pop().unwrap();
        key.push(if last == 'a' { 'b' } else { 'a' });
        assert!(!verify_key_checksum(&key, "sk_"));
    }

    #[test]
    fn test_verify_checksum_too_short() {
        assert!(!verify_key_checksum("sk_", "sk_"));
    }

    #[test]
    fn test_create_key_entry() {
        let key = generate_api_key("sk_live_", 32);
        let entry = create_key_entry(&key, "user123", 1700000000);
        assert_eq!(entry.user_id, "user123");
        assert_eq!(entry.created_at, 1700000000);
        assert_eq!(entry.key_prefix.len(), 12);
        assert_eq!(entry.key_hash.len(), 64);
    }

    #[test]
    fn test_key_entry_hash_matches() {
        let key = generate_api_key("sk_live_", 32);
        let entry = create_key_entry(&key, "user123", 1700000000);
        assert_eq!(entry.key_hash, hash_api_key(&key));
    }
}
