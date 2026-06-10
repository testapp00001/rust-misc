//! # Lesson 06: Key Migration from Classical to PQC
//!
//! ## The Migration Challenge
//!
//! Organizations have billions of keys, certificates, and encrypted data using
//! classical algorithms (RSA, ECDH, ECDSA). Migrating to PQC is a multi-year
//! effort that requires careful planning.
//!
//! ## Migration Phases
//!
//! ```
//! Phase 1: Inventory     → What keys exist? Where? How are they used?
//! Phase 2: Prioritize     → Which data must stay secret longest?
//! Phase 3: Hybrid Deploy  → Add PQC alongside classical
//! Phase 4: Classical Drop → Remove classical algorithms
//! Phase 5: Verify         → Confirm PQC-only operation
//! ```
//!
//! ## Key Hierarchy
//!
//! ```
//! Root CA (RSA/ECDSA)
//!   ├── Intermediate CA → Migrate to ML-DSA
//!   │     ├── Server cert → Migrate to hybrid
//!   │     └── Client cert → Migrate to hybrid
//!   └── Backup keys → MUST be re-encrypted with PQC
//! ```
//!
//! ## Attack: Stale Keys
//!
//! Keys that haven't been rotated are the highest risk. An attacker who
//! compromises an old key can decrypt all past traffic encrypted with it.
//! "Harvest now, decrypt later" targets these stale keys.

use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Represents a cryptographic key in an organization's inventory.
#[derive(Debug, Clone)]
pub struct CryptoKey {
    pub id: String,
    pub algorithm: String,
    pub key_size: u32,
    pub created_at: u64,      // timestamp
    pub expires_at: u64,      // timestamp
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
    HybridDeployed,   // PQC added alongside classical
    PqcOnly,          // Classical removed
    Deprecated,       // Key is being phased out
}

/// Exercise 1: Inventory — identify keys vulnerable to quantum attack.
///
/// Given a list of CryptoKeys, return the IDs of all keys using algorithms
/// vulnerable to Shor's algorithm (RSA, DH, ECDH, ECDSA, DSA).
///
/// Keys already marked as PQC are NOT vulnerable.
pub fn find_quantum_vulnerable_keys(keys: &[CryptoKey]) -> Vec<String> {
    todo!("Find keys vulnerable to quantum attacks")
}

/// Exercise 2: Prioritize keys by migration urgency.
///
/// Keys should be prioritized by:
/// 1. How long the data must remain secret (longer = more urgent)
/// 2. Certificate authority keys (highest priority)
/// 3. Keys expiring soonest (need renewal with PQC)
///
/// Return key IDs sorted by priority (most urgent first).
///
/// Priority scoring:
/// - CA key: +100 points
/// - Encryption key: +50 points
/// - Expiring within 1 year (365 days): +30 points
/// - Older than 2 years (730 days): +20 points
pub fn prioritize_migration(
    keys: &[CryptoKey],
    current_time: u64,
) -> Vec<String> {
    todo!("Prioritize keys for PQC migration")
}

/// Exercise 3: Create a hybrid key pair — add PQC to an existing classical key.
///
/// Given a classical key, create a new hybrid entry that combines both.
/// The hybrid key should:
/// - Have algorithm = "Hybrid-{classical_alg}+ML-KEM-768"
/// - Have the same usage as the original
/// - Be marked as PQC (is_pqc = true)
/// - Have the same expiration
pub fn create_hybrid_key(classical_key: &CryptoKey) -> CryptoKey {
    todo!("Create a hybrid key from a classical key")
}

/// Exercise 4: Check if a migration plan is complete.
///
/// A migration plan is a map of key_id -> MigrationStatus.
/// It is complete if ALL keys have status PqcOnly or Deprecated.
pub fn is_migration_complete(plan: &HashMap<String, MigrationStatus>) -> bool {
    todo!("Check if migration plan is complete")
}

/// Exercise 5: Estimate migration timeline.
///
/// Given the number of keys to migrate and a rate of keys per day,
/// estimate the number of days to complete migration.
///
/// Round up to the nearest whole day.
pub fn estimate_migration_days(total_keys: usize, keys_per_day: usize) -> u64 {
    todo!("Estimate migration timeline in days")
}

/// Exercise 6: Generate a migration report.
///
/// Given a list of keys and their migration statuses, return a summary:
/// (total_keys, vulnerable_count, migrated_count, pending_count)
///
/// - total_keys: total number of keys
/// - vulnerable_count: keys using quantum-vulnerable algorithms AND not yet PQC
/// - migrated_count: keys with PQC (is_pqc = true)
/// - pending_count: total - migrated_count
pub fn migration_report(keys: &[CryptoKey]) -> (usize, usize, usize, usize) {
    todo!("Generate migration status report")
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
            expires_at: 1000 + 86400 * 365 * 2, // 2 years
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
            make_key("k1", "RSA", KeyUsage::Encryption, true), // already PQC hybrid
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
        assert_eq!(estimate_migration_days(101, 10), 11); // rounds up
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
        assert_eq!(vulnerable, 1); // k1
        assert_eq!(migrated, 2); // k2, k3
        assert_eq!(pending, 1); // k1
    }
}
