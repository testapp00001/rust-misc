//! # Lesson 03: scrypt Password Hashing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use scrypt::{scrypt, Params};
use rand::RngCore;

/// Hash a password with scrypt using default parameters.
pub fn hash_password_scrypt(password: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let params = Params::new(14, 8, 1, 64)
        .map_err(|e| format!("Invalid scrypt params: {}", e))?;
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    let mut output = vec![0u8; 64];
    scrypt(password.as_bytes(), &salt, &params, &mut output)
        .map_err(|e| format!("scrypt failed: {}", e))?;
    Ok((salt.to_vec(), output))
}

/// Verify a password against a scrypt hash.
pub fn verify_password_scrypt(password: &str, salt: &[u8], hash: &[u8]) -> Result<bool, String> {
    let params = Params::new(14, 8, 1, 64)
        .map_err(|e| format!("Invalid scrypt params: {}", e))?;
    let mut output = vec![0u8; 64];
    scrypt(password.as_bytes(), salt, &params, &mut output)
        .map_err(|e| format!("scrypt failed: {}", e))?;
    // Constant-time comparison
    if output.len() != hash.len() {
        return Ok(false);
    }
    let mut result = 0u8;
    for (a, b) in output.iter().zip(hash.iter()) {
        result |= a ^ b;
    }
    Ok(result == 0)
}

/// Encode a scrypt hash as a single portable string.
pub fn encode_scrypt_hash(salt: &[u8], hash: &[u8], log_n: u8, r: u32, p: u32) -> String {
    format!("scrypt:{}:{}:{}:{}:{}", log_n, r, p, hex::encode(salt), hex::encode(hash))
}

/// Decode a scrypt hash string back to its components.
pub fn decode_scrypt_hash(encoded: &str) -> Result<(u8, u32, u32, Vec<u8>, Vec<u8>), String> {
    let parts: Vec<&str> = encoded.split(':').collect();
    if parts.len() != 6 || parts[0] != "scrypt" {
        return Err("Invalid scrypt hash format".to_string());
    }
    let log_n: u8 = parts[1].parse().map_err(|e| format!("Invalid log_n: {}", e))?;
    let r: u32 = parts[2].parse().map_err(|e| format!("Invalid r: {}", e))?;
    let p: u32 = parts[3].parse().map_err(|e| format!("Invalid p: {}", e))?;
    let salt = hex::decode(parts[4]).map_err(|e| format!("Invalid salt hex: {}", e))?;
    let hash = hex::decode(parts[5]).map_err(|e| format!("Invalid hash hex: {}", e))?;
    Ok((log_n, r, p, salt, hash))
}

/// Estimate memory usage for given scrypt parameters.
pub fn estimate_memory_usage(log_n: u8, _r: u32, p: u32) -> u64 {
    let n = 2u64.pow(log_n as u32);
    128 * n * (p as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let (salt, hash) = hash_password_scrypt("mypassword").unwrap();
        assert!(verify_password_scrypt("mypassword", &salt, &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let (salt, hash) = hash_password_scrypt("mypassword").unwrap();
        assert!(!verify_password_scrypt("wrongpassword", &salt, &hash).unwrap());
    }

    #[test]
    fn test_unique_salts() {
        let (salt1, _) = hash_password_scrypt("same").unwrap();
        let (salt2, _) = hash_password_scrypt("same").unwrap();
        assert_ne!(salt1, salt2, "Each hash should use a unique salt");
    }

    #[test]
    fn test_hash_length() {
        let (_, hash) = hash_password_scrypt("test").unwrap();
        assert_eq!(hash.len(), 64, "Hash should be 64 bytes (as configured)");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let (salt, hash) = hash_password_scrypt("test").unwrap();
        let encoded = encode_scrypt_hash(&salt, &hash, 14, 8, 1);
        let (log_n, r, p, decoded_salt, decoded_hash) = decode_scrypt_hash(&encoded).unwrap();
        assert_eq!(log_n, 14);
        assert_eq!(r, 8);
        assert_eq!(p, 1);
        assert_eq!(decoded_salt, salt);
        assert_eq!(decoded_hash, hash);
    }

    #[test]
    fn test_encode_format() {
        let encoded = encode_scrypt_hash(&[1, 2, 3], &[4, 5, 6], 10, 8, 1);
        assert!(encoded.starts_with("scrypt:10:8:1:"));
    }

    #[test]
    fn test_decode_invalid() {
        assert!(decode_scrypt_hash("invalid").is_err());
        assert!(decode_scrypt_hash("scrypt:abc:8:1:aabb:aabb").is_err());
    }

    #[test]
    fn test_memory_estimation() {
        // N=2^14=16384, p=1: 128 * 16384 * 1 = 2,097,152 bytes = ~2 MB
        let mem = estimate_memory_usage(14, 8, 1);
        assert_eq!(mem, 128 * 16384 * 1);

        // N=2^14, p=2: should double
        let mem2 = estimate_memory_usage(14, 8, 2);
        assert_eq!(mem2, mem * 2);
    }
}
