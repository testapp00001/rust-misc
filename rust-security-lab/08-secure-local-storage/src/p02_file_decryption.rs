//! # Lesson 02: File Decryption and Authentication Tag Verification
//!
//! ## The Decryption Process
//!
//! Decrypting a file encrypted with AES-256-GCM involves:
//! 1. **Extract the nonce** from the first 12 bytes
//! 2. **Decrypt the ciphertext** using the key and nonce
//! 3. **Verify the authentication tag** — this happens automatically in AES-GCM
//!
//! If the tag verification fails, the decryption returns an error. This means
//! the ciphertext was either tampered with, or the wrong key/nonce was used.
//!
//! ## Authentication Tag Verification
//!
//! The 16-byte authentication tag is computed over:
//! - The ciphertext
//! - Any additional authenticated data (AAD)
//! - The nonce
//!
//! ```text
//! Tag = GHASH(key, nonce, ciphertext, aad)
//! ```
//!
//! Any modification to any of these values causes tag verification to fail.
//!
//! ## Attack Scenario: Padding Oracle (Not Applicable to GCM)
//!
//! AES-CBC mode is vulnerable to padding oracle attacks where an attacker can
//! decrypt ciphertext by observing error messages. AES-GCM is NOT vulnerable
//! to this because it uses CTR mode (no padding) and verifies integrity before
//! decryption.
//!
//! ## Security Properties
//!
//! - **Ciphertext integrity**: Any bit flip in ciphertext is detected
//! - **AAD integrity**: Any bit flip in associated data is detected
//! - **Nonce integrity**: Nonce is included in the tag computation
//! - **Key authentication**: Wrong key produces random-looking plaintext with invalid tag

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
    Aes256Gcm, Key, Nonce,
};

/// Exercise 1: Extract the nonce from encrypted file data.
///
/// The nonce is the first 12 bytes of the encrypted data format:
/// `[nonce (12 bytes)] [ciphertext + tag]`
///
/// # Hints
/// - Check that encrypted_data has at least 12 bytes
/// - Return the first 12 bytes as a slice
pub fn extract_nonce(encrypted_data: &[u8]) -> Option<&[u8]> {
    todo!("Extract the 12-byte nonce from encrypted data")
}

/// Exercise 2: Extract the ciphertext (without nonce) from encrypted file data.
///
/// The ciphertext is everything after the first 12 bytes.
///
/// # Hints
/// - Return a slice starting at byte 12 to the end
pub fn extract_ciphertext(encrypted_data: &[u8]) -> Option<&[u8]> {
    todo!("Extract ciphertext portion (everything after the nonce)")
}

/// Exercise 3: Decrypt a file and verify its authentication tag.
///
/// This function should:
/// 1. Extract the nonce from the first 12 bytes
/// 2. Extract the ciphertext from the remaining bytes
/// 3. Decrypt and verify the authentication tag
///
/// # Hints
/// - Create cipher: `Aes256Gcm::new(key)`
/// - Convert nonce bytes: `Nonce::from_slice(nonce_bytes)`
/// - Decrypt: `cipher.decrypt(nonce, ciphertext)`
/// - The tag verification happens inside `decrypt()` — if it fails, you get an error
pub fn decrypt_and_verify(
    key: &Key<Aes256Gcm>,
    encrypted_data: &[u8],
) -> Result<Vec<u8>, String> {
    todo!("Decrypt file and verify authentication tag")
}

/// Exercise 4: Decrypt with AAD verification.
///
/// When the file was encrypted with AAD, you must provide the same AAD
/// during decryption. If the AAD doesn't match, the tag verification fails.
///
/// # Hints
/// - Use `Payload { msg: ciphertext, aad }` for the decrypt call
/// - The AAD is authenticated but not encrypted — it's verified against the tag
pub fn decrypt_with_aad_verification(
    key: &Key<Aes256Gcm>,
    encrypted_data: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, String> {
    todo!("Decrypt file with AAD verification")
}

/// Exercise 5: Demonstrate that tag verification catches all tampering types.
///
/// This function should:
/// 1. Encrypt plaintext with AAD
/// 2. Demonstrate that tampering with ciphertext fails
/// 3. Demonstrate that tampering with AAD fails
/// 4. Demonstrate that tampering with nonce fails
///
/// Returns a tuple of (ciphertext_tamper_detected, aad_tamper_detected, nonce_tamper_detected)
///
/// # Hints
/// - For each test, modify one byte in the respective section
/// - The nonce is bytes 0..12, the rest is ciphertext
pub fn demonstrate_tag_verification(
    key: &Key<Aes256Gcm>,
    plaintext: &[u8],
    aad: &[u8],
) -> (bool, bool, bool) {
    todo!("Demonstrate tag verification catches all tampering types")
}

/// Exercise 6: Decrypt a file from a hex-encoded string.
///
/// This is useful for reading encrypted files from text-based storage.
///
/// # Hints
/// - Decode hex: `hex::decode(encoded)` returns `Result<Vec<u8>, _>`
/// - Then decrypt the bytes normally
pub fn decrypt_from_hex(
    key: &Key<Aes256Gcm>,
    hex_data: &str,
) -> Result<Vec<u8>, String> {
    todo!("Decode hex string and decrypt")
}

/// Exercise 7: Validate encrypted file format without decrypting.
///
/// Check that the encrypted data has the minimum required structure:
/// - At least 12 bytes (nonce) + 16 bytes (tag) = 28 bytes minimum
/// - This is a quick sanity check before attempting decryption
///
/// # Hints
/// - Check length >= 28 (12 nonce + 16 tag minimum)
/// - This doesn't verify integrity — just the format
pub fn validate_encrypted_format(encrypted_data: &[u8]) -> bool {
    todo!("Validate encrypted file format without decrypting")
}

/// Exercise 8: Encrypt a file and include a version byte for format evolution.
///
/// Format: `[version (1 byte)] [nonce (12 bytes)] [ciphertext + tag]`
///
/// This allows future format changes without breaking existing files.
///
/// # Hints
/// - Prepend version byte `0x01` to the encrypted output
/// - The version byte is NOT authenticated (it's outside the AEAD)
pub fn encrypt_with_version(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> Vec<u8> {
    todo!("Encrypt file with version byte header")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encrypt_for_test(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> Vec<u8> {
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let mut output = nonce.to_vec();
        let ciphertext = cipher.encrypt(&nonce, plaintext).unwrap();
        output.extend_from_slice(&ciphertext);
        output
    }

    fn encrypt_with_aad_for_test(key: &Key<Aes256Gcm>, plaintext: &[u8], aad: &[u8]) -> Vec<u8> {
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let mut output = nonce.to_vec();
        let ciphertext = cipher.encrypt(&nonce, Payload { msg: plaintext, aad }).unwrap();
        output.extend_from_slice(&ciphertext);
        output
    }

    #[test]
    fn test_extract_nonce() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let encrypted = encrypt_for_test(&key, b"test");
        let nonce = extract_nonce(&encrypted).expect("Should extract nonce");
        assert_eq!(nonce.len(), 12, "Nonce should be 12 bytes");
    }

    #[test]
    fn test_extract_nonce_too_short() {
        let short_data = vec![0u8; 10];
        assert!(extract_nonce(&short_data).is_none(), "Too short data should return None");
    }

    #[test]
    fn test_extract_ciphertext() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let encrypted = encrypt_for_test(&key, b"hello");
        let ct = extract_ciphertext(&encrypted).expect("Should extract ciphertext");
        // Ciphertext = plaintext (5) + tag (16) = 21 bytes
        assert_eq!(ct.len(), 21, "Ciphertext should be plaintext + 16 byte tag");
    }

    #[test]
    fn test_decrypt_and_verify_success() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let plaintext = b"authenticated file contents";
        let encrypted = encrypt_for_test(&key, plaintext);

        let result = decrypt_and_verify(&key, &encrypted);
        assert!(result.is_ok(), "Decryption should succeed");
        assert_eq!(result.unwrap(), plaintext);
    }

    #[test]
    fn test_decrypt_and_verify_tampered() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let mut encrypted = encrypt_for_test(&key, b"test data");

        // Tamper with ciphertext (after nonce)
        if encrypted.len() > 15 {
            encrypted[15] ^= 0xff;
        }

        let result = decrypt_and_verify(&key, &encrypted);
        assert!(result.is_err(), "Tampered ciphertext should fail verification");
    }

    #[test]
    fn test_decrypt_with_aad_success() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let plaintext = b"secret file";
        let aad = b"filename: secret.txt";
        let encrypted = encrypt_with_aad_for_test(&key, plaintext, aad);

        let result = decrypt_with_aad_verification(&key, &encrypted, aad);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), plaintext);
    }

    #[test]
    fn test_decrypt_with_wrong_aad() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let encrypted = encrypt_with_aad_for_test(&key, b"data", b"correct aad");

        let result = decrypt_with_aad_verification(&key, &encrypted, b"wrong aad");
        assert!(result.is_err(), "Wrong AAD should fail verification");
    }

    #[test]
    fn test_tag_verification_demonstration() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let (ct_tamper, aad_tamper, nonce_tamper) =
            demonstrate_tag_verification(&key, b"test", b"aad");

        assert!(ct_tamper, "Ciphertext tampering should be detected");
        assert!(aad_tamper, "AAD tampering should be detected");
        assert!(nonce_tamper, "Nonce tampering should be detected");
    }

    #[test]
    fn test_hex_roundtrip() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let plaintext = b"hex test";
        let encrypted = encrypt_for_test(&key, plaintext);
        let hex_str = hex::encode(&encrypted);

        let decrypted = decrypt_from_hex(&key, &hex_str).expect("Hex decrypt should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_validate_format_valid() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let encrypted = encrypt_for_test(&key, b"test");
        assert!(validate_encrypted_format(&encrypted), "Valid format should pass");
    }

    #[test]
    fn test_validate_format_too_short() {
        let short = vec![0u8; 20];
        assert!(!validate_encrypted_format(&short), "Too short should fail validation");
    }

    #[test]
    fn test_version_header_roundtrip() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let plaintext = b"versioned file";

        let encrypted = encrypt_with_version(&key, plaintext);
        // Should start with version byte
        assert_eq!(encrypted[0], 0x01, "Version byte should be 0x01");

        // The rest should be valid encrypted data (nonce + ciphertext + tag)
        let inner = &encrypted[1..];
        let decrypted = decrypt_and_verify(&key, inner).expect("Versioned decrypt should work");
        assert_eq!(decrypted, plaintext);
    }
}
