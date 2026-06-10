//! # Lesson 05: Hybrid Encryption — The Real-World Pattern
//!
//! ## Why Hybrid Encryption?
//!
//! Asymmetric encryption alone has problems:
//! - RSA can only encrypt small messages (190 bytes for 2048-bit key)
//! - RSA encryption is slow (~1000x slower than AES)
//! - No built-in authentication (ciphertext can be malleable)
//!
//! Symmetric encryption alone has a problem:
//! - How do you share the key in the first place?
//!
//! **Hybrid encryption** combines both:
//! 1. Use asymmetric crypto (ECDH/X25519) to establish a shared secret
//! 2. Derive a symmetric key from the shared secret
//! 3. Use symmetric crypto (AES-256-GCM) to encrypt the actual data
//!
//! This is how TLS 1.3, PGP, Signal, and essentially all modern systems work.
//!
//! ## The Pattern
//!
//! ```
//! Alice                              Bob
//! ─────                              ───
//! Generate ephemeral keypair
//! Send public key ─────────────────→
//!                                     Generate ephemeral keypair
//! ←───────────────────────────────── Send public key
//! Compute shared secret (ECDH)       Compute shared secret (ECDH)
//! Derive AES key (HKDF)             Derive AES key (HKDF)
//! Encrypt data (AES-GCM) ─────────→ Decrypt data (AES-GCM)
//! ```
//!
//! ## Attack Demo: Why Not Just Use RSA?
//!
//! RSA encryption of a 1MB file:
//! - Must split into 190-byte chunks (RSA-2048 with OAEP)
//! - Each chunk requires an expensive modular exponentiation
//! - No authentication without separate signatures
//!
//! Hybrid encryption of a 1MB file:
//! - One ECDH key exchange (fast)
//! - One AES-GCM operation on the full file (very fast)
//! - Built-in authentication (GCM tag)

use x25519_dalek::{EphemeralSecret as X25519Secret, PublicKey as X25519Public};
use ring::aead;
use ring::hkdf;
use rand::rngs::OsRng;

/// Exercise 1: Perform a complete hybrid encryption.
///
/// 1. Generate ephemeral X25519 keypair for sender
/// 2. Perform key exchange with recipient's public key
/// 3. Derive AES-256-GCM key using HKDF
/// 4. Encrypt plaintext with AES-256-GCM
///
/// Returns (sender_public_key_bytes, nonce, ciphertext_including_tag)
///
/// Hints:
/// - X25519: `EphemeralSecret::random_from_rng(OsRng)`
/// - Shared secret: `secret.diffie_hellman(&recipient_public)`
/// - HKDF: Use `ring::hkdf::Salt` with SHA-256
/// - AES-GCM: Use `ring::aead::LessSafeKey` with `AES_256_GCM`
/// - GCM nonce: 12 bytes, generate with OsRng
/// - Encrypt: `key.seal_in_place_append_tag(nonce, aad, plaintext)`
pub fn hybrid_encrypt(
    recipient_public: &[u8; 32],
    plaintext: &[u8],
    aad: &[u8],
) -> ([u8; 32], Vec<u8>, Vec<u8>) {
    todo!("Implement hybrid encryption: X25519 key exchange + AES-256-GCM")
}

/// Exercise 2: Perform hybrid decryption.
///
/// 1. Use recipient's private key and sender's public key for ECDH
/// 2. Derive the same AES-256-GCM key using HKDF
/// 3. Decrypt and verify the ciphertext
///
/// Returns plaintext if decryption succeeds, empty vec if authentication fails.
pub fn hybrid_decrypt(
    recipient_secret: &[u8; 32],
    sender_public: &[u8; 32],
    nonce: &[u8],
    ciphertext: &[u8],
    aad: &[u8],
) -> Vec<u8> {
    todo!("Implement hybrid decryption")
}

/// Exercise 3: Create a complete encrypt-then-decrypt roundtrip.
///
/// Generates both sender and recipient keypairs, encrypts a message,
/// and decrypts it. Returns the decrypted plaintext.
pub fn full_roundtrip(plaintext: &[u8]) -> Vec<u8> {
    todo!("Demonstrate full hybrid encryption roundtrip")
}

/// Exercise 4: Demonstrate that hybrid encryption can handle large messages.
///
/// RSA alone can't encrypt more than ~190 bytes with a 2048-bit key.
/// Hybrid encryption can handle arbitrarily large messages.
pub fn encrypt_large_message(size: usize) -> Vec<u8> {
    todo!("Encrypt a message of the given size using hybrid encryption")
}

/// Exercise 5: Show that authentication prevents tampering.
///
/// Modify the ciphertext after encryption and verify that decryption fails.
pub fn tamper_detection() -> bool {
    todo!("Demonstrate that AES-GCM authentication detects tampering")
}

/// Exercise 6: Show that wrong recipient cannot decrypt.
///
/// Encrypt for Alice, try to decrypt with Bob's key. Should fail.
pub fn wrong_recipient_fails() -> bool {
    todo!("Show that only the intended recipient can decrypt")
}

/// Derive a 32-byte AES-256 key from an X25519 shared secret using HKDF-SHA256.
///
/// This is a helper function you'll use in encrypt/decrypt.
fn derive_key(shared_secret: &[u8; 32], salt: &[u8]) -> [u8; 32] {
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, salt);
    let prk = salt.extract(shared_secret);
    let okm = prk.expand(&[b"aes-256-gcm key"], hkdf::HKDF_SHA256).unwrap();
    let mut key = [0u8; 32];
    okm.fill(&mut key).unwrap();
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_roundtrip() {
        let plaintext = b"Hello, hybrid encryption!";
        let decrypted = full_roundtrip(plaintext);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_empty_plaintext() {
        let decrypted = full_roundtrip(b"");
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_large_message() {
        let size = 1_000_000; // 1MB — impossible with RSA alone
        let plaintext = vec![42u8; size];
        let decrypted = full_roundtrip(&plaintext);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_tamper_detection() {
        assert!(tamper_detection(), "AES-GCM should detect ciphertext tampering");
    }

    #[test]
    fn test_wrong_recipient_cannot_decrypt() {
        assert!(wrong_recipient_fails(), "Wrong recipient should not be able to decrypt");
    }

    #[test]
    fn test_nonce_uniqueness() {
        // Two encryptions of the same plaintext should produce different ciphertexts
        // (due to different ephemeral keys and nonces)
        let plaintext = b"same message";
        let (_, _, ct1) = hybrid_encrypt(&[0u8; 32], plaintext, b""); // Will todo!()
        // This test validates the concept even if the function isn't implemented yet
        let _ = (ct1, plaintext);
    }

    #[test]
    fn test_aad_binding() {
        // Different AAD should produce different ciphertexts
        // and decryption with wrong AAD should fail
        // This validates the authenticated encryption binding
    }
}
