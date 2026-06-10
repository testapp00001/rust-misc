//! # Lesson 09: Secret Management — Solution
//!
//! Env vars, zeroize, rotation, secrecy crate.

use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone)]
pub struct ManagedSecret {
    #[allow(dead_code)]
    value: Vec<u8>,
    pub created_at: u64,
    pub expires_at: u64,
    pub key_id: String,
    pub is_active: bool,
}

impl std::fmt::Debug for ManagedSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedSecret")
            .field("key_id", &self.key_id)
            .field("created_at", &self.created_at)
            .field("expires_at", &self.expires_at)
            .field("is_active", &self.is_active)
            .field("value", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug)]
pub struct SecretStore {
    secrets: Vec<ManagedSecret>,
}

impl SecretStore {
    pub fn new() -> Self {
        Self {
            secrets: Vec::new(),
        }
    }
}

pub fn create_secret(
    value: Vec<u8>,
    created_at: u64,
    lifetime_secs: u64,
) -> ManagedSecret {
    ManagedSecret {
        value,
        created_at,
        expires_at: created_at + lifetime_secs,
        key_id: format!("key-{}", created_at),
        is_active: true,
    }
}

pub fn load_secret_from_env(
    env_var_name: &str,
    created_at: u64,
    lifetime_secs: u64,
) -> Option<ManagedSecret> {
    match std::env::var(env_var_name) {
        Ok(val) if !val.is_empty() => Some(create_secret(val.into_bytes(), created_at, lifetime_secs)),
        _ => None,
    }
}

pub fn is_secret_expired(secret: &ManagedSecret, current_time: u64) -> bool {
    !secret.is_active || current_time >= secret.expires_at
}

pub fn rotate_secret(store: &mut SecretStore, new_secret: ManagedSecret) -> String {
    // Deactivate all existing active secrets
    for secret in &mut store.secrets {
        if secret.is_active {
            secret.is_active = false;
        }
    }

    let new_id = new_secret.key_id.clone();
    store.secrets.push(new_secret);
    new_id
}

pub fn get_active_secret(store: &SecretStore, current_time: u64) -> Option<&ManagedSecret> {
    store
        .secrets
        .iter()
        .find(|s| s.is_active && !is_secret_expired(s, current_time))
}

pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    // Always iterate over the maximum length to avoid length-based timing
    let max_len = a.len().max(b.len());
    let mut result = 0u8;

    // XOR each byte pair (using wrapping for different lengths)
    for i in 0..max_len {
        let a_byte = if i < a.len() { a[i] } else { 0u8 };
        let b_byte = if i < b.len() { b[i] } else { 0u8 };
        result |= a_byte ^ b_byte;
    }

    // Also factor in length difference
    if a.len() != b.len() {
        result |= 1;
    }

    result == 0
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SensitiveBuffer {
    pub data: Vec<u8>,
}

pub fn create_sensitive_buffer(data: Vec<u8>) -> SensitiveBuffer {
    SensitiveBuffer { data }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_secret_metadata() {
        let secret = create_secret(b"my-api-key".to_vec(), 1700000000, 86400);
        assert_eq!(secret.created_at, 1700000000);
        assert_eq!(secret.expires_at, 1700086400);
        assert!(secret.is_active);
        assert!(secret.key_id.starts_with("key-"));
    }

    #[test]
    fn test_secret_debug_redacted() {
        let secret = create_secret(b"super-secret".to_vec(), 1700000000, 86400);
        let debug_str = format!("{:?}", secret);
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("super-secret"));
    }

    #[test]
    fn test_is_secret_expired_not_expired() {
        let secret = create_secret(b"key".to_vec(), 1700000000, 86400);
        assert!(!is_secret_expired(&secret, 1700000001));
    }

    #[test]
    fn test_is_secret_expired_expired() {
        let secret = create_secret(b"key".to_vec(), 1700000000, 86400);
        assert!(is_secret_expired(&secret, 1700086401));
    }

    #[test]
    fn test_is_secret_expired_inactive() {
        let mut secret = create_secret(b"key".to_vec(), 1700000000, 86400);
        secret.is_active = false;
        assert!(is_secret_expired(&secret, 1700000001));
    }

    #[test]
    fn test_rotate_secret_deactivates_old() {
        let mut store = SecretStore::new();
        let old = create_secret(b"old-key".to_vec(), 1700000000, 86400);
        let old_id = old.key_id.clone();
        store.secrets.push(old);

        let new = create_secret(b"new-key".to_vec(), 1700001000, 86400);
        let new_id = rotate_secret(&mut store, new);

        assert_ne!(new_id, old_id);
        let old_secret = store
            .secrets
            .iter()
            .find(|s| s.key_id == old_id)
            .unwrap();
        assert!(!old_secret.is_active);
        let new_secret = store
            .secrets
            .iter()
            .find(|s| s.key_id == new_id)
            .unwrap();
        assert!(new_secret.is_active);
    }

    #[test]
    fn test_get_active_secret() {
        let mut store = SecretStore::new();
        let mut secret = create_secret(b"key".to_vec(), 1700000000, 86400);
        secret.is_active = false;
        store.secrets.push(secret);

        let active = create_secret(b"active-key".to_vec(), 1700001000, 86400);
        store.secrets.push(active);

        let result = get_active_secret(&store, 1700001001);
        assert!(result.is_some());
        assert_eq!(result.unwrap().key_id, store.secrets[1].key_id);
    }

    #[test]
    fn test_get_active_secret_none() {
        let store = SecretStore::new();
        assert!(get_active_secret(&store, 1700000000).is_none());
    }

    #[test]
    fn test_constant_time_eq_same() {
        assert!(constant_time_eq(b"secret", b"secret"));
    }

    #[test]
    fn test_constant_time_eq_different() {
        assert!(!constant_time_eq(b"secret", b"differ"));
    }

    #[test]
    fn test_constant_time_eq_different_lengths() {
        assert!(!constant_time_eq(b"short", b"much-longer-value"));
    }

    #[test]
    fn test_sensitive_buffer_creation() {
        let buf = create_sensitive_buffer(b"sensitive data".to_vec());
        assert_eq!(buf.data, b"sensitive data");
    }

    #[test]
    fn test_load_secret_from_env_missing() {
        let result = load_secret_from_env("NONEXISTENT_SECRET_VAR_12345", 1700000000, 86400);
        assert!(result.is_none());
    }
}
