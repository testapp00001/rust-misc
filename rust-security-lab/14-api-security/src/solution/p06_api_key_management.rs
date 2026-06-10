//! # Lesson 06: API Key Management (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Sha256, Digest};
use rand::Rng;
use rand::rngs::OsRng;

pub fn generate_api_key(prefix: &str, num_random_bytes: usize) -> String {
    let mut rng = OsRng;
    let mut bytes = vec![0u8; num_random_bytes];
    rng.fill(&mut bytes[..]);
    format!("{}{}", prefix, hex::encode(bytes))
}

pub fn hash_api_key(key: &str) -> String {
    let hash = Sha256::digest(key.as_bytes());
    hex::encode(hash)
}

pub fn extract_key_prefix(key: &str, prefix_len: usize) -> Option<String> {
    if key.len() >= prefix_len {
        Some(key[..prefix_len].to_string())
    } else {
        None
    }
}

pub fn validate_api_key(
    provided_key: &str,
    stored_hash: &str,
    expected_prefix: &str,
) -> Result<(), &'static str> {
    if !provided_key.starts_with(expected_prefix) {
        return Err("invalid_prefix");
    }

    let computed_hash = hash_api_key(provided_key);

    let computed_bytes = hex::decode(&computed_hash).map_err(|_| "invalid_key")?;
    let stored_bytes = hex::decode(stored_hash).map_err(|_| "invalid_key")?;

    if ring::constant_time::verify_slices_are_equal(&computed_bytes, &stored_bytes).is_ok() {
        Ok(())
    } else {
        Err("invalid_key")
    }
}

pub fn generate_key_with_checksum(prefix: &str, num_random_bytes: usize) -> String {
    let mut rng = OsRng;
    let mut bytes = vec![0u8; num_random_bytes];
    rng.fill(&mut bytes[..]);
    let random_hex = hex::encode(bytes);

    let checksum_hash = Sha256::digest(random_hex.as_bytes());
    let checksum = &hex::encode(checksum_hash)[..4];

    format!("{}{}{}", prefix, random_hex, checksum)
}

pub fn verify_key_checksum(key: &str, prefix: &str) -> bool {
    let without_prefix = match key.strip_prefix(prefix) {
        Some(s) => s,
        None => return false,
    };

    if without_prefix.len() < 4 {
        return false;
    }

    let (body, checksum) = without_prefix.split_at(without_prefix.len() - 4);

    let hash = Sha256::digest(body.as_bytes());
    let expected_checksum = &hex::encode(hash)[..4];

    checksum.eq_ignore_ascii_case(expected_checksum)
}

pub struct KeyEntry {
    pub key_prefix: String,
    pub key_hash: String,
    pub user_id: String,
    pub created_at: u64,
}

pub fn create_key_entry(key: &str, user_id: &str, created_at: u64) -> KeyEntry {
    KeyEntry {
        key_prefix: key.chars().take(12).collect(),
        key_hash: hash_api_key(key),
        user_id: user_id.to_string(),
        created_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_has_prefix() {
        let key = generate_api_key("sk_live_", 32);
        assert!(key.starts_with("sk_live_"));
    }

    #[test]
    fn test_generate_key_length() {
        let key = generate_api_key("sk_", 32);
        assert_eq!(key.len(), 3 + 64);
    }

    #[test]
    fn test_generate_keys_unique() {
        let k1 = generate_api_key("sk_", 32);
        let k2 = generate_api_key("sk_", 32);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_hash_deterministic() {
        let h1 = hash_api_key("sk_live_abc123");
        let h2 = hash_api_key("sk_live_abc123");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_is_hex() {
        let h = hash_api_key("test_key");
        assert!(hex::decode(&h).is_ok());
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn test_extract_prefix() {
        assert_eq!(extract_key_prefix("sk_live_abc123def", 8).unwrap(), "sk_live_");
    }

    #[test]
    fn test_extract_prefix_too_short() {
        assert!(extract_key_prefix("short", 8).is_none());
    }

    #[test]
    fn test_validate_api_key_valid() {
        let key = generate_api_key("sk_live_", 32);
        let hash = hash_api_key(&key);
        assert!(validate_api_key(&key, &hash, "sk_live_").is_ok());
    }

    #[test]
    fn test_validate_api_key_wrong_prefix() {
        let key = generate_api_key("sk_live_", 32);
        let hash = hash_api_key(&key);
        assert_eq!(validate_api_key(&key, &hash, "pk_").unwrap_err(), "invalid_prefix");
    }

    #[test]
    fn test_validate_api_key_wrong_hash() {
        let key = generate_api_key("sk_live_", 32);
        assert_eq!(validate_api_key(&key, "wrong_hash", "sk_live_").unwrap_err(), "invalid_key");
    }

    #[test]
    fn test_key_with_checksum_format() {
        let key = generate_key_with_checksum("sk_", 16);
        assert!(key.starts_with("sk_"));
        assert_eq!(key.len(), 3 + 32 + 4);
    }

    #[test]
    fn test_verify_checksum_valid() {
        let key = generate_key_with_checksum("sk_", 16);
        assert!(verify_key_checksum(&key, "sk_"));
    }

    #[test]
    fn test_verify_checksum_invalid() {
        let mut key = generate_key_with_checksum("sk_", 16);
        let last = key.pop().unwrap();
        key.push(if last == 'a' { 'b' } else { 'a' });
        assert!(!verify_key_checksum(&key, "sk_"));
    }

    #[test]
    fn test_verify_checksum_too_short() {
        assert!(!verify_key_checksum("sk_", "sk_"));
    }

    #[test]
    fn test_create_key_entry() {
        let key = generate_api_key("sk_live_", 32);
        let entry = create_key_entry(&key, "user123", 1700000000);
        assert_eq!(entry.user_id, "user123");
        assert_eq!(entry.created_at, 1700000000);
        assert_eq!(entry.key_prefix.len(), 12);
        assert_eq!(entry.key_hash.len(), 64);
    }

    #[test]
    fn test_key_entry_hash_matches() {
        let key = generate_api_key("sk_live_", 32);
        let entry = create_key_entry(&key, "user123", 1700000000);
        assert_eq!(entry.key_hash, hash_api_key(&key));
    }
}
