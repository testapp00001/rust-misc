//! # Lesson 07: Deterministic Signatures (RFC 6979)
//!
//! ## What is RFC 6979?
//!
//! RFC 6979 specifies a method for generating the nonce `k` in DSA/ECDSA
//! deterministically from the private key and message, using HMAC.
//!
//! Instead of: k = random()
//! We compute: k = HMAC-SHA256(private_key, message)
//!
//! This eliminates the need for a random number generator during signing,
//! while still producing a unique `k` for each (key, message) pair.
//!
//! ## Why Deterministic Nonces Matter
//!
//! The catastrophic attacks on ECDSA all involve bad nonces:
//!
//! 1. **Sony PS3 (2010)**: Used the same `k` for every signature. Private key
//!    recovered from two signatures. (k reused)
//!
//! 2. **Android Bitcoin wallet (2013)**: Java SecureRandom had a bug that
//!    sometimes produced the same `k`. Multiple wallets drained. (k repeated)
//!
//! 3. **ECDSA nonce bias**: If `k` has even a few bits of bias, the private key
//!    can be partially recovered. Many embedded RNGs have bias. (k biased)
//!
//! RFC 6979 eliminates ALL of these: `k` is deterministically derived, unique
//! per (key, message), and has no bias.
//!
//! ## Ed25519 Already Does This
//!
//! Ed25519 (RFC 8032) uses a similar deterministic nonce derivation built into
//! the algorithm. This is one reason Ed25519 is preferred over ECDSA.
//!
//! ## Attack Scenario: Nonce Bias
//!
//! If an ECDSA nonce `k` leaks even a few bits (e.g., top 4 bits are always 0),
//! the Hidden Number Problem (HNP) can be used to recover the private key
//! from enough signatures (~100-1000).

use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;

/// Exercise 1: Compute a deterministic nonce using HMAC.
///
/// k = HMAC-SHA256(key, message)
///
/// This is the core of RFC 6979: derive `k` deterministically.
///
/// Hints:
/// - Create HMAC: `HmacSha256::new_from_slice(key).unwrap()`
/// - Feed data: `mac.update(message)`
/// - Get result: `mac.finalize().into_bytes()`
pub fn compute_deterministic_nonce(key: &[u8], message: &[u8]) -> Vec<u8> {
    todo!("Compute deterministic nonce via HMAC-SHA256")
}

/// Exercise 2: Generate deterministic nonces with counter (RFC 6979 style).
///
/// RFC 6979 generates candidate nonces and checks they are valid (0 < k < n).
/// If not, it increments a counter and tries again.
///
/// Returns the first valid nonce (non-zero, correct length).
///
/// Hints:
/// - Start with k = HMAC-SHA256(key, message || [0x00])
/// - If k is zero or too large, try k = HMAC-SHA256(key, message || [0x01])
/// - Continue with incrementing counter
pub fn generate_rfc6979_nonce(
    key: &[u8],
    message: &[u8],
    curve_order: &[u8],
) -> Vec<u8> {
    todo!("Generate RFC 6979 deterministic nonce with counter")
}

/// Exercise 3: Demonstrate that deterministic nonces are repeatable.
///
/// Given the same key and message, the nonce should always be the same.
///
/// Returns (nonce1, nonce2) — they should be equal.
pub fn demonstrate_determinism(key: &[u8], message: &[u8]) -> (Vec<u8>, Vec<u8>) {
    todo!("Show that deterministic nonces are repeatable")
}

/// Exercise 4: Demonstrate that different messages get different nonces.
///
/// Given the same key but different messages, the nonces should differ.
///
/// Returns (nonce_for_msg1, nonce_for_msg2)
pub fn demonstrate_uniqueness(key: &[u8], msg1: &[u8], msg2: &[u8]) -> (Vec<u8>, Vec<u8>) {
    todo!("Show that different messages get different nonces")
}

/// Exercise 5: Implement a simplified deterministic ECDSA signing function.
///
/// Given a private key scalar and message:
/// 1. Compute deterministic k using HMAC
/// 2. "Sign" by computing a simplified signature (k || hash(message))
///    (This is NOT real ECDSA — just demonstrates the deterministic nonce concept)
///
/// Returns (k_bytes, signature_bytes)
pub fn deterministic_sign(private_key: &[u8], message: &[u8]) -> (Vec<u8>, Vec<u8>) {
    todo!("Simplified deterministic signing")
}

/// Exercise 6: Demonstrate the nonce-reuse attack.
///
/// If an implementation uses a static nonce (like Sony did), the private key
/// can be recovered. This function simulates the attack.
///
/// Given two signatures where k is known and the same:
/// privkey = (s1 * k - z1) * r^(-1) mod n
///
/// This function uses a simplified modular arithmetic to demonstrate.
/// Returns the "recovered" private key.
pub fn simulate_nonce_reuse_attack(
    k: &[u8],
    z1: &[u8],  // hash of message 1
    s1: &[u8],  // signature component for message 1
    z2: &[u8],  // hash of message 2
    s2: &[u8],  // signature component for message 2
) -> Vec<u8> {
    todo!("Simulate recovering private key from nonce reuse")
}

/// Exercise 7: Compare deterministic vs random nonce generation.
///
/// Returns (deterministic_nonce, random_nonce) for the same key and message.
/// The deterministic one is always the same; the random one changes.
pub fn compare_nonce_methods(key: &[u8], message: &[u8]) -> (Vec<u8>, Vec<u8>) {
    todo!("Compare deterministic and random nonce generation")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_nonce_not_empty() {
        let nonce = compute_deterministic_nonce(b"secret_key", b"message");
        assert!(!nonce.is_empty());
        assert_eq!(nonce.len(), 32, "HMAC-SHA256 output should be 32 bytes");
    }

    #[test]
    fn test_deterministic_nonce_repeatable() {
        let (n1, n2) = demonstrate_determinism(b"key", b"msg");
        assert_eq!(n1, n2, "Same key+message should produce same nonce");
    }

    #[test]
    fn test_deterministic_nonce_unique_per_message() {
        let (n1, n2) = demonstrate_uniqueness(b"key", b"msg1", b"msg2");
        assert_ne!(n1, n2, "Different messages should produce different nonces");
    }

    #[test]
    fn test_rfc6979_nonce_valid() {
        let order = vec![0xFFu8; 32]; // dummy large order
        let nonce = generate_rfc6979_nonce(b"key", b"msg", &order);
        assert!(!nonce.is_empty());
        // Nonce should be non-zero
        assert!(nonce.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_deterministic_sign_repeatable() {
        let key = b"private_key_material_here_32byte";
        let (k1, sig1) = deterministic_sign(key, b"message");
        let (k2, sig2) = deterministic_sign(key, b"message");
        assert_eq!(k1, k2, "k should be deterministic");
        assert_eq!(sig1, sig2, "signature should be deterministic");
    }

    #[test]
    fn test_deterministic_sign_varies_by_message() {
        let key = b"private_key_material_here_32byte";
        let (_, sig1) = deterministic_sign(key, b"message1");
        let (_, sig2) = deterministic_sign(key, b"message2");
        assert_ne!(sig1, sig2, "Different messages should produce different signatures");
    }

    #[test]
    fn test_nonce_reuse_attack() {
        // Simplified test — in real ECDSA this would involve modular arithmetic
        let k = [1u8; 32];
        let z1 = [2u8; 32];
        let s1 = [3u8; 32];
        let z2 = [4u8; 32];
        let s2 = [5u8; 32];
        let recovered = simulate_nonce_reuse_attack(&k, &z1, &s1, &z2, &s2);
        assert!(!recovered.is_empty(), "Should recover some key material");
    }

    #[test]
    fn test_compare_nonce_methods() {
        let key = b"test_key_32_bytes_padding_here!!";
        let message = b"test message";
        let (det, rand_nonce) = compare_nonce_methods(key, message);
        assert_eq!(det.len(), 32);
        assert_eq!(rand_nonce.len(), 32);
        // Deterministic is always the same
        let (det2, _) = compare_nonce_methods(key, message);
        assert_eq!(det, det2);
    }
}
