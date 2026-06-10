//! # Lesson 07: Padding Oracle Attack
//!
//! ## What is a Padding Oracle Attack?
//!
//! A padding oracle attack exploits a server that leaks whether decrypted ciphertext
//! has valid padding. This is possible in CBC mode because:
//!
//! 1. The server decrypts the last block to check padding
//! 2. If padding is invalid, it returns a different error than "bad MAC"
//! 3. The attacker uses this 1-bit information oracle to recover plaintext one byte at a time
//!
//! ## How CBC Padding Works (PKCS#7)
//!
//! ```text
//! Block size: 16 bytes
//! If plaintext is 11 bytes, add 5 bytes of value 0x05:
//!   [... 11 bytes ...] [0x05 0x05 0x05 0x05 0x05]
//!
//! If plaintext is 16 bytes, add a full block of 0x10:
//!   [... 16 bytes ...] [0x10 * 16]
//! ```
//!
//! ## The Attack (Simplified)
//!
//! To recover the last byte of a block:
//! 1. Attacker sends modified ciphertext to the server
//! 2. Tries all 256 values for the last byte of the preceding block
//! 3. When padding is valid (server says "good padding, bad MAC"), the attacker knows
//!    the XOR relationship: `modified_byte XOR guess = 0x01`
//! 4. This reveals the plaintext byte
//!
//! ## Timeline
//!
//! - **2002**: Vaudenay publishes the attack
//! - **2002-2010**: Used against TLS (BEAST, Lucky 13), ASP.NET, Java, XML encryption
//! - **Modern defense**: AEAD modes (GCM, Poly1305) verify authenticity BEFORE decryption
//!
//! ## Defense
//!
//! 1. **Use AEAD modes** (GCM, Poly1305) — they authenticate before decrypting
//! 2. **Constant-time comparison** for padding checks
//! 3. **MAC-then-Encrypt is wrong** — use Encrypt-then-MAC or AEAD
//! 4. **Never return different errors** for "bad padding" vs "bad MAC"

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};

/// Exercise 1: Implement PKCS#7 padding.
///
/// PKCS#7 padding adds N bytes of value N to make the plaintext a multiple
/// of the block size (16 bytes for AES).
///
/// Rules:
/// - If plaintext length is a multiple of 16, add a full block of 0x10 (16)
/// - Otherwise, add bytes to reach the next multiple
/// - Each padding byte has the value equal to the number of padding bytes
///
/// Hints:
/// - Calculate: `padding_len = 16 - (plaintext.len() % 16)`
/// - Append `padding_len` copies of `padding_len as u8`
pub fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
    todo!("Implement PKCS#7 padding")
}

/// Exercise 2: Implement PKCS#7 unpadding.
///
/// Remove PKCS#7 padding and return the original plaintext.
///
/// Rules:
/// - Read the last byte — it tells you how many padding bytes to remove
/// - Verify all padding bytes have the same value
/// - Return an error if padding is invalid
///
/// Hints:
/// - `let pad_len = *data.last().ok_or("empty")? as usize;`
/// - Verify `pad_len > 0 && pad_len <= block_size`
/// - Verify the last `pad_len` bytes are all equal to `pad_len as u8`
pub fn pkcs7_unpad(data: &[u8], block_size: usize) -> Result<&[u8], &'static str> {
    todo!("Implement PKCS#7 unpadding with validation")
}

/// Exercise 3: Simulate a padding oracle server.
///
/// This function simulates a vulnerable server that:
/// 1. Decrypts the ciphertext
/// 2. Checks if padding is valid
/// 3. Returns different errors for "bad padding" vs "bad MAC"
///
/// This is the VULNERABLE pattern — it leaks information about the padding.
///
/// Returns:
/// - Ok(plaintext) if decryption succeeds
/// - Err("bad_padding") if padding is invalid (LEAKS INFORMATION!)
/// - Err("bad_mac") if MAC/tag verification fails
///
/// For this exercise, we use AES-GCM which provides AEAD. To simulate the
/// padding oracle, we'll manually check padding after decryption.
/// In a real CBC scenario, the padding check happens before MAC verification.
///
/// Hints:
/// - Decrypt with AES-GCM (this already verifies the tag)
/// - Then check PKCS#7 padding on the result
/// - If tag fails → "bad_mac"
/// - If padding fails → "bad_padding" (this is the leak!)
pub fn vulnerable_server(
    key: &Key<Aes256Gcm>,
    nonce: &aes_gcm::aead::Nonce<Aes256Gcm>,
    ciphertext: &[u8],
) -> Result<Vec<u8>, &'static str> {
    todo!("Simulate a padding oracle server")
}

/// Exercise 4: Demonstrate why AEAD prevents padding oracle attacks.
///
/// With AEAD (AES-GCM), the authentication tag is verified BEFORE decryption.
/// If the tag is invalid, the error is the same regardless of what the padding
/// would be. This eliminates the oracle.
///
/// Returns true if the server consistently returns the same error type
/// for both "bad padding" and "bad MAC" scenarios.
///
/// Hints:
/// - Encrypt a message
/// - Tamper with the ciphertext (changes both padding and MAC)
/// - Show that AEAD always returns the same error (auth failure)
/// - Return true
pub fn demonstrate_aead_prevents_oracle(key: &Key<Aes256Gcm>) -> bool {
    todo!("Show that AEAD eliminates the padding oracle")
}

/// Exercise 5: Implement constant-time padding validation.
///
/// A secure implementation must check padding in constant time to prevent
/// timing side-channels. This function should:
/// 1. Always examine ALL bytes (no early return)
/// 2. Use bitwise operations instead of branches
/// 3. Return an error only after examining everything
///
/// Hints:
/// - Start with `valid = true`
/// - Loop over ALL bytes, checking each one
/// - Use `valid &= condition` instead of `if !condition { return Err }`
/// - After the loop, check `valid`
pub fn constant_time_pad_check(data: &[u8], block_size: usize) -> Result<&[u8], &'static str> {
    todo!("Implement constant-time PKCS#7 padding validation")
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
        let data = vec![0u8; 16]; // Exactly one block
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
        // Invalid padding: last byte says 4, but the bytes aren't all 4
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

        // Valid ciphertext should succeed
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
