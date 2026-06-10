//! # Lesson 01: Quantum Threat Model — Shor's and Grover's Algorithms
//!
//! ## Why PQC Matters Now
//!
//! Quantum computers exploit quantum mechanical phenomena (superposition, entanglement)
//! to solve certain mathematical problems exponentially faster than classical computers.
//! Two quantum algorithms threaten modern cryptography:
//!
//! - **Shor's Algorithm**: Factors large integers and computes discrete logarithms in
//!   polynomial time. Breaks RSA, DH, ECDH, ECDSA.
//! - **Grover's Algorithm**: Provides quadratic speedup for unstructured search.
//!   Effectively halves the security level of symmetric ciphers and hashes.
//!
//! ## Impact on Current Cryptography
//!
//! | Algorithm | Classical Security | Quantum Security | Status |
//! |-----------|-------------------|------------------|--------|
//! | RSA-2048 | 112-bit | ~0 (broken) | BROKEN |
//! | ECDH-P256 | 128-bit | ~0 (broken) | BROKEN |
//! | AES-128 | 128-bit | 64-bit | WEAKENED |
//! | AES-256 | 256-bit | 128-bit | ADEQUATE |
//! | SHA-256 | 128-bit collision | 128-bit collision | ADEQUATE |
//!
//! ## Attack Demo: Simulating Grover's Search
//!
//! Grover's algorithm searches an unsorted database of N items in O(sqrt(N)) steps.
//! For a brute-force key search over 2^n keys, this reduces to 2^(n/2) steps.
//!
//! We simulate this with a classical oracle-based search to demonstrate the concept.

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

/// Exercise 1: Classify the impact of Shor's algorithm on a given algorithm.
///
/// Return a `QuantumAttackResult` describing how Shor's algorithm affects the given
/// classical cryptographic algorithm and key size.
///
/// Rules:
/// - "RSA" with any key size: classical_bits = key_size / 10 (approximate), quantum_bits = 0,
///   is_broken = true, attack = "Shor's factoring"
/// - "DH" or "ECDH": classical_bits = key_size, quantum_bits = 0, is_broken = true,
///   attack = "Shor's discrete log"
/// - "ECDSA": classical_bits = key_size, quantum_bits = 0, is_broken = true,
///   attack = "Shor's discrete log"
/// - Anything else: is_broken = false, quantum_bits = classical_bits
pub fn classify_shor_impact(algorithm: &str, key_size: u32) -> QuantumAttackResult {
    todo!("Classify how Shor's algorithm affects the given algorithm")
}

/// Exercise 2: Calculate the effective quantum security level after Grover's attack.
///
/// Grover's algorithm provides a quadratic speedup, so the effective security is
/// halved. For example, AES-128 has 128-bit classical security but only 64-bit
/// quantum security.
///
/// Return the effective quantum security bits. If the input is 0, return 0.
pub fn grover_security_level(classical_bits: u32) -> u32 {
    todo!("Calculate effective security after Grover's algorithm")
}

/// Exercise 3: Determine if a symmetric cipher needs a key-size upgrade for quantum safety.
///
/// A cipher needs upgrading if its quantum security level (after Grover) is below
/// the `min_quantum_bits` threshold.
///
/// Return `true` if upgrading is needed, `false` if the cipher is already adequate.
pub fn needs_key_upgrade(classical_bits: u32, min_quantum_bits: u32) -> bool {
    todo!("Check if cipher needs key-size upgrade for quantum safety")
}

/// Exercise 4: Simulate a Grover's search oracle.
///
/// Given a target hash and a max search space (2^max_bits), find the preimage
/// by brute force (simulating what Grover's would do quadratically faster).
///
/// Return `Some(preimage_bytes)` if found, `None` if not found within the search space.
/// The preimage is a big-endian encoding of the integer that hashes to `target_hash`.
///
/// Hints:
/// - Iterate integers from 0 to 2^max_bits - 1
/// - Hash each integer (as big-endian bytes) with SHA-256
/// - Compare with target_hash
pub fn simulate_grover_search(target_hash: &[u8], max_bits: u32) -> Option<Vec<u8>> {
    todo!("Simulate Grover's oracle-based search")
}

/// Exercise 5: Estimate the number of logical qubits needed to break RSA.
///
/// Shor's algorithm to factor an n-bit RSA modulus requires approximately
/// 2n logical qubits (a common rough estimate).
///
/// Return the estimated qubit count.
pub fn estimate_rsa_qubits(key_size: u32) -> u32 {
    todo!("Estimate qubits needed to break RSA with Shor's algorithm")
}

/// Exercise 6: Generate a quantum risk report for a set of algorithms.
///
/// Given a list of (algorithm_name, key_size) tuples, return a list of
/// `QuantumAttackResult` entries classifying each one.
///
/// For RSA/DH/ECDH/ECDSA, use Shor's classification. For AES/ChaCha20,
/// use Grover's (halved bits, not broken). For SHA-256/SHA-3, report as
/// adequate (collision resistance unchanged by Grover for hash functions
/// at current sizes).
pub fn generate_risk_report(algorithms: &[(&str, u32)]) -> Vec<QuantumAttackResult> {
    todo!("Generate a quantum risk report for multiple algorithms")
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
        // AES-128 has 64-bit quantum security, below 128-bit threshold
        assert!(needs_key_upgrade(128, 128));
    }

    #[test]
    fn test_needs_upgrade_aes256() {
        // AES-256 has 128-bit quantum security, meets 128-bit threshold
        assert!(!needs_key_upgrade(256, 128));
    }

    #[test]
    fn test_simulate_grover_small() {
        // Search for a known preimage in a tiny space (4 bits = 16 values)
        let preimage: u8 = 7;
        let hash = sha256(&[preimage]);
        let result = simulate_grover_search(&hash, 4);
        assert_eq!(result, Some(vec![7]));
    }

    #[test]
    fn test_simulate_grover_not_found() {
        // Target hash not in search space
        let hash = sha256(&[255u8]);
        let result = simulate_grover_search(&hash, 3); // only 0..7
        assert_eq!(result, None);
    }

    #[test]
    fn test_estimate_rsa_qubits() {
        // RSA-2048 should need ~4096 qubits
        assert_eq!(estimate_rsa_qubits(2048), 4096);
    }

    #[test]
    fn test_risk_report() {
        let algorithms = vec![("RSA", 2048u32), ("AES", 256u32)];
        let report = generate_risk_report(&algorithms);
        assert_eq!(report.len(), 2);
        assert!(report[0].is_broken); // RSA is broken
        assert!(!report[1].is_broken); // AES is weakened but not broken
    }
}
