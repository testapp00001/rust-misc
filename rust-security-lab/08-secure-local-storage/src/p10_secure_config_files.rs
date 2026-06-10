//! # Lesson 10: Secure Configuration Files
//!
//! ## The Problem: Secrets in Config Files
//!
//! Applications often store sensitive configuration in files:
//! - Database passwords
//! - API keys
//! - TLS certificates and private keys
//! - Encryption keys
//!
//! These files are frequently:
//! - Committed to version control (accidentally)
//! - Left with world-readable permissions
//! - Stored in plaintext on disk
//! - Included in backups and logs
//!
//! ## Attack Scenario: Config File Exposure
//!
//! 1. Developer commits `.env` file to Git repository
//! 2. Repository is public (or becomes public)
//! 3. Automated scanners find the API keys within minutes
//! 4. Attacker uses the keys to access production systems
//!
//! ## Secure Config Strategies
//!
//! ### Strategy 1: Environment Variables
//! Store secrets as environment variables, not in files.
//!
//! ### Strategy 2: Encrypted Config Files
//! Encrypt the config file, decrypt at runtime with a key from the keychain.
//!
//! ### Strategy 3: Secret Managers
//! Use HashiCorp Vault, AWS Secrets Manager, etc.
//!
//! ### Strategy 4: Encrypted Config with Password
//! User provides a password at startup, which decrypts the config.
//!
//! ## This Lesson
//!
//! We implement Strategy 4: encrypted config files with password-based
//! key derivation. The config is encrypted with AES-256-GCM, and the
//! key is derived from a user password using Argon2id.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonOsRng, PasswordHasher, SaltString},
    Argon2, Params,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A configuration entry (key-value pair)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigEntry {
    /// Configuration key (e.g., "db_password")
    pub key: String,
    /// Configuration value (the secret)
    pub value: String,
    /// Whether this entry is sensitive (should be encrypted)
    pub sensitive: bool,
}

/// Exercise 1: Create a config file with mixed sensitive/insensitive entries.
///
/// Some config values are secrets (passwords, API keys), others are
/// not sensitive (hostnames, ports). This function separates them.
///
/// # Arguments
/// * `entries` - List of config entries
///
/// # Returns
/// A tuple of (plaintext_config_json, sensitive_keys_to_values)
///
/// # Hints
/// - Non-sensitive entries go into plaintext JSON
/// - Sensitive entries are returned separately for encryption
pub fn separate_config(entries: &[ConfigEntry]) -> (String, HashMap<String, String>) {
    todo!("Separate sensitive and non-sensitive config entries")
}

/// Exercise 2: Derive an encryption key from a config password.
///
/// Use Argon2id to derive a 32-byte key from the provided password.
/// The salt is stored alongside the encrypted config.
///
/// # Arguments
/// * `password` - The config encryption password
/// * `salt` - Random salt (16 bytes)
///
/// # Returns
/// 32-byte encryption key
///
/// # Hints
/// - Use Argon2id with moderate parameters (config decryption happens at startup)
/// - Parameters: 64MB memory, 3 iterations, 1 thread
pub fn derive_config_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    todo!("Derive config encryption key from password")
}

/// Exercise 3: Encrypt sensitive config values.
///
/// Takes a map of sensitive key-value pairs and encrypts them as a single blob.
///
/// # Arguments
/// * `password` - The config encryption password
/// * `secrets` - Map of key → secret value
///
/// # Returns
/// Encrypted blob with format: `[salt (16)] [nonce (12)] [ciphertext + tag]`
///
/// # Hints
/// - Generate a random salt
/// - Derive key from password + salt
/// - Serialize secrets to JSON
/// - Encrypt with AES-256-GCM
/// - Prepend salt + nonce to ciphertext
pub fn encrypt_config(password: &str, secrets: &HashMap<String, String>) -> Result<Vec<u8>, String> {
    todo!("Encrypt sensitive config values with password-derived key")
}

/// Exercise 4: Decrypt sensitive config values.
///
/// # Arguments
/// * `password` - The config encryption password
/// * `encrypted` - The encrypted config blob
///
/// # Returns
/// Map of key → decrypted secret value
///
/// # Hints
/// - Extract salt (first 16 bytes)
/// - Extract nonce (next 12 bytes)
/// - Derive key from password + salt
/// - Decrypt the rest
/// - Deserialize JSON
pub fn decrypt_config(
    password: &str,
    encrypted: &[u8],
) -> Result<HashMap<String, String>, String> {
    todo!("Decrypt sensitive config values")
}

/// Exercise 5: Create a complete encrypted config file.
///
/// Combines non-sensitive config (plaintext) with encrypted sensitive config
/// into a single file:
///
/// ```json
/// {
///   "version": 1,
///   "plaintext": { "host": "localhost", "port": "5432" },
///   "encrypted": "<base64-encoded-encrypted-blob>"
/// }
/// ```
///
/// # Hints
/// - Use `separate_config()` to split entries
/// - Use `encrypt_config()` for sensitive values
/// - Combine into a JSON structure
/// - Encode encrypted blob as base64
pub fn create_secure_config(
    password: &str,
    entries: &[ConfigEntry],
) -> Result<String, String> {
    todo!("Create a complete encrypted config file")
}

/// Exercise 6: Load and decrypt a secure config file.
///
/// Parses the JSON, decrypts the sensitive values, and returns
/// all config entries (both plaintext and decrypted).
///
/// # Hints
/// - Parse the JSON structure
/// - Decrypt the encrypted portion
/// - Merge plaintext and decrypted entries
pub fn load_secure_config(
    password: &str,
    config_json: &str,
) -> Result<Vec<ConfigEntry>, String> {
    todo!("Load and decrypt a secure config file")
}

/// Exercise 7: Rotate config encryption password.
///
/// Re-encrypt all sensitive config values with a new password.
///
/// # Arguments
/// * `old_password` - Current password
/// * `new_password` - New password
/// * `config_json` - Current encrypted config
///
/// # Returns
/// New encrypted config with the new password
///
/// # Hints
/// - Decrypt with old password
/// - Re-encrypt with new password
pub fn rotate_config_password(
    old_password: &str,
    new_password: &str,
    config_json: &str,
) -> Result<String, String> {
    todo!("Rotate config encryption password")
}

/// Exercise 8: Validate that a config file doesn't contain plaintext secrets.
///
/// Check that common secret patterns don't appear in the plaintext portion:
/// - API keys (long alphanumeric strings)
/// - Passwords (in keys like "password", "secret", "key", "token")
/// - Connection strings with embedded passwords
///
/// # Hints
/// - Parse the config JSON
/// - Check plaintext keys against a blocklist
/// - Check values for patterns that look like secrets
pub fn validate_no_plaintext_secrets(config_json: &str) -> Result<Vec<String>, String> {
    todo!("Validate that config doesn't contain plaintext secrets")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entries() -> Vec<ConfigEntry> {
        vec![
            ConfigEntry { key: "db_host".to_string(), value: "localhost".to_string(), sensitive: false },
            ConfigEntry { key: "db_port".to_string(), value: "5432".to_string(), sensitive: false },
            ConfigEntry { key: "db_password".to_string(), value: "super_secret_123".to_string(), sensitive: true },
            ConfigEntry { key: "api_key".to_string(), value: "sk-1234567890abcdef".to_string(), sensitive: true },
        ]
    }

    #[test]
    fn test_separate_config() {
        let entries = test_entries();
        let (plaintext, secrets) = separate_config(&entries);

        // Plaintext should contain non-sensitive entries
        assert!(plaintext.contains("db_host"));
        assert!(plaintext.contains("5432"));

        // Secrets should contain sensitive entries
        assert!(secrets.contains_key("db_password"));
        assert!(secrets.contains_key("api_key"));
        assert_eq!(secrets["db_password"], "super_secret_123");
    }

    #[test]
    fn test_derive_config_key() {
        let salt = b"0123456789abcdef";
        let key = derive_config_key("test_password", salt).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_encrypt_decrypt_config() {
        let password = "config_password_123";
        let mut secrets = HashMap::new();
        secrets.insert("db_password".to_string(), "secret123".to_string());
        secrets.insert("api_key".to_string(), "sk-abcdef".to_string());

        let encrypted = encrypt_config(password, &secrets).unwrap();
        let decrypted = decrypt_config(password, &encrypted).unwrap();

        assert_eq!(decrypted, secrets);
    }

    #[test]
    fn test_encrypt_decrypt_wrong_password() {
        let mut secrets = HashMap::new();
        secrets.insert("key".to_string(), "value".to_string());

        let encrypted = encrypt_config("correct_password", &secrets).unwrap();
        let result = decrypt_config("wrong_password", &encrypted);

        assert!(result.is_err(), "Wrong password should fail decryption");
    }

    #[test]
    fn test_create_and_load_secure_config() {
        let password = "my_config_pass";
        let entries = test_entries();

        let config_json = create_secure_config(password, &entries).unwrap();

        let loaded = load_secure_config(password, &config_json).unwrap();
        assert_eq!(loaded.len(), entries.len());

        // Check that all entries are present
        for entry in &entries {
            let found = loaded.iter().find(|e| e.key == entry.key);
            assert!(found.is_some(), "Missing entry: {}", entry.key);
            assert_eq!(found.unwrap().value, entry.value);
        }
    }

    #[test]
    fn test_rotate_config_password() {
        let entries = test_entries();
        let config_json = create_secure_config("old_pass", &entries).unwrap();

        let new_config = rotate_config_password("old_pass", "new_pass", &config_json).unwrap();

        // Should be decryptable with new password
        let loaded = load_secure_config("new_pass", &new_config).unwrap();
        assert_eq!(loaded.len(), entries.len());

        // Should NOT be decryptable with old password
        let result = load_secure_config("old_pass", &new_config);
        assert!(result.is_err(), "Old password should not work on new config");
    }

    #[test]
    fn test_validate_no_plaintext_secrets() {
        // Config with secrets in plaintext
        let bad_config = r#"{
            "version": 1,
            "plaintext": {"password": "secret123", "host": "localhost"},
            "encrypted": ""
        }"#;

        let warnings = validate_no_plaintext_secrets(bad_config).unwrap();
        assert!(!warnings.is_empty(), "Should detect plaintext secret");
    }

    #[test]
    fn test_config_different_encryption_each_time() {
        let password = "test_pass";
        let mut secrets = HashMap::new();
        secrets.insert("key".to_string(), "value".to_string());

        let enc1 = encrypt_config(password, &secrets).unwrap();
        let enc2 = encrypt_config(password, &secrets).unwrap();

        // Different salt/nonce → different ciphertext
        assert_ne!(enc1, enc2);
    }
}
