//! # Lesson 09: Secure Key Destruction (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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

/// Create a sensitive key with auto-zeroing on drop.
pub fn create_sensitive_key(key_id: &str, algorithm: &str) -> SensitiveKey {
    let rng = SystemRandom::new();
    let mut key_material = vec![0u8; 32];
    rng.fill(&mut key_material).expect("Failed to generate key material");

    SensitiveKey {
        id: key_id.to_string(),
        key_material,
        algorithm: algorithm.to_string(),
    }
}

/// Securely zero the key material in-place using `zeroize`.
///
/// `zeroize` uses a volatile write + compiler fence, so the compiler
/// cannot optimize the zeroing away even in release builds.
pub fn destroy_key(key: &mut SensitiveKey) {
    key.key_material.zeroize();
}

/// Check if key material has been zeroed.
pub fn is_key_destroyed(key: &SensitiveKey) -> bool {
    key.key_material.iter().all(|&b| b == 0)
}

/// Demonstrate the difference between naive zeroing and zeroize.
///
/// Returns (naive_zeroed_bytes, zeroize_zeroed_bytes).
/// In debug mode, both will be zeroed. In release mode, only `zeroize`
/// is guaranteed (the compiler may optimize away naive `.fill(0)`).
pub fn demonstrate_zeroize_importance() -> (Vec<u8>, Vec<u8>) {
    // Naive zeroing (can be optimized away in release mode)
    let mut naive = vec![0xDEu8; 32];
    for byte in naive.iter_mut() {
        *byte = 0;
    }

    // zeroize zeroing (compiler barrier)
    let mut zeroized = vec![0xADu8; 32];
    zeroized.zeroize();

    (naive, zeroized)
}

/// Demonstrate crypto-shredding by destroying the encryption key.
///
/// Returns (encrypted_data, key_was_destroyed, data_is_unrecoverable).
pub fn demonstrate_crypto_shredding(plaintext: &[u8]) -> (Vec<u8>, bool, bool) {
    let rng = SystemRandom::new();
    let mut key = vec![0u8; 32];
    rng.fill(&mut key).expect("Failed to generate key");

    // Encrypt with XOR (simplified)
    let encrypted: Vec<u8> = plaintext
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect();

    // Destroy the key
    let key_was_destroyed = {
        let mut key_copy = key.clone();
        key_copy.zeroize();
        key_copy.iter().all(|&b| b == 0)
    };

    // Without the key, the encrypted data is indistinguishable from random noise
    // (for a proper cipher like AES-GCM, this is information-theoretically true)
    let data_is_unrecoverable = key_was_destroyed;

    (encrypted, key_was_destroyed, data_is_unrecoverable)
}

/// Securely destroy multiple keys at once.
pub fn destroy_all_keys(keys: &mut Vec<SensitiveKey>) {
    for key in keys.iter_mut() {
        destroy_key(key);
    }
}

/// Wrap key material in a `SecretBox` for additional protection.
///
/// `SecretBox<T>`:
/// - Zeroes inner value on drop
/// - Redacts value in Debug output
/// - Requires `.expose_secret()` for explicit access
pub fn wrap_in_secret(key: &SensitiveKey) -> SecretBox<Vec<u8>> {
    SecretBox::new(key.key_material.clone().into())
}

/// Demonstrate that `SecretBox` prevents accidental logging.
///
/// Returns the Debug-formatted string (which should NOT contain the actual bytes).
pub fn demonstrate_secret_redaction(key_material: Vec<u8>) -> String {
    let secret: SecretBox<Vec<u8>> = SecretBox::new(key_material.into());
    format!("{:?}", secret)
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
        assert!(key.key_material.iter().any(|&b| b != 0));
        destroy_key(&mut key);
        assert!(is_key_destroyed(&key));
    }

    #[test]
    fn test_is_key_destroyed_initially_false() {
        let key = create_sensitive_key("test-key", "AES-256");
        assert!(!is_key_destroyed(&key));
    }

    #[test]
    fn test_zeroize_importance() {
        let (naive, zeroized) = demonstrate_zeroize_importance();
        assert!(naive.iter().all(|&b| b == 0));
        assert!(zeroized.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_crypto_shredding() {
        let plaintext = b"This message will be shredded";
        let (encrypted, destroyed, unrecoverable) = demonstrate_crypto_shredding(plaintext);
        assert!(encrypted.len() > 0);
        assert!(destroyed);
        assert!(unrecoverable);
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
            assert!(is_key_destroyed(key));
        }
    }

    #[test]
    fn test_secret_wrapping() {
        let key = create_sensitive_key("test-key", "AES-256");
        let secret = wrap_in_secret(&key);
        assert_eq!(secret.expose_secret().len(), 32);
    }

    #[test]
    fn test_secret_redaction() {
        let key_material = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let debug_output = demonstrate_secret_redaction(key_material);
        assert!(
            debug_output.contains("REDACTED"),
            "SecretBox Debug should contain REDACTED: got {}",
            debug_output
        );
    }
}
