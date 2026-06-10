//! # Lesson 04: Message Decryption — Auth Tag Verification (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce, Key,
};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// A wire-format encrypted message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub associated_data: Option<Vec<u8>>,
}

/// Decryption result with metadata.
#[derive(Debug)]
pub struct DecryptedMessage {
    pub plaintext: Vec<u8>,
    pub nonce: [u8; 12],
}

/// Create a cipher from key bytes.
pub fn create_cipher(key_bytes: &[u8; 32]) -> Aes256Gcm {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    Aes256Gcm::new(key)
}

/// Decrypt a message with AES-256-GCM, verifying the auth tag.
pub fn decrypt(
    cipher: &Aes256Gcm,
    msg: &EncryptedMessage,
) -> Result<DecryptedMessage, aes_gcm::Error> {
    let nonce = Nonce::from_slice(&msg.nonce);
    let plaintext = match &msg.associated_data {
        Some(aad) => cipher.decrypt(nonce, Payload { msg: &msg.ciphertext, aad })?,
        None => cipher.decrypt(nonce, msg.ciphertext.as_ref())?,
    };
    let mut nonce_arr = [0u8; 12];
    nonce_arr.copy_from_slice(&msg.nonce);
    Ok(DecryptedMessage {
        plaintext,
        nonce: nonce_arr,
    })
}

/// Decrypt with expected associated data check.
pub fn decrypt_with_expected_ad(
    cipher: &Aes256Gcm,
    msg: &EncryptedMessage,
    expected_ad: &[u8],
) -> Result<DecryptedMessage, String> {
    match &msg.associated_data {
        Some(ad) if ad == expected_ad => {
            decrypt(cipher, msg).map_err(|e| format!("Decryption failed: {:?}", e))
        }
        _ => Err("Associated data mismatch".to_string()),
    }
}

/// Decrypt a batch of messages, returning only successful decryptions.
pub fn decrypt_batch(
    cipher: &Aes256Gcm,
    messages: &[EncryptedMessage],
) -> Vec<DecryptedMessage> {
    messages
        .iter()
        .filter_map(|msg| decrypt(cipher, msg).ok())
        .collect()
}

/// Verify message integrity without decrypting.
pub fn verify_integrity(cipher: &Aes256Gcm, msg: &EncryptedMessage) -> bool {
    decrypt(cipher, msg).is_ok()
}

/// Securely wipe a DecryptedMessage.
pub fn wipe_message(mut msg: DecryptedMessage) {
    msg.plaintext.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes_gcm::aead::OsRng;

    fn test_key() -> [u8; 32] {
        [0x42u8; 32]
    }

    fn encrypt_helper(key: &[u8; 32], nonce_bytes: &[u8; 12], plaintext: &[u8], ad: Option<&[u8]>) -> EncryptedMessage {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(nonce_bytes);
        let ct = if let Some(aad) = ad {
            cipher.encrypt(nonce, Payload { msg: plaintext, aad })
        } else {
            cipher.encrypt(nonce, plaintext)
        }.unwrap();
        EncryptedMessage {
            nonce: nonce_bytes.to_vec(),
            ciphertext: ct,
            associated_data: ad.map(|a| a.to_vec()),
        }
    }

    #[test]
    fn test_decrypt_valid() {
        let key = test_key();
        let cipher = create_cipher(&key);
        let msg = encrypt_helper(&key, &[0u8; 12], b"Hello!", None);
        let result = decrypt(&cipher, &msg);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().plaintext, b"Hello!");
    }

    #[test]
    fn test_decrypt_with_aad() {
        let key = test_key();
        let cipher = create_cipher(&key);
        let msg = encrypt_helper(&key, &[1u8; 12], b"Secret", Some(b"sender:alice"));
        let result = decrypt(&cipher, &msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext() {
        let key = test_key();
        let cipher = create_cipher(&key);
        let mut msg = encrypt_helper(&key, &[2u8; 12], b"Original", None);
        msg.ciphertext[0] ^= 0xFF;
        let result = decrypt(&cipher, &msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key = test_key();
        let wrong_key = [0x99u8; 32];
        let cipher_wrong = create_cipher(&wrong_key);
        let msg = encrypt_helper(&key, &[3u8; 12], b"Secret", None);
        let result = decrypt(&cipher_wrong, &msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_expected_ad_mismatch() {
        let key = test_key();
        let cipher = create_cipher(&key);
        let msg = encrypt_helper(&key, &[4u8; 12], b"Msg", Some(b"actual_ad"));
        let result = decrypt_with_expected_ad(&cipher, &msg, b"wrong_ad");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_batch_mixed() {
        let key = test_key();
        let cipher = create_cipher(&key);
        let msg1 = encrypt_helper(&key, &[5u8; 12], b"Good1", None);
        let msg2 = encrypt_helper(&key, &[6u8; 12], b"Good2", None);
        let mut msg3 = encrypt_helper(&key, &[7u8; 12], b"Bad", None);
        msg3.ciphertext[0] ^= 0xFF;

        let results = decrypt_batch(&cipher, &[msg1, msg2, msg3]);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_verify_integrity() {
        let key = test_key();
        let cipher = create_cipher(&key);
        let msg = encrypt_helper(&key, &[8u8; 12], b"Check me", None);
        assert!(verify_integrity(&cipher, &msg));

        let mut tampered = msg.clone();
        tampered.ciphertext[0] ^= 0xFF;
        assert!(!verify_integrity(&cipher, &tampered));
    }

    #[test]
    fn test_wipe_message() {
        let key = test_key();
        let cipher = create_cipher(&key);
        let msg_enc = encrypt_helper(&key, &[9u8; 12], b"sensitive data", None);
        let msg = decrypt(&cipher, &msg_enc).unwrap();
        wipe_message(msg);
    }
}
