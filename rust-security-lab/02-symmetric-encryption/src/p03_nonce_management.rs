//! # Lesson 03: Nonce Management
//!
//! ## The Nonce Problem
//!
//! A **nonce** (Number Used Once) is the most critical operational detail in AEAD encryption.
//! It is NOT secret — it is sent alongside the ciphertext. But it MUST be unique per
//! (key, nonce) pair.
//!
//! ## Why Nonce Uniqueness Matters
//!
//! In AES-GCM, the nonce feeds into CTR mode to produce a keystream:
//!
//! ```text
//! Keystream = AES(key, nonce || counter_1) || AES(key, nonce || counter_2) || ...
//! Ciphertext = Plaintext XOR Keystream
//! ```
//!
//! If the same (key, nonce) is used twice with plaintexts P1 and P2:
//!
//! ```text
//! C1 = P1 XOR Keystream
//! C2 = P2 XOR Keystream
//! C1 XOR C2 = P1 XOR P2  → attacker recovers both plaintexts!
//! ```
//!
//! For GCM specifically, nonce reuse also leaks the GHASH authentication key,
//! allowing the attacker to **forge** authentication tags for any message.
//!
//! ## Nonce Strategies
//!
//! | Strategy | Size | Risk | Best For |
//! |----------|------|------|----------|
//! | Random 96-bit | 12 bytes | Birthday bound: safe up to ~2^32 encryptions | Multi-device, stateless |
//! | Counter | 12 bytes | Zero collision risk if monotonic | Single-device, database |
//! | SIV (Synthetic IV) | 256 bits | Deterministic, nonce-misuse resistant | Key-value stores |

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key,
};
use std::sync::atomic::{AtomicU64, Ordering};

/// Exercise 1: Random nonce generator.
///
/// Generate random 12-byte nonces. This is the simplest strategy — safe as long as
/// you don't exceed ~2^32 encryptions per key.
///
/// Hints:
/// - Use `Aes256Gcm::generate_nonce(&mut OsRng)`
pub fn random_nonce() -> aes_gcm::aead::Nonce<Aes256Gcm> {
    todo!("Generate a random nonce")
}

/// Exercise 2: Counter-based nonce generator.
///
/// Use an atomic counter to guarantee uniqueness. The 12-byte nonce encodes
/// a monotonically increasing 64-bit counter (big-endian) in the last 8 bytes.
///
/// Format: [0, 0, 0, 0, counter_bytes...]
///
/// Hints:
/// - Use `AtomicU64` for thread-safe incrementing
/// - Encode counter as big-endian: `counter.to_be_bytes()`
/// - Zero-pad the first 4 bytes
///
/// Security note: The counter state must persist across restarts or you risk reuse.
pub struct CounterNonce {
    counter: AtomicU64,
}

impl CounterNonce {
    pub fn new(initial_value: u64) -> Self {
        todo!("Initialize counter with given value")
    }

    pub fn next_nonce(&self) -> aes_gcm::aead::Nonce<Aes256Gcm> {
        todo!("Generate next nonce from counter")
    }
}

/// Exercise 3: Demonstrate the danger of counter wrap-around.
///
/// If a 64-bit counter wraps around to 0, the nonce repeats.
/// This function returns true if the counter-based nonce would be safe
/// for the given number of encryptions starting from the given initial value.
///
/// Hints:
/// - A 64-bit counter can safely handle 2^64 - 1 increments without wrapping
/// - But in practice, we should check for overflow
pub fn is_counter_safe(initial: u64, num_encryptions: u64) -> bool {
    todo!("Check if counter would wrap around")
}

/// Exercise 4: Encrypt with explicit nonce management (caller provides nonce).
///
/// This separates nonce generation from encryption, giving the caller control
/// over the nonce strategy.
///
/// Hints:
/// - Take key, nonce, and plaintext as parameters
/// - Encrypt using AES-256-GCM
/// - Return only the ciphertext (nonce is managed by caller)
pub fn encrypt_with_nonce(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    plaintext: &[u8],
) -> Vec<u8> {
    todo!("Encrypt with externally managed nonce")
}

/// Exercise 5: Nonce-misuse resistant encryption concept.
///
/// SIV (Synthetic IV) modes derive the nonce from the plaintext and key,
/// making them deterministic but nonce-misuse resistant. If the same plaintext
/// is encrypted twice, the same ciphertext is produced (no information leak
/// beyond what the deterministic encryption reveals).
///
/// This is a conceptual exercise — implement a simple deterministic "SIV-like"
/// mode using AES-GCM where the nonce is derived from the plaintext hash.
///
/// Hints:
/// - Hash the plaintext with SHA-256
/// - Take the first 12 bytes as the nonce
/// - Encrypt with that nonce
/// - Same plaintext → same nonce → same ciphertext (deterministic)
pub fn encrypt_deterministic(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> Vec<u8> {
    todo!("Implement deterministic encryption (SIV-like)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Key<Aes256Gcm> {
        Aes256Gcm::generate_key(&mut OsRng)
    }

    #[test]
    fn test_random_nonce_uniqueness() {
        let n1 = random_nonce();
        let n2 = random_nonce();
        assert_ne!(n1.as_slice(), n2.as_slice());
    }

    #[test]
    fn test_random_nonce_length() {
        let nonce = random_nonce();
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_counter_nonce_sequential() {
        let counter = CounterNonce::new(0);
        let n1 = counter.next_nonce();
        let n2 = counter.next_nonce();
        assert_ne!(n1.as_slice(), n2.as_slice(), "Sequential nonces must differ");

        // Verify counter increments
        let c1 = u64::from_be_bytes(n1[4..12].try_into().unwrap());
        let c2 = u64::from_be_bytes(n2[4..12].try_into().unwrap());
        assert_eq!(c2, c1 + 1);
    }

    #[test]
    fn test_counter_nonce_starting_value() {
        let counter = CounterNonce::new(1000);
        let nonce = counter.next_nonce();
        let val = u64::from_be_bytes(nonce[4..12].try_into().unwrap());
        assert_eq!(val, 1000);
    }

    #[test]
    fn test_is_counter_safe_normal() {
        assert!(is_counter_safe(0, 1_000_000));
    }

    #[test]
    fn test_is_counter_safe_overflow() {
        assert!(!is_counter_safe(u64::MAX, 2));
    }

    #[test]
    fn test_is_counter_safe_exact_boundary() {
        assert!(!is_counter_safe(u64::MAX, 1), "u64::MAX + 1 wraps to 0");
        assert!(is_counter_safe(u64::MAX - 1, 1));
    }

    #[test]
    fn test_encrypt_with_nonce_roundtrip() {
        let key = test_key();
        let nonce = random_nonce();
        let plaintext = b"explicit nonce test";

        let ciphertext = encrypt_with_nonce(&key, &nonce, plaintext);
        let cipher = Aes256Gcm::new(&key);
        let decrypted = cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_deterministic_encryption() {
        let key = test_key();
        let plaintext = b"deterministic test";

        let ct1 = encrypt_deterministic(&key, plaintext);
        let ct2 = encrypt_deterministic(&key, plaintext);
        assert_eq!(ct1, ct2, "Same plaintext should produce same ciphertext");
    }

    #[test]
    fn test_deterministic_different_plaintext() {
        let key = test_key();
        let ct1 = encrypt_deterministic(&key, b"message A");
        let ct2 = encrypt_deterministic(&key, b"message B");
        assert_ne!(ct1, ct2, "Different plaintexts must produce different ciphertexts");
    }
}
