//! # Lesson 03: Message Encryption — AES-256-GCM
//!
//! ## What Is AES-256-GCM?
//!
//! AES-256-GCM (Galois/Counter Mode) is an Authenticated Encryption with Associated Data
//! (AEAD) cipher. It provides:
//! - **Confidentiality**: Plaintext is encrypted with AES-256 in CTR mode
//! - **Integrity**: A 128-bit authentication tag (GHASH) detects tampering
//! - **Associated Data**: Additional unencrypted data that is authenticated
//!
//! ## Why AEAD?
//!
//! Without authentication, an attacker can modify ciphertext and potentially
//! learn information about the plaintext (e.g., padding oracle attacks).
//! AEAD prevents this by binding a MAC to the ciphertext.
//!
//! ## Nonce Management
//!
//! GCM requires a unique nonce for each encryption with the same key:
//! - 96 bits (12 bytes) is the standard nonce size
//! - NEVER reuse a nonce with the same key — this breaks both confidentiality and integrity
//! - Common strategy: use a counter that increments per message
//!
//! ## Attack: Nonce Reuse
//!
//! If the same (key, nonce) pair is used twice:
//! 1. An attacker can XOR the two ciphertexts to get XOR of plaintexts
//! 2. The authentication key is leaked, allowing forgery
//!
//! **Defense**: Use a monotonic counter or random 96-bit nonces (birthday bound ~2^32).

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key,
};
use serde::{Deserialize, Serialize};

/// A wire-format encrypted message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// The nonce used for this encryption (12 bytes).
    pub nonce: Vec<u8>,
    /// The ciphertext (includes the 16-byte GCM auth tag appended).
    pub ciphertext: Vec<u8>,
    /// Optional associated data that was authenticated but not encrypted.
    pub associated_data: Option<Vec<u8>>,
}

/// Exercise 1: Create an AES-256-GCM cipher from a 32-byte key.
///
/// Hints:
/// - Use `Key::<Aes256Gcm>::from_slice(&key_bytes)` to create a key
/// - Use `Aes256Gcm::new(&key)` to create the cipher
pub fn create_cipher(key_bytes: &[u8; 32]) -> Aes256Gcm {
    todo!("Create AES-256-GCM cipher from key bytes")
}

/// Exercise 2: Encrypt a message with AES-256-GCM.
///
/// Uses a counter-based nonce (provided as parameter).
/// The nonce must be exactly 12 bytes.
///
/// Hints:
/// - Create a `Nonce` from the nonce bytes: `Nonce::from_slice(nonce_bytes)`
/// - Use `cipher.encrypt(nonce, plaintext)` for no associated data
/// - For associated data, use `cipher.encrypt(nonce, Payload { msg, aad })`
/// - Returns the ciphertext (which includes the 16-byte auth tag at the end)
pub fn encrypt(
    cipher: &Aes256Gcm,
    nonce_bytes: &[u8; 12],
    plaintext: &[u8],
    associated_data: Option<&[u8]>,
) -> Result<EncryptedMessage, aes_gcm::Error> {
    todo!("Encrypt a message with AES-256-GCM")
}

/// Exercise 3: Generate a nonce from a counter value.
///
/// Creates a 12-byte nonce from a u64 counter (little-endian, zero-padded).
///
/// Hints:
/// - Convert counter to little-endian bytes: `counter.to_le_bytes()`
/// - Place them in the first 8 bytes of a 12-byte array
/// - Remaining 4 bytes are zero
pub fn nonce_from_counter(counter: u64) -> [u8; 12] {
    todo!("Generate a 12-byte nonce from a counter")
}

/// Exercise 4: Encrypt multiple messages with incrementing nonces.
///
/// Encrypts each plaintext with the next counter value.
/// Returns a vector of encrypted messages.
///
/// Hints:
/// - Start counter at `start_counter`
/// - For each message, generate nonce from counter, encrypt, increment counter
pub fn encrypt_batch(
    cipher: &Aes256Gcm,
    start_counter: u64,
    plaintexts: &[&[u8]],
) -> Result<Vec<EncryptedMessage>, aes_gcm::Error> {
    todo!("Encrypt a batch of messages with incrementing nonces")
}

/// Exercise 5: Serialize an encrypted message to JSON bytes.
///
/// Hints:
/// - Use `serde_json::to_vec(&msg)`
pub fn serialize_message(msg: &EncryptedMessage) -> Vec<u8> {
    todo!("Serialize EncryptedMessage to JSON bytes")
}

/// Exercise 6: Deserialize an encrypted message from JSON bytes.
///
/// Hints:
/// - Use `serde_json::from_slice::<EncryptedMessage>(&data)`
pub fn deserialize_message(data: &[u8]) -> Result<EncryptedMessage, serde_json::Error> {
    todo!("Deserialize EncryptedMessage from JSON bytes")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [0x42u8; 32]
    }

    #[test]
    fn test_create_cipher() {
        let cipher = create_cipher(&test_key());
        // Verify it can encrypt (basic smoke test)
        let nonce = nonce_from_counter(0);
        let nonce_ref = Nonce::from_slice(&nonce);
        let ct = cipher.encrypt(nonce_ref, b"test".as_ref());
        assert!(ct.is_ok(), "Cipher should be able to encrypt");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let cipher = create_cipher(&test_key());
        let nonce = nonce_from_counter(1);
        let plaintext = b"Hello, secure world!";

        let msg = encrypt(&cipher, &nonce, plaintext, None).unwrap();
        assert_eq!(msg.nonce, nonce.to_vec());
        assert!(msg.ciphertext.len() > plaintext.len(), "Ciphertext should include auth tag");
    }

    #[test]
    fn test_encrypt_with_associated_data() {
        let cipher = create_cipher(&test_key());
        let nonce = nonce_from_counter(2);
        let plaintext = b"Secret message";
        let aad = b"sender:alice";

        let msg = encrypt(&cipher, &nonce, plaintext, Some(aad)).unwrap();
        assert_eq!(msg.associated_data, Some(aad.to_vec()));
    }

    #[test]
    fn test_nonce_from_counter_unique() {
        let n0 = nonce_from_counter(0);
        let n1 = nonce_from_counter(1);
        let n2 = nonce_from_counter(2);
        assert_ne!(n0, n1);
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_nonce_from_counter_le() {
        let n = nonce_from_counter(1);
        // Counter 1 in LE is [1, 0, 0, 0, 0, 0, 0, 0]
        assert_eq!(n[0], 1);
        assert_eq!(n[1], 0);
        assert!(n[8..].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_encrypt_batch() {
        let cipher = create_cipher(&test_key());
        let plaintexts: Vec<&[u8]> = vec![b"msg1", b"msg2", b"msg3"];
        let msgs = encrypt_batch(&cipher, 0, &plaintexts).unwrap();
        assert_eq!(msgs.len(), 3);
        // Each should have a different nonce
        assert_ne!(msgs[0].nonce, msgs[1].nonce);
        assert_ne!(msgs[1].nonce, msgs[2].nonce);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let cipher = create_cipher(&test_key());
        let nonce = nonce_from_counter(5);
        let msg = encrypt(&cipher, &nonce, b"test", Some(b"aad")).unwrap();
        let bytes = serialize_message(&msg);
        let recovered = deserialize_message(&bytes).unwrap();
        assert_eq!(msg.nonce, recovered.nonce);
        assert_eq!(msg.ciphertext, recovered.ciphertext);
        assert_eq!(msg.associated_data, recovered.associated_data);
    }
}
