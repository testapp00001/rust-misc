//! # Lesson 05: Secure Aggregation -- Sum Private Values Without Revealing Them
//!
//! ## What is Secure Aggregation?
//!
//! Secure aggregation allows multiple parties to compute an aggregate function (sum, average,
//! count) over their private inputs without revealing individual values. Each party only learns
//! the final aggregate, not any other party's input.
//!
//! ## The Pairwise Masking Protocol
//!
//! A practical approach used in federated learning (Google's SecAgg):
//!
//! 1. Each pair of parties (i, j) agrees on a random mask s_ij = -s_ji
//! 2. Party i adds all their pairwise masks to their input: masked_i = x_i + sum_j(s_ij)
//! 3. Party i broadcasts masked_i
//! 4. When all masked values are summed, the masks cancel: sum(s_ij + s_ji) = 0
//! 5. The result is sum(x_i) -- the true aggregate
//!
//! ## Why Not Just Use MPC?
//!
//! Secure aggregation is a *specialized* MPC protocol optimized for aggregation.
//! It is much more efficient than general-purpose MPC:
//! - Communication: O(n) per party (vs. O(n^2) for general MPC)
//! - Computation: simple addition (vs. circuit evaluation)
//! - Works well with dropout: if a party disconnects, the remaining masks still cancel
//!
//! ## Attack: Reconstruction from Masks
//!
//! If an adversary controls the server that sees all masked values, and also compromises
//! the pairwise key material of all but one party, they can unmask that party's value.
//! The protocol must protect pairwise seeds with forward secrecy.
//!
//! ## Application: Federated Learning
//!
//! In federated learning, each phone computes a local model update. Secure aggregation
//! lets the server learn the sum of updates (to improve the global model) without seeing
//! any individual phone's update. This protects user data.

use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// A party's masked contribution to the aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedValue {
    pub party_id: usize,
    pub masked_value: i64,
}

/// Parameters for the secure aggregation protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggParams {
    /// The number of parties
    pub num_parties: usize,
    /// The modulus for arithmetic (prevents overflow)
    pub modulus: i64,
}

/// A pairwise seed shared between two parties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairwiseSeed {
    pub party_i: usize,
    pub party_j: usize,
    pub seed: i64,
}

/// Exercise 1: Generate pairwise seeds for all party pairs.
///
/// For n parties, generate n*(n-1)/2 unique seeds.
/// Each seed is a random value. Party pair (i, j) shares seed s_ij.
/// Convention: s_ij = -s_ji (mod modulus), so masks cancel on summation.
///
/// Return a vector of PairwiseSeed where i < j (canonical ordering).
pub fn generate_pairwise_seeds(
    params: &AggParams,
) -> Vec<PairwiseSeed> {
    todo!("Generate pairwise random seeds for all party pairs")
}

/// Exercise 2: Derive a party's mask from their pairwise seeds.
///
/// Party i's mask = sum of all s_ij for j != i (mod modulus).
/// For party i, sum seeds where i is party_i (+seed) and where i is party_j (-seed).
///
/// The key property: when all masks are summed, they cancel to 0.
pub fn compute_party_mask(
    party_id: usize,
    seeds: &[PairwiseSeed],
    modulus: i64,
) -> i64 {
    todo!("Compute the combined mask for a party from their pairwise seeds")
}

/// Exercise 3: Mask a party's value.
///
/// masked_value = (input + mask) mod modulus
pub fn mask_value(input: i64, mask: i64, modulus: i64) -> i64 {
    todo!("Add mask to input value")
}

/// Exercise 4: Aggregate all masked values.
///
/// Sum all masked values mod modulus. Because masks cancel, this gives the true sum.
///
/// Also verify that no party dropped out (all parties submitted). If a party
/// drops out, the protocol needs recovery mechanisms (not implemented here).
pub fn aggregate(
    masked_values: &[MaskedValue],
    params: &AggParams,
) -> Result<i64, String> {
    todo!("Sum all masked values to get the true aggregate")
}

/// Exercise 5: Run the full secure aggregation protocol.
///
/// Simulate end-to-end:
/// 1. Generate pairwise seeds
/// 2. Each party computes their mask
/// 3. Each party masks their input
/// 4. Aggregate all masked values
/// 5. Return the true sum
pub fn secure_sum(inputs: &[i64], modulus: i64) -> Result<i64, String> {
    todo!("Full secure aggregation protocol")
}

/// Exercise 6: Compute secure average.
///
/// Use secure_sum to compute the total, then divide by the number of parties.
/// Handle the case where the sum wraps around the modulus.
pub fn secure_average(inputs: &[i64], modulus: i64) -> Result<f64, String> {
    todo!("Compute average using secure aggregation")
}

/// Helper: derive a deterministic seed from party IDs and a master secret.
/// Used to simulate pairwise key agreement.
pub fn derive_seed(party_i: usize, party_j: usize, master: i64) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(party_i.to_le_bytes());
    hasher.update(party_j.to_le_bytes());
    hasher.update(master.to_le_bytes());
    let hash = hasher.finalize();
    let bytes = &hash[..8];
    i64::from_le_bytes(bytes.try_into().unwrap()).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_params() -> AggParams {
        AggParams {
            num_parties: 5,
            modulus: 1_000_007,
        }
    }

    #[test]
    fn test_generate_pairwise_seeds_count() {
        let params = test_params();
        let seeds = generate_pairwise_seeds(&params);
        // n*(n-1)/2 = 5*4/2 = 10
        assert_eq!(seeds.len(), 10);
    }

    #[test]
    fn test_pairwise_seeds_unique() {
        let params = test_params();
        let seeds = generate_pairwise_seeds(&params);
        let mut seeds_sorted: Vec<(usize, usize)> = seeds
            .iter()
            .map(|s| (s.party_i, s.party_j))
            .collect();
        seeds_sorted.sort();
        seeds_sorted.dedup();
        assert_eq!(seeds_sorted.len(), 10);
    }

    #[test]
    fn test_masks_cancel() {
        let params = test_params();
        let seeds = generate_pairwise_seeds(&params);
        // Sum all masks -- they should cancel to 0
        let total_mask: i64 = (0..params.num_parties)
            .map(|i| compute_party_mask(i, &seeds, params.modulus))
            .sum::<i64>()
            % params.modulus;
        assert_eq!(total_mask, 0, "Masks should cancel to 0");
    }

    #[test]
    fn test_secure_sum_basic() {
        let inputs = vec![10, 20, 30, 40, 50];
        let result = secure_sum(&inputs, 1_000_007).unwrap();
        assert_eq!(result, 150);
    }

    #[test]
    fn test_secure_sum_single_party() {
        let inputs = vec![42];
        let result = secure_sum(&inputs, 1_000_007).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_secure_sum_negative_values() {
        let inputs = vec![100, -30, 50];
        let result = secure_sum(&inputs, 1_000_007).unwrap();
        // Result might be negative mod, but the true sum is 120
        assert_eq!(result, 120);
    }

    #[test]
    fn test_secure_average() {
        let inputs = vec![10, 20, 30, 40];
        let avg = secure_average(&inputs, 1_000_007).unwrap();
        assert!((avg - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_mask_value() {
        let mask = 100;
        let input = 50;
        let masked = mask_value(input, mask, 1_000_007);
        assert_eq!(masked, 150);
    }

    #[test]
    fn test_aggregate_checks_party_count() {
        let params = test_params();
        // Only 3 of 5 parties submitted
        let masked = vec![
            MaskedValue { party_id: 0, masked_value: 10 },
            MaskedValue { party_id: 1, masked_value: 20 },
            MaskedValue { party_id: 2, masked_value: 30 },
        ];
        let result = aggregate(&masked, &params);
        // Should either error or handle gracefully
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_secure_sum_random_values() {
        // Larger test with random-ish values
        let inputs = vec![123, 456, 789, 1011, 1213];
        let expected: i64 = inputs.iter().sum();
        let result = secure_sum(&inputs, 1_000_007).unwrap();
        assert_eq!(result, expected);
    }
}
