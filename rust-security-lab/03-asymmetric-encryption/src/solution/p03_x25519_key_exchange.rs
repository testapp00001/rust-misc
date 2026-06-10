//! # Lesson 03: X25519 Key Exchange — Curve25519 Diffie-Hellman (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};
use ring::hkdf;
use rand::rngs::OsRng;

/// Perform a complete X25519 key exchange between two parties.
///
/// X25519 is an ECDH protocol on Curve25519:
/// - Private key: 32 random bytes (clamped)
/// - Public key: private_key * G (where G is the base point)
/// - Shared secret: alice_secret * bob_public = alice_secret * bob_secret * G
///
/// The clamping ensures the private key is a multiple of 8 and has the
/// correct bit length, preventing small-subgroup attacks automatically.
pub fn key_exchange() -> SharedSecret {
    let alice_secret = EphemeralSecret::random_from_rng(OsRng);
    let alice_public = PublicKey::from(&alice_secret);

    let bob_secret = EphemeralSecret::random_from_rng(OsRng);
    let bob_public = PublicKey::from(&bob_secret);

    // Both parties compute the same shared secret
    let alice_shared = alice_secret.diffie_hellman(&bob_public);
    let bob_shared = bob_secret.diffie_hellman(&alice_public);

    // Verify they match (in production, don't assert — just use one)
    assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());

    alice_shared
}

/// Verify that both parties derive the same shared secret.
///
/// This is the fundamental property of Diffie-Hellman:
/// a*(b*G) == b*(a*G) because scalar multiplication is commutative.
pub fn verify_shared_secret() -> bool {
    let alice_secret = EphemeralSecret::random_from_rng(OsRng);
    let alice_public = PublicKey::from(&alice_secret);

    let bob_secret = EphemeralSecret::random_from_rng(OsRng);
    let bob_public = PublicKey::from(&bob_secret);

    let alice_shared = alice_secret.diffie_hellman(&bob_public);
    let bob_shared = bob_secret.diffie_hellman(&alice_public);

    alice_shared.as_bytes() == bob_shared.as_bytes()
}

/// Demonstrate X25519 commutativity: a*B == b*A.
///
/// This mathematical property is what makes DH work. The attacker sees
/// A = a*G and B = b*G, but cannot compute a*b*G without knowing a or b
/// (the elliptic curve discrete logarithm problem).
pub fn verify_commutativity() -> bool {
    let secret_a = EphemeralSecret::random_from_rng(OsRng);
    let public_a = PublicKey::from(&secret_a);

    let secret_b = EphemeralSecret::random_from_rng(OsRng);
    let public_b = PublicKey::from(&secret_b);

    let shared_ab = secret_a.diffie_hellman(&public_b);
    let shared_ba = secret_b.diffie_hellman(&public_a);

    shared_ab.as_bytes() == shared_ba.as_bytes()
}

/// Derive a symmetric key from the X25519 shared secret using HKDF.
///
/// Raw shared secrets should NEVER be used directly as symmetric keys because:
/// 1. They may not have uniform distribution
/// 2. Different contexts should use different keys
/// 3. HKDF allows domain separation via "info" parameter
///
/// HKDF-SHA256: extract → expand → 32-byte key
pub fn derive_symmetric_key() -> Vec<u8> {
    let alice_secret = EphemeralSecret::random_from_rng(OsRng);

    let bob_secret = EphemeralSecret::random_from_rng(OsRng);
    let bob_public = PublicKey::from(&bob_secret);

    let shared = alice_secret.diffie_hellman(&bob_public);

    // Derive key using HKDF
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, b"x25519-session-key");
    let prk = salt.extract(shared.as_bytes());
    let okm = prk.expand(&[b"aes-256-gcm"], hkdf::HKDF_SHA256).unwrap();
    let mut key = vec![0u8; 32];
    okm.fill(&mut key).unwrap();
    key
}

/// Show that different keypairs produce different shared secrets.
///
/// Each random keypair generates a unique point on the curve,
/// so the shared secret will be different with overwhelming probability.
pub fn different_keys_different_secrets() -> bool {
    let s1 = EphemeralSecret::random_from_rng(OsRng);
    let s2 = EphemeralSecret::random_from_rng(OsRng);
    let p2 = PublicKey::from(&s2);

    let shared1 = s1.diffie_hellman(&p2);

    let s3 = EphemeralSecret::random_from_rng(OsRng);
    let s4 = EphemeralSecret::random_from_rng(OsRng);
    let p4 = PublicKey::from(&s4);

    let shared2 = s3.diffie_hellman(&p4);

    shared1.as_bytes() != shared2.as_bytes()
}

/// Extract raw bytes from an X25519 public key.
///
/// X25519 public keys are exactly 32 bytes — the x-coordinate of the
/// point on Curve25519 (the y-coordinate is not needed for DH).
pub fn public_key_bytes() -> [u8; 32] {
    let secret = EphemeralSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&secret);
    *public_key.as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_exchange_produces_secret() {
        let secret = key_exchange();
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
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_key_exchange_deterministic_for_same_keys() {
        let secret1 = EphemeralSecret::random_from_rng(OsRng);
        let pub1 = PublicKey::from(&secret1);
        let secret2 = EphemeralSecret::random_from_rng(OsRng);
        let pub2 = PublicKey::from(&secret2);

        let shared1 = secret1.diffie_hellman(&pub2);
        let shared2 = secret2.diffie_hellman(&pub1);
        assert_eq!(shared1.as_bytes(), shared2.as_bytes());
    }
}
