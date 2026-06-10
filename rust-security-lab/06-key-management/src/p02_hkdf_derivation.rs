//! # Lesson 02: HKDF — HMAC-based Key Derivation Function
//!
//! ## What Is HKDF?
//!
//! HKDF (RFC 5869) derives one or more cryptographic keys from a single high-entropy
//! input (like a master key or shared secret from a key exchange). It works in two phases:
//!
//! 1. **Extract**: Condenses input entropy into a fixed-length pseudorandom key (PRK)
//!    - `PRK = HMAC-SHA256(salt, input_key_material)`
//!    - The salt is optional but recommended
//!
//! 2. **Expand**: Generates output key material of any length from the PRK
//!    - `OKM = HMAC-SHA256(PRK, info || counter)` (repeated if more bytes needed)
//!    - The `info` parameter lets you derive different keys for different purposes
//!
//! ## When to Use HKDF
//!
//! - After a Diffie-Hellman key exchange (raw shared secret is not uniform)
//! - Deriving multiple keys from one master key (e.g., encryption key + MAC key)
//! - Any time you have high-entropy input and need structured key material
//!
//! ## Attack Scenario: Key Reuse Without Context Separation
//!
//! If you derive both an encryption key and a MAC key from the same master key
//! without using different `info` strings, they might be related. An attacker who
//! compromises one could potentially derive the other.
//!
//! HKDF with distinct `info` strings ensures cryptographically independent keys:
//! ```text
//! enc_key = HKDF(master, info="encryption")
//! mac_key = HKDF(master, info="authentication")
//! // enc_key and mac_key are independent — knowing one reveals nothing about the other
//! ```

use hkdf::Hkdf;
use sha2::Sha256;

/// Exercise 1: Extract a pseudorandom key (PRK) from input key material.
///
/// HKDF-Extract takes a salt and input key material, and produces a fixed-length PRK.
///
/// Hints:
/// - Create: `let hk = Hkdf::<Sha256>::new(Some(salt), ikm);`
/// - The PRK is 32 bytes (SHA-256 output)
/// - Use `hk.extract()` if you just need the PRK
pub fn extract(salt: &[u8], ikm: &[u8]) -> Vec<u8> {
    todo!("Implement HKDF-Extract using hkdf crate")
}

/// Exercise 2: Derive output key material from a PRK.
///
/// HKDF-Expand takes a PRK and an info string, and produces N bytes of output.
///
/// Hints:
/// - Create HKDF from PRK: `Hkdf::<Sha256>::from_prk(prk).unwrap()`
/// - Expand: `hk.expand(info, &mut okm).unwrap();`
/// - `okm` must be a `&mut [u8]` of the desired length
/// - Max output: 255 * 32 = 8160 bytes for SHA-256
pub fn expand(prk: &[u8], info: &[u8], length: usize) -> Vec<u8> {
    todo!("Implement HKDF-Expand using hkdf crate")
}

/// Exercise 3: One-shot HKDF (extract + expand combined).
///
/// Derive a key of the specified length from input key material, salt, and info.
///
/// Hints:
/// - Use `Hkdf::<Sha256>::new(Some(salt), ikm)` for extract
/// - Use `.expand(info, &mut okm)` for expand
pub fn derive_key(salt: &[u8], ikm: &[u8], info: &[u8], length: usize) -> Vec<u8> {
    todo!("Implement one-shot HKDF derivation")
}

/// Exercise 4: Derive multiple independent keys from one master key.
///
/// Each key is derived with a different `info` string to ensure independence.
///
/// Hints:
/// - Create HKDF once from the master key
/// - For each key, expand with a unique info string (e.g., "key-0", "key-1")
/// - Return a `Vec<Vec<u8>>`
pub fn derive_multiple_keys(master: &[u8], salt: &[u8], count: usize, key_length: usize) -> Vec<Vec<u8>> {
    todo!("Derive multiple independent keys from one master")
}

/// Exercise 5: Demonstrate why info strings matter.
///
/// Derive two keys from the same master with different info strings.
/// They must be different. Then derive two keys with the SAME info string.
/// They must be the same.
///
/// Return (key_with_info_a, key_with_info_b, key_same_info_1, key_same_info_2)
pub fn demonstrate_info_importance(master: &[u8], salt: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    todo!("Demonstrate HKDF info string semantics")
}

/// Exercise 6: Derive an encryption key and MAC key from a shared secret.
///
/// After a key exchange (e.g., ECDH), both parties share a raw secret.
/// Derive two purpose-specific keys using different info strings.
///
/// Returns (encryption_key, mac_key)
pub fn derive_enc_and_mac_keys(shared_secret: &[u8], salt: &[u8]) -> (Vec<u8>, Vec<u8>) {
    todo!("Derive encryption and MAC keys from shared secret")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_master() -> Vec<u8> {
        vec![0xABu8; 32]
    }

    fn test_salt() -> Vec<u8> {
        vec![0xCDu8; 16]
    }

    #[test]
    fn test_extract_length() {
        let prk = extract(&test_salt(), &test_master());
        assert_eq!(prk.len(), 32, "PRK from SHA-256 HKDF should be 32 bytes");
    }

    #[test]
    fn test_extract_deterministic() {
        let prk1 = extract(&test_salt(), &test_master());
        let prk2 = extract(&test_salt(), &test_master());
        assert_eq!(prk1, prk2);
    }

    #[test]
    fn test_expand_length() {
        let prk = extract(&test_salt(), &test_master());
        let okm = expand(&prk, b"test", 48);
        assert_eq!(okm.len(), 48);
    }

    #[test]
    fn test_expand_different_info() {
        let prk = extract(&test_salt(), &test_master());
        let okm1 = expand(&prk, b"encryption", 32);
        let okm2 = expand(&prk, b"authentication", 32);
        assert_ne!(okm1, okm2, "Different info must produce different keys");
    }

    #[test]
    fn test_expand_same_info_deterministic() {
        let prk = extract(&test_salt(), &test_master());
        let okm1 = expand(&prk, b"test", 32);
        let okm2 = expand(&prk, b"test", 32);
        assert_eq!(okm1, okm2, "Same info must produce same key");
    }

    #[test]
    fn test_derive_key_length() {
        let key = derive_key(&test_salt(), &test_master(), b"test", 64);
        assert_eq!(key.len(), 64);
    }

    #[test]
    fn test_derive_multiple_keys_count() {
        let keys = derive_multiple_keys(&test_master(), &test_salt(), 3, 32);
        assert_eq!(keys.len(), 3);
        for key in &keys {
            assert_eq!(key.len(), 32);
        }
    }

    #[test]
    fn test_derive_multiple_keys_independent() {
        let keys = derive_multiple_keys(&test_master(), &test_salt(), 3, 32);
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[1], keys[2]);
        assert_ne!(keys[0], keys[2]);
    }

    #[test]
    fn test_info_importance() {
        let master = test_master();
        let salt = test_salt();
        let (key_a, key_b, same1, same2) = demonstrate_info_importance(&master, &salt);
        // Different info -> different keys
        assert_ne!(key_a, key_b, "Keys with different info must differ");
        // Same info -> same keys
        assert_eq!(same1, same2, "Keys with same info must match");
    }

    #[test]
    fn test_enc_mac_keys_independent() {
        let (enc_key, mac_key) = derive_enc_and_mac_keys(&test_master(), &test_salt());
        assert_eq!(enc_key.len(), 32);
        assert_eq!(mac_key.len(), 32);
        assert_ne!(enc_key, mac_key, "Encryption and MAC keys must be independent");
    }

    #[test]
    fn test_derive_key_empty_salt() {
        // HKDF allows empty salt (uses a string of zeros as default)
        let key = derive_key(b"", &test_master(), b"test", 32);
        assert_eq!(key.len(), 32);
    }
}
