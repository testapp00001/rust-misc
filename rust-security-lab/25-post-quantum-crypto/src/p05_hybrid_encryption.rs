//! # Lesson 05: Hybrid Classical + PQC Encryption
//!
//! ## What is Hybrid Encryption?
//!
//! Hybrid encryption combines a classical key exchange (e.g., ECDH) with a
//! post-quantum KEM (e.g., ML-KEM). The shared secrets from both are combined
//! to derive the final encryption key.
//!
//! ```
//! Classical only:    ECDH → shared_secret → AES key
//! PQC only:          ML-KEM → shared_secret → AES key
//! Hybrid:            ECDH + ML-KEM → combined_secret → AES key
//! ```
//!
//! ## Why Hybrid?
//!
//! 1. **Defense in depth**: If PQC is broken, classical still protects. If
//!    quantum breaks classical, PQC still protects.
//! 2. **Gradual migration**: Deploy PQC without removing classical crypto.
//! 3. **Compliance**: Some standards require both during transition periods.
//!
//! ## Key Combination Methods
//!
//! - **Concatenation**: key = KDF(classical_ss || pqc_ss)
//! - **XOR + Hash**: key = KDF(classical_ss XOR pqc_ss)
//! - **Dual PRF**: key = PRF(classical_ss, pqc_ss)
//!
//! The safest approach is concatenation + KDF, as it preserves entropy from both.
//!
//! ## Attack Demo: Single-Point-of-Failure
//!
//! If you use only classical or only PQC, a single algorithm break compromises
//! everything. Hybrid ensures both must be broken simultaneously.

use sha2::{Digest, Sha256};
use ring::hkdf;
use ring::rand::{SecureRandom, SystemRandom};

/// Simulated classical key exchange result (e.g., ECDH).
#[derive(Debug, Clone)]
pub struct ClassicalKeyExchange {
    pub shared_secret: Vec<u8>,
    pub algorithm: String,
}

/// Simulated PQC key exchange result (e.g., ML-KEM).
#[derive(Debug, Clone)]
pub struct PqcKeyExchange {
    pub shared_secret: Vec<u8>,
    pub algorithm: String,
    pub ciphertext: Vec<u8>,
}

/// Combined hybrid shared secret.
#[derive(Debug, Clone, PartialEq)]
pub struct HybridSharedSecret {
    pub bytes: Vec<u8>,
    pub classical_alg: String,
    pub pqc_alg: String,
}

/// Exercise 1: Combine two shared secrets using concatenation + HKDF.
///
/// Given a classical shared secret and a PQC shared secret:
/// 1. Concatenate them: combined = classical || pqc
/// 2. Extract and expand using HKDF-SHA256
/// 3. Output 32 bytes
///
/// Hints:
/// - Use `ring::hkdf` for HKDF operations
/// - Salt should be empty, info should be b"hybrid-key-derivation"
/// - HKDF has two phases: extract (PRK) and expand (OKM)
pub fn combine_secrets_hkdf(
    classical_ss: &[u8],
    pqc_ss: &[u8],
) -> Vec<u8> {
    todo!("Combine shared secrets using HKDF")
}

/// Exercise 2: Combine two shared secrets using XOR + Hash.
///
/// 1. Pad the shorter secret with zeros to match lengths
/// 2. XOR them byte-by-byte
/// 3. SHA-256 hash the result
///
/// This is simpler than HKDF but slightly less flexible.
pub fn combine_secrets_xor_hash(
    classical_ss: &[u8],
    pqc_ss: &[u8],
) -> Vec<u8> {
    todo!("Combine shared secrets using XOR + SHA-256")
}

/// Exercise 3: Perform a hybrid key exchange simulation.
///
/// Simulates both a classical and PQC key exchange, then combines them.
/// Returns the combined shared secret and metadata.
///
/// For the simulation:
/// - Generate 32 random bytes for classical shared secret
/// - Generate 32 random bytes for PQC shared secret
/// - Combine using HKDF
pub fn hybrid_key_exchange(
    classical_alg: &str,
    pqc_alg: &str,
) -> (HybridSharedSecret, Vec<u8>) {
    todo!("Perform hybrid key exchange")
}

/// Exercise 4: Evaluate the security level of a hybrid scheme.
///
/// The hybrid security level is the MINIMUM of:
/// - Classical security level (quantum-attacked)
/// - PQC security level
///
/// For example:
/// - ECDH-P256 (128-bit classical, 0-bit quantum) + Kyber-768 (192-bit PQC)
///   → Classical-only: min(0, 192) = 0 (broken by quantum)
///   → Hybrid: the PQC part still provides 192-bit security
///   → Effective: 192-bit (PQC dominates)
///
/// Given the quantum security of classical and PQC parts, return the effective
/// hybrid quantum security level (which is the max of the two, since if EITHER
/// holds, the combined secret is safe).
pub fn hybrid_security_level(
    classical_quantum_bits: u32,
    pqc_quantum_bits: u32,
) -> u32 {
    todo!("Evaluate hybrid scheme security level")
}

/// Exercise 5: Validate that a hybrid key exchange is correct.
///
/// Given both the classical and PQC shared secrets, verify that combining them
/// produces a valid 32-byte key.
///
/// A valid key is one where:
/// 1. combine_secrets_hkdf succeeds (returns 32 bytes)
/// 2. The output is not all zeros
/// 3. The output is not identical to either input secret
pub fn validate_hybrid_exchange(
    classical_ss: &[u8],
    pqc_ss: &[u8],
) -> bool {
    todo!("Validate hybrid key exchange correctness")
}

/// Exercise 6: Compute the total bandwidth overhead of hybrid vs classical.
///
/// Hybrid adds PQC ciphertext overhead to the classical exchange.
/// Classical ECDH: 33 bytes (compressed point)
/// ML-KEM-768 ciphertext: 1088 bytes
/// Hybrid: 33 + 1088 = 1121 bytes
///
/// Given the PQC ciphertext size, return (hybrid_total, overhead_ratio vs classical).
pub fn hybrid_bandwidth_overhead(pqc_ct_size: usize) -> (usize, f64) {
    todo!("Compute hybrid bandwidth overhead")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_hkdf_length() {
        let classical = vec![1u8; 32];
        let pqc = vec![2u8; 32];
        let combined = combine_secrets_hkdf(&classical, &pqc);
        assert_eq!(combined.len(), 32);
    }

    #[test]
    fn test_combine_hkdf_deterministic() {
        let classical = vec![1u8; 32];
        let pqc = vec![2u8; 32];
        let c1 = combine_secrets_hkdf(&classical, &pqc);
        let c2 = combine_secrets_hkdf(&classical, &pqc);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_combine_hkdf_different_inputs() {
        let c1 = combine_secrets_hkdf(&[1u8; 32], &[2u8; 32]);
        let c2 = combine_secrets_hkdf(&[1u8; 32], &[3u8; 32]);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_combine_xor_hash_length() {
        let combined = combine_secrets_xor_hash(&[1u8; 32], &[2u8; 32]);
        assert_eq!(combined.len(), 32);
    }

    #[test]
    fn test_combine_xor_hash_deterministic() {
        let c1 = combine_secrets_xor_hash(&[1u8; 32], &[2u8; 32]);
        let c2 = combine_secrets_xor_hash(&[1u8; 32], &[2u8; 32]);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_hybrid_key_exchange() {
        let (ss, ct) = hybrid_key_exchange("ECDH-P256", "ML-KEM-768");
        assert_eq!(ss.bytes.len(), 32);
        assert_eq!(ss.classical_alg, "ECDH-P256");
        assert_eq!(ss.pqc_alg, "ML-KEM-768");
    }

    #[test]
    fn test_hybrid_security_level() {
        // ECDH broken by quantum (0 bits) + Kyber (192 bits) → 192 bits
        assert_eq!(hybrid_security_level(0, 192), 192);
        // Both contribute → max
        assert_eq!(hybrid_security_level(128, 192), 192);
        // Both zero
        assert_eq!(hybrid_security_level(0, 0), 0);
    }

    #[test]
    fn test_validate_hybrid_exchange() {
        let classical = vec![0x42u8; 32];
        let pqc = vec![0x37u8; 32];
        assert!(validate_hybrid_exchange(&classical, &pqc));
    }

    #[test]
    fn test_hybrid_bandwidth_overhead() {
        let (total, ratio) = hybrid_bandwidth_overhead(1088);
        assert_eq!(total, 33 + 1088);
        assert!((ratio - (1121.0 / 33.0)).abs() < 0.01);
    }

    #[test]
    fn test_combine_methods_differ() {
        // HKDF and XOR+Hash should produce different results
        let classical = vec![1u8; 32];
        let pqc = vec![2u8; 32];
        let hkdf_result = combine_secrets_hkdf(&classical, &pqc);
        let xor_result = combine_secrets_xor_hash(&classical, &pqc);
        assert_ne!(hkdf_result, xor_result);
    }
}
