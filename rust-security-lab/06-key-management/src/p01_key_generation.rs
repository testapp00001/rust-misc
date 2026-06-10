//! # Lesson 01: Generating Cryptographically Secure Random Keys
//!
//! ## What Is Key Generation?
//!
//! A cryptographic key is only as strong as the randomness used to create it.
//! Key generation must use a **Cryptographically Secure Pseudo-Random Number Generator
//! (CSPRNG)** — one that is unpredictable even if an attacker observes all previous outputs.
//!
//! In Rust, `ring::SystemRandom` is the recommended CSPRNG. It uses the operating
//! system's entropy source (`/dev/urandom` on Linux, `CryptGenRandom` on Windows).
//!
//! ## Why This Matters
//!
//! If an attacker can predict your key, encryption is worthless. Real-world failures:
//! - **Debian OpenSSL bug (2008)**: A code change reduced entropy to 15 bits. All SSH
//!   keys generated on Debian for 2 years were predictable.
//! - **Android SecureRandom (2013)**: Java's PRNG was not properly seeded on some Android
//!   devices, making Bitcoin wallet keys predictable. $5,500 stolen.
//! - **IoT devices**: Many embedded systems use `rand()` or `time()` as seed — trivially
//!   guessable.
//!
//! ## Attack Demo: Weak Key Generation
//!
//! A naive implementation using `rand::thread_rng()` with a fixed seed:
//!
//! ```rust,ignore
//! use rand::rngs::StdRng;
//! use rand::SeedableRng;
//!
//! // BAD: Predictable seed — attacker can reproduce this exact sequence
//! let mut rng = StdRng::seed_from_u64(42);
//! let mut key = [0u8; 32];
//! rng.fill(&mut key);
//! // key is always the same for seed 42!
//! ```
//!
//! ## Defense: Use `ring::SystemRandom`
//!
//! `ring::SystemRandom` pulls entropy from the OS kernel. There is no seed to guess.
//!
//! ```rust,ignore
//! use ring::rand::{SecureRandom, SystemRandom};
//!
//! let rng = SystemRandom::new();
//! let mut key = [0u8; 32];
//! rng.fill(&mut key).unwrap();
//! // key is different every time — no seed, no pattern
//! ```

use ring::rand::{SecureRandom, SystemRandom};

/// Exercise 1: Generate a random 256-bit (32-byte) key.
///
/// Use `ring::SystemRandom` to fill a 32-byte array with random data.
///
/// Hints:
/// - Create RNG: `let rng = SystemRandom::new();`
/// - Fill buffer: `rng.fill(&mut key).unwrap();`
/// - Return the key as `Vec<u8>`
pub fn generate_symmetric_key() -> Vec<u8> {
    todo!("Generate a 32-byte random key using ring::SystemRandom")
}

/// Exercise 2: Generate a random key of arbitrary length.
///
/// The caller specifies how many bytes of key material they need.
///
/// Hints:
/// - Create a `Vec<u8>` of the requested length: `vec![0u8; length]`
/// - Fill it with random bytes using `SystemRandom`
pub fn generate_key(length: usize) -> Vec<u8> {
    todo!("Generate a random key of specified length")
}

/// Exercise 3: Generate a random 96-bit (12-byte) nonce.
///
/// Nonces must be unique per key. For AES-GCM, the standard nonce is 12 bytes.
///
/// Hints:
/// - Same approach as key generation, but 12 bytes
/// - Use `ring::SystemRandom`
pub fn generate_nonce() -> Vec<u8> {
    todo!("Generate a 12-byte random nonce")
}

/// Exercise 4: Generate a random salt for key derivation.
///
/// Salts should be at least 16 bytes and must be unique per key derivation.
/// They are stored alongside the derived key (no secrecy required).
///
/// Hints:
/// - 16 bytes is a good salt size
/// - Use `ring::SystemRandom`
pub fn generate_salt() -> Vec<u8> {
    todo!("Generate a 16-byte random salt")
}

/// Exercise 5: Demonstrate that weak randomness produces predictable keys.
///
/// This function should use `rand::rngs::StdRng::seed_from_u64(42)` to show
/// that a fixed seed produces the same key every time. This is a teaching exercise —
/// never do this in production!
///
/// Hints:
/// - Use `rand::RngCore` trait for `.fill_bytes()`
/// - Use `rand::SeedableRng` for `StdRng::seed_from_u64()`
/// - Generate two keys with the same seed and verify they're identical
pub fn demonstrate_weak_rng(seed: u64) -> Vec<u8> {
    todo!("Demonstrate predictable key generation with a fixed seed")
}

/// Exercise 6: Generate multiple independent keys from one RNG instance.
///
/// In practice, you create one `SystemRandom` and use it for multiple key generations.
/// Each key must be independent — observing one key must not reveal others.
///
/// Hints:
/// - Create `SystemRandom` once
/// - Use it to generate `count` keys, each of `key_length` bytes
/// - Return as `Vec<Vec<u8>>`
pub fn generate_key_set(count: usize, key_length: usize) -> Vec<Vec<u8>> {
    todo!("Generate multiple independent keys from one RNG")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symmetric_key_length() {
        let key = generate_symmetric_key();
        assert_eq!(key.len(), 32, "AES-256 key must be 32 bytes");
    }

    #[test]
    fn test_symmetric_key_randomness() {
        let key1 = generate_symmetric_key();
        let key2 = generate_symmetric_key();
        assert_ne!(key1, key2, "Two generated keys must be different");
    }

    #[test]
    fn test_generate_key_length() {
        let key = generate_key(64);
        assert_eq!(key.len(), 64);
    }

    #[test]
    fn test_generate_key_zero_length() {
        let key = generate_key(0);
        assert_eq!(key.len(), 0);
    }

    #[test]
    fn test_nonce_length() {
        let nonce = generate_nonce();
        assert_eq!(nonce.len(), 12, "AES-GCM nonce must be 12 bytes");
    }

    #[test]
    fn test_nonce_randomness() {
        let n1 = generate_nonce();
        let n2 = generate_nonce();
        assert_ne!(n1, n2, "Two nonces must be different");
    }

    #[test]
    fn test_salt_length() {
        let salt = generate_salt();
        assert_eq!(salt.len(), 16, "Salt must be at least 16 bytes");
    }

    #[test]
    fn test_weak_rng_deterministic() {
        let key1 = demonstrate_weak_rng(42);
        let key2 = demonstrate_weak_rng(42);
        assert_eq!(key1, key2, "Same seed must produce same key (this is why it's weak!)");
    }

    #[test]
    fn test_weak_rng_different_seeds() {
        let key1 = demonstrate_weak_rng(42);
        let key2 = demonstrate_weak_rng(99);
        assert_ne!(key1, key2, "Different seeds must produce different keys");
    }

    #[test]
    fn test_key_set_count() {
        let keys = generate_key_set(5, 32);
        assert_eq!(keys.len(), 5);
        for key in &keys {
            assert_eq!(key.len(), 32);
        }
    }

    #[test]
    fn test_key_set_independence() {
        let keys = generate_key_set(3, 32);
        // All keys should be different (probability of collision is negligible)
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[1], keys[2]);
        assert_ne!(keys[0], keys[2]);
    }
}
