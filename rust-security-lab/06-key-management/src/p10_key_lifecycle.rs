//! # Lesson 10: Complete Key Lifecycle Management
//!
//! ## What Is Key Lifecycle Management?
//!
//! A key lifecycle covers every phase from creation to destruction. Proper lifecycle
//! management ensures keys are never used outside their intended scope or lifetime.
//!
//! ```text
//! +----------+    +----------+    +----------+    +----------+    +----------+
//! | Generate |--->| Activate |--->|   Use    |--->|  Rotate  |--->| Destroy  |
//! +----------+    +----------+    +----------+    +----------+    +----------+
//!      |                                                  |              ^
//!      |                                                  v              |
//!      |                                           +----------+          |
//!      +------------------------------------------>|  Retire  |----------+
//!                                                  +----------+
//! ```
//!
//! ## Key States
//!
//! | State | Description | Allowed Operations |
//! |-------|-------------|-------------------|
//! | Pending | Created but not yet active | None |
//! | Active | Currently in use | Encrypt, Decrypt, Sign |
//! | Retired | No longer used for new operations | Decrypt only (for old data) |
//! | Compromised | Potentially leaked | None (emergency) |
//! | Destroyed | Securely erased | None |
//!
//! ## NIST SP 800-57 Key Lifecycle
//!
//! NIST defines these phases:
//! 1. Pre-activation (key generated, not yet active)
//! 2. Active (authorized for use)
//! 3. Suspended (temporarily disabled)
//! 4. Deactivated (no longer for new operations)
//! 5. Destroyed (irreversibly removed)
//!
//! ## Security Notes
//!
//! - Automate lifecycle transitions (don't rely on humans to rotate keys)
//! - Monitor for overdue rotations (alert if key is past its expiry)
//! - Maintain a complete audit trail of all lifecycle transitions
//! - Test your destruction procedures regularly

use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

/// States in the key lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    Pending,
    Active,
    Retired,
    Compromised,
    Destroyed,
}

/// A lifecycle event (audit trail entry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub timestamp: u64,
    pub from_state: Option<LifecycleState>,
    pub to_state: LifecycleState,
    pub reason: String,
    pub actor: String,
}

/// A managed key with full lifecycle tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedKey {
    pub id: String,
    pub key_material: Vec<u8>,
    pub state: LifecycleState,
    pub created_at: u64,
    pub expires_at: u64,
    pub algorithm: String,
    pub lifecycle_events: Vec<LifecycleEvent>,
}

/// The key lifecycle manager.
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyLifecycleManager {
    pub keys: Vec<ManagedKey>,
}

impl KeyLifecycleManager {
    pub fn new() -> Self {
        KeyLifecycleManager { keys: Vec::new() }
    }
}

/// Exercise 1: Create a new managed key in Pending state.
///
/// Generate a 32-byte random key with the given metadata.
/// State should be Pending, and the first lifecycle event should record creation.
///
/// Hints:
/// - Use `ring::SystemRandom` for key material
/// - Add a LifecycleEvent: from_state=None, to_state=Pending, reason="Key created"
pub fn create_managed_key(
    key_id: &str,
    algorithm: &str,
    current_timestamp: u64,
    validity_seconds: u64,
    actor: &str,
) -> ManagedKey {
    todo!("Create a managed key in Pending state")
}

/// Exercise 2: Transition a key to Active state.
///
/// Validate the state transition (only Pending -> Active is allowed).
/// Record a lifecycle event.
///
/// Hints:
/// - Check current state is Pending
/// - Change state to Active
/// - Push a LifecycleEvent
/// - Return Err if transition is invalid
pub fn activate_key(key: &mut ManagedKey, timestamp: u64, actor: &str) -> Result<(), String> {
    todo!("Transition key from Pending to Active")
}

/// Exercise 3: Transition a key to Retired state.
///
/// Only Active -> Retired is allowed.
/// Record the event.
pub fn retire_key(key: &mut ManagedKey, timestamp: u64, actor: &str) -> Result<(), String> {
    todo!("Transition key from Active to Retired")
}

/// Exercise 4: Mark a key as Compromised (from any non-Destroyed state).
///
/// Compromised is an emergency state — it can be reached from Active, Retired,
/// or even Pending. Record the event with the reason.
pub fn mark_compromised(
    key: &mut ManagedKey,
    timestamp: u64,
    reason: &str,
    actor: &str,
) -> Result<(), String> {
    todo!("Mark a key as Compromised (emergency transition)")
}

/// Exercise 5: Destroy a key (transition to Destroyed state).
///
/// Only Retired or Compromised keys can be destroyed.
/// Zero the key material as part of destruction.
pub fn destroy_managed_key(key: &mut ManagedKey, timestamp: u64, actor: &str) -> Result<(), String> {
    todo!("Destroy a key (Retired/Compromised -> Destroyed)")
}

/// Exercise 6: Check if a key can be used for encryption.
///
/// Only Active keys that haven't expired should be usable for encryption.
///
/// Hints:
/// - State must be Active
/// - current_timestamp must be < expires_at
pub fn can_encrypt(key: &ManagedKey, current_timestamp: u64) -> bool {
    todo!("Check if key is valid for encryption")
}

/// Exercise 7: Check if a key can be used for decryption.
///
/// Active and Retired keys (not expired) can decrypt.
/// Compromised and Destroyed keys cannot.
pub fn can_decrypt(key: &ManagedKey, current_timestamp: u64) -> bool {
    todo!("Check if key is valid for decryption")
}

/// Exercise 8: Find keys that need rotation (expiring soon).
///
/// Return key IDs of Active keys that expire within `grace_seconds` of
/// `current_timestamp`.
pub fn find_keys_needing_rotation(
    manager: &KeyLifecycleManager,
    current_timestamp: u64,
    grace_seconds: u64,
) -> Vec<String> {
    todo!("Find Active keys expiring within grace period")
}

/// Exercise 9: Get the full audit trail for a key.
///
/// Return all lifecycle events for the given key ID, in chronological order.
pub fn get_audit_trail<'a>(manager: &'a KeyLifecycleManager, key_id: &str) -> Vec<&'a LifecycleEvent> {
    todo!("Get audit trail for a key")
}

/// Exercise 10: Demonstrate the complete lifecycle.
///
/// Create a key, activate it, use it, retire it, and destroy it.
/// Return the sequence of states the key went through.
pub fn demonstrate_lifecycle() -> Vec<LifecycleState> {
    todo!("Demonstrate the complete key lifecycle")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manager() -> KeyLifecycleManager {
        KeyLifecycleManager::new()
    }

    #[test]
    fn test_create_key_pending() {
        let key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        assert_eq!(key.state, LifecycleState::Pending);
        assert_eq!(key.key_material.len(), 32);
        assert_eq!(key.lifecycle_events.len(), 1);
    }

    #[test]
    fn test_activate_key() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        activate_key(&mut key, 1001, "admin").unwrap();
        assert_eq!(key.state, LifecycleState::Active);
        assert_eq!(key.lifecycle_events.len(), 2);
    }

    #[test]
    fn test_invalid_transition_pending_to_retired() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        let result = retire_key(&mut key, 1001, "admin");
        assert!(result.is_err(), "Pending -> Retired should be invalid");
    }

    #[test]
    fn test_retire_active_key() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        activate_key(&mut key, 1001, "admin").unwrap();
        retire_key(&mut key, 5000, "admin").unwrap();
        assert_eq!(key.state, LifecycleState::Retired);
    }

    #[test]
    fn test_compromise_from_active() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        activate_key(&mut key, 1001, "admin").unwrap();
        mark_compromised(&mut key, 2000, "Suspected breach", "security-team").unwrap();
        assert_eq!(key.state, LifecycleState::Compromised);
    }

    #[test]
    fn test_destroy_retired_key() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        activate_key(&mut key, 1001, "admin").unwrap();
        retire_key(&mut key, 5000, "admin").unwrap();
        destroy_managed_key(&mut key, 6000, "admin").unwrap();
        assert_eq!(key.state, LifecycleState::Destroyed);
        assert!(key.key_material.iter().all(|&b| b == 0), "Key material should be zeroed");
    }

    #[test]
    fn test_destroy_active_key_fails() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        activate_key(&mut key, 1001, "admin").unwrap();
        let result = destroy_managed_key(&mut key, 2000, "admin");
        assert!(result.is_err(), "Cannot destroy Active key directly");
    }

    #[test]
    fn test_can_encrypt_active_not_expired() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        activate_key(&mut key, 1001, "admin").unwrap();
        assert!(can_encrypt(&key, 5000));
        assert!(!can_encrypt(&key, 100000), "Should not encrypt with expired key");
    }

    #[test]
    fn test_can_decrypt_retired() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        activate_key(&mut key, 1001, "admin").unwrap();
        retire_key(&mut key, 5000, "admin").unwrap();
        assert!(can_decrypt(&key, 5001), "Retired key should still decrypt");
    }

    #[test]
    fn test_complete_lifecycle_states() {
        let states = demonstrate_lifecycle();
        assert!(states.contains(&LifecycleState::Pending));
        assert!(states.contains(&LifecycleState::Active));
        assert!(states.contains(&LifecycleState::Retired));
        assert!(states.contains(&LifecycleState::Destroyed));
    }

    #[test]
    fn test_audit_trail_count() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        activate_key(&mut key, 1001, "admin").unwrap();
        retire_key(&mut key, 5000, "admin").unwrap();
        assert_eq!(key.lifecycle_events.len(), 3, "Should have create + activate + retire events");
    }
}
