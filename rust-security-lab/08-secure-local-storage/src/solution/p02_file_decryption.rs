//! # Lesson 02: File Decryption and Authentication Tag Verification (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
    Aes256Gcm, Key, Nonce,
};

/// Extract the nonce from encrypted file data.
///
/// The nonce is always the first 12 bytes in our format.
/// Returns `None` if the data is too short to contain a nonce.
pub fn extract_nonce(encrypted_data: &[u8]) -> Option<&[u8]> {
    if encrypted_data.len() < 12 {
        return None;
    }
    Some(&encrypted_data[..12])
}

/// Extract the ciphertext (without nonce) from encrypted file data.
///
/// Returns everything after the first 12 bytes.
pub fn extract_ciphertext(encrypted_data: &[u8]) -> Option<&[u8]> {
    if encrypted_data.len() < 12 {
        return None;
    }
    Some(&encrypted_data[12..])
}

/// Decrypt a file and verify its authentication tag.
///
/// This is a convenience wrapper that extracts the nonce, creates the cipher,
/// and decrypts in one step. The authentication tag is verified inside
/// `cipher.decrypt()` — if it fails, the data was tampered with.
pub fn decrypt_and_verify(
    key: &Key<Aes256Gcm>,
    encrypted_data: &[u8],
) -> Result<Vec<u8>, String> {
    let nonce_bytes = extract_nonce(encrypted_data)
        .ok_or("Data too short to contain nonce")?;
    let ciphertext = extract_ciphertext(encrypted_data)
        .ok_or("Data too short to contain ciphertext")?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(key);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption/verification failed: {:?}", e))
}

/// Decrypt with AAD verification.
///
/// The AAD is included in the authentication tag computation. If the AAD
/// provided during decryption differs from the AAD used during encryption,
/// the tag verification fails.
pub fn decrypt_with_aad_verification(
    key: &Key<Aes256Gcm>,
    encrypted_data: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, String> {
    let nonce_bytes = extract_nonce(encrypted_data)
        .ok_or("Data too short")?;
    let ciphertext = extract_ciphertext(encrypted_data)
        .ok_or("Data too short")?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(key);

    cipher
        .decrypt(nonce, Payload { msg: ciphertext, aad })
        .map_err(|e| format!("Decryption with AAD failed: {:?}", e))
}

/// Demonstrate that tag verification catches all tampering types.
///
/// Tests three scenarios:
/// 1. Ciphertext modification — detected by tag
/// 2. AAD modification — detected by tag (AAD is included in tag computation)
/// 3. Nonce modification — detected by tag (nonce is included in tag computation)
pub fn demonstrate_tag_verification(
    key: &Key<Aes256Gcm>,
    plaintext: &[u8],
    aad: &[u8],
) -> (bool, bool, bool) {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, Payload { msg: plaintext, aad }).unwrap();

    // Build the full encrypted data: [nonce][ciphertext]
    let mut encrypted = nonce.to_vec();
    encrypted.extend_from_slice(&ciphertext);

    // Test 1: Tamper with ciphertext
    let mut ct_tampered = encrypted.clone();
    if ct_tampered.len() > 15 {
        ct_tampered[15] ^= 0xff;
    }
    let ct_detected = decrypt_and_verify(key, &ct_tampered).is_err();

    // Test 2: Tamper with AAD
    let wrong_aad = b"tampered aad data";
    let aad_detected = decrypt_with_aad_verification(key, &encrypted, wrong_aad).is_err();

    // Test 3: Tamper with nonce
    let mut nonce_tampered = encrypted.clone();
    nonce_tampered[5] ^= 0xff;
    let nonce_detected = decrypt_and_verify(key, &nonce_tampered).is_err();

    (ct_detected, aad_detected, nonce_detected)
}

/// Decrypt a file from a hex-encoded string.
///
/// Hex encoding is useful for displaying encrypted data in logs,
/// config files, or anywhere binary data needs to be represented as text.
pub fn decrypt_from_hex(
    key: &Key<Aes256Gcm>,
    hex_data: &str,
) -> Result<Vec<u8>, String> {
    let encrypted = hex::decode(hex_data)
        .map_err(|e| format!("Hex decode failed: {}", e))?;
    decrypt_and_verify(key, &encrypted)
}

/// Validate encrypted file format without decrypting.
///
/// Checks the structural validity of the encrypted data:
/// - Minimum size: 12 (nonce) + 16 (auth tag) = 28 bytes
/// - This is a quick sanity check before attempting decryption
///
/// Note: This does NOT verify integrity — only format validity.
pub fn validate_encrypted_format(encrypted_data: &[u8]) -> bool {
    // Minimum: 12 bytes nonce + 16 bytes auth tag
    encrypted_data.len() >= 28
}

/// Encrypt a file with a version byte header.
///
/// Format: `[version (1)] [nonce (12)] [ciphertext + tag]`
///
/// The version byte allows future format evolution. It's prepended
/// before the nonce, so it's outside the AEAD scope (not authenticated).
pub fn encrypt_with_version(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).expect("Encryption failed");

    let mut output = Vec::with_capacity(1 + 12 + ciphertext.len());
    output.push(0x01); // Version byte
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    output
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
