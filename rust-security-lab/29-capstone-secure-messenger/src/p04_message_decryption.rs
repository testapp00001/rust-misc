//! # Lesson 04: Message Decryption — Auth Tag Verification
//!
//! ## What Happens During Decryption?
//!
//! AES-256-GCM decryption:
//! 1. Verify the authentication tag against the ciphertext + associated data
//! 2. If tag is valid: decrypt the ciphertext to plaintext
//! 3. If tag is invalid: return error (DO NOT return partial plaintext)
//!
//! The auth tag is a 16-byte value appended to the ciphertext by the encryptor.
//! It is computed over: the ciphertext + any associated data.
//!
//! ## Why Auth Tag Verification Matters
//!
//! Without AEAD, an attacker can:
//! - Flip bits in ciphertext to flip bits in plaintext (CTR mode property)
//! - Use a padding oracle to decrypt messages byte-by-byte
//! - Modify routing information in headers while keeping the body valid
//!
//! AEAD prevents all of these by rejecting any modification.
//!
//! ## Attack: Padding Oracle
//!
//! In non-authenticated modes (e.g., CBC), an attacker sends modified ciphertexts
//! and observes whether decryption produces valid padding. Each query leaks one
//! byte of plaintext. **Defense**: Always use AEAD; never decrypt without verifying.
//!
//! ## Attack: Truncation
//!
//! An attacker removes bytes from the end of a message. Without a length field
//! or auth tag, this might go undetected. **Defense**: AEAD tags cover the entire
//! ciphertext; any truncation is detected.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key,
};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// A wire-format encrypted message (same as p03).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub associated_data: Option<Vec<u8>>,
}

/// Decryption result with metadata.
#[derive(Debug)]
pub struct DecryptedMessage {
    /// The recovered plaintext bytes.
    pub plaintext: Vec<u8>,
    /// The nonce that was used (for counter tracking).
    pub nonce: [u8; 12],
}

/// Exercise 1: Create a cipher from key bytes (same as p03).
pub fn create_cipher(key_bytes: &[u8; 32]) -> Aes256Gcm {
    todo!("Create AES-256-GCM cipher from key bytes")
}

/// Exercise 2: Decrypt a message with AES-256-GCM.
///
/// Verifies the auth tag and returns the plaintext if valid.
///
/// Hints:
/// - Create a Nonce from msg.nonce
/// - If associated_data is Some, use `Payload { msg: &msg.ciphertext, aad }`
/// - Otherwise use `cipher.decrypt(nonce, msg.ciphertext.as_ref())`
/// - On auth tag mismatch, aes-gcm returns an Error
pub fn decrypt(
    cipher: &Aes256Gcm,
    msg: &EncryptedMessage,
) -> Result<DecryptedMessage, aes_gcm::Error> {
    todo!("Decrypt a message and verify auth tag")
}

/// Exercise 3: Decrypt with a expected associated data check.
///
/// Decrypts the message and verifies that the associated data matches
/// the expected value. Returns an error if the AD doesn't match or
/// decryption fails.
///
/// Hints:
/// - First check msg.associated_data matches expected_ad
/// - Then decrypt normally
/// - Return error if either check fails
pub fn decrypt_with_expected_ad(
    cipher: &Aes256Gcm,
    msg: &EncryptedMessage,
    expected_ad: &[u8],
) -> Result<DecryptedMessage, String> {
    todo!("Decrypt and verify expected associated data")
}

/// Exercise 4: Decrypt a batch of messages, returning only successful decryptions.
///
/// Silently skips messages that fail to decrypt (e.g., tampered or corrupted).
///
/// Hints:
/// - Iterate over messages
/// - Try to decrypt each one
/// - Collect successful results
pub fn decrypt_batch(
    cipher: &Aes256Gcm,
    messages: &[EncryptedMessage],
) -> Vec<DecryptedMessage> {
    todo!("Decrypt batch, skipping failures")
}

/// Exercise 5: Verify message integrity without decrypting.
///
/// Returns true if the message's auth tag is valid.
/// Useful for checking message integrity before committing to processing.
///
/// Hints:
/// - Attempt to decrypt
/// - Return true if Ok, false if Err
/// - The plaintext is discarded
pub fn verify_integrity(cipher: &Aes256Gcm, msg: &EncryptedMessage) -> bool {
    todo!("Verify auth tag without returning plaintext")
}

/// Exercise 6: Securely wipe a DecryptedMessage.
///
/// Zero out the plaintext bytes after use to prevent memory leaks.
///
/// Hints:
/// - Use `msg.plaintext.zeroize()` (from the zeroize crate)
/// - The Zeroize trait is implemented for Vec<u8>
pub fn wipe_message(mut msg: DecryptedMessage) {
    todo!("Securely zeroize the plaintext")
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
            cipher.encrypt(nonce, aes_gcm::aead::Payload { msg: plaintext, aad })
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
        // Flip a bit in the ciphertext
        msg.ciphertext[0] ^= 0xFF;
        let result = decrypt(&cipher, &msg);
        assert!(result.is_err(), "Tampered ciphertext should fail decryption");
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key = test_key();
        let wrong_key = [0x99u8; 32];
        let cipher_wrong = create_cipher(&wrong_key);
        let msg = encrypt_helper(&key, &[3u8; 12], b"Secret", None);
        let result = decrypt(&cipher_wrong, &msg);
        assert!(result.is_err(), "Wrong key should fail decryption");
    }

    #[test]
    fn test_decrypt_with_expected_ad_mismatch() {
        let key = test_key();
        let cipher = create_cipher(&key);
        let msg = encrypt_helper(&key, &[4u8; 12], b"Msg", Some(b"actual_ad"));
        let result = decrypt_with_expected_ad(&cipher, &msg, b"wrong_ad");
        assert!(result.is_err(), "AD mismatch should fail");
    }

    #[test]
    fn test_decrypt_batch_mixed() {
        let key = test_key();
        let cipher = create_cipher(&key);
        let mut msg1 = encrypt_helper(&key, &[5u8; 12], b"Good1", None);
        let msg2 = encrypt_helper(&key, &[6u8; 12], b"Good2", None);
        let mut msg3 = encrypt_helper(&key, &[7u8; 12], b"Bad", None);
        msg3.ciphertext[0] ^= 0xFF; // tamper

        let results = decrypt_batch(&cipher, &[msg1, msg2, msg3]);
        assert_eq!(results.len(), 2, "Should decrypt 2 valid messages, skip 1 tampered");
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
        let ptr = msg.plaintext.as_ptr();
        let len = msg.plaintext.len();
        wipe_message(msg);
        // After wiping, the memory should be zeroed (best effort check)
        // Note: we can't safely read the Vec after drop, but the function should not panic
    }
}
