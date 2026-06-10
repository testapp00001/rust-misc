//! # Lesson 10: Secure Configuration Files (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonOsRng, SaltString},
    Argon2, Algorithm, Params, Version,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A configuration entry (key-value pair)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub sensitive: bool,
}

/// Separate sensitive and non-sensitive config entries.
///
/// Non-sensitive entries can be stored in plaintext (e.g., hostnames, ports).
/// Sensitive entries (passwords, API keys) are returned separately for encryption.
pub fn separate_config(entries: &[ConfigEntry]) -> (String, HashMap<String, String>) {
    let mut plaintext_parts = Vec::new();
    let mut secrets = HashMap::new();

    for entry in entries {
        if entry.sensitive {
            secrets.insert(entry.key.clone(), entry.value.clone());
        } else {
            plaintext_parts.push(format!("{}={}", entry.key, entry.value));
        }
    }

    (plaintext_parts.join("\n"), secrets)
}

/// Derive an encryption key from a config password.
///
/// Uses Argon2id with moderate parameters suitable for config file encryption
/// (faster than password hashing since it happens at startup).
pub fn derive_config_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let params = Params::new(65536, 3, 1, Some(32))
        .map_err(|e| format!("Invalid params: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    Ok(output)
}

/// Encrypt sensitive config values.
///
/// Format: `[salt (16)] [nonce (12)] [ciphertext + tag]`
///
/// The salt is included so the key can be re-derived during decryption.
/// All sensitive values are serialized to JSON and encrypted as a single blob.
pub fn encrypt_config(password: &str, secrets: &HashMap<String, String>) -> Result<Vec<u8>, String> {
    // Generate random salt
    let salt = SaltString::generate(&mut ArgonOsRng);
    let salt_bytes = salt.as_str().as_bytes();

    // Derive key
    let key_bytes = derive_config_key(password, &salt_bytes[..16.min(salt_bytes.len())])?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

    // Serialize secrets to JSON
    let plaintext = serde_json::to_vec(secrets)
        .map_err(|e| format!("Serialization failed: {}", e))?;

    // Encrypt
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_slice())
        .map_err(|e| format!("Encryption failed: {:?}", e))?;

    // Build output: [salt][nonce][ciphertext]
    let mut output = Vec::with_capacity(salt_bytes.len() + 12 + ciphertext.len());
    output.extend_from_slice(&salt_bytes[..16.min(salt_bytes.len())]);
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypt sensitive config values.
///
/// Extracts the salt, re-derives the key, and decrypts the secrets.
pub fn decrypt_config(
    password: &str,
    encrypted: &[u8],
) -> Result<HashMap<String, String>, String> {
    if encrypted.len() < 28 {
        return Err("Encrypted data too short".to_string());
    }

    // Extract salt (first 16 bytes)
    let salt = &encrypted[..16];

    // Extract nonce (next 12 bytes)
    let nonce = Nonce::from_slice(&encrypted[16..28]);

    // Derive key
    let key_bytes = derive_config_key(password, salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

    // Decrypt
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher
        .decrypt(nonce, &encrypted[28..])
        .map_err(|e| format!("Decryption failed: {:?}", e))?;

    // Deserialize
    serde_json::from_slice(&plaintext)
        .map_err(|e| format!("Deserialization failed: {}", e))
}

/// Create a complete encrypted config file.
///
/// Combines plaintext config with encrypted sensitive values into a
/// JSON structure that can be stored as a single file.
pub fn create_secure_config(
    password: &str,
    entries: &[ConfigEntry],
) -> Result<String, String> {
    let (plaintext_config, secrets) = separate_config(entries);

    let encrypted = if secrets.is_empty() {
        Vec::new()
    } else {
        encrypt_config(password, &secrets)?
    };

    use base64::Engine;
    let config = serde_json::json!({
        "version": 1,
        "plaintext": plaintext_config,
        "encrypted": base64::engine::general_purpose::STANDARD.encode(&encrypted),
    });

    serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Config serialization failed: {}", e))
}

/// Load and decrypt a secure config file.
///
/// Parses the JSON, decrypts the sensitive values, and returns
/// all config entries merged together.
pub fn load_secure_config(
    password: &str,
    config_json: &str,
) -> Result<Vec<ConfigEntry>, String> {
    use base64::Engine;

    let config: serde_json::Value = serde_json::from_str(config_json)
        .map_err(|e| format!("Config parse failed: {}", e))?;

    let mut entries = Vec::new();

    // Parse plaintext entries
    if let Some(plaintext) = config["plaintext"].as_str() {
        for line in plaintext.lines() {
            if let Some((key, value)) = line.split_once('=') {
                entries.push(ConfigEntry {
                    key: key.to_string(),
                    value: value.to_string(),
                    sensitive: false,
                });
            }
        }
    }

    // Decrypt sensitive entries
    let encrypted_b64 = config["encrypted"]
        .as_str()
        .unwrap_or("");

    if !encrypted_b64.is_empty() {
        let encrypted = base64::engine::general_purpose::STANDARD
            .decode(encrypted_b64)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        let secrets = decrypt_config(password, &encrypted)?;

        for (key, value) in secrets {
            entries.push(ConfigEntry {
                key,
                value,
                sensitive: true,
            });
        }
    }

    Ok(entries)
}

/// Rotate config encryption password.
///
/// Decrypts with the old password and re-encrypts with the new password.
/// This is a common security operation when the config password needs
/// to be changed (e.g., employee leaving the organization).
pub fn rotate_config_password(
    old_password: &str,
    new_password: &str,
    config_json: &str,
) -> Result<String, String> {
    use base64::Engine;

    let config: serde_json::Value = serde_json::from_str(config_json)
        .map_err(|e| format!("Config parse failed: {}", e))?;

    let encrypted_b64 = config["encrypted"]
        .as_str()
        .ok_or("No encrypted section found")?;

    let encrypted = base64::engine::general_purpose::STANDARD
        .decode(encrypted_b64)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    // Decrypt with old password
    let secrets = decrypt_config(old_password, &encrypted)?;

    // Re-encrypt with new password
    let new_encrypted = encrypt_config(new_password, &secrets)?;

    // Build new config
    let plaintext = config["plaintext"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let new_config = serde_json::json!({
        "version": 1,
        "plaintext": plaintext,
        "encrypted": base64::engine::general_purpose::STANDARD.encode(&new_encrypted),
    });

    serde_json::to_string_pretty(&new_config)
        .map_err(|e| format!("Config serialization failed: {}", e))
}

/// Validate that a config file doesn't contain plaintext secrets.
///
/// Checks for patterns that indicate secrets in the plaintext section:
/// - Keys containing "password", "secret", "key", "token", "credential"
/// - Values that look like API keys or tokens
pub fn validate_no_plaintext_secrets(config_json: &str) -> Result<Vec<String>, String> {
    let config: serde_json::Value = serde_json::from_str(config_json)
        .map_err(|e| format!("Config parse failed: {}", e))?;

    let mut warnings = Vec::new();

    let plaintext = config["plaintext"]
        .as_str()
        .unwrap_or("");

    let secret_keywords = ["password", "secret", "key", "token", "credential", "api_key", "apikey"];

    for line in plaintext.lines() {
        if let Some((key, _value)) = line.split_once('=') {
            let lower_key = key.to_lowercase();
            if secret_keywords.iter().any(|kw| lower_key.contains(kw)) {
                warnings.push(format!("Plaintext secret detected in key: '{}'", key));
            }
        }
    }

    Ok(warnings)
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
            "plaintext": "password=secret123\nhost=localhost",
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
