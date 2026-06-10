//! # Lesson 05: Pepper -- Application-Wide Secret (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::hmac;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

/// Apply a pepper using HMAC-SHA256.
pub fn apply_pepper_hmac(password: &str, pepper: &str) -> Vec<u8> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, pepper.as_bytes());
    let tag = hmac::sign(&key, password.as_bytes());
    tag.as_ref().to_vec()
}

/// Apply a pepper using simple concatenation.
pub fn apply_pepper_concat(password: &str, pepper: &str) -> Vec<u8> {
    format!("{}:{}", pepper, password).into_bytes()
}

/// Hash a peppered password with Argon2id.
pub fn hash_peppered_argon2id(password: &str, pepper: &str) -> Result<String, String> {
    let peppered = apply_pepper_hmac(password, pepper);
    let peppered_hex = hex::encode(&peppered);
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(peppered_hex.as_bytes(), &salt)
        .map_err(|e| format!("Hashing failed: {}", e))?;
    Ok(hash.to_string())
}

/// Verify a peppered password against an Argon2id hash.
pub fn verify_peppered_argon2id(password: &str, pepper: &str, hash_str: &str) -> Result<bool, String> {
    let peppered = apply_pepper_hmac(password, pepper);
    let peppered_hex = hex::encode(&peppered);
    let parsed_hash = PasswordHash::new(hash_str)
        .map_err(|e| format!("Failed to parse hash: {}", e))?;
    match Argon2::default().verify_password(peppered_hex.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(format!("Verification error: {}", e)),
    }
}

/// Demonstrate that different peppers produce different hashes.
pub fn demonstrate_pepper_effect(password: &str, pepper1: &str, pepper2: &str) -> (Vec<u8>, Vec<u8>) {
    (apply_pepper_hmac(password, pepper1), apply_pepper_hmac(password, pepper2))
}

/// Create a pepper rotation helper.
pub fn rotate_pepper(password: &str, _old_pepper: &str, new_pepper: &str) -> Result<String, String> {
    hash_peppered_argon2id(password, new_pepper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_pepper_deterministic() {
        let h1 = apply_pepper_hmac("password", "pepper");
        let h2 = apply_pepper_hmac("password", "pepper");
        assert_eq!(h1, h2, "Same inputs should produce same HMAC");
    }

    #[test]
    fn test_different_peppers_different_output() {
        let h1 = apply_pepper_hmac("password", "pepper1");
        let h2 = apply_pepper_hmac("password", "pepper2");
        assert_ne!(h1, h2, "Different peppers should produce different HMACs");
    }

    #[test]
    fn test_different_passwords_different_output() {
        let h1 = apply_pepper_hmac("password1", "pepper");
        let h2 = apply_pepper_hmac("password2", "pepper");
        assert_ne!(h1, h2, "Different passwords should produce different HMACs");
    }

    #[test]
    fn test_concat_pepper() {
        let result = apply_pepper_concat("mypassword", "mysecret");
        let expected = format!("{}:{}", "mysecret", "mypassword");
        assert_eq!(result, expected.as_bytes());
    }

    #[test]
    fn test_peppered_argon2id_roundtrip() {
        let pepper = "my-application-pepper-secret";
        let hash = hash_peppered_argon2id("testpassword", pepper).unwrap();
        assert!(verify_peppered_argon2id("testpassword", pepper, &hash).unwrap());
        assert!(!verify_peppered_argon2id("wrongpassword", pepper, &hash).unwrap());
    }

    #[test]
    fn test_wrong_pepper_fails() {
        let hash = hash_peppered_argon2id("testpassword", "correct-pepper").unwrap();
        assert!(!verify_peppered_argon2id("testpassword", "wrong-pepper", &hash).unwrap());
    }

    #[test]
    fn test_pepper_rotation() {
        let old_pepper = "old-secret";
        let new_pepper = "new-secret";
        let new_hash = rotate_pepper("mypassword", old_pepper, new_pepper).unwrap();
        assert!(verify_peppered_argon2id("mypassword", new_pepper, &new_hash).unwrap());
    }

    #[test]
    fn test_demonstrate_pepper_effect() {
        let (h1, h2) = demonstrate_pepper_effect("samepassword", "pepper_a", "pepper_b");
        assert_ne!(h1, h2);
    }
}
