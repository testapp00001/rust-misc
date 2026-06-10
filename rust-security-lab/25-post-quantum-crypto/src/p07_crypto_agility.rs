//! # Lesson 07: Crypto Agility — Swap Algorithms Easily
//!
//! ## What is Crypto Agility?
//!
//! Crypto agility is the ability of a system to switch between cryptographic
//! algorithms WITHOUT changing the overall system architecture. This is critical
//! for PQC migration because:
//!
//! 1. PQC standards are new — algorithms may need to change
//! 2. Regulatory requirements may mandate specific algorithms
//! 3. A vulnerability in one algorithm requires rapid switching
//!
//! ## Design Principles
//!
//! ```
//! Bad:   fn encrypt(data: &[u8]) -> Vec<u8> { aes_gcm_encrypt(data, key) }
//! Good:  fn encrypt(data: &[u8], algo: Algorithm) -> Vec<u8> { algo.encrypt(data, key) }
//! ```
//!
//! Key patterns:
//! - **Algorithm identifiers**: OIDs or string names, not hardcoded
//! - **Trait-based dispatch**: Define traits, implement per algorithm
//! - **Negotiation**: Client and server agree on algorithms
//! - **Versioning**: Key metadata includes algorithm info
//!
//! ## Attack: Algorithm Downgrade
//!
//! An attacker forces the use of a weaker algorithm. Mitigation:
//! - Maintain minimum security levels
//! - Reject algorithms below threshold
//! - Log and alert on algorithm negotiation

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
    pub oid: String,  // Object Identifier
}

/// A policy that specifies minimum security requirements.
#[derive(Debug, Clone)]
pub struct CryptoPolicy {
    pub min_security_bits: u32,
    pub require_pqc: bool,
    pub allowed_algorithms: Vec<String>,
    pub blocked_algorithms: Vec<String>,
}

/// Exercise 1: Create a registry of algorithms.
///
/// Return a HashMap mapping algorithm name to Algorithm struct.
/// Include at least these algorithms:
/// - "AES-128-GCM": Symmetric, 128-bit, not PQC, OID "2.16.840.1.101.3.4.1.6"
/// - "AES-256-GCM": Symmetric, 256-bit, not PQC, OID "2.16.840.1.101.3.4.1.46"
/// - "ChaCha20-Poly1305": Symmetric, 256-bit, not PQC, OID "1.2.840.113549.1.9.16.3.18"
/// - "ECDH-P256": KeyExchange, 128-bit, not PQC, OID "1.2.840.10045.3.1.7"
/// - "ML-KEM-768": KeyExchange, 192-bit, PQC, OID "2.16.840.1.101.3.4.4.2"
/// - "ECDSA-P256": Signature, 128-bit, not PQC, OID "1.2.840.10045.4.3.2"
/// - "ML-DSA-65": Signature, 192-bit, PQC, OID "2.16.840.1.101.3.4.3.18"
/// - "SPHINCS+-SHA256-128s": Signature, 128-bit, PQC, OID "2.16.840.1.101.3.4.3.20"
/// - "SHA-256": Hash, 128-bit, not PQC, OID "2.16.840.1.101.3.4.2.1"
/// - "SHA-384": Hash, 192-bit, not PQC, OID "2.16.840.1.101.3.4.2.2"
pub fn create_algorithm_registry() -> HashMap<String, Algorithm> {
    todo!("Create algorithm registry with known algorithms")
}

/// Exercise 2: Check if an algorithm meets a crypto policy.
///
/// An algorithm is acceptable if:
/// 1. It meets the minimum security bits requirement
/// 2. If require_pqc is true, it must be PQC
/// 3. It is not in the blocked list
/// 4. If the allowed list is non-empty, it must be in the allowed list
pub fn is_algorithm_allowed(algo: &Algorithm, policy: &CryptoPolicy) -> bool {
    todo!("Check if algorithm meets policy requirements")
}

/// Exercise 3: Find the best algorithm for a given family and policy.
///
/// From the registry, find algorithms that:
/// 1. Match the requested family
/// 2. Meet the policy requirements
/// 3. Return the one with the highest security bits
///
/// Return None if no algorithm meets the requirements.
pub fn select_best_algorithm(
    registry: &HashMap<String, Algorithm>,
    family: &AlgorithmFamily,
    policy: &CryptoPolicy,
) -> Option<Algorithm> {
    todo!("Select best algorithm meeting policy requirements")
}

/// Exercise 4: Detect algorithm downgrade attack.
///
/// Given a client's offered algorithms and a server's selection, detect
/// if the server chose a weaker algorithm than the client's strongest.
///
/// Return true if a downgrade is detected (server's choice has fewer
/// security bits than the maximum offered by the client).
pub fn detect_downgrade(
    offered: &[Algorithm],
    selected: &Algorithm,
) -> bool {
    todo!("Detect algorithm downgrade attack")
}

/// Exercise 5: Negotiate algorithm between client and server.
///
/// Client offers a list of algorithms. Server has a policy.
/// Find the first client algorithm that meets the server's policy.
///
/// Return Some(algorithm_name) if agreement is possible, None otherwise.
pub fn negotiate_algorithm(
    client_offers: &[Algorithm],
    server_policy: &CryptoPolicy,
) -> Option<String> {
    todo!("Negotiate algorithm between client and server")
}

/// Exercise 6: Generate an algorithm migration plan.
///
/// Given current algorithms and a target policy, return a list of
/// (current_algorithm, recommended_algorithm) pairs where migration is needed.
///
/// Only include algorithms that don't meet the target policy.
/// The recommended algorithm should be the best available in the same family.
pub fn migration_plan(
    current_algorithms: &[Algorithm],
    registry: &HashMap<String, Algorithm>,
    target_policy: &CryptoPolicy,
) -> Vec<(String, String)> {
    todo!("Generate algorithm migration plan")
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
