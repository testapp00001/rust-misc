//! # Lesson 01: Key Generation — Identity and Ephemeral Keypairs (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ed25519_dalek::{SigningKey as Ed25519SigningKey, VerifyingKey as Ed25519VerifyingKey, Signature, Signer, Verifier};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};

/// A complete identity for a messenger user.
#[derive(Clone)]
pub struct Identity {
    pub signing_key: Ed25519SigningKey,
}

impl Identity {
    /// Generate a new random identity.
    pub fn generate() -> Self {
        Self {
            signing_key: Ed25519SigningKey::generate(&mut OsRng),
        }
    }

    /// Get the verifying (public) key.
    pub fn verifying_key(&self) -> Ed25519VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Sign a message with this identity's signing key.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Verify a signature against this identity's public key.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        self.signing_key.verifying_key().verify(message, signature).is_ok()
    }

    /// Generate an ephemeral X25519 keypair for key exchange.
    pub fn generate_ephemeral() -> (EphemeralSecret, X25519PublicKey) {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public = X25519PublicKey::from(&secret);
        (secret, public)
    }

    /// Generate a signed prekey — an X25519 keypair signed by the identity.
    pub fn generate_signed_prekey(&self) -> (StaticSecret, X25519PublicKey, Signature) {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = X25519PublicKey::from(&secret);
        let signature = self.signing_key.sign(public.as_bytes());
        (secret, public, signature)
    }
}

/// Derive a fingerprint from an Ed25519 verifying key.
pub fn fingerprint(key: &Ed25519VerifyingKey) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generate() {
        let identity = Identity::generate();
        let _vk = identity.verifying_key();
    }

    #[test]
    fn test_sign_and_verify() {
        let identity = Identity::generate();
        let message = b"Hello, secure world!";
        let signature = identity.sign(message);
        assert!(identity.verify(message, &signature), "Valid signature should verify");
    }

    #[test]
    fn test_verify_wrong_message() {
        let identity = Identity::generate();
        let message = b"Original message";
        let signature = identity.sign(message);
        let tampered = b"Tampered message";
        assert!(!identity.verify(tampered, &signature), "Signature on wrong message should fail");
    }

    #[test]
    fn test_verify_wrong_key() {
        let alice = Identity::generate();
        let bob = Identity::generate();
        let message = b"Hello from Alice";
        let signature = alice.sign(message);
        assert!(
            bob.verifying_key().verify(message, &signature).is_err(),
            "Wrong key should not verify signature"
        );
    }

    #[test]
    fn test_ephemeral_keypair() {
        let (secret, public) = Identity::generate_ephemeral();
        let (secret2, public2) = Identity::generate_ephemeral();
        let shared1 = secret.diffie_hellman(&public2);
        let shared2 = secret2.diffie_hellman(&public);
        assert_eq!(shared1.as_bytes(), shared2.as_bytes(), "DH should produce same shared secret");
    }

    #[test]
    fn test_signed_prekey_signature() {
        let identity = Identity::generate();
        let (_secret, public, signature) = identity.generate_signed_prekey();
        assert!(
            identity.verify(public.as_bytes(), &signature),
            "Signed prekey signature should be valid"
        );
    }

    #[test]
    fn test_fingerprint_deterministic() {
        let identity = Identity::generate();
        let fp1 = fingerprint(&identity.verifying_key());
        let fp2 = fingerprint(&identity.verifying_key());
        assert_eq!(fp1, fp2, "Fingerprint should be deterministic");
    }

    #[test]
    fn test_fingerprint_different_keys() {
        let alice = Identity::generate();
        let bob = Identity::generate();
        let fp_alice = fingerprint(&alice.verifying_key());
        let fp_bob = fingerprint(&bob.verifying_key());
        assert_ne!(fp_alice, fp_bob, "Different keys should have different fingerprints");
    }
}
