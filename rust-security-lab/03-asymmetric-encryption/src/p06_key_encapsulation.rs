//! # Lesson 06: Key Encapsulation (KEM) — Wrapping Secrets with Public Keys
//!
//! ## What is Key Encapsulation?
//!
//! A Key Encapsulation Mechanism (KEM) is a simplified form of public-key encryption
//! specifically designed to transport symmetric keys. Instead of encrypting arbitrary
//! data, a KEM has three operations:
//!
//! 1. **Keygen** → (public_key, private_key)
//! 2. **Encapsulate**(public_key) → (ciphertext, shared_secret)
//! 3. **Decapsulate**(private_key, ciphertext) → shared_secret
//!
//! The sender generates a random shared secret and "wraps" it with the recipient's
//! public key. Only the recipient can "unwrap" it with their private key.
//!
//! ## Why KEM Instead of Direct Encryption?
//!
//! - **Simpler API**: No padding oracle attacks (unlike RSA-OAEP encrypt/decrypt)
//! - **Better security proofs**: Easier to prove secure in formal models
//! - **Post-quantum ready**: NIST PQC standards (Kyber/ML-KEM) use KEM, not encryption
//! - **Standard pattern**: TLS 1.3 uses KEM-like key exchange
//!
//! ## Real-World Usage
//!
//! - TLS 1.3: Server encapsulates a pre-shared key
//! - Signal Protocol: X3DH key agreement is essentially a KEM
//! - Hybrid PQC: Classical KEM + post-quantum KEM combined
//!
//! ## Attack Demo: Why Not Just Use RSA Encrypt?
//!
//! RSA-OAEP encrypt is essentially a KEM, but the API is more complex and
//! error-prone. A dedicated KEM API makes it harder to misuse.

use x25519_dalek::{EphemeralSecret, PublicKey};
use ring::hkdf;
use rand::rngs::OsRng;

/// A KEM ciphertext (encrypted key material) and the associated shared secret.
pub struct KemOutput {
    /// The encapsulated key material to send to the recipient.
    pub ciphertext: Vec<u8>,
    /// The shared secret derived by both parties (32 bytes).
    pub shared_secret: [u8; 32],
}

/// Exercise 1: Implement KEM key generation.
///
/// Generate an X25519 keypair for use with the KEM.
///
/// Returns (public_key_bytes, secret_key_bytes).
///
/// Hints:
/// - Generate `EphemeralSecret::random_from_rng(OsRng)`
/// - Get public key: `PublicKey::from(&secret)`
/// - Return both as byte arrays
pub fn kem_keygen() -> ([u8; 32], [u8; 32]) {
    todo!("Generate a KEM keypair (X25519)")
}

/// Exercise 2: Implement KEM encapsulation.
///
/// Given a recipient's public key, generate a random shared secret and
/// encapsulate it so only the recipient can recover it.
///
/// Returns a `KemOutput` containing the ciphertext and shared secret.
///
/// Hints:
/// - Generate an ephemeral keypair (the "encapsulation key")
/// - Perform X25519 DH with recipient's public key
/// - Derive the shared secret using HKDF
/// - The "ciphertext" is the ephemeral public key (sent to recipient)
pub fn kem_encapsulate(recipient_public: &[u8; 32]) -> KemOutput {
    todo!("Encapsulate a shared secret using recipient's public key")
}

/// Exercise 3: Implement KEM decapsulation.
///
/// Given the recipient's private key and the ciphertext (ephemeral public key),
/// recover the same shared secret that the sender derived.
///
/// Returns the 32-byte shared secret.
///
/// Hints:
/// - Parse the ciphertext as the ephemeral public key
/// - Perform X25519 DH with recipient's secret
/// - Derive the shared secret using the same HKDF parameters
pub fn kem_decapsulate(recipient_secret: &[u8; 32], ciphertext: &[u8]) -> [u8; 32] {
    todo!("Decapsulate the shared secret using recipient's private key")
}

/// Exercise 4: Verify that encapsulate/decapsulate produces matching secrets.
///
/// Returns `true` if both sides derive the same shared secret.
pub fn verify_kem_correctness() -> bool {
    todo!("Verify KEM correctness: encapsulate and decapsulate agree")
}

/// Exercise 5: Demonstrate that decapsulation with the wrong key fails.
///
/// The wrong private key should produce a different shared secret
/// (not an error — just a wrong value, which will cause authentication failure later).
pub fn wrong_key_wrong_secret() -> bool {
    todo!("Show that wrong key produces wrong shared secret")
}

/// Exercise 6: Use the KEM shared secret to encrypt a message.
///
/// Demonstrate the typical workflow: KEM to get shared secret, then
/// use it as an AES-256-GCM key.
pub fn kem_encrypt_message(
    recipient_public: &[u8; 32],
    plaintext: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    todo!("KEM + AES-GCM: encapsulate key, then encrypt message")
}

/// Derive a 32-byte key from KEM shared secret using HKDF.
fn derive_kem_key(shared_secret: &[u8], ephemeral_public: &[u8]) -> [u8; 32] {
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, ephemeral_public);
    let prk = salt.extract(shared_secret);
    let okm = prk.expand(&[b"kem-shared-secret"], hkdf::HKDF_SHA256).unwrap();
    let mut key = [0u8; 32];
    okm.fill(&mut key).unwrap();
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kem_keygen() {
        let (pub_key, _sec_key) = kem_keygen();
        assert_eq!(pub_key.len(), 32, "Public key should be 32 bytes");
    }

    #[test]
    fn test_kem_roundtrip() {
        assert!(verify_kem_correctness(), "KEM encapsulate/decapsulate should produce matching secrets");
    }

    #[test]
    fn test_kem_shared_secret_is_32_bytes() {
        let (pub_key, _sec_key) = kem_keygen();
        let output = kem_encapsulate(&pub_key);
        assert_eq!(output.shared_secret.len(), 32);
    }

    #[test]
    fn test_kem_ciphertext_is_ephemeral_public_key() {
        let (pub_key, _sec_key) = kem_keygen();
        let output = kem_encapsulate(&pub_key);
        // The ciphertext should be the ephemeral public key (32 bytes)
        assert_eq!(output.ciphertext.len(), 32);
    }

    #[test]
    fn test_wrong_key_fails() {
        assert!(wrong_key_wrong_secret(), "Wrong key should produce wrong shared secret");
    }

    #[test]
    fn test_two_encapsulations_differ() {
        let (pub_key, _) = kem_keygen();
        let out1 = kem_encapsulate(&pub_key);
        let out2 = kem_encapsulate(&pub_key);
        // Different ephemeral keys → different shared secrets
        assert_ne!(out1.shared_secret, out2.shared_secret,
            "Two encapsulations should produce different secrets");
        assert_ne!(out1.ciphertext, out2.ciphertext,
            "Two encapsulations should produce different ciphertexts");
    }

    #[test]
    fn test_kem_encrypt_decrypt() {
        let (pub_key, sec_key) = kem_keygen();
        let plaintext = b"KEM-encrypted message";
        let (ciphertext, kem_ct) = kem_encrypt_message(&pub_key, plaintext);
        // Decrypt: decapsulate key, then decrypt
        let shared = kem_decapsulate(&sec_key, &kem_ct);
        let _ = (shared, ciphertext, plaintext);
        // Full verification would require implementing decryption too
    }
}
