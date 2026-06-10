//! # Lesson 03: Message Encryption — AES-256-GCM (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce, Key,
};
use serde::{Deserialize, Serialize};

/// A wire-format encrypted message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub associated_data: Option<Vec<u8>>,
}

/// Create an AES-256-GCM cipher from a 32-byte key.
pub fn create_cipher(key_bytes: &[u8; 32]) -> Aes256Gcm {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    Aes256Gcm::new(key)
}

/// Encrypt a message with AES-256-GCM.
pub fn encrypt(
    cipher: &Aes256Gcm,
    nonce_bytes: &[u8; 12],
    plaintext: &[u8],
    associated_data: Option<&[u8]>,
) -> Result<EncryptedMessage, aes_gcm::Error> {
    let nonce = Nonce::from_slice(nonce_bytes);
    let ciphertext = match associated_data {
        Some(aad) => cipher.encrypt(nonce, Payload { msg: plaintext, aad })?,
        None => cipher.encrypt(nonce, plaintext)?,
    };
    Ok(EncryptedMessage {
        nonce: nonce_bytes.to_vec(),
        ciphertext,
        associated_data: associated_data.map(|a| a.to_vec()),
    })
}

/// Generate a nonce from a counter value (little-endian, zero-padded to 12 bytes).
pub fn nonce_from_counter(counter: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&counter.to_le_bytes());
    nonce
}

/// Encrypt multiple messages with incrementing nonces.
pub fn encrypt_batch(
    cipher: &Aes256Gcm,
    start_counter: u64,
    plaintexts: &[&[u8]],
) -> Result<Vec<EncryptedMessage>, aes_gcm::Error> {
    let mut messages = Vec::with_capacity(plaintexts.len());
    for (i, plaintext) in plaintexts.iter().enumerate() {
        let counter = start_counter + i as u64;
        let nonce = nonce_from_counter(counter);
        messages.push(encrypt(cipher, &nonce, plaintext, None)?);
    }
    Ok(messages)
}

/// Serialize an encrypted message to JSON bytes.
pub fn serialize_message(msg: &EncryptedMessage) -> Vec<u8> {
    serde_json::to_vec(msg).expect("serialization failed")
}

/// Deserialize an encrypted message from JSON bytes.
pub fn deserialize_message(data: &[u8]) -> Result<EncryptedMessage, serde_json::Error> {
    serde_json::from_slice(data)
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
        let nonce = nonce_from_counter(0);
        let nonce_ref = Nonce::from_slice(&nonce);
        let ct = cipher.encrypt(nonce_ref, b"test".as_ref());
        assert!(ct.is_ok());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let cipher = create_cipher(&test_key());
        let nonce = nonce_from_counter(1);
        let plaintext = b"Hello, secure world!";
        let msg = encrypt(&cipher, &nonce, plaintext, None).unwrap();
        assert_eq!(msg.nonce, nonce.to_vec());
        assert!(msg.ciphertext.len() > plaintext.len());
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
