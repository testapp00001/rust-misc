//! # Lesson 09: Additional Authenticated Data (AAD)
//!
//! ## What is AAD?
//!
//! AEAD ciphers can authenticate data that is NOT encrypted. This is called
//! **Additional Authenticated Data (AAD)** or **Associated Data**.
//!
//! ```text
//! (ciphertext, tag) = AES-GCM(key, nonce, plaintext, aad)
//! ```
//!
//! The AAD is:
//! - **Authenticated**: Any modification of AAD is detected (tag check fails)
//! - **NOT encrypted**: The AAD is visible to everyone
//! - **NOT in the ciphertext**: It must be provided separately for verification
//!
//! ## When to Use AAD
//!
//! | Use Case | AAD Content |
//! |----------|------------|
//! | Network protocol | Packet headers, routing info |
//! | Database encryption | Row ID, table name, version |
//! | File encryption | Filename, permissions, timestamps |
//! | API request | Method, path, content-type |
//! | TLS record | Sequence number, content type |
//!
//! ## Why AAD Matters
//!
//! Without AAD, an attacker could:
//! 1. Take a valid ciphertext from context A and replay it in context B
//! 2. Swap ciphertexts between different database rows
//! 3. Reorder encrypted network packets
//!
//! AAD binds the ciphertext to its context, preventing these attacks.
//!
//! ## Attack Scenario: Context Confusion
//!
//! Without AAD:
//! ```text
//! Alice encrypts: encrypt(key, nonce, "Pay Bob $100")
//! Mallory replays the ciphertext in a different context
//! Server decrypts: "Pay Bob $100" — looks valid!
//! ```
//!
//! With AAD:
//! ```text
//! Alice encrypts: encrypt(key, nonce, "Pay Bob $100", aad="txn_id=42")
//! Mallory replays with aad="txn_id=99"
//! Server decrypts: ERROR — AAD mismatch!
//! ```

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};

/// Exercise 1: Encrypt with AAD.
///
/// Encrypt plaintext with additional authenticated data. The AAD is authenticated
/// but NOT encrypted — it will be visible in transit.
///
/// Returns (ciphertext, nonce). The AAD must be provided separately during decryption.
///
/// Hints:
/// - Create cipher: `Aes256Gcm::new(&key)`
/// - Use `aes_gcm::aead::Payload` to combine plaintext and AAD:
///   `cipher.encrypt(&nonce, Payload { msg: plaintext, aad: aad })`
/// - Or use `cipher.encrypt(&nonce, plaintext)` for no AAD
pub fn encrypt_with_aad(
    key: &Key<Aes256Gcm>,
    plaintext: &[u8],
    aad: &[u8],
) -> (Vec<u8>, aes_gcm::aead::Nonce<Aes256Gcm>) {
    todo!("Encrypt plaintext with additional authenticated data")
}

/// Exercise 2: Decrypt with AAD verification.
///
/// Decrypt ciphertext and verify that the AAD matches what was used during encryption.
/// If the AAD has been tampered with, decryption fails.
///
/// Hints:
/// - Create cipher: `Aes256Gcm::new(&key)`
/// - Use `Payload` for decryption too:
///   `cipher.decrypt(&nonce, Payload { msg: ciphertext, aad: aad })`
pub fn decrypt_with_aad(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    todo!("Decrypt with AAD verification")
}

/// Exercise 3: Demonstrate AAD tamper detection.
///
/// Encrypt with AAD, then try to decrypt with modified AAD.
/// Returns true if the tampered AAD is detected.
///
/// Hints:
/// - Encrypt with some AAD
/// - Try to decrypt with different AAD
/// - Should return Err
pub fn demonstrate_aad_tamper_detection(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> bool {
    todo!("Show that modified AAD causes decryption to fail")
}

/// Exercise 4: Implement a sealed message with context binding.
///
/// A "sealed message" binds the ciphertext to a specific context (e.g., a user ID,
/// a request path, a database row). The context is used as AAD.
///
/// Format: [nonce: 12 bytes] [ciphertext+tag]
/// The context (AAD) is not stored — it must be provided during decryption.
///
/// Hints:
/// - Generate a fresh nonce
/// - Encrypt with the context as AAD
/// - Prepend nonce to the ciphertext
pub fn seal_with_context(key: &Key<Aes256Gcm>, plaintext: &[u8], context: &[u8]) -> Vec<u8> {
    todo!("Seal a message bound to a specific context")
}

/// Exercise 5: Open a sealed message with context verification.
///
/// Parse and decrypt a message produced by `seal_with_context`.
/// The context must match exactly — any change causes decryption to fail.
///
/// Hints:
/// - Split nonce (first 12 bytes) from ciphertext
/// - Decrypt with the provided context as AAD
pub fn open_with_context(
    key: &Key<Aes256Gcm>,
    sealed: &[u8],
    context: &[u8],
) -> Result<Vec<u8>, &'static str> {
    todo!("Open a sealed message and verify context")
}

/// Exercise 6: Demonstrate replay attack prevention with AAD.
///
/// Show that a ciphertext encrypted for one context cannot be used in another.
///
/// Returns true if the replay is prevented (decryption fails with wrong context).
///
/// Hints:
/// - Seal a message with context "user:alice"
/// - Try to open it with context "user:bob"
/// - Should fail
pub fn demonstrate_replay_prevention(key: &Key<Aes256Gcm>) -> bool {
    todo!("Show that AAD prevents context replay attacks")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Key<Aes256Gcm> {
        Aes256Gcm::generate_key(&mut OsRng)
    }

    #[test]
    fn test_encrypt_decrypt_with_aad() {
        let key = test_key();
        let plaintext = b"secret message";
        let aad = b"context: user=alice, txn=42";

        let (ciphertext, nonce) = encrypt_with_aad(&key, plaintext, aad);
        let decrypted = decrypt_with_aad(&key, &nonce, &ciphertext, aad).expect("Decryption failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_aad_fails() {
        let key = test_key();
        let plaintext = b"secret message";
        let aad = b"correct context";

        let (ciphertext, nonce) = encrypt_with_aad(&key, plaintext, aad);
        let wrong_aad = b"wrong context";
        let result = decrypt_with_aad(&key, &nonce, &ciphertext, wrong_aad);
        assert!(result.is_err(), "Wrong AAD must fail decryption");
    }

    #[test]
    fn test_no_aad_works() {
        let key = test_key();
        let plaintext = b"no aad message";
        let aad = b"";

        let (ciphertext, nonce) = encrypt_with_aad(&key, plaintext, aad);
        let decrypted = decrypt_with_aad(&key, &nonce, &ciphertext, aad).expect("Failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aad_tamper_detection() {
        let key = test_key();
        let detected = demonstrate_aad_tamper_detection(&key, b"AAD protects context");
        assert!(detected, "Modified AAD must be detected");
    }

    #[test]
    fn test_seal_open_with_context() {
        let key = test_key();
        let plaintext = b"context-bound data";
        let context = b"row_id=12345";

        let sealed = seal_with_context(&key, plaintext, context);
        let opened = open_with_context(&key, &sealed, context).expect("Open failed");
        assert_eq!(opened, plaintext);
    }

    #[test]
    fn test_seal_open_wrong_context() {
        let key = test_key();
        let plaintext = b"context-bound data";
        let context = b"row_id=12345";

        let sealed = seal_with_context(&key, plaintext, context);
        let result = open_with_context(&key, &sealed, b"row_id=99999");
        assert!(result.is_err(), "Wrong context must fail");
    }

    #[test]
    fn test_replay_prevention() {
        let key = test_key();
        let prevented = demonstrate_replay_prevention(&key);
        assert!(prevented, "AAD should prevent replay across contexts");
    }

    #[test]
    fn test_aad_visible_in_transit() {
        let key = test_key();
        let plaintext = b"encrypted content";
        let aad = b"public metadata";

        let (ciphertext, nonce) = encrypt_with_aad(&key, plaintext, aad);

        // AAD is not in the ciphertext — it's provided externally
        // The ciphertext length is plaintext_len + 16 (tag), no AAD included
        assert_eq!(ciphertext.len(), plaintext.len() + 16);
        // Decrypt still works when AAD is provided
        let decrypted = decrypt_with_aad(&key, &nonce, &ciphertext, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
