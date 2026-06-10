//! # Lesson 08: OS Keychain Integration (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;

/// Simulated keychain for testing and learning.
///
/// In production, you would use the `keyring` crate:
/// ```rust,no_run
/// let entry = keyring::Entry::new("com.myapp", "user@example.com")?;
/// entry.set_password("secret")?;
/// let password = entry.get_password()?;
/// ```
///
/// This simulation teaches the concepts without requiring a GUI session.
#[derive(Debug)]
pub struct SimulatedKeychain {
    service: String,
    entries: HashMap<String, String>,
}

impl SimulatedKeychain {
    /// Create a new simulated keychain.
    ///
    /// The service name acts as a namespace, preventing collisions between
    /// different applications using the same keychain.
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_string(),
            entries: HashMap::new(),
        }
    }

    /// Store a secret in the keychain.
    ///
    /// In a real keychain, this would encrypt the secret with the user's
    /// login password (or hardware key) and store it in the OS-managed
    /// secure storage.
    pub fn set_secret(&mut self, account: &str, secret: &str) -> Result<(), String> {
        let key = format!("{}:{}", self.service, account);
        self.entries.insert(key, secret.to_string());
        Ok(())
    }

    /// Retrieve a secret from the keychain.
    ///
    /// Returns `None` if no secret is stored for the given account.
    pub fn get_secret(&self, account: &str) -> Option<String> {
        let key = format!("{}:{}", self.service, account);
        self.entries.get(&key).cloned()
    }

    /// Delete a secret from the keychain.
    pub fn delete_secret(&mut self, account: &str) -> Result<(), String> {
        let key = format!("{}:{}", self.service, account);
        self.entries.remove(&key);
        Ok(())
    }

    /// List all accounts stored in the keychain.
    ///
    /// Returns account names only — not the secrets themselves.
    /// Strips the service prefix to return clean account names.
    pub fn list_accounts(&self) -> Vec<String> {
        let prefix = format!("{}:", self.service);
        self.entries
            .keys()
            .filter_map(|k| k.strip_prefix(&prefix).map(|s| s.to_string()))
            .collect()
    }

    /// Check if a secret exists for the given account.
    pub fn has_secret(&self, account: &str) -> bool {
        let key = format!("{}:{}", self.service, account);
        self.entries.contains_key(&key)
    }
}

/// Store an encryption key in the keychain.
///
/// Generates a random 32-byte key and stores it as a hex string.
/// This is the recommended pattern: generate a strong key and store
/// it in the keychain rather than deriving it from a password each time.
pub fn store_encryption_key(keychain: &mut SimulatedKeychain, key_name: &str) -> String {
    use rand::Rng;
    let key: [u8; 32] = rand::thread_rng().gen();
    let key_hex = hex::encode(key);
    keychain
        .set_secret(key_name, &key_hex)
        .expect("Failed to store key");
    key_hex
}

/// Retrieve an encryption key from the keychain.
///
/// Decodes the hex-encoded key back to bytes for use with
/// encryption algorithms.
pub fn retrieve_encryption_key(
    keychain: &SimulatedKeychain,
    key_name: &str,
) -> Option<Vec<u8>> {
    let key_hex = keychain.get_secret(key_name)?;
    hex::decode(key_hex).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_keychain() {
        let kc = SimulatedKeychain::new("com.test.app");
        assert_eq!(kc.service, "com.test.app");
        assert!(kc.list_accounts().is_empty());
    }

    #[test]
    fn test_set_and_get_secret() {
        let mut kc = SimulatedKeychain::new("com.test.app");
        kc.set_secret("user@example.com", "super_secret_api_key").unwrap();

        let secret = kc.get_secret("user@example.com");
        assert_eq!(secret, Some("super_secret_api_key".to_string()));
    }

    #[test]
    fn test_get_nonexistent() {
        let kc = SimulatedKeychain::new("com.test.app");
        assert!(kc.get_secret("nobody").is_none());
    }

    #[test]
    fn test_delete_secret() {
        let mut kc = SimulatedKeychain::new("com.test.app");
        kc.set_secret("user", "secret").unwrap();
        assert!(kc.get_secret("user").is_some());

        kc.delete_secret("user").unwrap();
        assert!(kc.get_secret("user").is_none());
    }

    #[test]
    fn test_list_accounts() {
        let mut kc = SimulatedKeychain::new("com.test.app");
        kc.set_secret("alice", "secret1").unwrap();
        kc.set_secret("bob", "secret2").unwrap();
        kc.set_secret("charlie", "secret3").unwrap();

        let accounts = kc.list_accounts();
        assert_eq!(accounts.len(), 3);
        assert!(accounts.contains(&"alice".to_string()));
        assert!(accounts.contains(&"bob".to_string()));
    }

    #[test]
    fn test_has_secret() {
        let mut kc = SimulatedKeychain::new("com.test.app");
        assert!(!kc.has_secret("user"));

        kc.set_secret("user", "secret").unwrap();
        assert!(kc.has_secret("user"));

        kc.delete_secret("user").unwrap();
        assert!(!kc.has_secret("user"));
    }

    #[test]
    fn test_store_and_retrieve_key() {
        let mut kc = SimulatedKeychain::new("com.test.keys");
        let key_hex = store_encryption_key(&mut kc, "file_key");
        assert_eq!(key_hex.len(), 64, "Hex key should be 64 chars (32 bytes)");

        let key_bytes = retrieve_encryption_key(&kc, "file_key").unwrap();
        assert_eq!(key_bytes.len(), 32, "Key should be 32 bytes");
        assert_eq!(hex::encode(&key_bytes), key_hex);
    }

    #[test]
    fn test_retrieve_nonexistent_key() {
        let kc = SimulatedKeychain::new("com.test.keys");
        assert!(retrieve_encryption_key(&kc, "missing_key").is_none());
    }
}
