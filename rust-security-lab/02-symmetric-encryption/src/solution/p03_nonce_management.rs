//! # Lesson 03: Nonce Management (Reference Solution)
//!
//! See the exercise file for full documentation on nonce strategies and their trade-offs.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};
use std::sync::atomic::{AtomicU64, Ordering};

/// Generate a random 12-byte nonce using OsRng.
///
/// Safe for up to ~2^32 encryptions per key before birthday-bound collisions
/// become non-negligible.
pub fn random_nonce() -> aes_gcm::aead::Nonce<Aes256Gcm> {
    Aes256Gcm::generate_nonce(&mut OsRng)
}

/// Counter-based nonce generator.
///
/// Uses an atomic counter for thread-safe, collision-free nonce generation.
/// The 96-bit (12-byte) nonce format:
///   [0, 0, 0, 0, counter_byte_7, ..., counter_byte_0]
///
/// Security note: The counter MUST persist across process restarts. If the process
/// crashes and restarts with counter=0, nonces will repeat with high probability.
pub struct CounterNonce {
    counter: AtomicU64,
}

impl CounterNonce {
    pub fn new(initial_value: u64) -> Self {
        Self {
            counter: AtomicU64::new(initial_value),
        }
    }

    pub fn next_nonce(&self) -> aes_gcm::aead::Nonce<Aes256Gcm> {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        let bytes = count.to_be_bytes();
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..12].copy_from_slice(&bytes);
        *Nonce::<Aes256Gcm>::from_slice(&nonce_bytes)
    }
}

/// Check if the counter would safely cover the requested number of encryptions.
///
/// A 64-bit counter starting at `initial` can safely handle
/// `u64::MAX - initial` increments before wrapping to 0.
pub fn is_counter_safe(initial: u64, num_encryptions: u64) -> bool {
    initial.checked_add(num_encryptions).is_some()
}

/// Encrypt with an externally managed nonce.
///
/// This pattern separates nonce management from encryption, giving the caller
/// full control over the nonce strategy (counter, random, SIV, etc.).
pub fn encrypt_with_nonce(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    plaintext: &[u8],
) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    cipher
        .encrypt(nonce, plaintext)
        .expect("Encryption should not fail")
}

/// Deterministic encryption (SIV-like).
///
/// Derives the nonce from the plaintext hash, making encryption deterministic:
/// same (key, plaintext) always produces the same ciphertext.
///
/// WARNING: Deterministic encryption leaks whether two plaintexts are equal.
/// Only use when this leakage is acceptable (e.g., encrypting database indexes).
/// In production, use AES-SIV (RFC 5297) or AES-GCM-SIV (RFC 8452) instead.
pub fn encrypt_deterministic(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> Vec<u8> {
    let hash = ring::digest::digest(&ring::digest::SHA256, plaintext);
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes.copy_from_slice(&hash.as_ref()[..12]);
    let nonce = Nonce::<Aes256Gcm>::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new(key);
    cipher
        .encrypt(nonce, plaintext)
        .expect("Encryption should not fail")
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
        assert_ne!(n1.as_slice(), n2.as_slice());

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
        assert!(!is_counter_safe(u64::MAX, 1));
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
        assert_eq!(ct1, ct2);
    }

    #[test]
    fn test_deterministic_different_plaintext() {
        let key = test_key();
        let ct1 = encrypt_deterministic(&key, b"message A");
        let ct2 = encrypt_deterministic(&key, b"message B");
        assert_ne!(ct1, ct2);
    }
}
