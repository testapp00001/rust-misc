//! # Lesson 05: Hybrid Classical + PQC Encryption (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};
use ring::hkdf;
use ring::rand::{SecureRandom, SystemRandom};

/// Simulated classical key exchange result.
#[derive(Debug, Clone)]
pub struct ClassicalKeyExchange {
    pub shared_secret: Vec<u8>,
    pub algorithm: String,
}

/// Simulated PQC key exchange result.
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

/// Combine shared secrets using HKDF-SHA256.
///
/// Concatenates the two secrets and runs them through HKDF extract+expand.
pub fn combine_secrets_hkdf(
    classical_ss: &[u8],
    pqc_ss: &[u8],
) -> Vec<u8> {
    // Concatenate: IKM = classical || pqc
    let mut ikm = Vec::with_capacity(classical_ss.len() + pqc_ss.len());
    ikm.extend_from_slice(classical_ss);
    ikm.extend_from_slice(pqc_ss);

    // HKDF extract
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, &[]);
    let prk = salt.extract(&ikm);

    // HKDF expand
    let info = b"hybrid-key-derivation";
    let okm = prk.expand(info, hkdf::HKDF_SHA256).unwrap();
    let mut output = [0u8; 32];
    okm.fill(&mut output).unwrap();

    output.to_vec()
}

/// Combine shared secrets using XOR + SHA-256.
///
/// Pads shorter input with zeros, XORs, and hashes.
pub fn combine_secrets_xor_hash(
    classical_ss: &[u8],
    pqc_ss: &[u8],
) -> Vec<u8> {
    let max_len = classical_ss.len().max(pqc_ss.len());
    let mut xored = vec![0u8; max_len];

    for i in 0..max_len {
        let c = if i < classical_ss.len() { classical_ss[i] } else { 0 };
        let p = if i < pqc_ss.len() { pqc_ss[i] } else { 0 };
        xored[i] = c ^ p;
    }

    Sha256::digest(&xored).to_vec()
}

/// Perform a hybrid key exchange simulation.
pub fn hybrid_key_exchange(
    classical_alg: &str,
    pqc_alg: &str,
) -> (HybridSharedSecret, Vec<u8>) {
    let rng = SystemRandom::new();

    let mut classical_ss = vec![0u8; 32];
    let mut pqc_ss = vec![0u8; 32];
    let mut pqc_ct = vec![0u8; 64]; // simulated PQC ciphertext
    rng.fill(&mut classical_ss).unwrap();
    rng.fill(&mut pqc_ss).unwrap();
    rng.fill(&mut pqc_ct).unwrap();

    let combined = combine_secrets_hkdf(&classical_ss, &pqc_ss);

    let ss = HybridSharedSecret {
        bytes: combined,
        classical_alg: classical_alg.to_string(),
        pqc_alg: pqc_alg.to_string(),
    };

    (ss, pqc_ct)
}

/// Evaluate hybrid security level.
///
/// Since the final key depends on BOTH secrets (via KDF), the attacker must
/// break BOTH. So the effective quantum security is the MAX of the two,
/// because the attacker targets the weaker link, but the combined key
/// retains the security of the stronger component.
pub fn hybrid_security_level(
    classical_quantum_bits: u32,
    pqc_quantum_bits: u32,
) -> u32 {
    classical_quantum_bits.max(pqc_quantum_bits)
}

/// Validate a hybrid key exchange.
pub fn validate_hybrid_exchange(
    classical_ss: &[u8],
    pqc_ss: &[u8],
) -> bool {
    let combined = combine_secrets_hkdf(classical_ss, pqc_ss);

    // Check length
    if combined.len() != 32 {
        return false;
    }

    // Check not all zeros
    if combined.iter().all(|&b| b == 0) {
        return false;
    }

    // Check not identical to either input
    if combined == classical_ss || combined == pqc_ss {
        return false;
    }

    true
}

/// Compute hybrid bandwidth overhead.
pub fn hybrid_bandwidth_overhead(pqc_ct_size: usize) -> (usize, f64) {
    let classical_size = 33; // ECDH compressed point
    let total = classical_size + pqc_ct_size;
    let ratio = total as f64 / classical_size as f64;
    (total, ratio)
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
        assert_eq!(hybrid_security_level(0, 192), 192);
        assert_eq!(hybrid_security_level(128, 192), 192);
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
        let classical = vec![1u8; 32];
        let pqc = vec![2u8; 32];
        let hkdf_result = combine_secrets_hkdf(&classical, &pqc);
        let xor_result = combine_secrets_xor_hash(&classical, &pqc);
        assert_ne!(hkdf_result, xor_result);
    }
}
