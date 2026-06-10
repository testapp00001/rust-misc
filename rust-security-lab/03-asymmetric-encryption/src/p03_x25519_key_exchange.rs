//! # Lesson 03: X25519 Key Exchange — Curve25519 Diffie-Hellman
//!
//! ## What is X25519?
//!
//! X25519 is an elliptic-curve Diffie-Hellman (ECDH) key agreement protocol
//! based on Curve25519. It was designed by Daniel Bernstein to be:
//! - **Constant-time**: No timing side-channels
//! - **Fast**: ~150,000 key exchanges per second on modern hardware
//! - **Simple**: Hard to misuse compared to NIST curves
//! - **Widely adopted**: TLS 1.3, Signal, WireGuard, SSH
//!
//! ## How X25519 Works
//!
//! 1. Alice generates private key `a` (random 32 bytes), public key `A = a * G`
//! 2. Bob generates private key `b`, public key `B = b * G`
//! 3. Alice computes: `shared = a * B = a * b * G`
//! 4. Bob computes:   `shared = b * A = b * a * G`
//! 5. Both have the same `shared = a * b * G` — the Diffie-Hellman secret
//!
//! An eavesdropper sees only A and B, and cannot compute `a * b * G`
//! (the elliptic curve discrete logarithm problem).
//!
//! ## Why X25519 over RSA for Key Exchange?
//!
//! | Property | RSA-3072 | X25519 |
//! |----------|----------|--------|
//! | Key size | 3072 bits | 256 bits |
//! | Key exchange speed | ~1000/s | ~150,000/s |
//! | Forward secrecy | No (with static keys) | Yes (with ephemeral keys) |
//! | Constant-time | Hard to implement | Built-in |
//!
//! ## Attack Demo: Shared Secret Without Key Agreement
//!
//! Without a key exchange protocol, two parties must somehow share a secret key
//! over an insecure channel — a chicken-and-egg problem. X25519 solves this:
//! both parties contribute entropy, and the shared secret emerges from the combination.

use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};
use rand::rngs::OsRng;

/// Exercise 1: Perform a complete X25519 key exchange between two parties.
///
/// Returns the shared secret that both parties compute independently.
///
/// Hints:
/// - Generate Alice's secret: `EphemeralSecret::random_from_rng(OsRng)`
/// - Get Alice's public key: `PublicKey::from(&alice_secret)`
/// - Do the same for Bob
/// - Alice computes shared secret: `alice_secret.diffie_hellman(&bob_public)`
/// - Bob computes shared secret: `bob_secret.diffie_hellman(&alice_public)`
/// - Both `.as_bytes()` should be equal
pub fn key_exchange() -> SharedSecret {
    todo!("Perform X25519 key exchange between Alice and Bob")
}

/// Exercise 2: Verify that both parties derive the same shared secret.
///
/// Returns `true` if Alice's and Bob's computed secrets match.
///
/// Hints:
/// - Perform key exchange for both parties
/// - Compare the raw bytes: `alice_shared.as_bytes() == bob_shared.as_bytes()`
pub fn verify_shared_secret() -> bool {
    todo!("Verify both parties compute the same shared secret")
}

/// Exercise 3: Demonstrate that the shared secret is the same regardless of party order.
///
/// X25519 is commutative: a*(b*G) == b*(a*G).
/// This function should verify that property.
pub fn verify_commutativity() -> bool {
    todo!("Demonstrate X25519 commutativity")
}

/// Exercise 4: Derive a symmetric key from the X25519 shared secret.
///
/// Raw shared secrets should not be used directly as symmetric keys.
/// Use HKDF to derive a proper key from the shared secret.
///
/// Hints:
/// - Perform X25519 key exchange
/// - Use `ring::hkdf` to derive a key from the shared secret
/// - Use SHA-256 as the hash function
/// - Output a 32-byte key suitable for AES-256
pub fn derive_symmetric_key() -> Vec<u8> {
    todo!("Derive AES-256 key from X25519 shared secret using HKDF")
}

/// Exercise 5: Demonstrate that different key pairs produce different shared secrets.
///
/// Two independent key exchanges should produce different shared secrets
/// (with overwhelming probability).
pub fn different_keys_different_secrets() -> bool {
    todo!("Show that different keypairs produce different shared secrets")
}

/// Exercise 6: Extract raw public key bytes from an X25519 public key.
///
/// X25519 public keys are 32 bytes. Demonstrate serialization.
///
/// Hints:
/// - Generate a secret and public key
/// - Use `public_key.as_bytes()` to get `&[u8; 32]`
pub fn public_key_bytes() -> [u8; 32] {
    todo!("Extract raw bytes from X25519 public key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_exchange_produces_secret() {
        let secret = key_exchange();
        // X25519 shared secrets are 32 bytes
        assert_eq!(secret.as_bytes().len(), 32);
    }

    #[test]
    fn test_both_parties_agree() {
        assert!(verify_shared_secret(), "Both parties should compute the same shared secret");
    }

    #[test]
    fn test_commutativity() {
        assert!(verify_commutativity(), "X25519 should be commutative");
    }

    #[test]
    fn test_derived_key_is_32_bytes() {
        let key = derive_symmetric_key();
        assert_eq!(key.len(), 32, "Derived key should be 32 bytes for AES-256");
    }

    #[test]
    fn test_different_keys_different_secrets() {
        assert!(different_keys_different_secrets(),
            "Different keypairs should produce different shared secrets");
    }

    #[test]
    fn test_public_key_is_32_bytes() {
        let bytes = public_key_bytes();
        // Just verify it runs and returns 32 bytes
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_key_exchange_deterministic_for_same_keys() {
        // If we use the same key material, the result should be the same
        let secret1 = EphemeralSecret::random_from_rng(OsRng);
        let pub1 = PublicKey::from(&secret1);
        let secret2 = EphemeralSecret::random_from_rng(OsRng);
        let pub2 = PublicKey::from(&secret2);

        let shared1 = secret1.diffie_hellman(&pub2);
        let shared2 = secret2.diffie_hellman(&pub1);
        assert_eq!(shared1.as_bytes(), shared2.as_bytes());
    }
}
