//! # Lesson 07: Crypto Agility — Swap Algorithms Easily (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Supported cryptographic algorithm families.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AlgorithmFamily {
    Symmetric,
    KeyExchange,
    Signature,
    Hash,
}

/// A cryptographic algorithm with its properties.
#[derive(Debug, Clone)]
pub struct Algorithm {
    pub name: String,
    pub family: AlgorithmFamily,
    pub security_bits: u32,
    pub is_pqc: bool,
    pub oid: String,
}

/// A policy specifying minimum security requirements.
#[derive(Debug, Clone)]
pub struct CryptoPolicy {
    pub min_security_bits: u32,
    pub require_pqc: bool,
    pub allowed_algorithms: Vec<String>,
    pub blocked_algorithms: Vec<String>,
}

/// Create a registry of standard algorithms.
pub fn create_algorithm_registry() -> HashMap<String, Algorithm> {
    let entries = vec![
        ("AES-128-GCM", AlgorithmFamily::Symmetric, 128, false, "2.16.840.1.101.3.4.1.6"),
        ("AES-256-GCM", AlgorithmFamily::Symmetric, 256, false, "2.16.840.1.101.3.4.1.46"),
        ("ChaCha20-Poly1305", AlgorithmFamily::Symmetric, 256, false, "1.2.840.113549.1.9.16.3.18"),
        ("ECDH-P256", AlgorithmFamily::KeyExchange, 128, false, "1.2.840.10045.3.1.7"),
        ("ML-KEM-768", AlgorithmFamily::KeyExchange, 192, true, "2.16.840.1.101.3.4.4.2"),
        ("ECDSA-P256", AlgorithmFamily::Signature, 128, false, "1.2.840.10045.4.3.2"),
        ("ML-DSA-65", AlgorithmFamily::Signature, 192, true, "2.16.840.1.101.3.4.3.18"),
        ("SPHINCS+-SHA256-128s", AlgorithmFamily::Signature, 128, true, "2.16.840.1.101.3.4.3.20"),
        ("SHA-256", AlgorithmFamily::Hash, 128, false, "2.16.840.1.101.3.4.2.1"),
        ("SHA-384", AlgorithmFamily::Hash, 192, false, "2.16.840.1.101.3.4.2.2"),
    ];

    let mut map = HashMap::new();
    for (name, family, bits, pqc, oid) in entries {
        map.insert(
            name.to_string(),
            Algorithm {
                name: name.to_string(),
                family,
                security_bits: bits,
                is_pqc: pqc,
                oid: oid.to_string(),
            },
        );
    }
    map
}

/// Check if an algorithm meets a crypto policy.
pub fn is_algorithm_allowed(algo: &Algorithm, policy: &CryptoPolicy) -> bool {
    // Check blocked list
    if policy.blocked_algorithms.contains(&algo.name) {
        return false;
    }

    // Check allowed list (if non-empty, must be in it)
    if !policy.allowed_algorithms.is_empty() && !policy.allowed_algorithms.contains(&algo.name) {
        return false;
    }

    // Check minimum security
    if algo.security_bits < policy.min_security_bits {
        return false;
    }

    // Check PQC requirement
    if policy.require_pqc && !algo.is_pqc {
        return false;
    }

    true
}

/// Select the best algorithm for a family meeting the policy.
pub fn select_best_algorithm(
    registry: &HashMap<String, Algorithm>,
    family: &AlgorithmFamily,
    policy: &CryptoPolicy,
) -> Option<Algorithm> {
    registry
        .values()
        .filter(|a| a.family == *family && is_algorithm_allowed(a, policy))
        .max_by_key(|a| a.security_bits)
        .cloned()
}

/// Detect algorithm downgrade: server chose weaker than client's best.
pub fn detect_downgrade(
    offered: &[Algorithm],
    selected: &Algorithm,
) -> bool {
    let max_offered = offered.iter().map(|a| a.security_bits).max().unwrap_or(0);
    selected.security_bits < max_offered
}

/// Negotiate algorithm between client offers and server policy.
pub fn negotiate_algorithm(
    client_offers: &[Algorithm],
    server_policy: &CryptoPolicy,
) -> Option<String> {
    client_offers
        .iter()
        .find(|a| is_algorithm_allowed(a, server_policy))
        .map(|a| a.name.clone())
}

/// Generate algorithm migration plan.
pub fn migration_plan(
    current_algorithms: &[Algorithm],
    registry: &HashMap<String, Algorithm>,
    target_policy: &CryptoPolicy,
) -> Vec<(String, String)> {
    current_algorithms
        .iter()
        .filter(|a| !is_algorithm_allowed(a, target_policy))
        .filter_map(|a| {
            select_best_algorithm(registry, &a.family, target_policy)
                .map(|recommended| (a.name.clone(), recommended.name.clone()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_algo(name: &str, family: AlgorithmFamily, bits: u32, is_pqc: bool) -> Algorithm {
        Algorithm {
            name: name.to_string(),
            family,
            security_bits: bits,
            is_pqc,
            oid: "1.2.3.4".to_string(),
        }
    }

    fn test_policy() -> CryptoPolicy {
        CryptoPolicy {
            min_security_bits: 128,
            require_pqc: false,
            allowed_algorithms: vec![],
            blocked_algorithms: vec![],
        }
    }

    fn pqc_policy() -> CryptoPolicy {
        CryptoPolicy {
            min_security_bits: 128,
            require_pqc: true,
            allowed_algorithms: vec![],
            blocked_algorithms: vec![],
        }
    }

    #[test]
    fn test_registry_size() {
        let reg = create_algorithm_registry();
        assert_eq!(reg.len(), 10);
    }

    #[test]
    fn test_registry_contains_ml_kem() {
        let reg = create_algorithm_registry();
        assert!(reg.contains_key("ML-KEM-768"));
        assert!(reg["ML-KEM-768"].is_pqc);
    }

    #[test]
    fn test_algorithm_allowed_basic() {
        let algo = make_algo("AES-256-GCM", AlgorithmFamily::Symmetric, 256, false);
        let policy = test_policy();
        assert!(is_algorithm_allowed(&algo, &policy));
    }

    #[test]
    fn test_algorithm_blocked() {
        let algo = make_algo("AES-128-GCM", AlgorithmFamily::Symmetric, 128, false);
        let policy = CryptoPolicy {
            min_security_bits: 128,
            require_pqc: false,
            allowed_algorithms: vec![],
            blocked_algorithms: vec!["AES-128-GCM".to_string()],
        };
        assert!(!is_algorithm_allowed(&algo, &policy));
    }

    #[test]
    fn test_algorithm_requires_pqc() {
        let classical = make_algo("ECDH-P256", AlgorithmFamily::KeyExchange, 128, false);
        let pqc = make_algo("ML-KEM-768", AlgorithmFamily::KeyExchange, 192, true);
        let policy = pqc_policy();
        assert!(!is_algorithm_allowed(&classical, &policy));
        assert!(is_algorithm_allowed(&pqc, &policy));
    }

    #[test]
    fn test_select_best_symmetric() {
        let mut reg = HashMap::new();
        reg.insert("AES-128".to_string(), make_algo("AES-128", AlgorithmFamily::Symmetric, 128, false));
        reg.insert("AES-256".to_string(), make_algo("AES-256", AlgorithmFamily::Symmetric, 256, false));
        let policy = test_policy();
        let best = select_best_algorithm(&reg, &AlgorithmFamily::Symmetric, &policy);
        assert_eq!(best.unwrap().name, "AES-256");
    }

    #[test]
    fn test_detect_downgrade() {
        let offered = vec![
            make_algo("AES-256", AlgorithmFamily::Symmetric, 256, false),
            make_algo("AES-128", AlgorithmFamily::Symmetric, 128, false),
        ];
        let selected = make_algo("AES-128", AlgorithmFamily::Symmetric, 128, false);
        assert!(detect_downgrade(&offered, &selected));
    }

    #[test]
    fn test_no_downgrade() {
        let offered = vec![
            make_algo("AES-256", AlgorithmFamily::Symmetric, 256, false),
            make_algo("AES-128", AlgorithmFamily::Symmetric, 128, false),
        ];
        let selected = make_algo("AES-256", AlgorithmFamily::Symmetric, 256, false);
        assert!(!detect_downgrade(&offered, &selected));
    }

    #[test]
    fn test_negotiate_success() {
        let offers = vec![
            make_algo("AES-128", AlgorithmFamily::Symmetric, 128, false),
            make_algo("AES-256", AlgorithmFamily::Symmetric, 256, false),
        ];
        let policy = CryptoPolicy {
            min_security_bits: 256,
            require_pqc: false,
            allowed_algorithms: vec![],
            blocked_algorithms: vec![],
        };
        let result = negotiate_algorithm(&offers, &policy);
        assert_eq!(result, Some("AES-256".to_string()));
    }

    #[test]
    fn test_negotiate_failure() {
        let offers = vec![
            make_algo("AES-128", AlgorithmFamily::Symmetric, 128, false),
        ];
        let policy = CryptoPolicy {
            min_security_bits: 256,
            require_pqc: false,
            allowed_algorithms: vec![],
            blocked_algorithms: vec![],
        };
        let result = negotiate_algorithm(&offers, &policy);
        assert_eq!(result, None);
    }
}
