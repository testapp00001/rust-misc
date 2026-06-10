//! # Lesson 04: Entry Management (Reference Solution)
//!
//! See the exercise file for full documentation on CRUD operations for vault entries.

use serde::{Deserialize, Serialize};
use chrono::Utc;
use zeroize::Zeroize;

/// A single password entry in the vault.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct VaultEntry {
    pub name: String,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub modified_at: String,
}

/// The vault data structure, containing all entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultData {
    pub version: u32,
    pub entries: Vec<VaultEntry>,
    pub last_modified: String,
}

impl VaultData {
    pub fn new() -> Self {
        Self {
            version: 1,
            entries: Vec::new(),
            last_modified: Utc::now().to_rfc3339(),
        }
    }
}

/// Add a new entry to the vault.
pub fn add_entry(
    vault: &mut VaultData,
    name: String,
    username: String,
    password: String,
    url: Option<String>,
    notes: Option<String>,
) -> Result<(), String> {
    if vault.entries.iter().any(|e| e.name == name) {
        return Err(format!("Entry '{}' already exists", name));
    }
    let now = Utc::now().to_rfc3339();
    let entry = VaultEntry {
        name,
        username,
        password,
        url,
        notes,
        created_at: now.clone(),
        modified_at: now,
    };
    vault.entries.push(entry);
    vault.last_modified = Utc::now().to_rfc3339();
    Ok(())
}

/// Retrieve an entry by name.
pub fn get_entry<'a>(vault: &'a VaultData, name: &str) -> Option<&'a VaultEntry> {
    vault.entries.iter().find(|e| e.name == name)
}

/// Update an existing entry.
pub fn update_entry(
    vault: &mut VaultData,
    name: &str,
    new_username: Option<String>,
    new_password: Option<String>,
    new_url: Option<Option<String>>,
    new_notes: Option<Option<String>>,
) -> Result<(), String> {
    let entry = vault
        .entries
        .iter_mut()
        .find(|e| e.name == name)
        .ok_or_else(|| format!("Entry '{}' not found", name))?;
    if let Some(u) = new_username {
        entry.username = u;
    }
    if let Some(p) = new_password {
        entry.password = p;
    }
    if let Some(u) = new_url {
        entry.url = u;
    }
    if let Some(n) = new_notes {
        entry.notes = n;
    }
    entry.modified_at = Utc::now().to_rfc3339();
    vault.last_modified = Utc::now().to_rfc3339();
    Ok(())
}

/// Delete an entry by name.
pub fn delete_entry(vault: &mut VaultData, name: &str) -> Result<(), String> {
    let idx = vault
        .entries
        .iter()
        .position(|e| e.name == name)
        .ok_or_else(|| format!("Entry '{}' not found", name))?;
    vault.entries.remove(idx);
    vault.last_modified = Utc::now().to_rfc3339();
    Ok(())
}

/// Serialize vault data to JSON bytes.
pub fn serialize_vault(vault: &VaultData) -> Result<Vec<u8>, String> {
    serde_json::to_vec(vault).map_err(|e| e.to_string())
}

/// Deserialize vault data from JSON bytes.
pub fn deserialize_vault(data: &[u8]) -> Result<VaultData, String> {
    serde_json::from_slice(data).map_err(|e| e.to_string())
}

/// List all entry names.
pub fn list_entry_names(vault: &VaultData) -> Vec<String> {
    vault.entries.iter().map(|e| e.name.clone()).collect()
}

/// Count entries in the vault.
pub fn count_entries(vault: &VaultData) -> usize {
    vault.entries.len()
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
