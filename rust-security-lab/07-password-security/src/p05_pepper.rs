//! # Lesson 05: Pepper -- Application-Wide Secret
//!
//! ## What is a Pepper?
//!
//! A pepper is an application-wide secret that is added to the password BEFORE
//! hashing, but is NOT stored in the database. It is typically kept in:
//! - Environment variables
//! - Hardware Security Modules (HSMs)
//! - Secret management systems (Vault, AWS Secrets Manager)
//!
//! The idea: even if an attacker steals the entire database (hashes + salts),
//! they still cannot crack passwords without the pepper.
//!
//! ## Pepper vs Salt
//!
//! | Property  | Salt                          | Pepper                        |
//! |-----------|-------------------------------|-------------------------------|
//! | Per-user  | Yes (unique per password)     | No (shared across all users)  |
//! | Stored    | In the database               | Outside the database          |
//! | Secret    | No (stored in cleartext)      | Yes (protected secret)        |
//! | Purpose   | Defeat rainbow tables         | Defense-in-depth for DB leak  |
//! | Required  | Yes                           | Optional (but recommended)    |
//!
//! ## How to Apply a Pepper
//!
//! Method 1 (concatenation): `hash(pepper + password + salt)`
//! Method 2 (HMAC): `hash(HMAC(pepper, password) + salt)`
//!
//! Method 2 is preferred because HMAC provides a well-studied security proof.
//! The pepper acts as the HMAC key, and the password is the message.
//!
//! ## Security Analysis
//!
//! The pepper adds defense-in-depth:
//! - **SQL injection only**: Attacker gets hashes but not pepper. Cannot crack.
//! - **Full server compromise**: Attacker gets hashes AND pepper. Same as no pepper.
//!
//! The pepper is NOT a substitute for proper hashing. It is an additional layer.
//!
//! ## Rotating a Pepper
//!
//! Rotating a pepper requires re-hashing all passwords, since the old pepper is
//! needed to verify existing passwords. This is the same problem as migrating
//! hash algorithms (see p10_password_migration). You can:
//! 1. Keep old pepper for verification
//! 2. Re-hash with new pepper on next login
//! 3. After migration period, remove old pepper

use ring::hmac;

/// Exercise 1: Apply a pepper using HMAC-SHA256.
///
/// Compute HMAC(pepper, password) and return the result.
/// This is the recommended way to apply a pepper.
///
/// Hints:
/// - Create HMAC key: `hmac::Key::new(hmac::HMAC_SHA256, pepper.as_bytes())`
/// - Sign: `hmac::sign(&key, password.as_bytes())`
/// - Return the HMAC tag as bytes: `tag.as_ref().to_vec()`
pub fn apply_pepper_hmac(password: &str, pepper: &str) -> Vec<u8> {
    todo!("Implement HMAC-based pepper application")
}

/// Exercise 2: Apply a pepper using simple concatenation.
///
/// Returns `pepper:password` as bytes. This is simpler but less principled
/// than HMAC.
///
/// Hints:
/// - Format: `format!("{}:{}", pepper, password)`
/// - Return as bytes
pub fn apply_pepper_concat(password: &str, pepper: &str) -> Vec<u8> {
    todo!("Implement concatenation-based pepper application")
}

/// Exercise 3: Hash a peppered password with Argon2id.
///
/// Apply the pepper with HMAC, then hash the result with Argon2id.
///
/// Hints:
/// - Apply pepper: `apply_pepper_hmac(password, pepper)`
/// - Hash with Argon2id: use `argon2` crate
/// - Use SaltString::generate for a random salt
/// - Convert HMAC output to base64 or hex for Argon2 input
pub fn hash_peppered_argon2id(password: &str, pepper: &str) -> Result<String, String> {
    todo!("Implement peppered Argon2id hashing")
}

/// Exercise 4: Verify a peppered password against an Argon2id hash.
///
/// Hints:
/// - Apply pepper: `apply_pepper_hmac(password, pepper)`
/// - Parse hash: `PasswordHash::new(hash_str)?`
/// - Verify with Argon2
pub fn verify_peppered_argon2id(password: &str, pepper: &str, hash_str: &str) -> Result<bool, String> {
    todo!("Implement peppered password verification")
}

/// Exercise 5: Demonstrate that different peppers produce different hashes.
///
/// Hash the same password with two different peppers using HMAC.
/// Return both HMAC outputs.
///
/// Hints:
/// - Call `apply_pepper_hmac` with two different peppers
pub fn demonstrate_pepper_effect(password: &str, pepper1: &str, pepper2: &str) -> (Vec<u8>, Vec<u8>) {
    todo!("Implement pepper effect demonstration")
}

/// Exercise 6: Create a pepper rotation helper.
///
/// Given a password verified with the old pepper, re-hash it with the new pepper.
/// Returns the new Argon2id hash.
///
/// This simulates what happens during a pepper rotation:
/// 1. User logs in with old pepper (verification succeeds)
/// 2. Re-hash with new pepper
/// 3. Store new hash
///
/// Hints:
/// - Hash with new pepper using `hash_peppered_argon2id`
pub fn rotate_pepper(password: &str, old_pepper: &str, new_pepper: &str) -> Result<String, String> {
    todo!("Implement pepper rotation")
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
