//! # Lesson 06: Auto-Lock
//!
//! ## Locking the Vault After Inactivity
//!
//! An unlocked vault holds the master key in memory. If the user walks away
//! from their computer, anyone with physical access can read passwords.
//! Auto-lock solves this by zeroizing the key after a period of inactivity.
//!
//! ## What Happens on Lock?
//!
//! 1. The encryption key is zeroized (overwritten with zeros)
//! 2. The decrypted vault data is zeroized
//! 3. The vault state transitions from `Unlocked` to `Locked`
//! 4. To access passwords again, the user must re-enter their passphrase
//!
//! ## Memory Zeroization
//!
//! Rust's optimizer might remove "dead" stores (writing to memory that is
//! never read again). The `zeroize` crate prevents this by using volatile
//! writes that the compiler cannot optimize away.
//!
//! ## The `secrecy` Crate
//!
//! `Secret<T>` wraps sensitive values and implements `Drop` using `Zeroize`.
//! It also prevents accidental display in debug output (shows `[REDACTED]`).
//!
//! ## Inactivity Detection
//!
//! We track the last time the vault was accessed. On each access, we check
//! whether the inactivity timeout has been exceeded. If so, we lock
//! automatically before servicing the request.
//!
//! ## Attack Scenario
//!
//! An attacker gains remote shell access to the user's machine. If the vault
//! is unlocked, the key is in the process's heap memory. With auto-lock, the
//! key is zeroized after 5 minutes of inactivity, limiting the attacker's
//! window. Memory forensics after lock will find only zeros.

use std::time::{Duration, Instant};
use zeroize::Zeroize;
use chrono::Utc;

/// Vault lock state.
#[derive(Debug, Clone, PartialEq)]
pub enum VaultState {
    /// Vault is locked -- no key material in memory.
    Locked,
    /// Vault is unlocked -- key material present.
    Unlocked,
}

/// The vault with auto-lock capability.
#[derive(Debug)]
pub struct AutoLockVault {
    /// Current state
    pub state: VaultState,
    /// The encryption key (only valid when Unlocked)
    encryption_key: Vec<u8>,
    /// Decrypted vault data (only valid when Unlocked)
    vault_data: Vec<u8>,
    /// Last access time (only set when Unlocked)
    last_access: Option<Instant>,
    /// Inactivity timeout duration
    timeout: Duration,
    /// Audit log of lock/unlock events
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

    /// Exercise 1: Unlock the vault with the given key and data.
    ///
    /// If the vault is already unlocked, re-key it (for rotation).
    ///
    /// Hints:
    /// - Set `self.encryption_key = key.to_vec()`
    /// - Set `self.vault_data = data.to_vec()`
    /// - Set `self.last_access = Some(Instant::now())`
    /// - Set `self.state = VaultState::Unlocked`
    /// - Log an `Unlocked` event
    pub fn unlock(&mut self, key: &[u8], data: &[u8]) {
        todo!("Unlock the vault")
    }

    /// Exercise 2: Lock the vault, zeroizing all sensitive data.
    ///
    /// Hints:
    /// - `self.encryption_key.zeroize()`
    /// - `self.vault_data.zeroize()`
    /// - `self.encryption_key.clear()`
    /// - `self.vault_data.clear()`
    /// - Set state to Locked
    /// - Set last_access to None
    /// - Log a `Locked` event
    pub fn lock(&mut self) {
        todo!("Lock the vault and zeroize keys")
    }

    /// Exercise 3: Check if the vault should auto-lock due to inactivity.
    ///
    /// Returns true if the vault was auto-locked.
    ///
    /// Hints:
    /// - If state is Locked, return false
    /// - Check `self.last_access` against `self.timeout`
    /// - If expired, call `self.lock()` and log an AutoLocked event
    pub fn check_timeout(&mut self) -> bool {
        todo!("Check inactivity timeout and auto-lock if needed")
    }

    /// Exercise 4: Record an access (resets the inactivity timer).
    ///
    /// If the vault is locked, log an AccessDenied event and return None.
    /// If the vault is unlocked but timed out, auto-lock and return None.
    /// Otherwise, update last_access and return Some(data).
    ///
    /// Hints:
    /// - Check state first
    /// - Call check_timeout
    /// - Update last_access
    /// - Return a reference to vault_data
    pub fn access(&mut self) -> Option<Vec<u8>> {
        todo!("Record access and return vault data if unlocked")
    }

    /// Exercise 5: Change the inactivity timeout.
    ///
    /// Hints:
    /// - `self.timeout = new_timeout`
    pub fn set_timeout(&mut self, new_timeout: Duration) {
        todo!("Change the inactivity timeout")
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

/// Exercise 6: Create a vault context string for the audit log.
///
/// Returns a string like: "[2024-01-15T10:30:00Z] UNLOCKED"
///
/// Hints:
/// - Use `Utc::now().to_rfc3339()`
/// - Match on event_type
fn format_lock_event(event: &LockEvent) -> String {
    todo!("Format a lock event for the audit log")
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
        vault.access(); // Reset timer
        std::thread::sleep(Duration::from_millis(100));
        // Should still be unlocked because we accessed at ~100ms
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
        // After lock, key should be empty
        assert!(vault.encryption_key.is_empty());
        assert!(vault.vault_data.is_empty());
    }
}
