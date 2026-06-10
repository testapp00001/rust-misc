//! # Lesson 08: OS Keychain Integration
//!
//! ## Why Use the OS Keychain?
//!
//! Storing encryption keys in files is risky — they can be copied, accidentally
//! committed to version control, or left in backups. OS keychains provide
//! secure, hardware-backed storage for secrets:
//!
//! | OS | Keychain | Security |
//! |----|----------|----------|
//! | macOS | Keychain | T2/Secure Enclave, user password required |
//! | Windows | Credential Manager | DPAPI, tied to user account |
//! | Linux | GNOME Keyring / KDE Wallet | D-Bus Secret Service |
//!
//! ## How OS Keychains Work
//!
//! 1. **Store**: Your app asks the OS to store a secret with a unique identifier
//! 2. **Retrieve**: Your app asks for the secret by identifier
//! 3. **Delete**: Your app can remove stored secrets
//!
//! The OS encrypts the keychain with the user's login password (or hardware key).
//! Secrets are only accessible when the user is logged in.
//!
//! ## Attack Scenario: Key in File
//!
//! 1. App stores API key in `~/.myapp/config.json`
//! 2. Attacker gains read access to the filesystem
//! 3. Attacker reads the API key directly
//!
//! With OS keychain:
//! 1. App stores API key in the OS keychain
//! 2. Attacker gains read access to the filesystem
//! 3. API key is NOT in any file — only accessible via the keychain API
//!
//! ## Rust Crates for Keychain Access
//!
//! - `keyring` crate: Cross-platform keychain access
//! - `secret-service` crate: Linux D-Bus Secret Service
//! - `security-framework` crate: macOS Keychain (lower level)
//!
//! ## Limitations
//!
//! - **Not portable**: Keys are tied to the user account on a specific machine
//! - **No backup**: If the OS keychain is lost, secrets are gone
//! - **Testing**: Hard to test without a real keychain (mock it)
//! - **CI/CD**: No keychain in headless environments

use std::collections::HashMap;

/// Simulated keychain for testing and learning.
/// In production, use the `keyring` crate for real OS keychain access.
///
/// This simulates the OS keychain API to teach the concepts without
/// requiring actual keychain access (which requires a GUI session).
#[derive(Debug)]
pub struct SimulatedKeychain {
    /// Service/namespace for the keychain entries
    service: String,
    /// Stored secrets: (service, account) -> secret
    entries: HashMap<String, String>,
}

impl SimulatedKeychain {
    /// Exercise 1: Create a new simulated keychain.
    ///
    /// The service name identifies your application's namespace in the keychain.
    ///
    /// # Arguments
    /// * `service` - Service name (e.g., "com.myapp.secrets")
    pub fn new(service: &str) -> Self {
        todo!("Create a new SimulatedKeychain")
    }

    /// Exercise 2: Store a secret in the keychain.
    ///
    /// In a real keychain, this would encrypt the secret and store it
    /// in the OS-managed secure storage.
    ///
    /// # Arguments
    /// * `account` - The account/username identifier
    /// * `secret` - The secret value to store
    ///
    /// # Hints
    /// - Use `(service, account)` as the key
    /// - Store the secret in the HashMap
    pub fn set_secret(&mut self, account: &str, secret: &str) -> Result<(), String> {
        todo!("Store a secret in the keychain")
    }

    /// Exercise 3: Retrieve a secret from the keychain.
    ///
    /// # Arguments
    /// * `account` - The account/username identifier
    ///
    /// # Hints
    /// - Look up the secret by `(service, account)`
    /// - Return `None` if not found
    pub fn get_secret(&self, account: &str) -> Option<String> {
        todo!("Retrieve a secret from the keychain")
    }

    /// Exercise 4: Delete a secret from the keychain.
    ///
    /// # Arguments
    /// * `account` - The account/username identifier
    pub fn delete_secret(&mut self, account: &str) -> Result<(), String> {
        todo!("Delete a secret from the keychain")
    }

    /// Exercise 5: List all accounts stored in the keychain.
    ///
    /// Returns a list of account names (not the secrets themselves).
    pub fn list_accounts(&self) -> Vec<String> {
        todo!("List all account names in the keychain")
    }

    /// Exercise 6: Check if a secret exists in the keychain.
    pub fn has_secret(&self, account: &str) -> bool {
        todo!("Check if a secret exists for the given account")
    }
}

/// Exercise 7: Store an encryption key in the keychain.
///
/// This demonstrates the recommended pattern: generate a key, store it
/// in the keychain, and use it for file encryption.
///
/// # Arguments
/// * `keychain` - The keychain to store the key in
/// * `key_name` - Name for the key entry
///
/// # Returns
/// The generated key as a hex string
///
/// # Hints
/// - Generate a random 32-byte key
/// - Encode as hex
/// - Store in the keychain
pub fn store_encryption_key(keychain: &mut SimulatedKeychain, key_name: &str) -> String {
    todo!("Generate and store an encryption key in the keychain")
}

/// Exercise 8: Retrieve an encryption key from the keychain.
///
/// # Arguments
/// * `keychain` - The keychain to retrieve from
/// * `key_name` - Name of the key entry
///
/// # Returns
/// The key as a Vec<u8>, or None if not found
pub fn retrieve_encryption_key(
    keychain: &SimulatedKeychain,
    key_name: &str,
) -> Option<Vec<u8>> {
    todo!("Retrieve and decode an encryption key from the keychain")
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
