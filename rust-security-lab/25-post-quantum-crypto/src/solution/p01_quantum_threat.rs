//! # Lesson 01: Quantum Threat Model — Shor's and Grover's Algorithms (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};

/// Represents a quantum attack simulation result.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumAttackResult {
    pub algorithm: String,
    pub classical_bits: u32,
    pub quantum_bits: u32,
    pub is_broken: bool,
    pub attack_name: String,
}

/// Classify the impact of Shor's algorithm on a given algorithm.
///
/// Shor's algorithm breaks all public-key systems based on integer factoring
/// or discrete logarithms (including elliptic curves).
pub fn classify_shor_impact(algorithm: &str, key_size: u32) -> QuantumAttackResult {
    match algorithm {
        "RSA" => QuantumAttackResult {
            algorithm: algorithm.to_string(),
            classical_bits: key_size / 10,
            quantum_bits: 0,
            is_broken: true,
            attack_name: "Shor's factoring".to_string(),
        },
        "DH" | "ECDH" | "ECDSA" => QuantumAttackResult {
            algorithm: algorithm.to_string(),
            classical_bits: key_size,
            quantum_bits: 0,
            is_broken: true,
            attack_name: "Shor's discrete log".to_string(),
        },
        _ => QuantumAttackResult {
            algorithm: algorithm.to_string(),
            classical_bits: key_size,
            quantum_bits: key_size,
            is_broken: false,
            attack_name: "None".to_string(),
        },
    }
}

/// Calculate the effective quantum security level after Grover's attack.
///
/// Grover's algorithm provides a quadratic speedup for brute-force search:
/// a 2^n key space is searched in 2^(n/2) quantum operations.
pub fn grover_security_level(classical_bits: u32) -> u32 {
    classical_bits / 2
}

/// Determine if a symmetric cipher needs a key-size upgrade for quantum safety.
///
/// Compare the post-Grover security level against the minimum acceptable threshold.
pub fn needs_key_upgrade(classical_bits: u32, min_quantum_bits: u32) -> bool {
    grover_security_level(classical_bits) < min_quantum_bits
}

/// Simulate a Grover's search oracle by brute force.
///
/// In a real quantum computer, Grover's algorithm would find the preimage in
/// ~2^(max_bits/2) steps instead of 2^max_bits. We simulate the classical
/// equivalent to demonstrate the concept.
pub fn simulate_grover_search(target_hash: &[u8], max_bits: u32) -> Option<Vec<u8>> {
    let limit = 1u32 << max_bits;
    for i in 0..limit {
        let bytes = i.to_be_bytes();
        // Use only the relevant bytes for the search space
        let start = 4 - ((max_bits + 7) / 8) as usize;
        let preimage = &bytes[start..];
        let hash = sha256(preimage);
        if hash == target_hash {
            return Some(preimage.to_vec());
        }
    }
    None
}

/// Estimate the number of logical qubits needed to break RSA.
///
/// Shor's algorithm for factoring an n-bit modulus requires approximately 2n
/// logical qubits (modular exponentiation circuit + measurement).
pub fn estimate_rsa_qubits(key_size: u32) -> u32 {
    key_size * 2
}

/// Generate a quantum risk report for a set of algorithms.
///
/// Classifies each algorithm based on whether it is vulnerable to Shor's
/// (public-key) or Grover's (symmetric) attacks.
pub fn generate_risk_report(algorithms: &[(&str, u32)]) -> Vec<QuantumAttackResult> {
    algorithms
        .iter()
        .map(|(name, size)| {
            // Public-key algorithms: Shor's breaks them
            if *name == "RSA" || *name == "DH" || *name == "ECDH" || *name == "ECDSA" {
                classify_shor_impact(name, *size)
            }
            // Symmetric/hash algorithms: Grover's weakens them
            else {
                let quantum_bits = grover_security_level(*size);
                QuantumAttackResult {
                    algorithm: name.to_string(),
                    classical_bits: *size,
                    quantum_bits,
                    is_broken: false,
                    attack_name: "Grover's (weakened)".to_string(),
                }
            }
        })
        .collect()
}

/// Helper: compute SHA-256 hash
fn sha256(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_shor_rsa() {
        let result = classify_shor_impact("RSA", 2048);
        assert!(result.is_broken);
        assert_eq!(result.quantum_bits, 0);
        assert_eq!(result.attack_name, "Shor's factoring");
    }

    #[test]
    fn test_classify_shor_ecdh() {
        let result = classify_shor_impact("ECDH", 256);
        assert!(result.is_broken);
        assert_eq!(result.quantum_bits, 0);
        assert_eq!(result.attack_name, "Shor's discrete log");
    }

    #[test]
    fn test_classify_shor_unknown() {
        let result = classify_shor_impact("AES", 256);
        assert!(!result.is_broken);
        assert_eq!(result.quantum_bits, 256);
    }

    #[test]
    fn test_grover_aes128() {
        assert_eq!(grover_security_level(128), 64);
    }

    #[test]
    fn test_grover_aes256() {
        assert_eq!(grover_security_level(256), 128);
    }

    #[test]
    fn test_grover_zero() {
        assert_eq!(grover_security_level(0), 0);
    }

    #[test]
    fn test_needs_upgrade_aes128() {
        assert!(needs_key_upgrade(128, 128));
    }

    #[test]
    fn test_needs_upgrade_aes256() {
        assert!(!needs_key_upgrade(256, 128));
    }

    #[test]
    fn test_simulate_grover_small() {
        let preimage: u8 = 7;
        let hash = sha256(&[preimage]);
        let result = simulate_grover_search(&hash, 4);
        assert_eq!(result, Some(vec![7]));
    }

    #[test]
    fn test_simulate_grover_not_found() {
        let hash = sha256(&[255u8]);
        let result = simulate_grover_search(&hash, 3);
        assert_eq!(result, None);
    }

    #[test]
    fn test_estimate_rsa_qubits() {
        assert_eq!(estimate_rsa_qubits(2048), 4096);
    }

    #[test]
    fn test_risk_report() {
        let algorithms = vec![("RSA", 2048u32), ("AES", 256u32)];
        let report = generate_risk_report(&algorithms);
        assert_eq!(report.len(), 2);
        assert!(report[0].is_broken);
        assert!(!report[1].is_broken);
    }
}
