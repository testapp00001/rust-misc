//! # Lesson 10: Complete Key Lifecycle Management (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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

/// Create a new managed key in Pending state.
pub fn create_managed_key(
    key_id: &str,
    algorithm: &str,
    current_timestamp: u64,
    validity_seconds: u64,
    actor: &str,
) -> ManagedKey {
    let rng = SystemRandom::new();
    let mut key_material = vec![0u8; 32];
    rng.fill(&mut key_material).expect("Failed to generate key");

    ManagedKey {
        id: key_id.to_string(),
        key_material,
        state: LifecycleState::Pending,
        created_at: current_timestamp,
        expires_at: current_timestamp + validity_seconds,
        algorithm: algorithm.to_string(),
        lifecycle_events: vec![LifecycleEvent {
            timestamp: current_timestamp,
            from_state: None,
            to_state: LifecycleState::Pending,
            reason: "Key created".to_string(),
            actor: actor.to_string(),
        }],
    }
}

/// Transition a key to Active state (only from Pending).
pub fn activate_key(key: &mut ManagedKey, timestamp: u64, actor: &str) -> Result<(), String> {
    if key.state != LifecycleState::Pending {
        return Err(format!("Cannot activate key in {:?} state", key.state));
    }
    key.state = LifecycleState::Active;
    key.lifecycle_events.push(LifecycleEvent {
        timestamp,
        from_state: Some(LifecycleState::Pending),
        to_state: LifecycleState::Active,
        reason: "Key activated".to_string(),
        actor: actor.to_string(),
    });
    Ok(())
}

/// Transition a key to Retired state (only from Active).
pub fn retire_key(key: &mut ManagedKey, timestamp: u64, actor: &str) -> Result<(), String> {
    if key.state != LifecycleState::Active {
        return Err(format!("Cannot retire key in {:?} state", key.state));
    }
    key.state = LifecycleState::Retired;
    key.lifecycle_events.push(LifecycleEvent {
        timestamp,
        from_state: Some(LifecycleState::Active),
        to_state: LifecycleState::Retired,
        reason: "Key retired".to_string(),
        actor: actor.to_string(),
    });
    Ok(())
}

/// Mark a key as Compromised (emergency — from any non-Destroyed state).
pub fn mark_compromised(
    key: &mut ManagedKey,
    timestamp: u64,
    reason: &str,
    actor: &str,
) -> Result<(), String> {
    if key.state == LifecycleState::Destroyed {
        return Err("Cannot compromise a destroyed key".to_string());
    }
    let from = key.state;
    key.state = LifecycleState::Compromised;
    key.lifecycle_events.push(LifecycleEvent {
        timestamp,
        from_state: Some(from),
        to_state: LifecycleState::Compromised,
        reason: reason.to_string(),
        actor: actor.to_string(),
    });
    Ok(())
}

/// Destroy a key (only from Retired or Compromised).
///
/// Zeroes the key material as part of destruction.
pub fn destroy_managed_key(key: &mut ManagedKey, timestamp: u64, actor: &str) -> Result<(), String> {
    if key.state != LifecycleState::Retired && key.state != LifecycleState::Compromised {
        return Err(format!("Cannot destroy key in {:?} state (must be Retired or Compromised)", key.state));
    }
    let from = key.state;
    key.state = LifecycleState::Destroyed;
    // Zero the key material
    for byte in key.key_material.iter_mut() {
        *byte = 0;
    }
    key.lifecycle_events.push(LifecycleEvent {
        timestamp,
        from_state: Some(from),
        to_state: LifecycleState::Destroyed,
        reason: "Key destroyed (material zeroed)".to_string(),
        actor: actor.to_string(),
    });
    Ok(())
}

/// Check if a key can be used for encryption.
///
/// Only Active keys that haven't expired.
pub fn can_encrypt(key: &ManagedKey, current_timestamp: u64) -> bool {
    key.state == LifecycleState::Active && current_timestamp < key.expires_at
}

/// Check if a key can be used for decryption.
///
/// Active or Retired keys (not expired, not compromised, not destroyed).
pub fn can_decrypt(key: &ManagedKey, current_timestamp: u64) -> bool {
    (key.state == LifecycleState::Active || key.state == LifecycleState::Retired)
        && current_timestamp < key.expires_at
}

/// Find Active keys that expire within the grace period.
pub fn find_keys_needing_rotation(
    manager: &KeyLifecycleManager,
    current_timestamp: u64,
    grace_seconds: u64,
) -> Vec<String> {
    manager
        .keys
        .iter()
        .filter(|k| {
            k.state == LifecycleState::Active
                && k.expires_at <= current_timestamp + grace_seconds
        })
        .map(|k| k.id.clone())
        .collect()
}

/// Get the full audit trail for a key.
pub fn get_audit_trail<'a>(manager: &'a KeyLifecycleManager, key_id: &str) -> Vec<&'a LifecycleEvent> {
    manager
        .keys
        .iter()
        .find(|k| k.id == key_id)
        .map(|k| k.lifecycle_events.iter().collect())
        .unwrap_or_default()
}

/// Demonstrate the complete key lifecycle.
///
/// Returns the sequence of states the key went through.
pub fn demonstrate_lifecycle() -> Vec<LifecycleState> {
    let mut key = create_managed_key("demo-key", "AES-256-GCM", 1000, 86400, "system");
    let mut states = vec![key.state]; // Pending

    activate_key(&mut key, 1001, "admin").unwrap();
    states.push(key.state); // Active

    retire_key(&mut key, 5000, "admin").unwrap();
    states.push(key.state); // Retired

    destroy_managed_key(&mut key, 6000, "admin").unwrap();
    states.push(key.state); // Destroyed

    states
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(result.is_err());
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
        assert!(key.key_material.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_destroy_active_key_fails() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        activate_key(&mut key, 1001, "admin").unwrap();
        let result = destroy_managed_key(&mut key, 2000, "admin");
        assert!(result.is_err());
    }

    #[test]
    fn test_can_encrypt_active_not_expired() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        activate_key(&mut key, 1001, "admin").unwrap();
        assert!(can_encrypt(&key, 5000));
        assert!(!can_encrypt(&key, 100000));
    }

    #[test]
    fn test_can_decrypt_retired() {
        let mut key = create_managed_key("test-key", "AES-256-GCM", 1000, 86400, "admin");
        activate_key(&mut key, 1001, "admin").unwrap();
        retire_key(&mut key, 5000, "admin").unwrap();
        assert!(can_decrypt(&key, 5001));
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
        assert_eq!(key.lifecycle_events.len(), 3);
    }
}
