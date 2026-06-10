//! # Lesson 05: Hybrid Encryption — The Real-World Pattern (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.
//!
//! This implements the same pattern used by TLS 1.3, Signal, WireGuard, etc.:
//! 1. ECDH key exchange (X25519) to establish shared secret
//! 2. HKDF to derive symmetric key
//! 3. AES-256-GCM to encrypt data with authentication

use x25519_dalek::{EphemeralSecret, PublicKey};
use ring::aead;
use ring::hkdf;
use rand::rngs::OsRng;
use rand::RngCore;

/// Perform hybrid encryption.
///
/// 1. Generate ephemeral X25519 keypair
/// 2. ECDH with recipient's public key -> shared secret
/// 3. HKDF-SHA256 -> AES-256-GCM key
/// 4. AES-256-GCM encrypt with AAD
///
/// Returns: (sender_public_key, nonce, ciphertext_with_tag)
pub fn hybrid_encrypt(
    recipient_public: &[u8; 32],
    plaintext: &[u8],
    aad: &[u8],
) -> ([u8; 32], Vec<u8>, Vec<u8>) {
    // 1. Generate ephemeral keypair
    let sender_secret = EphemeralSecret::random_from_rng(OsRng);
    let sender_public = PublicKey::from(&sender_secret);

    // 2. ECDH key exchange
    let recipient_pub = PublicKey::from(*recipient_public);
    let shared_secret = sender_secret.diffie_hellman(&recipient_pub);

    // 3. Derive AES-256-GCM key
    let key_bytes = derive_key(shared_secret.as_bytes(), b"hybrid-encryption");

    // 4. Encrypt with AES-256-GCM
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
        .expect("invalid key");
    let key = aead::LessSafeKey::new(unbound_key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, aead::Aad::from(aad), &mut in_out)
        .expect("encryption failed");

    (*sender_public.as_bytes(), nonce_bytes.to_vec(), in_out)
}

/// Perform hybrid decryption.
///
/// 1. ECDH with sender's public key -> same shared secret
/// 2. HKDF-SHA256 -> same AES-256-GCM key
/// 3. AES-256-GCM decrypt and verify
///
/// Returns plaintext if successful, empty vec if authentication fails.
pub fn hybrid_decrypt(
    recipient_secret: &[u8; 32],
    sender_public: &[u8; 32],
    nonce: &[u8],
    ciphertext: &[u8],
    aad: &[u8],
) -> Vec<u8> {
    // 1. ECDH key exchange
    let recipient_sec = x25519_dalek::StaticSecret::from(*recipient_secret);
    let sender_pub = PublicKey::from(*sender_public);
    let shared_secret = recipient_sec.diffie_hellman(&sender_pub);

    // 2. Derive same AES-256-GCM key
    let key_bytes = derive_key(shared_secret.as_bytes(), b"hybrid-encryption");

    // 3. Decrypt
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
        .expect("invalid key");
    let key = aead::LessSafeKey::new(unbound_key);

    let nonce = aead::Nonce::assume_unique_for_key(
        nonce.try_into().expect("nonce must be 12 bytes")
    );

    let mut in_out = ciphertext.to_vec();
    match key.open_in_place(nonce, aead::Aad::from(aad), &mut in_out) {
        Ok(plaintext) => plaintext.to_vec(),
        Err(_) => vec![], // Authentication failed
    }
}

/// Full encrypt-then-decrypt roundtrip.
pub fn full_roundtrip(plaintext: &[u8]) -> Vec<u8> {
    // Generate recipient keypair using StaticSecret (so we can extract bytes)
    let recipient_secret = x25519_dalek::StaticSecret::random_from_rng(OsRng);
    let recipient_public = PublicKey::from(&recipient_secret);

    // Encrypt
    let (sender_pub, nonce, ciphertext) = hybrid_encrypt(
        recipient_public.as_bytes(),
        plaintext,
        b"additional-data",
    );

    // Decrypt
    hybrid_decrypt(
        &recipient_secret.to_bytes(),
        &sender_pub,
        &nonce,
        &ciphertext,
        b"additional-data",
    )
}

/// Encrypt a large message (demonstrating hybrid encryption advantage).
///
/// RSA-2048 can only encrypt 190 bytes. AES-256-GCM can encrypt up to ~64GB.
pub fn encrypt_large_message(size: usize) -> Vec<u8> {
    let plaintext = vec![42u8; size];
    full_roundtrip(&plaintext)
}

/// Demonstrate AES-GCM tamper detection.
///
/// AES-GCM is an AEAD cipher: it provides both confidentiality AND integrity.
/// Any modification to the ciphertext (including the authentication tag)
/// causes decryption to fail.
pub fn tamper_detection() -> bool {
    let recipient_secret = x25519_dalek::StaticSecret::random_from_rng(OsRng);
    let recipient_public = PublicKey::from(&recipient_secret);

    let plaintext = b"important message";
    let (sender_pub, nonce, mut ciphertext) = hybrid_encrypt(
        recipient_public.as_bytes(),
        plaintext,
        b"aad",
    );

    // Tamper with ciphertext
    if !ciphertext.is_empty() {
        ciphertext[0] ^= 0x01;
    }

    // Decryption should fail
    let decrypted = hybrid_decrypt(
        &recipient_secret.to_bytes(),
        &sender_pub,
        &nonce,
        &ciphertext,
        b"aad",
    );

    decrypted.is_empty() // Empty = authentication failed
}

/// Demonstrate that wrong recipient cannot decrypt.
pub fn wrong_recipient_fails() -> bool {
    let alice_secret = x25519_dalek::StaticSecret::random_from_rng(OsRng);
    let alice_public = PublicKey::from(&alice_secret);

    let bob_secret = x25519_dalek::StaticSecret::random_from_rng(OsRng);

    // Encrypt FOR Alice
    let (sender_pub, nonce, ciphertext) = hybrid_encrypt(
        alice_public.as_bytes(),
        b"secret for alice",
        b"",
    );

    // Try to decrypt with Bob's key
    let decrypted = hybrid_decrypt(
        &bob_secret.to_bytes(),
        &sender_pub,
        &nonce,
        &ciphertext,
        b"",
    );

    decrypted.is_empty() // Should fail
}

/// Derive a 32-byte AES-256 key from an X25519 shared secret using HKDF.
pub fn derive_key(shared_secret: &[u8; 32], context: &[u8]) -> [u8; 32] {
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, b"hybrid-encryption-salt");
    let prk = salt.extract(shared_secret);
    let info = [context];
    let okm = prk.expand(&info, hkdf::HKDF_SHA256).unwrap();
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
        let size = 1_000_000; // 1MB
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
        let recipient_secret = x25519_dalek::StaticSecret::random_from_rng(OsRng);
        let recipient_public = PublicKey::from(&recipient_secret);

        let plaintext = b"same message";
        let (_, _, ct1) = hybrid_encrypt(recipient_public.as_bytes(), plaintext, b"");
        let (_, _, ct2) = hybrid_encrypt(recipient_public.as_bytes(), plaintext, b"");

        // Different ephemeral keys and nonces -> different ciphertexts
        assert_ne!(ct1, ct2, "Two encryptions should produce different ciphertexts");
    }

    #[test]
    fn test_aad_binding() {
        let recipient_secret = x25519_dalek::StaticSecret::random_from_rng(OsRng);
        let recipient_public = PublicKey::from(&recipient_secret);

        let plaintext = b"message";

        // Encrypt with AAD1
        let (sender1, nonce1, ct1) = hybrid_encrypt(
            recipient_public.as_bytes(), plaintext, b"context-1",
        );

        // Decrypt with wrong AAD should fail
        let decrypted = hybrid_decrypt(
            &recipient_secret.to_bytes(), &sender1, &nonce1, &ct1, b"context-2",
        );

        assert!(decrypted.is_empty(), "Wrong AAD should cause decryption to fail");
    }
}
