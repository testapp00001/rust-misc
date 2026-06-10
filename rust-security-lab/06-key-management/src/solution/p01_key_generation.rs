//! # Lesson 01: Generating Cryptographically Secure Random Keys (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::rand::{SecureRandom, SystemRandom};

/// Generate a random 256-bit (32-byte) key using OS entropy.
///
/// `ring::SystemRandom` uses the OS CSPRNG:
/// - Linux: `/dev/urandom` (or `getrandom` syscall)
/// - Windows: `BCryptGenRandom`
/// - macOS: `SecRandomCopyBytes`
///
/// Never use `rand::thread_rng()` for cryptographic keys unless you're certain
/// it's backed by a CSPRNG on your target platform.
pub fn generate_symmetric_key() -> Vec<u8> {
    let rng = SystemRandom::new();
    let mut key = vec![0u8; 32];
    rng.fill(&mut key).expect("Failed to generate random key");
    key
}

/// Generate a random key of arbitrary length.
pub fn generate_key(length: usize) -> Vec<u8> {
    let rng = SystemRandom::new();
    let mut key = vec![0u8; length];
    rng.fill(&mut key).expect("Failed to generate random key");
    key
}

/// Generate a random 96-bit (12-byte) nonce.
///
/// In AES-GCM, the nonce must be unique for every encryption with the same key.
/// A 96-bit random nonce gives a collision probability of ~2^-48 after 2^48 encryptions
/// (birthday bound), which is safe for virtually all use cases.
pub fn generate_nonce() -> Vec<u8> {
    let rng = SystemRandom::new();
    let mut nonce = vec![0u8; 12];
    rng.fill(&mut nonce).expect("Failed to generate random nonce");
    nonce
}

/// Generate a random 16-byte salt for key derivation.
///
/// Salts prevent rainbow table attacks on password-derived keys.
/// They don't need to be secret — just unique per user/password.
pub fn generate_salt() -> Vec<u8> {
    let rng = SystemRandom::new();
    let mut salt = vec![0u8; 16];
    rng.fill(&mut salt).expect("Failed to generate random salt");
    salt
}

/// Demonstrate deterministic (WEAK) key generation with a fixed seed.
///
/// This is a teaching exercise. It shows why seeding a PRNG with a constant
/// is catastrophic — the key is fully predictable to anyone who knows the seed.
///
/// NEVER use this in production.
pub fn demonstrate_weak_rng(seed: u64) -> Vec<u8> {
    use rand::rngs::StdRng;
    use rand::{RngCore, SeedableRng};

    let mut rng = StdRng::seed_from_u64(seed);
    let mut key = vec![0u8; 32];
    rng.fill_bytes(&mut key);
    key
}

/// Generate multiple independent keys from one RNG instance.
///
/// `SystemRandom` maintains internal state from the OS entropy pool.
/// Each call to `fill()` advances the internal state, so successive keys
/// are independent even though they come from the same `SystemRandom` instance.
pub fn generate_key_set(count: usize, key_length: usize) -> Vec<Vec<u8>> {
    let rng = SystemRandom::new();
    let mut keys = Vec::with_capacity(count);
    for _ in 0..count {
        let mut key = vec![0u8; key_length];
        rng.fill(&mut key).expect("Failed to generate random key");
        keys.push(key);
    }
    keys
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
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[1], keys[2]);
        assert_ne!(keys[0], keys[2]);
    }
}
