//! # Lesson 04: Why Salts Prevent Rainbow Tables
//!
//! ## The Problem: Rainbow Tables
//!
//! A rainbow table is a precomputed lookup table that maps hashes back to passwords.
//! For MD5, a complete rainbow table for all 8-character alphanumeric passwords
//! exists on the internet. Without salts, an attacker who steals your hash database
//! can look up every hash in seconds.
//!
//! Example rainbow table entry:
//! ```
//! "password" → 5f4dcc3b5aa765d61d8327deb882cf99
//! "123456"   → e10adc3949ba59abbe56e057f20f883e
//! ```
//!
//! If you hash without a salt, two users with the same password have the same hash.
//! An attacker can precompute hashes for millions of common passwords and reverse
//! your entire database instantly.
//!
//! ## The Solution: Unique Salt Per Password
//!
//! A salt is a random value prepended (or appended) to each password before hashing.
//! Each user gets a unique salt, stored alongside the hash.
//!
//! ```
//! User A: password="hello", salt="a1b2c3" → hash("a1b2c3:hello") → "..."
//! User B: password="hello", salt="d4e5f6" → hash("d4e5f6:hello") → "..."
//! ```
//!
//! Even though both users chose "password", their hashes are completely different.
//! A rainbow table built for unsalted hashes is useless. To attack salted hashes,
//! an attacker must rebuild the table for EACH salt -- making the attack
//! O(users * passwords) instead of O(passwords).
//!
//! ## Salt Requirements
//!
//! - **Unique per password**: NEVER reuse salts across users
//! - **Random**: Use a CSPRNG (cryptographically secure random number generator)
//! - **Sufficient length**: 16 bytes (128 bits) minimum
//! - **Stored with the hash**: Salts are not secret -- they're stored alongside the hash
//!
//! ## How Argon2id/bcrypt/scrypt Handle Salts
//!
//! All three algorithms build salt generation into their API:
//! - Argon2id: `SaltString::generate(&mut OsRng)`
//! - bcrypt: Generated internally by `bcrypt::hash()`
//! - scrypt: You generate and pass it manually
//!
//! You typically don't manage salts yourself. But understanding WHY they exist
//! is critical for security reasoning.

use ring::digest;

/// Exercise 1: Demonstrate why unsalted hashes are dangerous.
///
/// Hash the same password twice without a salt. Both hashes should be identical,
/// proving that an attacker can build a single rainbow table for all users.
///
/// Hints:
/// - Just SHA-256 hash the password
/// - Return both hashes (they'll be equal)
pub fn unsalted_hash(password: &str) -> (Vec<u8>, Vec<u8>) {
    todo!("Implement unsalted hashing demo")
}

/// Exercise 2: Hash a password with a random 16-byte salt.
///
/// Return (salt, hash). The hash is computed as SHA-256(salt || password).
///
/// Hints:
/// - Generate 16 random bytes with `rand::thread_rng().gen::<[u8; 16]>()`
/// - Concatenate salt + password bytes
/// - SHA-256 the concatenation
pub fn hash_with_salt(password: &str) -> (Vec<u8>, Vec<u8>) {
    todo!("Implement salted hashing")
}

/// Exercise 3: Verify a password against a salted hash.
///
/// Recompute SHA-256(salt || password) and compare with constant-time comparison.
///
/// Hints:
/// - Concatenate salt + password bytes
/// - SHA-256 the concatenation
/// - Compare with `ring::constant_time::verify_slices_are_equal`
pub fn verify_salted_hash(password: &str, salt: &[u8], expected_hash: &[u8]) -> bool {
    todo!("Implement salted hash verification")
}

/// Exercise 4: Show that the same password with different salts produces different hashes.
///
/// Hash the same password with two different salts and verify the hashes differ.
///
/// Hints:
/// - Call `hash_with_salt` twice
/// - Compare the hashes (they should be different due to different salts)
pub fn demonstrate_salt_uniqueness(password: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    todo!("Implement salt uniqueness demonstration")
}

/// Exercise 5: Encode a salted hash as a single string for storage.
///
/// Format: `<hex_salt>:<hex_hash>`
///
/// Hints:
/// - Use `hex::encode` for both salt and hash
pub fn encode_salted_hash(salt: &[u8], hash: &[u8]) -> String {
    todo!("Implement salted hash encoding")
}

/// Exercise 6: Decode a salted hash string.
///
/// Parse the format from `encode_salted_hash`.
/// Returns (salt, hash).
///
/// Hints:
/// - Split by ':'
/// - Use `hex::decode` for each part
pub fn decode_salted_hash(encoded: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    todo!("Implement salted hash decoding")
}

/// Exercise 7: Compute how many bytes of salt are needed for N years of security.
///
/// Rule of thumb: 16 bytes (128 bits) is sufficient for any realistic scenario.
/// This exercise helps you understand the math.
///
/// With k bits of salt and n users, the probability of a salt collision is
/// approximately n^2 / 2^(k+1) (birthday paradox).
///
/// Returns the minimum number of salt bytes needed so that the collision
/// probability for `num_users` users is below 2^-32.
///
/// Hints:
/// - Birthday paradox: P(collision) ≈ n^2 / 2^(k+1)
/// - We want P < 2^-32
/// - So n^2 / 2^(k+1) < 2^-32
/// - k+1 > 2*log2(n) + 32
/// - k > 2*log2(n) + 31
/// - bytes = ceil(k / 8)
pub fn recommended_salt_length(num_users: u64) -> usize {
    todo!("Implement salt length recommendation")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsalted_hashes_identical() {
        let (h1, h2) = unsalted_hash("password123");
        assert_eq!(h1, h2, "Unsalted hashes of the same password should be identical");
    }

    #[test]
    fn test_salted_hashes_unique() {
        let (salt1, hash1) = hash_with_salt("password123");
        let (salt2, hash2) = hash_with_salt("password123");
        assert_ne!(salt1, salt2, "Salts should be unique");
        assert_ne!(hash1, hash2, "Hashes should differ with different salts");
    }

    #[test]
    fn test_verify_correct_password() {
        let (salt, hash) = hash_with_salt("mypassword");
        assert!(verify_salted_hash("mypassword", &salt, &hash));
    }

    #[test]
    fn test_verify_wrong_password() {
        let (salt, hash) = hash_with_salt("mypassword");
        assert!(!verify_salted_hash("wrongpassword", &salt, &hash));
    }

    #[test]
    fn test_salt_length() {
        let (salt, _) = hash_with_salt("test");
        assert_eq!(salt.len(), 16, "Salt should be 16 bytes");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let (salt, hash) = hash_with_salt("test");
        let encoded = encode_salted_hash(&salt, &hash);
        let (decoded_salt, decoded_hash) = decode_salted_hash(&encoded).unwrap();
        assert_eq!(decoded_salt, salt);
        assert_eq!(decoded_hash, hash);
    }

    #[test]
    fn test_recommended_salt_length() {
        // For 1 billion users, should still recommend 16 bytes (128 bits)
        let len = recommended_salt_length(1_000_000_000);
        assert!(len >= 16, "Should recommend at least 16 bytes for any realistic scenario");
    }

    #[test]
    fn test_demonstrate_salt_uniqueness() {
        let (salt1, hash1, salt2, hash2) = demonstrate_salt_uniqueness("samepassword");
        assert_ne!(salt1, salt2);
        assert_ne!(hash1, hash2);
    }
}
