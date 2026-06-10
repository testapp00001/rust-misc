//! # Lesson 06: Key Migration from Classical to PQC (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Represents a cryptographic key in an organization's inventory.
#[derive(Debug, Clone)]
pub struct CryptoKey {
    pub id: String,
    pub algorithm: String,
    pub key_size: u32,
    pub created_at: u64,
    pub expires_at: u64,
    pub usage: KeyUsage,
    pub is_pqc: bool,
}

/// Key usage categories.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyUsage {
    Encryption,
    Signing,
    KeyExchange,
    CertificateAuthority,
}

/// Migration status for a key.
#[derive(Debug, Clone, PartialEq)]
pub enum MigrationStatus {
    NotStarted,
    HybridDeployed,
    PqcOnly,
    Deprecated,
}

/// Find keys vulnerable to Shor's algorithm.
///
/// Vulnerable algorithms: RSA, DH, ECDH, ECDSA, DSA (all factoring/DLP based).
pub fn find_quantum_vulnerable_keys(keys: &[CryptoKey]) -> Vec<String> {
    let vulnerable_algs = ["RSA", "DH", "ECDH", "ECDH-P256", "ECDH-P384",
                           "ECDSA", "ECDSA-P256", "ECDSA-P384", "DSA"];

    keys.iter()
        .filter(|k| !k.is_pqc && vulnerable_algs.iter().any(|&a| k.algorithm.contains(a)))
        .map(|k| k.id.clone())
        .collect()
}

/// Prioritize keys by migration urgency.
pub fn prioritize_migration(
    keys: &[CryptoKey],
    current_time: u64,
) -> Vec<String> {
    let one_year = 365 * 86400;
    let two_years = 2 * 365 * 86400;

    let mut scored: Vec<(String, i32)> = keys
        .iter()
        .map(|k| {
            let mut score = 0i32;

            // CA keys are highest priority
            if k.usage == KeyUsage::CertificateAuthority {
                score += 100;
            }

            // Encryption keys are high priority (data at rest risk)
            if k.usage == KeyUsage::Encryption {
                score += 50;
            }

            // Expiring soon
            if k.expires_at.saturating_sub(current_time) < one_year {
                score += 30;
            }

            // Old keys
            if current_time.saturating_sub(k.created_at) > two_years {
                score += 20;
            }

            (k.id.clone(), score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().map(|(id, _)| id).collect()
}

/// Create a hybrid key from a classical key.
pub fn create_hybrid_key(classical_key: &CryptoKey) -> CryptoKey {
    CryptoKey {
        id: format!("hybrid-{}", classical_key.id),
        algorithm: format!("Hybrid-{}+ML-KEM-768", classical_key.algorithm),
        key_size: classical_key.key_size,
        created_at: classical_key.created_at,
        expires_at: classical_key.expires_at,
        usage: classical_key.usage.clone(),
        is_pqc: true,
    }
}

/// Check if migration is complete: all keys must be PqcOnly or Deprecated.
pub fn is_migration_complete(plan: &HashMap<String, MigrationStatus>) -> bool {
    plan.values().all(|status| {
        *status == MigrationStatus::PqcOnly || *status == MigrationStatus::Deprecated
    })
}

/// Estimate migration timeline in days (rounded up).
pub fn estimate_migration_days(total_keys: usize, keys_per_day: usize) -> u64 {
    if keys_per_day == 0 || total_keys == 0 {
        return 0;
    }
    ((total_keys as f64) / (keys_per_day as f64)).ceil() as u64
}

/// Generate a migration status report.
pub fn migration_report(keys: &[CryptoKey]) -> (usize, usize, usize, usize) {
    let total = keys.len();
    let migrated = keys.iter().filter(|k| k.is_pqc).count();
    let vulnerable = keys.iter().filter(|k| {
        !k.is_pqc && find_quantum_vulnerable_keys(&[k.clone()]).len() > 0
    }).count();
    let pending = total - migrated;
    (total, vulnerable, migrated, pending)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(id: &str, alg: &str, usage: KeyUsage, is_pqc: bool) -> CryptoKey {
        CryptoKey {
            id: id.to_string(),
            algorithm: alg.to_string(),
            key_size: 2048,
            created_at: 1000,
            expires_at: 1000 + 86400 * 365 * 2,
            usage,
            is_pqc,
        }
    }

    #[test]
    fn test_find_vulnerable_rsa() {
        let keys = vec![
            make_key("k1", "RSA", KeyUsage::Encryption, false),
            make_key("k2", "ML-KEM-768", KeyUsage::KeyExchange, true),
        ];
        let vulnerable = find_quantum_vulnerable_keys(&keys);
        assert_eq!(vulnerable, vec!["k1"]);
    }

    #[test]
    fn test_find_vulnerable_ecdh() {
        let keys = vec![
            make_key("k1", "ECDH-P256", KeyUsage::KeyExchange, false),
            make_key("k2", "ECDSA-P256", KeyUsage::Signing, false),
        ];
        let vulnerable = find_quantum_vulnerable_keys(&keys);
        assert_eq!(vulnerable.len(), 2);
    }

    #[test]
    fn test_find_vulnerable_pqc_not_counted() {
        let keys = vec![
            make_key("k1", "RSA", KeyUsage::Encryption, true),
        ];
        let vulnerable = find_quantum_vulnerable_keys(&keys);
        assert!(vulnerable.is_empty());
    }

    #[test]
    fn test_prioritize_ca_first() {
        let keys = vec![
            make_key("server", "RSA", KeyUsage::Encryption, false),
            make_key("ca", "RSA", KeyUsage::CertificateAuthority, false),
        ];
        let prioritized = prioritize_migration(&keys, 1000);
        assert_eq!(prioritized[0], "ca");
    }

    #[test]
    fn test_create_hybrid_key() {
        let classical = make_key("k1", "RSA", KeyUsage::Encryption, false);
        let hybrid = create_hybrid_key(&classical);
        assert!(hybrid.algorithm.contains("Hybrid"));
        assert!(hybrid.algorithm.contains("ML-KEM-768"));
        assert!(hybrid.is_pqc);
        assert_eq!(hybrid.usage, KeyUsage::Encryption);
    }

    #[test]
    fn test_migration_complete() {
        let mut plan = HashMap::new();
        plan.insert("k1".to_string(), MigrationStatus::PqcOnly);
        plan.insert("k2".to_string(), MigrationStatus::Deprecated);
        assert!(is_migration_complete(&plan));
    }

    #[test]
    fn test_migration_not_complete() {
        let mut plan = HashMap::new();
        plan.insert("k1".to_string(), MigrationStatus::PqcOnly);
        plan.insert("k2".to_string(), MigrationStatus::HybridDeployed);
        assert!(!is_migration_complete(&plan));
    }

    #[test]
    fn test_estimate_migration_days() {
        assert_eq!(estimate_migration_days(100, 10), 10);
        assert_eq!(estimate_migration_days(101, 10), 11);
        assert_eq!(estimate_migration_days(0, 10), 0);
    }

    #[test]
    fn test_migration_report() {
        let keys = vec![
            make_key("k1", "RSA", KeyUsage::Encryption, false),
            make_key("k2", "ECDH", KeyUsage::KeyExchange, true),
            make_key("k3", "ML-KEM-768", KeyUsage::KeyExchange, true),
        ];
        let (total, vulnerable, migrated, pending) = migration_report(&keys);
        assert_eq!(total, 3);
        assert_eq!(vulnerable, 1);
        assert_eq!(migrated, 2);
        assert_eq!(pending, 1);
    }
}
