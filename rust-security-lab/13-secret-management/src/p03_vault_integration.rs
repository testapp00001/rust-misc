//! # Lesson 03: HashiCorp Vault Integration Concepts
//!
//! ## What is Vault?
//!
//! HashiCorp Vault is a secrets management tool that provides:
//! - **Centralized secret storage**: One place for all secrets
//! - **Dynamic secrets**: Generated on-demand with automatic expiration
//! - **Lease-based access**: Secrets have a TTL and auto-revoke
//! - **Audit logging**: Every access is recorded
//! - **Encryption as a service**: Encrypt/decrypt without exposing keys
//!
//! ## How Lease-Based Secrets Work
//!
//! ```text
//! 1. App requests a secret from Vault
//! 2. Vault generates (or retrieves) the secret
//! 3. Vault assigns a "lease" — a TTL (time-to-live)
//! 4. App uses the secret for the lease duration
//! 5. When the lease expires, the secret is automatically revoked
//! 6. App must renew the lease or request a new secret
//! ```
//!
//! ## Dynamic Secrets Example
//!
//! Instead of sharing a single database password:
//! ```text
//! Traditional:  All apps share "db_admin_password"
//! Dynamic:      Each app gets its own unique credential
//!               App A → "app_a_abc123" (TTL: 1h)
//!               App B → "app_b_def456" (TTL: 1h)
//!               When App A's lease expires → credential is revoked
//! ```
//!
//! ## Benefits
//!
//! - **Blast radius reduction**: Compromised credential only affects one app
//! - **Automatic rotation**: Credentials expire and are replaced automatically
//! - **Audit trail**: Every credential request is logged
//! - **No long-lived secrets**: Everything has a TTL
//!
//! ## This Exercise
//!
//! We'll simulate a Vault-like system with leases, TTLs, and dynamic secret generation.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Represents a lease for a secret — the secret is valid for a limited time.
#[derive(Debug, Clone)]
pub struct SecretLease {
    /// The secret value.
    pub secret: String,
    /// When the lease was created.
    pub created_at: Instant,
    /// How long the lease is valid.
    pub ttl: Duration,
    /// The lease ID for renewal/revocation.
    pub lease_id: String,
    /// Whether this lease has been explicitly revoked.
    pub revoked: bool,
}

impl SecretLease {
    /// Check if the lease has expired.
    pub fn is_expired(&self) -> bool {
        self.revoked || self.created_at.elapsed() >= self.ttl
    }

    /// Get the remaining TTL.
    pub fn remaining_ttl(&self) -> Duration {
        if self.revoked {
            Duration::ZERO
        } else {
            let elapsed = self.created_at.elapsed();
            if elapsed >= self.ttl {
                Duration::ZERO
            } else {
                self.ttl - elapsed
            }
        }
    }
}

/// A simulated Vault that manages secrets with leases.
#[derive(Debug)]
pub struct SecretVault {
    /// Stored secrets indexed by path (e.g., "database/creds").
    secrets: HashMap<String, String>,
    /// Active leases indexed by lease_id.
    leases: HashMap<String, SecretLease>,
    /// Counter for generating unique lease IDs.
    lease_counter: u64,
    /// Default TTL for new leases.
    default_ttl: Duration,
}

impl SecretVault {
    /// Create a new empty vault with a default TTL.
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            secrets: HashMap::new(),
            leases: HashMap::new(),
            lease_counter: 0,
            default_ttl,
        }
    }

    /// Store a secret at the given path.
    pub fn put_secret(&mut self, path: &str, value: &str) {
        self.secrets.insert(path.to_string(), value.to_string());
    }

    /// Generate a unique lease ID.
    fn next_lease_id(&mut self) -> String {
        self.lease_counter += 1;
        format!("lease-{:08}", self.lease_counter)
    }
}

/// Exercise 1: Request a secret from the vault with a lease.
///
/// Retrieve the secret at the given path and create a lease for it.
/// Return `Ok(SecretLease)` if the secret exists, `Err(msg)` if not.
///
/// The lease should use the vault's default TTL.
///
/// Hints:
/// - Check if the path exists in `vault.secrets`
/// - Generate a lease ID with `vault.next_lease_id()`
/// - Create a `SecretLease` with the current instant, the default TTL, etc.
/// - Store the lease in `vault.leases`
/// - Return the lease
pub fn request_secret(vault: &mut SecretVault, path: &str) -> Result<SecretLease, String> {
    todo!("Request a secret from the vault, creating a lease")
}

/// Exercise 2: Request a secret with a custom TTL.
///
/// Same as `request_secret`, but use the provided TTL instead of the default.
///
/// Hints:
/// - Same as above but use the `custom_ttl` parameter for the lease TTL
pub fn request_secret_with_ttl(
    vault: &mut SecretVault,
    path: &str,
    custom_ttl: Duration,
) -> Result<SecretLease, String> {
    todo!("Request a secret with a custom TTL")
}

/// Exercise 3: Renew a lease, extending its TTL.
///
/// Given a lease_id and a new TTL, extend the lease.
/// Return `Ok(new_remaining_ttl)` if successful, `Err(msg)` if the lease
/// doesn't exist or has already expired/been revoked.
///
/// Hints:
/// - Look up the lease by lease_id in `vault.leases`
/// - Check if it's expired or revoked
/// - Update the lease: set `created_at` to now and `ttl` to the new TTL
/// - Return the new remaining TTL
pub fn renew_lease(
    vault: &mut SecretVault,
    lease_id: &str,
    new_ttl: Duration,
) -> Result<Duration, String> {
    todo!("Renew a lease, extending its TTL")
}

/// Exercise 4: Revoke a lease immediately.
///
/// Mark the lease as revoked. The secret should no longer be usable.
/// Return `Ok(())` if successful, `Err(msg)` if the lease doesn't exist.
///
/// Hints:
/// - Look up the lease by lease_id
/// - Set `revoked = true`
pub fn revoke_lease(vault: &mut SecretVault, lease_id: &str) -> Result<(), String> {
    todo!("Revoke a lease immediately")
}

/// Exercise 5: Get a dynamic secret — generate a unique credential per request.
///
/// Instead of returning a stored secret, generate a unique credential each time.
/// The format should be: `{path}_{lease_id}` (e.g., "database/creds_lease-00000001").
///
/// Still create a lease for the generated credential.
///
/// Hints:
/// - Generate a lease ID first
/// - Build the dynamic secret: format!("{}_{}", path, lease_id)
/// - Create a lease with the dynamic secret
/// - Store the lease
/// - Return the lease
pub fn request_dynamic_secret(
    vault: &mut SecretVault,
    path: &str,
    ttl: Duration,
) -> Result<SecretLease, String> {
    todo!("Generate a dynamic secret with a lease")
}

/// Exercise 6: Check which leases are still valid (not expired, not revoked).
///
/// Return a Vec of lease_ids that are still valid.
///
/// Hints:
/// - Iterate over all leases in `vault.leases`
/// - Filter by `!lease.is_expired()`
/// - Collect the lease_ids
pub fn list_active_leases(vault: &SecretVault) -> Vec<String> {
    todo!("List all active (non-expired, non-revoked) leases")
}

/// Exercise 7: Revoke all leases for a given secret path.
///
/// Find all leases whose secret matches the value at the given path,
/// and revoke them all. Return the number of leases revoked.
///
/// Hints:
/// - Get the secret value at the path
/// - Iterate over all leases
/// - Revoke any lease whose secret matches the value
/// - Count how many were revoked
pub fn revoke_all_for_path(vault: &mut SecretVault, path: &str) -> usize {
    todo!("Revoke all leases for a given secret path")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vault() -> SecretVault {
        let mut vault = SecretVault::new(Duration::from_secs(3600));
        vault.put_secret("database/creds", "db_password_123");
        vault.put_secret("api/stripe", "sk_test_abcdef");
        vault.put_secret("api/sendgrid", "SG.xyz789");
        vault
    }

    #[test]
    fn test_request_secret_success() {
        let mut vault = create_test_vault();
        let lease = request_secret(&mut vault, "database/creds").unwrap();
        assert_eq!(lease.secret, "db_password_123");
        assert!(!lease.is_expired());
        assert!(lease.remaining_ttl() > Duration::ZERO);
    }

    #[test]
    fn test_request_secret_not_found() {
        let mut vault = create_test_vault();
        let result = request_secret(&mut vault, "nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_request_secret_with_custom_ttl() {
        let mut vault = create_test_vault();
        let lease = request_secret_with_ttl(&mut vault, "api/stripe", Duration::from_secs(60)).unwrap();
        assert_eq!(lease.secret, "sk_test_abcdef");
        assert!(lease.remaining_ttl() <= Duration::from_secs(60));
    }

    #[test]
    fn test_revoke_lease() {
        let mut vault = create_test_vault();
        let lease = request_secret(&mut vault, "database/creds").unwrap();
        let lease_id = lease.lease_id.clone();

        revoke_lease(&mut vault, &lease_id).unwrap();

        // Lease should now be expired
        let stored_lease = vault.leases.get(&lease_id).unwrap();
        assert!(stored_lease.is_expired());
    }

    #[test]
    fn test_revoke_nonexistent_lease() {
        let mut vault = create_test_vault();
        let result = revoke_lease(&mut vault, "lease-99999999");
        assert!(result.is_err());
    }

    #[test]
    fn test_dynamic_secret_unique() {
        let mut vault = create_test_vault();
        let lease1 = request_dynamic_secret(&mut vault, "database/creds", Duration::from_secs(3600)).unwrap();
        let lease2 = request_dynamic_secret(&mut vault, "database/creds", Duration::from_secs(3600)).unwrap();

        // Dynamic secrets should be different each time
        assert_ne!(lease1.secret, lease2.secret);
        // But both should contain the path
        assert!(lease1.secret.starts_with("database/creds_"));
        assert!(lease2.secret.starts_with("database/creds_"));
    }

    #[test]
    fn test_list_active_leases() {
        let mut vault = create_test_vault();
        let l1 = request_secret(&mut vault, "database/creds").unwrap();
        let l2 = request_secret(&mut vault, "api/stripe").unwrap();
        let _l3 = request_secret(&mut vault, "api/sendgrid").unwrap();

        revoke_lease(&mut vault, &l1.lease_id).unwrap();

        let active = list_active_leases(&vault);
        assert_eq!(active.len(), 2);
        assert!(!active.contains(&l1.lease_id));
        assert!(active.contains(&l2.lease_id));
    }

    #[test]
    fn test_renew_lease() {
        let mut vault = create_test_vault();
        let lease = request_secret_with_ttl(&mut vault, "api/stripe", Duration::from_secs(1)).unwrap();
        let lease_id = lease.lease_id.clone();

        // Renew with a longer TTL
        let remaining = renew_lease(&mut vault, &lease_id, Duration::from_secs(3600)).unwrap();
        assert!(remaining > Duration::from_secs(100));

        // Lease should still be active
        let stored = vault.leases.get(&lease_id).unwrap();
        assert!(!stored.is_expired());
    }

    #[test]
    fn test_revoke_all_for_path() {
        let mut vault = create_test_vault();
        let _l1 = request_secret(&mut vault, "database/creds").unwrap();
        let _l2 = request_secret(&mut vault, "database/creds").unwrap();
        let _l3 = request_secret(&mut vault, "api/stripe").unwrap();

        let revoked = revoke_all_for_path(&mut vault, "database/creds");
        assert_eq!(revoked, 2);

        // Only the api/stripe lease should remain active
        let active = list_active_leases(&vault);
        assert_eq!(active.len(), 1);
    }
}
