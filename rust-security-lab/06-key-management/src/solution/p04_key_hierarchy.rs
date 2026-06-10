//! # Lesson 04: Key Hierarchy Design (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use hkdf::Hkdf;
use sha2::Sha256;

/// Represents a key in the hierarchy with its metadata.
#[derive(Debug, Clone)]
pub struct KeyEntry {
    pub id: String,
    pub key: Vec<u8>,
    pub domain: String,
}

/// Derive a domain key from a master key using HKDF.
///
/// The domain name serves as the HKDF `info` parameter, ensuring each domain
/// gets a cryptographically independent key from the same master.
pub fn derive_domain_key(master_key: &[u8], salt: &[u8], domain: &str) -> Vec<u8> {
    let hk = Hkdf::<Sha256>::new(Some(salt), master_key);
    let mut okm = vec![0u8; 32];
    let info = format!("domain:{}", domain);
    hk.expand(info.as_bytes(), &mut okm).expect("Valid HKDF expand");
    okm
}

/// Derive an operational key from a domain key using HKDF.
///
/// Two-level derivation: master -> domain -> operational.
/// Even if two domains happen to use the same key_id, the resulting
/// operational keys are different because they come from different domain keys.
pub fn derive_operational_key(domain_key: &[u8], salt: &[u8], key_id: &str) -> Vec<u8> {
    let hk = Hkdf::<Sha256>::new(Some(salt), domain_key);
    let mut okm = vec![0u8; 32];
    let info = format!("operational:{}", key_id);
    hk.expand(info.as_bytes(), &mut okm).expect("Valid HKDF expand");
    okm
}

/// Build a complete key hierarchy.
///
/// This demonstrates the standard pattern used in KMS systems:
/// - Master key (in HSM) derives domain keys
/// - Domain keys derive operational keys
/// - Each level has unique info strings for HKDF
pub fn build_hierarchy(
    master_key: &[u8],
    salt: &[u8],
    domains: &[&str],
    keys_per_domain: usize,
) -> Vec<KeyEntry> {
    let mut entries = Vec::new();
    for domain in domains {
        let domain_key = derive_domain_key(master_key, salt, domain);
        for i in 0..keys_per_domain {
            let key_id = format!("{}-{}", domain, i);
            let key = derive_operational_key(&domain_key, salt, &key_id);
            entries.push(KeyEntry {
                id: key_id,
                key,
                domain: domain.to_string(),
            });
        }
    }
    entries
}

/// Find a key by its ID in the hierarchy.
pub fn find_key(hierarchy: &[KeyEntry], key_id: &str) -> Option<Vec<u8>> {
    hierarchy
        .iter()
        .find(|entry| entry.id == key_id)
        .map(|entry| entry.key.clone())
}

/// Demonstrate that keys from different domains are independent.
///
/// Even with the same operational key ID pattern, the domain isolation
/// ensures complete independence.
pub fn demonstrate_domain_isolation(
    master_key: &[u8],
    salt: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    let domain_a = derive_domain_key(master_key, salt, "domain-a");
    let domain_b = derive_domain_key(master_key, salt, "domain-b");
    (domain_a, domain_b)
}

/// Demonstrate that HKDF is one-way (pre-image resistant).
///
/// You cannot reverse an operational key to recover the domain key or master key.
/// This is because HKDF uses HMAC, which is built on a cryptographic hash function.
pub fn demonstrate_one_way(
    master_key: &[u8],
    salt: &[u8],
) -> (Vec<u8>, Vec<u8>, bool) {
    let domain_key = derive_domain_key(master_key, salt, "test-domain");
    let op_key = derive_operational_key(&domain_key, salt, "test-key-0");

    // There's no "reverse HKDF" — the only way to find the domain key
    // from the operational key would be to brute-force all possible domain keys,
    // which is computationally infeasible.
    // We demonstrate this by confirming that there's no function to reverse it.
    // The "attempted_reverse_is_none" flag shows the concept.
    let attempted_reverse_is_none = true; // HKDF cannot be reversed

    (domain_key, op_key, attempted_reverse_is_none)
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
