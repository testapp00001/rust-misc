//! # Lesson 05: Range Proofs (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};

/// Decompose a value into binary representation (LSB first).
pub fn decompose_bits(value: u64, num_bits: usize) -> Vec<u8> {
    (0..num_bits).map(|i| ((value >> i) & 1) as u8).collect()
}

/// Reconstruct a value from its binary representation.
pub fn reconstruct_from_bits(bits: &[u8]) -> u64 {
    bits.iter()
        .enumerate()
        .map(|(i, &b)| (b as u64) * (1u64 << i))
        .sum()
}

/// Prove that a bit is 0 or 1 using a hash-based binding.
pub fn prove_bit_valid(bit: u8, salt: &[u8]) -> (u8, Vec<u8>) {
    let mut hasher = Sha256::new();
    hasher.update([bit]);
    hasher.update(salt);
    let proof_hash = hasher.finalize().to_vec();
    (bit, proof_hash)
}

/// Verify a bit validity proof.
pub fn verify_bit_valid(bit: u8, salt: &[u8], proof_hash: &[u8]) -> bool {
    if bit > 1 {
        return false;
    }
    let mut hasher = Sha256::new();
    hasher.update([bit]);
    hasher.update(salt);
    hasher.finalize().to_vec() == proof_hash
}

/// Range proof: prove value is in [0, 2^num_bits) using bit decomposition.
pub fn range_prove(value: u64, num_bits: usize) -> (Vec<(u8, Vec<u8>, Vec<u8>)>, u64) {
    let bits = decompose_bits(value, num_bits);
    let proofs: Vec<(u8, Vec<u8>, Vec<u8>)> = bits
        .iter()
        .map(|&bit| {
            let salt: Vec<u8> = rand::random::<[u8; 16]>().to_vec();
            let (b, hash) = prove_bit_valid(bit, &salt);
            (b, salt, hash)
        })
        .collect();
    let reconstructed = reconstruct_from_bits(&bits);
    (proofs, reconstructed)
}

/// Verify a range proof: all bits must be valid.
pub fn range_verify(
    bit_proofs: &[(u8, Vec<u8>, Vec<u8>)],
    expected_bits: usize,
) -> bool {
    if bit_proofs.len() != expected_bits {
        return false;
    }
    bit_proofs
        .iter()
        .all(|(bit, salt, hash)| verify_bit_valid(*bit, salt, hash))
}

/// Prove value is in [lo, hi] by proving (value - lo) is in [0, hi - lo].
pub fn prove_in_range(value: u64, lo: u64, hi: u64) -> bool {
    if value < lo || value > hi {
        return false;
    }
    let offset = value - lo;
    let range_size = hi - lo;
    let num_bits = (range_size.next_power_of_two().trailing_zeros() as usize) + 1;
    let (proofs, _) = range_prove(offset, num_bits);
    range_verify(&proofs, num_bits)
}

/// Simple range check without ZK (leaks the value).
pub fn naive_range_check(value: u64, lo: u64, hi: u64) -> bool {
    value >= lo && value <= hi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_bits_basic() {
        let bits = decompose_bits(5, 4);
        assert_eq!(bits, vec![1, 0, 1, 0]);
    }

    #[test]
    fn test_decompose_bits_zero() {
        let bits = decompose_bits(0, 4);
        assert_eq!(bits, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_reconstruct_round_trip() {
        let value = 42u64;
        let bits = decompose_bits(value, 8);
        assert_eq!(reconstruct_from_bits(&bits), value);
    }

    #[test]
    fn test_bit_valid_proof() {
        let salt = vec![1, 2, 3, 4];
        let (bit, proof_hash) = prove_bit_valid(1, &salt);
        assert_eq!(bit, 1);
        assert!(verify_bit_valid(1, &salt, &proof_hash));
    }

    #[test]
    fn test_bit_zero_valid() {
        let salt = vec![5, 6, 7, 8];
        let (bit, proof_hash) = prove_bit_valid(0, &salt);
        assert_eq!(bit, 0);
        assert!(verify_bit_valid(0, &salt, &proof_hash));
    }

    #[test]
    fn test_range_prove_and_verify() {
        let value = 42u64;
        let num_bits = 8;
        let (proofs, _reconstructed) = range_prove(value, num_bits);
        assert_eq!(proofs.len(), num_bits);
        assert!(range_verify(&proofs, num_bits));
    }

    #[test]
    fn test_range_proof_max_value() {
        let value = 255u64;
        let (proofs, _) = range_prove(value, 8);
        assert!(range_verify(&proofs, 8));
    }

    #[test]
    fn test_naive_range_check() {
        assert!(naive_range_check(50, 0, 100));
        assert!(!naive_range_check(150, 0, 100));
        assert!(naive_range_check(0, 0, 100));
        assert!(naive_range_check(100, 0, 100));
    }

    #[test]
    fn test_prove_in_range_basic() {
        assert!(prove_in_range(50, 0, 100));
        assert!(prove_in_range(0, 0, 100));
        assert!(prove_in_range(100, 0, 100));
    }
}
