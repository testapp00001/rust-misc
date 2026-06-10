//! # Lesson 03: HashiCorp Vault Integration Concepts (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SecretLease {
    pub secret: String,
    pub created_at: Instant,
    pub ttl: Duration,
    pub lease_id: String,
    pub revoked: bool,
}

impl SecretLease {
    pub fn is_expired(&self) -> bool {
        self.revoked || self.created_at.elapsed() >= self.ttl
    }

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

#[derive(Debug)]
pub struct SecretVault {
    secrets: HashMap<String, String>,
    leases: HashMap<String, SecretLease>,
    lease_counter: u64,
    default_ttl: Duration,
}

impl SecretVault {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            secrets: HashMap::new(),
            leases: HashMap::new(),
            lease_counter: 0,
            default_ttl,
        }
    }

    pub fn put_secret(&mut self, path: &str, value: &str) {
        self.secrets.insert(path.to_string(), value.to_string());
    }

    fn next_lease_id(&mut self) -> String {
        self.lease_counter += 1;
        format!("lease-{:08}", self.lease_counter)
    }
}

pub fn request_secret(vault: &mut SecretVault, path: &str) -> Result<SecretLease, String> {
    let secret = vault.secrets.get(path).cloned().ok_or_else(|| format!("Secret not found: {}", path))?;
    let lease_id = vault.next_lease_id();
    let lease = SecretLease {
        secret,
        created_at: Instant::now(),
        ttl: vault.default_ttl,
        lease_id: lease_id.clone(),
        revoked: false,
    };
    vault.leases.insert(lease_id, lease.clone());
    Ok(lease)
}

pub fn request_secret_with_ttl(
    vault: &mut SecretVault,
    path: &str,
    custom_ttl: Duration,
) -> Result<SecretLease, String> {
    let secret = vault.secrets.get(path).cloned().ok_or_else(|| format!("Secret not found: {}", path))?;
    let lease_id = vault.next_lease_id();
    let lease = SecretLease {
        secret,
        created_at: Instant::now(),
        ttl: custom_ttl,
        lease_id: lease_id.clone(),
        revoked: false,
    };
    vault.leases.insert(lease_id, lease.clone());
    Ok(lease)
}

pub fn renew_lease(
    vault: &mut SecretVault,
    lease_id: &str,
    new_ttl: Duration,
) -> Result<Duration, String> {
    let lease = vault.leases.get_mut(lease_id).ok_or_else(|| format!("Lease not found: {}", lease_id))?;
    if lease.revoked {
        return Err("Lease has been revoked".to_string());
    }
    lease.created_at = Instant::now();
    lease.ttl = new_ttl;
    Ok(lease.remaining_ttl())
}

pub fn revoke_lease(vault: &mut SecretVault, lease_id: &str) -> Result<(), String> {
    let lease = vault.leases.get_mut(lease_id).ok_or_else(|| format!("Lease not found: {}", lease_id))?;
    lease.revoked = true;
    Ok(())
}

pub fn request_dynamic_secret(
    vault: &mut SecretVault,
    path: &str,
    ttl: Duration,
) -> Result<SecretLease, String> {
    // Dynamic secrets don't require the path to exist in the vault
    let lease_id = vault.next_lease_id();
    let dynamic_secret = format!("{}_{}", path, lease_id);
    let lease = SecretLease {
        secret: dynamic_secret,
        created_at: Instant::now(),
        ttl,
        lease_id: lease_id.clone(),
        revoked: false,
    };
    vault.leases.insert(lease_id, lease.clone());
    Ok(lease)
}

pub fn list_active_leases(vault: &SecretVault) -> Vec<String> {
    vault
        .leases
        .iter()
        .filter(|(_, lease)| !lease.is_expired())
        .map(|(id, _)| id.clone())
        .collect()
}

pub fn revoke_all_for_path(vault: &mut SecretVault, path: &str) -> usize {
    let secret_value = match vault.secrets.get(path) {
        Some(v) => v.clone(),
        None => return 0,
    };
    let mut count = 0;
    for lease in vault.leases.values_mut() {
        if lease.secret == secret_value && !lease.revoked {
            lease.revoked = true;
            count += 1;
        }
    }
    count
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

        assert_ne!(lease1.secret, lease2.secret);
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

        let remaining = renew_lease(&mut vault, &lease_id, Duration::from_secs(3600)).unwrap();
        assert!(remaining > Duration::from_secs(100));

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

        let active = list_active_leases(&vault);
        assert_eq!(active.len(), 1);
    }
}
