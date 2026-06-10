//! # Lesson 09: Secure Key Destruction
//!
//! ## What Is Secure Key Destruction?
//!
//! Secure key destruction (also called "crypto-shredding" or "key erasure") ensures
//! that a cryptographic key is completely and irrecoverably removed from memory and
//! storage. This is the foundation of **crypto-shredding**: instead of deleting
//! encrypted data, you destroy the key — making the data permanently unreadable.
//!
//! ```text
//! Crypto-Shredding:
//!   Data encrypted with Key K
//!   Destroy K
//!   Data is now indistinguishable from random noise
//!   No forensic technique can recover it (if K was truly destroyed)
//! ```
//!
//! ## Why Not Just Delete the File?
//!
//! - `delete` removes the file name, not the data (recoverable with forensics)
//! - Memory may be swapped to disk (page file, hibernation)
//! - SSDs have wear-leveling — "overwritten" blocks may persist
//! - Rust's compiler may optimize away "unnecessary" memory zeroing
//!
//! ## The Problem in Rust
//!
//! Rust's `Vec<u8>` and `String` types do NOT zero memory on drop. The compiler
//! may even elid explicit zeroing if it determines the values are "dead." The
//! `zeroize` crate solves this with compiler barriers.
//!
//! ## Security Notes
//!
//! - Use the `zeroize` crate for memory zeroing (compiler cannot elide it)
//! - Use `secrecy::Secret<T>` to wrap sensitive values (zeroizes on drop)
//! - For SSDs, use hardware encryption — destroying the key effectively
//!   "erases" all data encrypted with it (crypto-erase)
//! - Document your key destruction policy and verify it works

use ring::rand::{SecureRandom, SystemRandom};
use secrecy::SecretBox;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A sensitive key that should be securely destroyed.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SensitiveKey {
    pub id: String,
    pub key_material: Vec<u8>,
    #[zeroize(skip)]
    pub algorithm: String,
}

/// Exercise 1: Create a sensitive key.
///
/// Generate a 32-byte random key wrapped in a SensitiveKey struct.
///
/// Hints:
/// - Use `ring::SystemRandom` for key material
/// - Return a `SensitiveKey` (which auto-zeroes on drop thanks to `ZeroizeOnDrop`)
pub fn create_sensitive_key(key_id: &str, algorithm: &str) -> SensitiveKey {
    todo!("Create a sensitive key with auto-zeroing on drop")
}

/// Exercise 2: Securely zero a key's material in-place.
///
/// Use the `zeroize` trait to explicitly zero the key material.
/// After calling this, `key.key_material` should be all zeros.
///
/// Hints:
/// - Call `key.key_material.zeroize()` (from the `Zeroize` trait)
/// - The `#[derive(Zeroize)]` on SensitiveKey makes this available
pub fn destroy_key(key: &mut SensitiveKey) {
    todo!("Securely zero the key material in-place")
}

/// Exercise 3: Verify that a key has been destroyed.
///
/// Check that all bytes in key_material are zero.
///
/// Hints:
/// - Use `.iter().all(|&b| b == 0)`
pub fn is_key_destroyed(key: &SensitiveKey) -> bool {
    todo!("Check if key material has been zeroed")
}

/// Exercise 4: Demonstrate that zeroize works (unlike naive zeroing).
///
/// This function creates a key, then demonstrates that:
/// 1. Naive zeroing (`key.fill(0)`) CAN be optimized away by the compiler
/// 2. `zeroize` CANNOT be optimized away (compiler barrier)
///
/// For safety, both approaches zero the bytes in this test, but only `zeroize`
/// is guaranteed to survive compiler optimizations in release builds.
///
/// Returns (naive_zeroed_bytes, zeroize_zeroed_bytes)
pub fn demonstrate_zeroize_importance() -> (Vec<u8>, Vec<u8>) {
    todo!("Demonstrate the difference between naive zeroing and zeroize")
}

/// Exercise 5: Crypto-shredding — destroy the key to make data unrecoverable.
///
/// Given encrypted data and its key, demonstrate that destroying the key
/// makes the data permanently unreadable.
///
/// Returns (encrypted_data, key_was_destroyed, data_is_unrecoverable)
pub fn demonstrate_crypto_shredding(plaintext: &[u8]) -> (Vec<u8>, bool, bool) {
    todo!("Demonstrate crypto-shredding by destroying the encryption key")
}

/// Exercise 6: Batch destruction — securely zero multiple keys at once.
///
/// Given a vector of SensitiveKey, zero all of them.
///
/// Hints:
/// - Iterate and call `destroy_key` on each
pub fn destroy_all_keys(keys: &mut Vec<SensitiveKey>) {
    todo!("Securely destroy multiple keys")
}

/// Exercise 7: Wrap a key in a SecretBox for additional protection.
///
/// The `secrecy::SecretBox` type:
/// - Prevents accidental logging (Debug is redacted)
/// - Zeroes the inner value on drop
/// - Requires `.expose_secret()` to access (makes access explicit)
///
/// Create a SecretBox<Vec<u8>> from a key's material.
pub fn wrap_in_secret(key: &SensitiveKey) -> SecretBox<Vec<u8>> {
    todo!("Wrap key material in a SecretBox for additional protection")
}

/// Exercise 8: Demonstrate that SecretBox prevents accidental logging.
///
/// Create a SecretBox<Vec<u8>> and show that Debug formatting redacts the value.
///
/// Returns the Debug-formatted string (should contain "REDACTED")
pub fn demonstrate_secret_redaction(key_material: Vec<u8>) -> String {
    todo!("Show that SecretBox prevents accidental logging of sensitive data")
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn test_create_sensitive_key() {
        let key = create_sensitive_key("test-key", "AES-256");
        assert_eq!(key.id, "test-key");
        assert_eq!(key.key_material.len(), 32);
    }

    #[test]
    fn test_destroy_key_zeros_material() {
        let mut key = create_sensitive_key("test-key", "AES-256");
        assert!(key.key_material.iter().any(|&b| b != 0), "Key should have non-zero bytes initially");
        destroy_key(&mut key);
        assert!(is_key_destroyed(&key), "Key material should be all zeros after destruction");
    }

    #[test]
    fn test_is_key_destroyed_initially_false() {
        let key = create_sensitive_key("test-key", "AES-256");
        assert!(!is_key_destroyed(&key), "Fresh key should not be destroyed");
    }

    #[test]
    fn test_zeroize_importance() {
        let (naive, zeroized) = demonstrate_zeroize_importance();
        // Both should be zeroed in our test (but only zeroize is guaranteed in release)
        assert!(naive.iter().all(|&b| b == 0));
        assert!(zeroized.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_crypto_shredding() {
        let plaintext = b"This message will be shredded";
        let (encrypted, destroyed, unrecoverable) = demonstrate_crypto_shredding(plaintext);
        assert!(encrypted.len() > 0, "Should have encrypted data");
        assert!(destroyed, "Key should be destroyed");
        assert!(unrecoverable, "Data should be unrecoverable without key");
    }

    #[test]
    fn test_batch_destruction() {
        let mut keys = vec![
            create_sensitive_key("key-1", "AES-256"),
            create_sensitive_key("key-2", "AES-256"),
            create_sensitive_key("key-3", "AES-256"),
        ];
        destroy_all_keys(&mut keys);
        for key in &keys {
            assert!(is_key_destroyed(key), "All keys should be destroyed");
        }
    }

    #[test]
    fn test_secret_wrapping() {
        let key = create_sensitive_key("test-key", "AES-256");
        let secret = wrap_in_secret(&key);
        // Secret should contain the same bytes when exposed
        assert_eq!(secret.expose_secret().len(), 32);
    }

    #[test]
    fn test_secret_redaction() {
        let key_material = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let debug_output = demonstrate_secret_redaction(key_material);
        // SecretBox's Debug output should contain "REDACTED" and NOT the actual bytes
        assert!(
            debug_output.contains("REDACTED"),
            "SecretBox Debug should redact actual values: got {}",
            debug_output
        );
    }
}
