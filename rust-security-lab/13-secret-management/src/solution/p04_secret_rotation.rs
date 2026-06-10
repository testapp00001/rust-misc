//! # Lesson 04: Secret Rotation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone)]
pub struct SecretVersion {
    pub value: String,
    pub version: u32,
    pub created_at: Instant,
    pub active: bool,
}

#[derive(Debug)]
pub struct RotatingSecretStore {
    secrets: HashMap<String, Vec<SecretVersion>>,
    max_versions: usize,
    max_age: Duration,
}

impl RotatingSecretStore {
    pub fn new(max_versions: usize, max_age: Duration) -> Self {
        Self {
            secrets: HashMap::new(),
            max_versions,
            max_age,
        }
    }

    pub fn get_current(&self, name: &str) -> Option<&SecretVersion> {
        self.secrets
            .get(name)
            .and_then(|versions| versions.iter().find(|v| v.active))
    }
}

pub fn rotate_secret(store: &mut RotatingSecretStore, name: &str, new_value: &str) -> u32 {
    let versions = store.secrets.entry(name.to_string()).or_default();

    // Deactivate all existing versions
    for v in versions.iter_mut() {
        v.active = false;
    }

    let new_version_num = versions.iter().map(|v| v.version).max().unwrap_or(0) + 1;
    let new_entry = SecretVersion {
        value: new_value.to_string(),
        version: new_version_num,
        created_at: Instant::now(),
        active: true,
    };

    versions.insert(0, new_entry);

    // Trim to max_versions
    versions.truncate(store.max_versions);

    new_version_num
}

pub fn needs_rotation(store: &RotatingSecretStore, name: &str) -> bool {
    match store.get_current(name) {
        Some(version) => version.created_at.elapsed() >= store.max_age,
        None => false,
    }
}

pub fn list_secrets_needing_rotation(store: &RotatingSecretStore) -> Vec<String> {
    store
        .secrets
        .keys()
        .filter(|name| needs_rotation(store, name))
        .cloned()
        .collect()
}

pub fn validate_secret(store: &RotatingSecretStore, name: &str, value: &str) -> bool {
    match store.secrets.get(name) {
        Some(versions) => versions.iter().any(|v| v.active && v.value == value),
        None => false,
    }
}

pub fn deactivate_old_versions(store: &mut RotatingSecretStore, name: &str, keep_count: usize) -> usize {
    match store.secrets.get_mut(name) {
        Some(versions) => {
            let mut deactivated = 0;
            let mut kept = 0;
            for v in versions.iter_mut() {
                if v.active {
                    if kept < keep_count {
                        kept += 1;
                    } else {
                        v.active = false;
                        deactivated += 1;
                    }
                }
            }
            deactivated
        }
        None => 0,
    }
}

pub fn get_version_history(store: &RotatingSecretStore, name: &str) -> Vec<(u32, bool, u64)> {
    match store.secrets.get(name) {
        Some(versions) => versions
            .iter()
            .map(|v| (v.version, v.active, v.created_at.elapsed().as_secs()))
            .collect(),
        None => Vec::new(),
    }
}

pub fn purge_inactive_versions(store: &mut RotatingSecretStore) -> usize {
    let mut total_purged = 0;
    for versions in store.secrets.values_mut() {
        let before = versions.len();
        versions.retain(|v| v.active);
        total_purged += before - versions.len();
    }
    total_purged
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_store() -> RotatingSecretStore {
        RotatingSecretStore::new(5, Duration::from_secs(3600))
    }

    #[test]
    fn test_rotate_secret_new() {
        let mut store = create_test_store();
        let version = rotate_secret(&mut store, "db_password", "old_pass");
        assert_eq!(version, 1);
        assert_eq!(store.get_current("db_password").unwrap().value, "old_pass");
    }

    #[test]
    fn test_rotate_secret_existing() {
        let mut store = create_test_store();
        rotate_secret(&mut store, "db_password", "old_pass");
        let version = rotate_secret(&mut store, "db_password", "new_pass");
        assert_eq!(version, 2);
        assert_eq!(store.get_current("db_password").unwrap().value, "new_pass");
    }

    #[test]
    fn test_validate_secret_current() {
        let mut store = create_test_store();
        rotate_secret(&mut store, "api_key", "key_v1");
        assert!(validate_secret(&store, "api_key", "key_v1"));
    }

    #[test]
    fn test_validate_secret_wrong_value() {
        let mut store = create_test_store();
        rotate_secret(&mut store, "api_key", "key_v1");
        assert!(!validate_secret(&store, "api_key", "wrong_key"));
    }

    #[test]
    fn test_validate_secret_nonexistent() {
        let store = create_test_store();
        assert!(!validate_secret(&store, "nonexistent", "anything"));
    }

    #[test]
    fn test_version_history() {
        let mut store = create_test_store();
        rotate_secret(&mut store, "db_password", "v1");
        rotate_secret(&mut store, "db_password", "v2");
        rotate_secret(&mut store, "db_password", "v3");

        let history = get_version_history(&store, "db_password");
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].0, 3);
        assert!(history[0].1);
        assert!(!history[1].1);
        assert!(!history[2].1);
    }

    #[test]
    fn test_deactivate_old_versions() {
        let mut store = create_test_store();
        rotate_secret(&mut store, "key", "v1");
        rotate_secret(&mut store, "key", "v2");
        rotate_secret(&mut store, "key", "v3");

        let purged = purge_inactive_versions(&mut store);
        assert_eq!(purged, 2);
        let history = get_version_history(&store, "key");
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_list_secrets_needing_rotation() {
        let mut store = RotatingSecretStore::new(5, Duration::from_millis(1));
        rotate_secret(&mut store, "fast_expiring", "value");
        std::thread::sleep(Duration::from_millis(10));
        let needing = list_secrets_needing_rotation(&store);
        assert!(needing.contains(&"fast_expiring".to_string()));
    }
}
