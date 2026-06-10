//! # Lesson 01: AES-256-GCM Basics (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};

/// Generate a random AES-256 key (32 bytes).
///
/// `KeyInit::generate_key` uses a cryptographically secure RNG (OsRng)
/// to produce a key with 256 bits of entropy.
pub fn generate_key() -> Key<Aes256Gcm> {
    Aes256Gcm::generate_key(&mut OsRng)
}

/// Generate a random 96-bit nonce (12 bytes).
///
/// Random nonces are safe when the number of encryptions per key is small.
/// For AES-GCM, a birthday collision at 2^32 messages has probability ~2^-33,
/// which is acceptably low for most applications.
pub fn generate_nonce() -> aes_gcm::aead::Nonce<Aes256Gcm> {
    Aes256Gcm::generate_nonce(&mut OsRng)
}

/// Encrypt plaintext using AES-256-GCM.
///
/// Returns (ciphertext_with_tag, nonce). The 16-byte authentication tag is
/// appended to the ciphertext by the `aes-gcm` crate.
///
/// Security notes:
/// - A fresh random nonce is generated for each encryption
/// - The nonce MUST be stored/transmitted with the ciphertext
/// - NEVER reuse a (key, nonce) pair — see p08_nonce_reuse_attack
pub fn encrypt(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> (Vec<u8>, aes_gcm::aead::Nonce<Aes256Gcm>) {
    let cipher = Aes256Gcm::new(key);
    let nonce = generate_nonce();
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .expect("Encryption should not fail");
    (ciphertext, nonce)
}

/// Decrypt ciphertext using AES-256-GCM.
///
/// Returns plaintext if the authentication tag is valid.
/// Returns an error if the tag doesn't match (data was tampered with).
///
/// This is the power of AEAD: you get an explicit error on any tampering,
/// rather than silently returning garbage plaintext.
pub fn decrypt(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    ciphertext: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new(key);
    cipher.decrypt(nonce, ciphertext)
}

/// Demonstrate that GCM detects tampering.
///
/// Any modification to the ciphertext — even a single bit — causes the
/// authentication tag check to fail, and decryption returns an error.
pub fn demonstrate_tamper_detection(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> bool {
    let (mut ciphertext, nonce) = encrypt(key, plaintext);

    // Tamper with one byte
    if !ciphertext.is_empty() {
        ciphertext[0] ^= 0xff;
    }

    // Decryption must fail
    decrypt(key, &nonce, &ciphertext).is_err()
}

/// Encrypt with a password-derived key.
///
/// WARNING: In production, use Argon2id for password-based key derivation.
/// SHA-256 hashing provides no defense against brute-force or dictionary attacks.
/// This function is for educational purposes only.
pub fn encrypt_with_password(password: &str, plaintext: &[u8]) -> (Vec<u8>, aes_gcm::aead::Nonce<Aes256Gcm>) {
    let hash = ring::digest::digest(&ring::digest::SHA256, password.as_bytes());
    let key = Key::<Aes256Gcm>::clone_from_slice(hash.as_ref());
    encrypt(&key, plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = generate_key();
        assert_eq!(key.len(), 32, "AES-256 key must be 32 bytes");
    }

    #[test]
    fn test_nonce_generation() {
        let nonce = generate_nonce();
        assert_eq!(nonce.len(), 12, "GCM nonce must be 12 bytes");
    }

    #[test]
    fn test_nonce_uniqueness() {
        let n1 = generate_nonce();
        let n2 = generate_nonce();
        assert_ne!(n1.as_slice(), n2.as_slice(), "Random nonces must be unique");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_key();
        let plaintext = b"Hello, AES-GCM! This is a secret message.";
        let (ciphertext, nonce) = encrypt(&key, plaintext);

        assert_ne!(ciphertext, plaintext, "Ciphertext must differ from plaintext");
        assert_eq!(ciphertext.len(), plaintext.len() + 16, "GCM adds 16-byte auth tag");

        let decrypted = decrypt(&key, &nonce, &ciphertext).expect("Decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_empty_plaintext() {
        let key = generate_key();
        let (ciphertext, nonce) = encrypt(&key, b"");
        assert_eq!(ciphertext.len(), 16, "Empty plaintext produces 16-byte tag only");
        let decrypted = decrypt(&key, &nonce, &ciphertext).expect("Should decrypt empty");
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_key();
        let key2 = generate_key();
        let (ciphertext, nonce) = encrypt(&key1, b"secret");
        let result = decrypt(&key2, &nonce, &ciphertext);
        assert!(result.is_err(), "Wrong key must fail decryption");
    }

    #[test]
    fn test_wrong_nonce_fails() {
        let key = generate_key();
        let (ciphertext, _nonce) = encrypt(&key, b"secret");
        let wrong_nonce = generate_nonce();
        let result = decrypt(&key, &wrong_nonce, &ciphertext);
        assert!(result.is_err(), "Wrong nonce must fail decryption");
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = generate_key();
        let plaintext = b"authenticated data";
        let tampered = demonstrate_tamper_detection(&key, plaintext);
        assert!(tampered, "Tampered ciphertext must be detected");
    }
}
