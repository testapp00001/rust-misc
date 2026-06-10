//! # Lesson 04: Why Salts Prevent Rainbow Tables (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use rand::Rng;

/// Demonstrate why unsalted hashes are dangerous.
pub fn unsalted_hash(password: &str) -> (Vec<u8>, Vec<u8>) {
    let h1 = digest::digest(&digest::SHA256, password.as_bytes());
    let h2 = digest::digest(&digest::SHA256, password.as_bytes());
    (h1.as_ref().to_vec(), h2.as_ref().to_vec())
}

/// Hash a password with a random 16-byte salt.
pub fn hash_with_salt(password: &str) -> (Vec<u8>, Vec<u8>) {
    let salt: [u8; 16] = rand::thread_rng().gen();
    let mut data = salt.to_vec();
    data.extend_from_slice(password.as_bytes());
    let hash = digest::digest(&digest::SHA256, &data);
    (salt.to_vec(), hash.as_ref().to_vec())
}

/// Verify a password against a salted hash.
pub fn verify_salted_hash(password: &str, salt: &[u8], expected_hash: &[u8]) -> bool {
    let mut data = salt.to_vec();
    data.extend_from_slice(password.as_bytes());
    let hash = digest::digest(&digest::SHA256, &data);
    // Constant-time comparison
    let computed = hash.as_ref();
    if computed.len() != expected_hash.len() {
        return false;
    }
    let mut result = 0u8;
    for (a, b) in computed.iter().zip(expected_hash.iter()) {
        result |= a ^ b;
    }
    result == 0
}

/// Show that the same password with different salts produces different hashes.
pub fn demonstrate_salt_uniqueness(password: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    let (salt1, hash1) = hash_with_salt(password);
    let (salt2, hash2) = hash_with_salt(password);
    (salt1, hash1, salt2, hash2)
}

/// Encode a salted hash as a single string for storage.
pub fn encode_salted_hash(salt: &[u8], hash: &[u8]) -> String {
    format!("{}:{}", hex::encode(salt), hex::encode(hash))
}

/// Decode a salted hash string.
pub fn decode_salted_hash(encoded: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let parts: Vec<&str> = encoded.split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid format: expected salt:hash".to_string());
    }
    let salt = hex::decode(parts[0]).map_err(|e| format!("Invalid salt hex: {}", e))?;
    let hash = hex::decode(parts[1]).map_err(|e| format!("Invalid hash hex: {}", e))?;
    Ok((salt, hash))
}

/// Compute how many bytes of salt are needed for N years of security.
pub fn recommended_salt_length(num_users: u64) -> usize {
    if num_users == 0 {
        return 16;
    }
    // k > 2*log2(n) + 31
    let log2_n = (64 - num_users.leading_zeros()) as f64;
    let k = (2.0 * log2_n + 31.0).ceil() as usize;
    // bytes = ceil(k / 8), minimum 16
    let bytes = (k + 7) / 8;
    bytes.max(16)
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
