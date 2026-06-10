//! # Lesson 04: Secret Rotation
//!
//! ## Why Rotate Secrets?
//!
//! Secrets should not live forever. Regular rotation limits the window of exposure
//! if a secret is compromised. Many compliance frameworks (PCI-DSS, SOC2, HIPAA)
//! mandate periodic rotation.
//!
//! ## The Rotation Problem
//!
//! Rotating a secret is not as simple as "change the password":
//!
//! 1. **Multiple consumers**: Many services may use the same secret
//! 2. **Zero-downtime**: You can't just change a DB password while apps are running
//! 3. **Atomicity**: All consumers must switch to the new secret, or none
//! 4. **Rollback**: If the new secret fails, you need to roll back quickly
//! 5. **Overlap period**: Old and new secrets must both work during transition
//!
//! ## Rotation Strategies
//!
//! - **Big bang**: Change the secret, restart all consumers (downtime risk)
//! - **Blue/green**: Deploy new secret alongside old, switch traffic
//! - **Overlap**: Accept both old and new secrets during a transition window
//! - **Versioned**: Keep multiple versions, retire the oldest
//!
//! ## Attack: Stale Secrets
//!
//! Former employees, deprecated services, and forgotten integrations all retain
//! access if secrets are not rotated. The longer a secret lives, the more copies
//! exist in logs, backups, and developer machines.
//!
//! ## Defense
//!
//! 1. Automate rotation with a secrets manager
//! 2. Use short-lived secrets (minutes/hours, not months)
//! 3. Implement overlap periods for zero-downtime rotation
//! 4. Track which consumers use which secret version
//! 5. Alert on secrets that exceed their maximum age

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

/// A versioned secret with metadata.
#[derive(Debug, Clone)]
pub struct SecretVersion {
    /// The secret value.
    pub value: String,
    /// Version number (incremented on each rotation).
    pub version: u32,
    /// When this version was created.
    pub created_at: Instant,
    /// Whether this version is still active (can be used for authentication).
    pub active: bool,
}

/// A secret store that supports versioned secrets with rotation.
#[derive(Debug)]
pub struct RotatingSecretStore {
    /// Secret name -> list of versions (newest first).
    secrets: HashMap<String, Vec<SecretVersion>>,
    /// Maximum number of versions to keep per secret.
    max_versions: usize,
    /// Maximum age before a secret should be rotated.
    max_age: Duration,
}

impl RotatingSecretStore {
    /// Create a new store with version limits and max age.
    pub fn new(max_versions: usize, max_age: Duration) -> Self {
        Self {
            secrets: HashMap::new(),
            max_versions,
            max_age,
        }
    }

    /// Get the current (newest active) version of a secret.
    pub fn get_current(&self, name: &str) -> Option<&SecretVersion> {
        self.secrets
            .get(name)
            .and_then(|versions| versions.iter().find(|v| v.active))
    }
}

/// Exercise 1: Store a new secret or rotate an existing one.
///
/// If the secret doesn't exist, create version 1.
/// If it exists, create a new version (increment version number),
/// deactivate all previous versions, and keep at most `max_versions`.
///
/// Return the new version number.
///
/// Hints:
/// - Get the current max version from the existing list (or 0 if new)
/// - Create a new SecretVersion with version = max + 1
/// - Deactivate all existing versions
/// - Insert the new version at the front of the list
/// - Trim the list to max_versions
pub fn rotate_secret(store: &mut RotatingSecretStore, name: &str, new_value: &str) -> u32 {
    todo!("Store a new secret or rotate an existing one")
}

/// Exercise 2: Check if a secret needs rotation based on its age.
///
/// Return true if the current version's age exceeds the store's `max_age`.
/// Return false if the secret doesn't exist (nothing to rotate).
///
/// Hints:
/// - Get the current version
/// - If None, return false
/// - Compare `created_at.elapsed()` with `max_age`
pub fn needs_rotation(store: &RotatingSecretStore, name: &str) -> bool {
    todo!("Check if a secret has exceeded its maximum age")
}

/// Exercise 3: List all secrets that need rotation.
///
/// Return a Vec of secret names whose current version exceeds max_age.
///
/// Hints:
/// - Iterate over all secrets in the store
/// - For each, check `needs_rotation`
/// - Collect the names that need rotation
pub fn list_secrets_needing_rotation(store: &RotatingSecretStore) -> Vec<String> {
    todo!("List all secrets that need rotation")
}

/// Exercise 4: Validate a secret value against any active version.
///
/// Return true if the given value matches ANY active version of the named secret.
/// This supports the overlap period during rotation where both old and new
/// secrets are valid.
///
/// Hints:
/// - Get all versions for the secret name
/// - Filter by `active == true`
/// - Check if any version's value matches the given value
pub fn validate_secret(store: &RotatingSecretStore, name: &str, value: &str) -> bool {
    todo!("Validate a secret against any active version")
}

/// Exercise 5: Deactivate old versions, keeping only the N most recent active.
///
/// Given a secret name and `keep_count`, deactivate all versions except the
/// `keep_count` most recent ones. Return the number of versions deactivated.
///
/// Hints:
/// - Get the versions list for the secret
/// - Skip the first `keep_count` active versions
/// - Deactivate the rest
/// - Count how many were deactivated
pub fn deactivate_old_versions(store: &mut RotatingSecretStore, name: &str, keep_count: usize) -> usize {
    todo!("Deactivate old secret versions")
}

/// Exercise 6: Get the version history of a secret.
///
/// Return a Vec of (version_number, active_status, age_in_seconds) tuples
/// for all versions of the named secret, ordered from newest to oldest.
///
/// Hints:
/// - Get the versions list
/// - Map each version to (version, active, created_at.elapsed().as_secs())
/// - Collect into a Vec
pub fn get_version_history(store: &RotatingSecretStore, name: &str) -> Vec<(u32, bool, u64)> {
    todo!("Get version history of a secret")
}

/// Exercise 7: Purge all inactive versions of all secrets.
///
/// Remove all versions where `active == false` from every secret.
/// Return the total number of versions purged.
///
/// Hints:
/// - Iterate over all secrets
/// - For each, retain only versions where `active == true`
/// - Count the difference in lengths
pub fn purge_inactive_versions(store: &mut RotatingSecretStore) -> usize {
    todo!("Purge all inactive secret versions")
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
        // Newest first
        assert_eq!(history[0].0, 3);
        assert!(history[0].1); // newest should be active
        // Older ones should be inactive
        assert!(!history[1].1);
        assert!(!history[2].1);
    }

    #[test]
    fn test_deactivate_old_versions() {
        let mut store = create_test_store();
        rotate_secret(&mut store, "key", "v1");
        rotate_secret(&mut store, "key", "v2");
        rotate_secret(&mut store, "key", "v3");
        // After rotation, only v3 is active. Re-activate all for this test.
        // Let's test purge instead since deactivate works on active versions.
        let purged = purge_inactive_versions(&mut store);
        assert_eq!(purged, 2); // v1 and v2 should be purged
        let history = get_version_history(&store, "key");
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_list_secrets_needing_rotation() {
        let mut store = RotatingSecretStore::new(5, Duration::from_millis(1));
        rotate_secret(&mut store, "fast_expiring", "value");
        // Wait for the secret to exceed max_age
        std::thread::sleep(Duration::from_millis(10));
        let needing = list_secrets_needing_rotation(&store);
        assert!(needing.contains(&"fast_expiring".to_string()));
    }
}
