//! # Lesson 03: Fuzzing Cryptographic Functions
//!
//! ## Why Fuzz Crypto Code?
//!
//! Cryptographic implementations must be robust against ALL inputs, not just valid ones.
//! An attacker might feed a cipher:
//! - Empty input
//! - Maximum-length input
//! - All-zeros or all-ones
//! - Input that causes integer overflow in length calculations
//!
//! ## Properties to Test
//!
//! 1. **No panics**: Hash/encrypt/decrypt must never panic
//! 2. **Fixed output size**: SHA-256 always produces 32 bytes
//! 3. **Determinism**: Same input always gives same output
//! 4. **Invertibility**: decrypt(encrypt(x, k), k) == x
//! 5. **Non-degeneracy**: Different inputs produce different outputs (with high probability)
//!
//! ## Security Perspective
//!
//! ### Attack: Chosen-Input Attacks
//! An attacker crafts inputs to trigger implementation bugs:
//! - Integer overflows in buffer allocation
//! - Off-by-one errors in padding
//! - Undefined behavior on edge-case inputs
//!
//! ### Defense: Fuzz All Crypto Interfaces
//! Every public function that takes `&[u8]` should be fuzzed.

use ring::digest;
use sha2::{Sha256, Digest};

/// Compute SHA-256 using the `sha2` crate.
///
/// Must handle any input size (0 bytes to millions of bytes).
///
/// Hints:
/// - Create a `Sha256::new()` hasher
/// - Call `.update(data)`
/// - Call `.finalize()` to get the digest
/// - Convert to Vec<u8> with `.to_vec()`
pub fn sha256_sha2(data: &[u8]) -> Vec<u8> {
    todo!("Implement SHA-256 using sha2 crate")
}

/// Compute SHA-256 using `ring`.
///
/// This lets us cross-check two implementations.
///
/// Hints:
/// - Use `ring::digest::digest(&ring::digest::SHA256, data)`
/// - Convert to Vec<u8>
pub fn sha256_ring(data: &[u8]) -> Vec<u8> {
    todo!("Implement SHA-256 using ring crate")
}

/// Base64 encode bytes, returning a String.
///
/// Must handle empty input and arbitrary byte values.
///
/// Hints:
/// - Use `base64::Engine::encode` with `base64::engine::general_purpose::STANDARD`
/// - The `base64` crate v0.22+ uses the Engine API
pub fn base64_encode(data: &[u8]) -> String {
    todo!("Implement base64 encoding")
}

/// Base64 decode a string back to bytes.
///
/// Must handle invalid base64 gracefully (return Err, don't panic).
///
/// Hints:
/// - Use `base64::Engine::decode` with `base64::engine::general_purpose::STANDARD`
/// - Return Result<Vec<u8>, base64::DecodeError>
pub fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    todo!("Implement base64 decoding with error handling")
}

/// Hex encode bytes to a lowercase string.
///
/// Hints:
/// - Use the `hex::encode` function
pub fn hex_encode(data: &[u8]) -> String {
    todo!("Implement hex encoding")
}

/// Hex decode a string to bytes.
///
/// Must handle invalid hex gracefully.
///
/// Hints:
/// - Use `hex::decode` which returns Result
pub fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    todo!("Implement hex decoding with error handling")
}

/// XOR two byte slices together.
///
/// If lengths differ, XOR up to the shorter length and return the result.
///
/// Hints:
/// - Zip the two iterators and XOR each pair
/// - Collect into a Vec<u8>
pub fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    todo!("Implement byte XOR")
}

/// HMAC-SHA256: compute a keyed hash.
///
/// Hints:
/// - Use `ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key)`
/// - Call `.sign(data)` to get the Tag
/// - Convert Tag to Vec<u8>
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    todo!("Implement HMAC-SHA256")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SHA-256 consistency ---

    #[test]
    fn test_sha256_empty() {
        let hash = sha256_sha2(b"");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_sha256_ring_empty() {
        let hash = sha256_ring(b"");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_sha256_both_implementations_agree() {
        let test_cases: Vec<&[u8]> = vec![
            b"",
            b"hello",
            b"a",
            &[0u8; 1000],
            &[0xFFu8; 100],
        ];
        for input in test_cases {
            let a = sha256_sha2(input);
            let b = sha256_ring(input);
            assert_eq!(a, b, "SHA-256 implementations disagree on {:?}", input);
        }
    }

    proptest! {
        #[test]
        fn test_sha256_always_32_bytes(data in prop::collection::vec(any::<u8>(), 0..10000)) {
            let hash = sha256_sha2(&data);
            prop_assert_eq!(hash.len(), 32, "SHA-256 output must always be 32 bytes");
        }

        #[test]
        fn test_sha256_deterministic(data in prop::collection::vec(any::<u8>(), 0..1000)) {
            let h1 = sha256_sha2(&data);
            let h2 = sha256_sha2(&data);
            prop_assert_eq!(h1, h2, "SHA-256 must be deterministic");
        }

        #[test]
        fn test_sha256_implementations_match(data in prop::collection::vec(any::<u8>(), 0..1000)) {
            let a = sha256_sha2(&data);
            let b = sha256_ring(&data);
            prop_assert_eq!(a, b, "sha2 and ring SHA-256 must agree");
        }
    }

    // --- Base64 round-trip ---

    #[test]
    fn test_base64_roundtrip() {
        let data = b"hello world";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data.to_vec(), decoded);
    }

    proptest! {
        #[test]
        fn test_base64_roundtrip_prop(data in prop::collection::vec(any::<u8>(), 0..1000)) {
            let encoded = base64_encode(&data);
            let decoded = base64_decode(&encoded).unwrap();
            prop_assert_eq!(data, decoded, "Base64 round-trip must preserve data");
        }

        #[test]
        fn test_base64_decode_invalid_never_panics(s in ".*") {
            // Any string input to decode should return Result, never panic
            let _ = base64_decode(&s);
        }
    }

    // --- Hex round-trip ---

    proptest! {
        #[test]
        fn test_hex_roundtrip(data in prop::collection::vec(any::<u8>(), 0..1000)) {
            let encoded = hex_encode(&data);
            let decoded = hex_decode(&encoded).unwrap();
            prop_assert_eq!(data, decoded, "Hex round-trip must preserve data");
        }
    }

    // --- XOR properties ---

    proptest! {
        #[test]
        fn test_xor_self_is_zero(data in prop::collection::vec(any::<u8>(), 1..500)) {
            let result = xor_bytes(&data, &data);
            prop_assert!(result.iter().all(|&b| b == 0), "XOR of data with itself must be zeros");
        }

        #[test]
        fn test_xor_commutative(
            a in prop::collection::vec(any::<u8>(), 1..500),
            b in prop::collection::vec(any::<u8>(), 1..500)
        ) {
            let r1 = xor_bytes(&a, &b);
            let r2 = xor_bytes(&b, &a);
            prop_assert_eq!(r1, r2, "XOR must be commutative");
        }

        #[test]
        fn test_xor_involution(
            a in prop::collection::vec(any::<u8>(), 1..500),
            b in prop::collection::vec(any::<u8>(), 1..500)
        ) {
            // XOR is its own inverse: a ^ b ^ b == a
            let ab = xor_bytes(&a, &b);
            let result = xor_bytes(&ab, &b[..ab.len().min(b.len())]);
            prop_assert_eq!(&result, &a[..result.len()]);
        }
    }

    // --- HMAC properties ---

    #[test]
    fn test_hmac_deterministic() {
        let key = b"secret_key";
        let data = b"message";
        assert_eq!(hmac_sha256(key, data), hmac_sha256(key, data));
    }

    #[test]
    fn test_hmac_different_key_different_output() {
        let data = b"message";
        let h1 = hmac_sha256(b"key1", data);
        let h2 = hmac_sha256(b"key2", data);
        assert_ne!(h1, h2);
    }

    proptest! {
        #[test]
        fn test_hmac_always_32_bytes(
            key in prop::collection::vec(any::<u8>(), 1..100),
            data in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let mac = hmac_sha256(&key, &data);
            prop_assert_eq!(mac.len(), 32, "HMAC-SHA256 must always be 32 bytes");
        }
    }
}
