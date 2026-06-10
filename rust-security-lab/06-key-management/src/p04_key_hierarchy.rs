//! # Lesson 04: Key Hierarchy Design
//!
//! ## What Is a Key Hierarchy?
//!
//! A key hierarchy organizes cryptographic keys in a tree structure where higher-level
//! keys (master keys) derive or encrypt lower-level keys (domain keys, operational keys).
//!
//! ```text
//!                     Master Key (MK)
//!                    /       |        \
//!            User Key    Payment Key   Log Key
//!            /     \        |            \
//!        DEK_u1  DEK_u2  DEK_pay1     DEK_log1
//! ```
//!
//! ## Why Use a Hierarchy?
//!
//! 1. **Compromise isolation**: If DEK_u1 is compromised, other keys are safe
//! 2. **Independent rotation**: Rotate payment keys without touching user keys
//! 3. **Access control**: Different teams/services hold different domain keys
//! 4. **Reduced master key exposure**: Master key is used rarely (only to derive/unwrap)
//!
//! ## Attack Scenario: Flat Key Structure
//!
//! If every key is independent and stored separately:
//! - 1000 data encryption keys = 1000 keys to manage
//! - No way to bulk-rotate or revoke
//! - Single breach exposes everything
//!
//! With a hierarchy:
//! - Master key derives domain keys
//! - Rotating the domain key automatically re-keys all operational keys under it
//! - Master key can be stored in an HSM, rarely accessed
//!
//! ## Security Principles
//!
//! 1. Master key should ONLY derive/unwrap — never encrypt data directly
//! 2. Each level should use HKDF with unique info strings
//! 3. Compromise of a child key must not reveal the parent
//! 4. Document the hierarchy for operational clarity

use hkdf::Hkdf;
use sha2::Sha256;

/// Represents a key in the hierarchy with its metadata.
#[derive(Debug, Clone)]
pub struct KeyEntry {
    pub id: String,
    pub key: Vec<u8>,
    pub domain: String,
}

/// Exercise 1: Derive a domain key from a master key.
///
/// Use HKDF with the master key as input and a domain name as the info string.
///
/// Hints:
/// - Use `Hkdf::<Sha256>::new(salt, master_key)`
/// - Expand with `domain_name.as_bytes()` as info
/// - 32-byte output for AES-256
pub fn derive_domain_key(master_key: &[u8], salt: &[u8], domain: &str) -> Vec<u8> {
    todo!("Derive a domain key from master key using HKDF")
}

/// Exercise 2: Derive an operational key from a domain key.
///
/// Operational keys are leaf-level keys used for actual encryption/signing.
/// They're derived from domain keys using HKDF.
///
/// Hints:
/// - Use HKDF with domain key as input
/// - info = "operational:{key_id}"
pub fn derive_operational_key(domain_key: &[u8], salt: &[u8], key_id: &str) -> Vec<u8> {
    todo!("Derive an operational key from a domain key")
}

/// Exercise 3: Build a complete key hierarchy.
///
/// Given a master key, derive domain keys for each domain, and then derive
/// `keys_per_domain` operational keys under each domain.
///
/// Returns a flat list of KeyEntry with (id, key, domain).
pub fn build_hierarchy(
    master_key: &[u8],
    salt: &[u8],
    domains: &[&str],
    keys_per_domain: usize,
) -> Vec<KeyEntry> {
    todo!("Build a complete key hierarchy")
}

/// Exercise 4: Look up a key by its ID from a hierarchy.
///
/// Given a hierarchy (list of KeyEntry) and a key_id, return the matching key.
///
/// Hints:
/// - Search through the entries for a matching `id`
/// - Return `None` if not found
pub fn find_key(hierarchy: &[KeyEntry], key_id: &str) -> Option<Vec<u8>> {
    todo!("Find a key by ID in the hierarchy")
}

/// Exercise 5: Demonstrate domain isolation.
///
/// Derive keys from two different domains. Keys from different domains must be
/// different, even with the same key_id suffix.
///
/// Returns (domain1_key, domain2_key)
pub fn demonstrate_domain_isolation(
    master_key: &[u8],
    salt: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    todo!("Show that keys from different domains are independent")
}

/// Exercise 6: Demonstrate that compromising a child doesn't reveal the parent.
///
/// Given an operational key and the salt used, verify that you CANNOT derive
/// the domain key or master key from it. HKDF is one-way in the expand direction.
///
/// This function should:
/// 1. Derive a domain key from master
/// 2. Derive an operational key from domain key
/// 3. Demonstrate that the operational key cannot be "reversed" to get the domain key
///
/// Returns (domain_key, operational_key, attempted_reverse_is_none)
pub fn demonstrate_one_way(
    master_key: &[u8],
    salt: &[u8],
) -> (Vec<u8>, Vec<u8>, bool) {
    todo!("Demonstrate that HKDF is one-way")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_master() -> Vec<u8> {
        vec![0xABu8; 32]
    }

    fn test_salt() -> Vec<u8> {
        vec![0xCDu8; 16]
    }

    #[test]
    fn test_domain_key_length() {
        let key = derive_domain_key(&test_master(), &test_salt(), "users");
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_domain_key_deterministic() {
        let key1 = derive_domain_key(&test_master(), &test_salt(), "users");
        let key2 = derive_domain_key(&test_master(), &test_salt(), "users");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_domains_different_keys() {
        let key1 = derive_domain_key(&test_master(), &test_salt(), "users");
        let key2 = derive_domain_key(&test_master(), &test_salt(), "payments");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_operational_key_length() {
        let domain_key = derive_domain_key(&test_master(), &test_salt(), "users");
        let op_key = derive_operational_key(&domain_key, &test_salt(), "user-001");
        assert_eq!(op_key.len(), 32);
    }

    #[test]
    fn test_operational_keys_different_ids() {
        let domain_key = derive_domain_key(&test_master(), &test_salt(), "users");
        let key1 = derive_operational_key(&domain_key, &test_salt(), "user-001");
        let key2 = derive_operational_key(&domain_key, &test_salt(), "user-002");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_build_hierarchy_count() {
        let hierarchy = build_hierarchy(&test_master(), &test_salt(), &["users", "payments"], 3);
        assert_eq!(hierarchy.len(), 6, "2 domains * 3 keys = 6 entries");
    }

    #[test]
    fn test_find_key() {
        let hierarchy = build_hierarchy(&test_master(), &test_salt(), &["users"], 3);
        let key = find_key(&hierarchy, "users-0");
        assert!(key.is_some());
        let missing = find_key(&hierarchy, "nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_domain_isolation() {
        let (key1, key2) = demonstrate_domain_isolation(&test_master(), &test_salt());
        assert_ne!(key1, key2, "Keys from different domains must differ");
    }

    #[test]
    fn test_hierarchy_uniqueness() {
        let hierarchy = build_hierarchy(&test_master(), &test_salt(), &["a", "b", "c"], 5);
        // All keys should be unique
        for i in 0..hierarchy.len() {
            for j in (i + 1)..hierarchy.len() {
                assert_ne!(
                    hierarchy[i].key, hierarchy[j].key,
                    "Keys {} and {} should differ",
                    hierarchy[i].id, hierarchy[j].id
                );
            }
        }
    }
}
