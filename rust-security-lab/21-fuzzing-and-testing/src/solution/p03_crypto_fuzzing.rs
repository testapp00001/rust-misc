//! # Lesson 03: Fuzzing Cryptographic Functions (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Sha256, Digest};
use proptest::prelude::*;

/// Compute SHA-256 using the sha2 crate.
pub fn sha256_sha2(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute SHA-256 using ring.
pub fn sha256_ring(data: &[u8]) -> Vec<u8> {
    ring::digest::digest(&ring::digest::SHA256, data).as_ref().to_vec()
}

/// Base64 encode bytes.
pub fn base64_encode(data: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
}

/// Base64 decode a string.
pub fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)
        .map_err(|e| e.to_string())
}

/// Hex encode bytes to lowercase string.
pub fn hex_encode(data: &[u8]) -> String {
    hex::encode(data)
}

/// Hex decode a string to bytes.
pub fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    hex::decode(s).map_err(|e| e.to_string())
}

/// XOR two byte slices together up to the shorter length.
pub fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

/// HMAC-SHA256: compute a keyed hash.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    ring::hmac::sign(&key, data).as_ref().to_vec()
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

    proptest::proptest! {
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

    proptest::proptest! {
        #[test]
        fn test_base64_roundtrip_prop(data in prop::collection::vec(any::<u8>(), 0..1000)) {
            let encoded = base64_encode(&data);
            let decoded = base64_decode(&encoded).unwrap();
            prop_assert_eq!(data, decoded, "Base64 round-trip must preserve data");
        }

        #[test]
        fn test_base64_decode_invalid_never_panics(s in ".*") {
            let _ = base64_decode(&s);
        }
    }

    // --- Hex round-trip ---

    proptest::proptest! {
        #[test]
        fn test_hex_roundtrip(data in prop::collection::vec(any::<u8>(), 0..1000)) {
            let encoded = hex_encode(&data);
            let decoded = hex_decode(&encoded).unwrap();
            prop_assert_eq!(data, decoded, "Hex round-trip must preserve data");
        }
    }

    // --- XOR properties ---

    proptest::proptest! {
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

    proptest::proptest! {
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
