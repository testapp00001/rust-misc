//! # Lesson 06: Auto-Lock (Reference Solution)
//!
//! See the exercise file for full documentation on auto-lock and memory zeroization.

use std::time::{Duration, Instant};
use zeroize::Zeroize;
use chrono::Utc;

/// Vault lock state.
#[derive(Debug, Clone, PartialEq)]
pub enum VaultState {
    Locked,
    Unlocked,
}

/// The vault with auto-lock capability.
#[derive(Debug)]
pub struct AutoLockVault {
    pub state: VaultState,
    encryption_key: Vec<u8>,
    vault_data: Vec<u8>,
    last_access: Option<Instant>,
    timeout: Duration,
    pub audit_log: Vec<LockEvent>,
}

/// An audit log entry for lock/unlock events.
#[derive(Debug, Clone)]
pub struct LockEvent {
    pub event_type: LockEventType,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub enum LockEventType {
    Unlocked,
    Locked,
    AutoLocked,
    AccessDenied,
}

impl AutoLockVault {
    /// Create a new locked vault with the given inactivity timeout.
    pub fn new(timeout: Duration) -> Self {
        Self {
            state: VaultState::Locked,
            encryption_key: Vec::new(),
            vault_data: Vec::new(),
            last_access: None,
            timeout,
            audit_log: Vec::new(),
        }
    }

    /// Unlock the vault with the given key and data.
    pub fn unlock(&mut self, key: &[u8], data: &[u8]) {
        self.encryption_key = key.to_vec();
        self.vault_data = data.to_vec();
        self.last_access = Some(Instant::now());
        self.state = VaultState::Unlocked;
        self.audit_log.push(LockEvent {
            event_type: LockEventType::Unlocked,
            timestamp: Utc::now().to_rfc3339(),
        });
    }

    /// Lock the vault, zeroizing all sensitive data.
    pub fn lock(&mut self) {
        self.encryption_key.zeroize();
        self.vault_data.zeroize();
        self.encryption_key.clear();
        self.vault_data.clear();
        self.state = VaultState::Locked;
        self.last_access = None;
        self.audit_log.push(LockEvent {
            event_type: LockEventType::Locked,
            timestamp: Utc::now().to_rfc3339(),
        });
    }

    /// Check if the vault should auto-lock due to inactivity.
    pub fn check_timeout(&mut self) -> bool {
        if self.state == VaultState::Locked {
            return false;
        }
        if let Some(last) = self.last_access {
            if last.elapsed() >= self.timeout {
                self.encryption_key.zeroize();
                self.vault_data.zeroize();
                self.encryption_key.clear();
                self.vault_data.clear();
                self.state = VaultState::Locked;
                self.last_access = None;
                self.audit_log.push(LockEvent {
                    event_type: LockEventType::AutoLocked,
                    timestamp: Utc::now().to_rfc3339(),
                });
                return true;
            }
        }
        false
    }

    /// Record an access and return vault data if unlocked.
    pub fn access(&mut self) -> Option<Vec<u8>> {
        if self.state == VaultState::Locked {
            self.audit_log.push(LockEvent {
                event_type: LockEventType::AccessDenied,
                timestamp: Utc::now().to_rfc3339(),
            });
            return None;
        }
        if self.check_timeout() {
            return None;
        }
        self.last_access = Some(Instant::now());
        Some(self.vault_data.clone())
    }

    /// Change the inactivity timeout.
    pub fn set_timeout(&mut self, new_timeout: Duration) {
        self.timeout = new_timeout;
    }

    /// Get the current timeout duration.
    pub fn get_timeout(&self) -> Duration {
        self.timeout
    }
}

impl Drop for AutoLockVault {
    fn drop(&mut self) {
        self.encryption_key.zeroize();
        self.vault_data.zeroize();
    }
}

/// Format a lock event for the audit log.
#[allow(dead_code)]
fn format_lock_event(event: &LockEvent) -> String {
    let action = match event.event_type {
        LockEventType::Unlocked => "UNLOCKED",
        LockEventType::Locked => "LOCKED",
        LockEventType::AutoLocked => "AUTO_LOCKED",
        LockEventType::AccessDenied => "ACCESS_DENIED",
    };
    format!("[{}] {}", event.timestamp, action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vault_is_locked() {
        let vault = AutoLockVault::new(Duration::from_secs(300));
        assert_eq!(vault.state, VaultState::Locked);
    }

    #[test]
    fn test_unlock_and_lock() {
        let mut vault = AutoLockVault::new(Duration::from_secs(300));
        vault.unlock(b"key123", b"vaultdata");
        assert_eq!(vault.state, VaultState::Unlocked);
        vault.lock();
        assert_eq!(vault.state, VaultState::Locked);
    }

    #[test]
    fn test_access_when_locked() {
        let mut vault = AutoLockVault::new(Duration::from_secs(300));
        assert!(vault.access().is_none());
    }

    #[test]
    fn test_access_when_unlocked() {
        let mut vault = AutoLockVault::new(Duration::from_secs(300));
        vault.unlock(b"key", b"data");
        let data = vault.access();
        assert!(data.is_some());
        assert_eq!(data.unwrap(), b"data");
    }

    #[test]
    fn test_auto_lock_on_timeout() {
        let mut vault = AutoLockVault::new(Duration::from_millis(50));
        vault.unlock(b"key", b"data");
        std::thread::sleep(Duration::from_millis(100));
        assert!(vault.check_timeout());
        assert_eq!(vault.state, VaultState::Locked);
    }

    #[test]
    fn test_access_resets_timer() {
        let mut vault = AutoLockVault::new(Duration::from_millis(200));
        vault.unlock(b"key", b"data");
        std::thread::sleep(Duration::from_millis(100));
        vault.access();
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(vault.state, VaultState::Unlocked);
    }

    #[test]
    fn test_set_timeout() {
        let mut vault = AutoLockVault::new(Duration::from_secs(300));
        vault.set_timeout(Duration::from_secs(60));
        assert_eq!(vault.get_timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_audit_log() {
        let mut vault = AutoLockVault::new(Duration::from_secs(300));
        vault.unlock(b"key", b"data");
        vault.lock();
        assert!(vault.audit_log.len() >= 2);
    }

    #[test]
    fn test_lock_zeroizes_key() {
        let mut vault = AutoLockVault::new(Duration::from_secs(300));
        vault.unlock(b"secretkeydata12345678", b"vault");
        vault.lock();
        assert!(vault.encryption_key.is_empty());
        assert!(vault.vault_data.is_empty());
    }
}
