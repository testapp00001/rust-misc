//! # Lesson 03: HMAC — Hash-based Message Authentication Code
//!
//! ## What is HMAC?
//!
//! HMAC proves that a message came from someone who knows a secret key AND that
//! the message hasn't been tampered with. It combines a hash function with a secret key.
//!
//! ```text
//! HMAC(key, msg) = H((key ^ opad) || H((key ^ ipad) || msg))
//! ```
//!
//! Where:
//! - `ipad` = 0x36 repeated
//! - `opad` = 0x5C repeated
//! - `H` = hash function (SHA-256, SHA-512, etc.)
//!
//! ## Why Not Just Hash(key || msg)?
//!
//! Because of **length extension attacks** (see Lesson 04). If you use
//! `H(key || msg)`, an attacker who knows `H(key || msg)` can compute
//! `H(key || msg || padding || evil_data)` without knowing the key.
//!
//! HMAC's double-hashing structure prevents this because the attacker would
//! need to compute the outer hash, which requires the key.
//!
//! ## Security Properties
//!
//! 1. **Authenticity**: Only someone with the key can produce a valid MAC
//! 2. **Integrity**: Any change to the message invalidates the MAC
//! 3. **No repudiation**: The key holder cannot deny sending the message
//!
//! ## Common Uses
//!
//! - API authentication (HMAC-SHA256 signatures on requests)
//! - JWT tokens (HS256 = HMAC-SHA256)
//! - TLS record authentication
//! - Cookie signing
//!
//! ## Attack Scenario
//!
//! An attacker intercepts a message with its HMAC. They want to modify the message.
//! Without the key, they cannot compute a valid HMAC for the modified message.
//! Even a 1-bit change produces a completely different HMAC.
//!
//! ## Key Management
//!
//! - Generate keys with `ring::rand::SystemRandom`
//! - Key should be at least as long as the hash output (32 bytes for SHA-256)
//! - Never reuse HMAC keys for encryption or vice versa

use ring::hmac;

/// Exercise 1: Generate a random HMAC key.
///
/// The key should be suitable for use with HMAC-SHA256.
///
/// Hints:
/// - Use `ring::rand::SystemRandom::new()` to get a CSPRNG
/// - Use `hmac::Key::generate(hmac::HMAC_SHA256, &rng)` to generate the key
/// - This produces a cryptographically random 32-byte key
pub fn generate_hmac_key() -> hmac::Key {
    todo!("Generate a random HMAC-SHA256 key")
}

/// Exercise 2: Compute HMAC-SHA256 for a message.
///
/// Returns the HMAC tag as bytes.
///
/// Hints:
/// - Use `hmac::sign(&key, message)` to compute the MAC
/// - The result is `hmac::Tag` — call `.as_ref()` to get `&[u8]`
pub fn compute_hmac(key: &hmac::Key, message: &[u8]) -> Vec<u8> {
    todo!("Compute HMAC-SHA256 tag")
}

/// Exercise 3: Verify an HMAC tag against a message.
///
/// Returns true if the tag is valid for the given key and message.
/// This MUST use constant-time comparison internally.
///
/// Hints:
/// - Use `hmac::verify(&key, message, tag_bytes)` to verify
/// - This returns `Result<(), Unspecified>` — convert to bool
pub fn verify_hmac(key: &hmac::Key, message: &[u8], tag: &[u8]) -> bool {
    todo!("Verify HMAC-SHA256 tag (must be constant-time)")
}

/// Exercise 4: Compute HMAC and return as hex string.
///
/// Hints:
/// - Compute the HMAC with `compute_hmac()`
/// - Convert to hex with `hex::encode()`
pub fn hmac_hex(key: &hmac::Key, message: &[u8]) -> String {
    todo!("Compute HMAC and return as hex string")
}

/// Exercise 5: Create a signed message (message + HMAC).
///
/// Returns (message, hmac_tag) as a tuple.
///
/// Hints:
/// - Compute HMAC of the message
/// - Return both the message bytes and the tag
pub fn sign_message(key: &hmac::Key, message: &[u8]) -> (Vec<u8>, Vec<u8>) {
    todo!("Create a signed message")
}

/// Exercise 6: Verify a signed message.
///
/// Returns the message if verification succeeds, None otherwise.
///
/// Hints:
/// - Verify the HMAC tag against the message
/// - If valid, return Some(message), else None
pub fn verify_signed_message(key: &hmac::Key, message: &[u8], tag: &[u8]) -> Option<Vec<u8>> {
    todo!("Verify a signed message and return it if valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = generate_hmac_key();
        // Key was successfully created — verify it works by signing something
        let tag = compute_hmac(&key, b"test");
        assert_eq!(tag.len(), 32, "HMAC-SHA256 tag should be 32 bytes");
    }

    #[test]
    fn test_compute_and_verify() {
        let key = generate_hmac_key();
        let message = b"Hello, HMAC!";
        let tag = compute_hmac(&key, message);
        assert!(verify_hmac(&key, message, &tag), "Valid HMAC should verify");
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_hmac_key();
        let key2 = generate_hmac_key();
        let message = b"authenticated message";
        let tag = compute_hmac(&key1, message);
        assert!(!verify_hmac(&key2, message, &tag), "Wrong key should fail verification");
    }

    #[test]
    fn test_tampered_message_fails() {
        let key = generate_hmac_key();
        let message = b"original message";
        let tag = compute_hmac(&key, message);
        let tampered = b"tampered message";
        assert!(!verify_hmac(&key, tampered, &tag), "Tampered message should fail");
    }

    #[test]
    fn test_tampered_tag_fails() {
        let key = generate_hmac_key();
        let message = b"authentic message";
        let mut tag = compute_hmac(&key, message);
        tag[0] ^= 0xFF; // Flip bits in the tag
        assert!(!verify_hmac(&key, message, &tag), "Tampered tag should fail");
    }

    #[test]
    fn test_empty_message() {
        let key = generate_hmac_key();
        let tag = compute_hmac(&key, b"");
        assert_eq!(tag.len(), 32, "HMAC of empty message should still be 32 bytes");
        assert!(verify_hmac(&key, b"", &tag), "Empty message HMAC should verify");
    }

    #[test]
    fn test_long_message() {
        let key = generate_hmac_key();
        let message = vec![0xABu8; 1_000_000]; // 1MB
        let tag = compute_hmac(&key, &message);
        assert!(verify_hmac(&key, &message, &tag), "Long message HMAC should verify");
    }

    #[test]
    fn test_hmac_hex() {
        let key = generate_hmac_key();
        let hex_tag = hmac_hex(&key, b"test");
        assert_eq!(hex_tag.len(), 64, "Hex HMAC should be 64 characters (32 bytes)");
        // Verify it's valid hex
        assert!(hex_tag.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let key = generate_hmac_key();
        let message = b"roundtrip test";
        let (msg, tag) = sign_message(&key, message);
        let result = verify_signed_message(&key, &msg, &tag);
        assert_eq!(result, Some(message.to_vec()));
    }

    #[test]
    fn test_verify_signed_message_tampered() {
        let key = generate_hmac_key();
        let message = b"original";
        let (msg, tag) = sign_message(&key, message);
        let mut tampered_msg = msg.clone();
        tampered_msg[0] = b'X';
        let result = verify_signed_message(&key, &tampered_msg, &tag);
        assert_eq!(result, None, "Tampered message should return None");
    }

    #[test]
    fn test_deterministic_with_same_key() {
        let key = generate_hmac_key();
        let message = b"deterministic test";
        let tag1 = compute_hmac(&key, message);
        let tag2 = compute_hmac(&key, message);
        assert_eq!(tag1, tag2, "HMAC should be deterministic for same key+message");
    }
}
