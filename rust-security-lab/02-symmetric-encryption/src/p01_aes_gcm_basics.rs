//! # Lesson 01: AES-256-GCM Basics
//!
//! ## What is AES-GCM?
//!
//! AES-GCM (Galois/Counter Mode) is an **Authenticated Encryption with Associated Data (AEAD)**
//! cipher. It simultaneously provides:
//! - **Confidentiality**: Plaintext is encrypted — unreadable without the key
//! - **Integrity**: Any tampering with ciphertext is detected
//! - **Authenticity**: The ciphertext is verified to come from someone with the key
//!
//! GCM combines CTR mode encryption with a GHASH authentication tag.
//!
//! ## Why AEAD Matters
//!
//! Traditional encryption (AES-CBC) provides confidentiality ONLY. An attacker can modify
//! ciphertext and the decryption will produce garbage plaintext — but you won't know it's
//! garbage. AEAD solves this by producing a **tag** (MAC) alongside the ciphertext.
//!
//! ```text
//! Encryption:  (ciphertext, tag) = AES-GCM(key, nonce, plaintext, aad)
//! Decryption:  plaintext = AES-GCM-Open(key, nonce, ciphertext, aad, tag)
//!              → returns ERROR if tag doesn't match (tampered!)
//! ```
//!
//! ## Attack Scenario: Without Authentication
//!
//! In CBC mode without MAC:
//! 1. Attacker flips bits in ciphertext block N
//! 2. Block N decrypts to garbage, but block N-1 has controlled bit flips
//! 3. Attacker can manipulate plaintext of the previous block
//!
//! With GCM, ANY modification to ciphertext or AAD causes decryption to fail entirely.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};

/// Exercise 1: Generate a random AES-256 key.
///
/// AES-256 requires a 32-byte (256-bit) key.
///
/// Hints:
/// - Use `Aes2566Gcm::generate_key(&mut OsRng)` — returns `Key<Aes256Gcm>` (32 bytes)
/// - Alternatively, use `rand::Rng` to fill a 32-byte array
pub fn generate_key() -> Key<Aes256Gcm> {
    todo!("Generate a random AES-256 key")
}

/// Exercise 2: Generate a random 96-bit nonce.
///
/// A nonce for AES-GCM is 12 bytes (96 bits). It must NEVER repeat with the same key.
///
/// Hints:
/// - Use `Aes256Gcm::generate_nonce(&mut OsRng)`
/// - Returns `Nonce<Aes256Gcm>` which is 12 bytes
pub fn generate_nonce() -> aes_gcm::aead::Nonce<Aes256Gcm> {
    todo!("Generate a random 96-bit nonce")
}

/// Exercise 3: Encrypt plaintext using AES-256-GCM.
///
/// Returns (ciphertext, nonce) — you must send the nonce along with the ciphertext
/// so the recipient can decrypt.
///
/// Hints:
/// - Create cipher: `Aes2566Gcm::new(&key)`
/// - Generate a fresh nonce
/// - Encrypt: `cipher.encrypt(&nonce, plaintext)`
/// - Returns `Result<Vec<u8>, Error>` — unwrap for now (we'll handle errors later)
pub fn encrypt(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> (Vec<u8>, aes_gcm::aead::Nonce<Aes256Gcm>) {
    todo!("Encrypt plaintext with AES-256-GCM")
}

/// Exercise 4: Decrypt ciphertext using AES-256-GCM.
///
/// Returns plaintext if the tag is valid, or an error if tampered.
///
/// Hints:
/// - Create cipher: `Aes2566Gcm::new(&key)`
/// - Decrypt: `cipher.decrypt(&nonce, ciphertext)`
/// - Returns `Result<Vec<u8>, Error>` — propagate the error with `?`
pub fn decrypt(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    ciphertext: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    todo!("Decrypt ciphertext with AES-256-GCM")
}

/// Exercise 5: Demonstrate that GCM detects tampering.
///
/// Modify one byte of the ciphertext and attempt decryption. It should fail.
///
/// Hints:
/// - Encrypt a message
/// - Flip a bit: `ciphertext[0] ^= 0xff`
/// - Attempt decryption — should return Err
pub fn demonstrate_tamper_detection(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> bool {
    todo!("Encrypt, tamper with ciphertext, verify decryption fails")
}

/// Exercise 6: Encrypt with a password-derived key (for educational purposes).
///
/// In production, use Argon2id for password-based key derivation (Module 07).
/// Here we use a simple SHA-256 hash of the password as a teaching exercise.
///
/// Hints:
/// - Hash the password with SHA-256 to get 32 bytes
/// - Create `Key<Aes2566Gcm>` from the hash
/// - Encrypt the plaintext
pub fn encrypt_with_password(password: &str, plaintext: &[u8]) -> (Vec<u8>, aes_gcm::aead::Nonce<Aes256Gcm>) {
    todo!("Derive key from password and encrypt")
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
        // GCM produces a 16-byte tag even for empty plaintext
        assert_eq!(ciphertext.len(), 16, "Empty plaintext → 16-byte tag only");
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
