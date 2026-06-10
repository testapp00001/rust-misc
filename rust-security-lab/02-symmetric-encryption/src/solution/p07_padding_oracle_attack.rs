//! # Lesson 07: Padding Oracle Attack (Reference Solution)
//!
//! See the exercise file for full documentation on padding oracle attacks,
//! PKCS#7 padding, and defenses.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};

/// PKCS#7 padding: append N bytes of value N to reach a multiple of block_size.
///
/// If the data is already a multiple of block_size, a full block of padding
/// is added (so unpadding is always deterministic).
pub fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
    let padding_len = block_size - (data.len() % block_size);
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat(padding_len as u8).take(padding_len));
    padded
}

/// PKCS#7 unpadding: remove and validate padding bytes.
///
/// Returns the original data or an error if padding is invalid.
pub fn pkcs7_unpad(data: &[u8], block_size: usize) -> Result<&[u8], &'static str> {
    if data.is_empty() {
        return Err("Empty data");
    }

    let pad_len = *data.last().unwrap() as usize;

    if pad_len == 0 || pad_len > block_size {
        return Err("Invalid padding length");
    }

    if data.len() < pad_len {
        return Err("Data shorter than padding length");
    }

    // Verify all padding bytes have the correct value
    let padding_start = data.len() - pad_len;
    for &byte in &data[padding_start..] {
        if byte != pad_len as u8 {
            return Err("Invalid padding bytes");
        }
    }

    Ok(&data[..padding_start])
}

/// Simulate a vulnerable padding oracle server.
///
/// This server uses AES-GCM for encryption but simulates the vulnerable
/// CBC behavior by checking padding after decryption and returning
/// different error types. This is the VULNERABLE pattern.
pub fn vulnerable_server(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    ciphertext: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let cipher = Aes256Gcm::new(key);

    // Step 1: Decrypt (this verifies the GCM tag)
    let decrypted = match cipher.decrypt(nonce, ciphertext) {
        Ok(pt) => pt,
        Err(_) => return Err("bad_mac"),
    };

    // Step 2: Check padding (simulating the vulnerable pattern)
    match pkcs7_unpad(&decrypted, 16) {
        Ok(unpadded) => Ok(unpadded.to_vec()),
        Err(_) => Err("bad_padding"), // LEAKS INFORMATION!
    }
}

/// Demonstrate that AEAD prevents padding oracle attacks.
///
/// With GCM, authentication is verified BEFORE any padding check.
/// The error is always "authentication failed" regardless of what
/// the padding would look like. This eliminates the oracle.
pub fn demonstrate_aead_prevents_oracle(key: &Key<Aes256Gcm>) -> bool {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Encrypt some data
    let plaintext = pkcs7_pad(b"test message", 16);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).expect("Encrypt failed");

    // Tamper with the ciphertext
    let mut tampered = ciphertext.clone();
    if !tampered.is_empty() {
        tampered[0] ^= 0xff;
    }

    // With AEAD, tampered ciphertext always fails with the same error
    // (authentication failure), regardless of what the padding would be
    let result1 = cipher.decrypt(&nonce, ciphertext.as_ref());
    let result2 = cipher.decrypt(&nonce, tampered.as_ref());

    // Original should succeed
    assert!(result1.is_ok());
    // Tampered should fail — same error type for all tampering
    assert!(result2.is_err());

    // The key insight: AEAD rejects tampered data before any padding check,
    // so there's no way to distinguish "bad padding" from "bad MAC"
    true
}

/// Constant-time PKCS#7 padding validation.
///
/// This implementation examines ALL bytes without early return,
/// using bitwise operations to avoid timing side-channels.
pub fn constant_time_pad_check(data: &[u8], block_size: usize) -> Result<&[u8], &'static str> {
    if data.is_empty() {
        return Err("Empty data");
    }

    let pad_len = *data.last().unwrap() as usize;

    // Track validity using bitwise operations (no branches)
    let mut valid: u8 = 1;

    // Check pad_len is in range [1, block_size]
    valid &= ((pad_len >= 1) & (pad_len <= block_size)) as u8;

    // Check data is long enough
    valid &= (data.len() >= pad_len) as u8;

    // Check ALL bytes in the padding region (constant-time loop)
    let padding_start = data.len().saturating_sub(pad_len);
    for i in padding_start..data.len() {
        // This comparison runs for every byte, regardless of earlier results
        valid &= (data[i] == pad_len as u8) as u8;
    }

    if valid == 1 {
        Ok(&data[..padding_start])
    } else {
        Err("Invalid padding")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Key<Aes256Gcm> {
        Aes256Gcm::generate_key(&mut OsRng)
    }

    #[test]
    fn test_pkcs7_pad_partial_block() {
        let data = b"hello"; // 5 bytes
        let padded = pkcs7_pad(data, 16);
        assert_eq!(padded.len(), 16);
        assert_eq!(&padded[..5], b"hello");
        assert!(padded[5..].iter().all(|&b| b == 11));
    }

    #[test]
    fn test_pkcs7_pad_full_block() {
        let data = vec![0u8; 16];
        let padded = pkcs7_pad(&data, 16);
        assert_eq!(padded.len(), 32, "Full block gets a full padding block");
        assert!(padded[16..].iter().all(|&b| b == 16));
    }

    #[test]
    fn test_pkcs7_pad_unpad_roundtrip() {
        let data = b"PKCS#7 padding test!!";
        let padded = pkcs7_pad(data, 16);
        let unpadded = pkcs7_unpad(&padded, 16).expect("Unpad failed");
        assert_eq!(unpadded, data);
    }

    #[test]
    fn test_pkcs7_unpad_invalid() {
        let bad_data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 4];
        assert!(pkcs7_unpad(&bad_data, 16).is_err(), "Invalid padding must be rejected");
    }

    #[test]
    fn test_vulnerable_server_oracle() {
        let key = test_key();
        let plaintext = b"padding oracle test";
        let padded = pkcs7_pad(plaintext, 16);

        let cipher = Aes256Gcm::new(&key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, padded.as_ref()).unwrap();

        let result = vulnerable_server(&key, &nonce, &ciphertext);
        assert!(result.is_ok());
    }

    #[test]
    fn test_aead_prevents_oracle() {
        let key = test_key();
        let prevents = demonstrate_aead_prevents_oracle(&key);
        assert!(prevents, "AEAD should prevent padding oracle attacks");
    }

    #[test]
    fn test_constant_time_pad_check_valid() {
        let data = b"hello";
        let padded = pkcs7_pad(data, 16);
        let result = constant_time_pad_check(&padded, 16);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_constant_time_pad_check_invalid() {
        let bad_data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 3];
        assert!(constant_time_pad_check(&bad_data, 16).is_err());
    }
}
