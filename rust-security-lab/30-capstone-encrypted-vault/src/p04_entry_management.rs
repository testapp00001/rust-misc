//! # Lesson 04: Entry Management
//!
//! ## Managing Encrypted Password Entries
//!
//! This lesson builds the CRUD (Create, Read, Update, Delete) operations for
//! vault entries. Each entry contains a name, username, password, URL, and notes.
//! All operations work on encrypted data -- the vault is decrypted only in memory,
//! modified, then re-encrypted.
//!
//! ## Data Model
//!
//! ```text
//! VaultEntry {
//!     name: "GitHub",           // unique identifier
//!     username: "user@email.com",
//!     password: "s3cret!",
//!     url: "https://github.com",
//!     notes: "2FA enabled",
//!     created_at: "2024-01-15T10:30:00Z",
//!     modified_at: "2024-06-01T14:22:00Z",
//! }
//! ```
//!
//! ## Serialization
//!
//! Entries are serialized to JSON, then encrypted as a single blob. This means
//! the entire vault is one ciphertext -- an attacker cannot see how many entries
//! exist or their individual sizes without the key.
//!
//! ## Naming Convention
//!
//! Entry names must be unique within the vault. This is enforced at the
//! application level, not the crypto level. Duplicate names would cause
//! ambiguity when retrieving entries.
//!
//! ## Security Considerations
//!
//! - Passwords are never stored in plaintext, even in memory (use `secrecy`)
//! - Deleted entries are overwritten with zeros before deallocation
//! - The vault is re-encrypted after every modification

use serde::{Deserialize, Serialize};
use chrono::Utc;
use zeroize::Zeroize;

/// A single password entry in the vault.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct VaultEntry {
    /// Unique name for this entry (e.g., "GitHub", "Gmail")
    pub name: String,
    /// Username or email for this entry
    pub username: String,
    /// The password (encrypted at rest, zeroized when dropped)
    pub password: String,
    /// Optional URL
    pub url: Option<String>,
    /// Optional notes
    pub notes: Option<String>,
    /// ISO 8601 timestamp of creation
    pub created_at: String,
    /// ISO 8601 timestamp of last modification
    pub modified_at: String,
}

/// The vault data structure, containing all entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultData {
    /// Vault format version
    pub version: u32,
    /// All entries
    pub entries: Vec<VaultEntry>,
    /// ISO 8601 timestamp of last save
    pub last_modified: String,
}

impl VaultData {
    /// Create a new empty vault.
    pub fn new() -> Self {
        Self {
            version: 1,
            entries: Vec::new(),
            last_modified: Utc::now().to_rfc3339(),
        }
    }
}

/// Exercise 1: Add a new entry to the vault.
///
/// Check for duplicate names. Update `last_modified` timestamp.
///
/// Hints:
/// - Check if an entry with the same name exists: `vault.entries.iter().any(|e| e.name == name)`
/// - Set created_at and modified_at to `Utc::now().to_rfc3339()`
/// - Push the new entry and update vault.last_modified
pub fn add_entry(
    vault: &mut VaultData,
    name: String,
    username: String,
    password: String,
    url: Option<String>,
    notes: Option<String>,
) -> Result<(), String> {
    todo!("Add a new entry to the vault")
}

/// Exercise 2: Retrieve an entry by name.
///
/// Returns a reference to the entry if found.
///
/// Hints:
/// - `vault.entries.iter().find(|e| e.name == name)`
pub fn get_entry<'a>(vault: &'a VaultData, name: &str) -> Option<&'a VaultEntry> {
    todo!("Retrieve an entry by name")
}

/// Exercise 3: Update an existing entry's password and username.
///
/// Updates the `modified_at` timestamp. Returns an error if the entry doesn't exist.
///
/// Hints:
/// - Find the entry: `vault.entries.iter_mut().find(|e| e.name == name)`
/// - Update fields and set modified_at
pub fn update_entry(
    vault: &mut VaultData,
    name: &str,
    new_username: Option<String>,
    new_password: Option<String>,
    new_url: Option<Option<String>>,
    new_notes: Option<Option<String>>,
) -> Result<(), String> {
    todo!("Update an existing entry")
}

/// Exercise 4: Delete an entry by name.
///
/// The entry's memory is zeroized before removal.
///
/// Hints:
/// - Find the index: `vault.entries.iter().position(|e| e.name == name)`
/// - Zeroize and remove: the `Zeroize` derive handles zeroing on drop
/// - Update vault.last_modified
pub fn delete_entry(vault: &mut VaultData, name: &str) -> Result<(), String> {
    todo!("Delete an entry by name")
}

/// Exercise 5: Serialize vault data to JSON bytes.
///
/// Hints:
/// - `serde_json::to_vec(vault).map_err(|e| e.to_string())`
pub fn serialize_vault(vault: &VaultData) -> Result<Vec<u8>, String> {
    todo!("Serialize vault to JSON bytes")
}

/// Exercise 6: Deserialize vault data from JSON bytes.
///
/// Hints:
/// - `serde_json::from_slice(bytes).map_err(|e| e.to_string())`
pub fn deserialize_vault(data: &[u8]) -> Result<VaultData, String> {
    todo!("Deserialize vault from JSON bytes")
}

/// Exercise 7: List all entry names (for display).
///
/// Hints:
/// - `vault.entries.iter().map(|e| e.name.clone()).collect()`
pub fn list_entry_names(vault: &VaultData) -> Vec<String> {
    todo!("List all entry names")
}

/// Exercise 8: Count entries in the vault.
///
/// Hints:
/// - `vault.entries.len()`
pub fn count_entries(vault: &VaultData) -> usize {
    todo!("Count entries")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(name: &str) -> (String, String, String, Option<String>, Option<String>) {
        (
            name.to_string(),
            "user@test.com".to_string(),
            "password123".to_string(),
            Some("https://example.com".to_string()),
            Some("test notes".to_string()),
        )
    }

    #[test]
    fn test_add_and_get_entry() {
        let mut vault = VaultData::new();
        let (name, user, pass, url, notes) = sample_entry("GitHub");
        add_entry(&mut vault, name.clone(), user, pass, url, notes).unwrap();
        let entry = get_entry(&vault, "GitHub").unwrap();
        assert_eq!(entry.name, "GitHub");
        assert_eq!(entry.username, "user@test.com");
    }

    #[test]
    fn test_add_duplicate_name() {
        let mut vault = VaultData::new();
        let (n, u, p, url, notes) = sample_entry("GitHub");
        add_entry(&mut vault, n.clone(), u.clone(), p.clone(), url.clone(), notes.clone()).unwrap();
        let result = add_entry(&mut vault, n, u, p, url, notes);
        assert!(result.is_err(), "Should reject duplicate names");
    }

    #[test]
    fn test_get_nonexistent() {
        let vault = VaultData::new();
        assert!(get_entry(&vault, "DoesNotExist").is_none());
    }

    #[test]
    fn test_update_entry() {
        let mut vault = VaultData::new();
        let (n, u, p, url, notes) = sample_entry("GitHub");
        add_entry(&mut vault, n, u, p, url, notes).unwrap();
        update_entry(&mut vault, "GitHub", Some("new@email.com".into()), Some("newpass".into()), None, None).unwrap();
        let entry = get_entry(&vault, "GitHub").unwrap();
        assert_eq!(entry.username, "new@email.com");
        assert_eq!(entry.password, "newpass");
    }

    #[test]
    fn test_update_nonexistent() {
        let mut vault = VaultData::new();
        assert!(update_entry(&mut vault, "Nope", Some("x".into()), None, None, None).is_err());
    }

    #[test]
    fn test_delete_entry() {
        let mut vault = VaultData::new();
        let (n, u, p, url, notes) = sample_entry("GitHub");
        add_entry(&mut vault, n, u, p, url, notes).unwrap();
        assert_eq!(count_entries(&vault), 1);
        delete_entry(&mut vault, "GitHub").unwrap();
        assert_eq!(count_entries(&vault), 0);
        assert!(get_entry(&vault, "GitHub").is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut vault = VaultData::new();
        assert!(delete_entry(&mut vault, "Nope").is_err());
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let mut vault = VaultData::new();
        let (n, u, p, url, notes) = sample_entry("Test");
        add_entry(&mut vault, n, u, p, url, notes).unwrap();
        let bytes = serialize_vault(&vault).unwrap();
        let restored = deserialize_vault(&bytes).unwrap();
        assert_eq!(restored.entries.len(), 1);
        assert_eq!(restored.entries[0].name, "Test");
    }

    #[test]
    fn test_list_entry_names() {
        let mut vault = VaultData::new();
        add_entry(&mut vault, "A".into(), "u".into(), "p".into(), None, None).unwrap();
        add_entry(&mut vault, "B".into(), "u".into(), "p".into(), None, None).unwrap();
        add_entry(&mut vault, "C".into(), "u".into(), "p".into(), None, None).unwrap();
        let names = list_entry_names(&vault);
        assert_eq!(names, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_count_entries_empty() {
        let vault = VaultData::new();
        assert_eq!(count_entries(&vault), 0);
    }
}
