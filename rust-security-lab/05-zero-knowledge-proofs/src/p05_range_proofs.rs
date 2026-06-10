//! # Lesson 05: Range Proofs
//!
//! ## What is a Range Proof?
//!
//! A range proof lets you prove that a committed value lies within a specific range
//! [a, b] without revealing the value itself.
//!
//! Example: "I have at least $1000 in my bank account" without revealing the exact balance.
//!
//! ## Why Range Proofs Matter
//!
//! Without range proofs, a malicious user could commit to a negative number in a cryptocurrency
//! transaction, effectively creating money from nothing. Range proofs prevent this.
//!
//! ## ATTACK: No Range Proof
//!
//! In a confidential transaction system, amounts are hidden behind commitments.
//! Without range proofs:
//! - Alice commits to sending -1000 coins (negative amount!)
//! - The homomorphic property means the system computes: old_balance + (-1000) = old_balance - 1000
//! - But the commitment hides this — the network thinks it's a valid transfer
//! - Alice effectively steals 1000 coins
//!
//! ## Range Proof Approaches
//!
//! 1. **Bit decomposition**: Prove each bit of the value is 0 or 1
//! 2. **Bulletproofs**: Efficient range proofs using inner product arguments
//! 3. **Simple approach**: For teaching, we use bit decomposition with commitments
//!
//! ## Pedersen Commitment Range Proof (Simplified)
//!
//! To prove v in [0, 2^n):
//! 1. Decompose v = b_0 + 2*b_1 + 4*b_2 + ... + 2^(n-1)*b_(n-1)
//! 2. Commit to each bit: C_i = commit(b_i, r_i)
//! 3. Prove each b_i is 0 or 1 (using a ZK proof of bit validity)
//! 4. Prove the weighted sum equals the original commitment
//!
//! For teaching, we implement a simplified version.

/// Exercise 1: Decompose a value into its binary representation.
///
/// Returns a vector of bits (LSB first) of length `num_bits`.
///
/// Hints:
/// - For each bit position i: bit = (value >> i) & 1
/// - Collect into a Vec<u8>
pub fn decompose_bits(value: u64, num_bits: usize) -> Vec<u8> {
    todo!("Decompose value into binary representation")
}

/// Exercise 2: Reconstruct a value from its binary representation.
///
/// value = sum(bit[i] * 2^i)
///
/// Hints:
/// - Iterate over bits with enumerate
/// - Accumulate: sum += bit * (1 << i)
pub fn reconstruct_from_bits(bits: &[u8]) -> u64 {
    todo!("Reconstruct value from bits")
}

/// Exercise 3: Prove that a bit is valid (0 or 1) using a simple hash-based proof.
///
/// For bit b, prove b*(1-b) = 0.
/// This is a simplified "proof" — in practice, use a proper ZK protocol.
///
/// Returns a proof struct containing:
/// - The bit value (for verification)
/// - A hash binding the prover to their claim
///
/// Hints:
/// - Compute hash = SHA256(bit_bytes || salt)
/// - Generate random salt
pub fn prove_bit_valid(bit: u8, salt: &[u8]) -> (u8, Vec<u8>) {
    todo!("Prove a bit is 0 or 1")
}

/// Exercise 4: Verify a bit validity proof.
///
/// Hints:
/// - Check bit is 0 or 1
/// - Recompute hash and compare
pub fn verify_bit_valid(bit: u8, salt: &[u8], proof_hash: &[u8]) -> bool {
    todo!("Verify bit validity proof")
}

/// Exercise 5: Range proof — prove that a value is in [0, 2^num_bits).
///
/// Decomposes the value into bits and proves each bit is valid.
/// Returns (bit_proofs, reconstructed_value) where each bit_proof is (bit, salt, hash).
///
/// Hints:
/// - Decompose value into num_bits bits
/// - For each bit, generate random salt and compute bit proof
/// - Return all proofs
pub fn range_prove(value: u64, num_bits: usize) -> (Vec<(u8, Vec<u8>, Vec<u8>)>, u64) {
    todo!("Prove value is in range [0, 2^num_bits)")
}

/// Exercise 6: Verify a range proof.
///
/// Check:
/// 1. All bit proofs are valid
/// 2. Reconstructed value matches expected (if provided)
/// 3. Number of bits matches expected
///
/// Hints:
/// - Verify each bit proof using verify_bit_valid
/// - Reconstruct value from bits
pub fn range_verify(
    bit_proofs: &[(u8, Vec<u8>, Vec<u8>)],
    expected_bits: usize,
) -> bool {
    todo!("Verify range proof")
}

/// Exercise 7: Check if a value is in a specific range [lo, hi].
///
/// This extends the basic range proof to arbitrary ranges.
/// value in [lo, hi] iff (value - lo) in [0, hi - lo].
///
/// Hints:
/// - Compute offset = value - lo (handle underflow)
/// - Compute range_size = hi - lo
/// - Determine num_bits needed: range_size.next_power_of_two().trailing_zeros() + 1
/// - Prove offset in [0, 2^num_bits)
pub fn prove_in_range(value: u64, lo: u64, hi: u64) -> bool {
    todo!("Prove value is in range [lo, hi]")
}

/// Exercise 8: Simple range check without ZK (for comparison — leaks the value).
///
/// Hints:
/// - Just check lo <= value && value <= hi
/// - This is what we do WITHOUT zero-knowledge
pub fn naive_range_check(value: u64, lo: u64, hi: u64) -> bool {
    todo!("Simple range check (no ZK)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_bits_basic() {
        let bits = decompose_bits(5, 4); // 5 = 101 in binary
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
        let value = 255u64; // max for 8 bits
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
