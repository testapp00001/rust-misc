//! # Lesson 09: Privacy-Preserving Computation -- Secure Aggregation
//!
//! ## What is Privacy-Preserving Computation?
//!
//! Privacy-preserving computation allows multiple parties to compute aggregate
//! statistics over their combined data without revealing individual inputs.
//!
//! ## Techniques
//!
//! 1. **Secure Aggregation**: Sum/average values without seeing individual contributions
//! 2. **Homomorphic Encryption**: Compute on encrypted data (Module 03 covers RSA/EC)
//! 3. **Secure Multi-Party Computation (MPC)**: Protocol for joint computation
//! 4. **Federated Learning**: Train ML models on distributed data
//!
//! ## This Lesson: Secure Sum Protocol
//!
//! The simplest privacy-preserving computation: N parties want to compute the sum
//! of their private values without revealing individual values.
//!
//! ### Shamir-style Additive Secret Sharing
//!
//! 1. Each party splits their value into N random shares that sum to their value
//! 2. Party i sends share j to party j (for all j != i)
//! 3. Each party sums all shares they received
//! 4. The sum of all parties' local sums = the true total sum
//!
//! No single party sees any other party's value.
//!
//! ## Attack: Collusion in Secret Sharing
//!
//! If N-1 parties collude, they can reconstruct the remaining party's value.
//! Security holds only if fewer than N parties collude.
//!
//! ## Application: Private Statistics
//!
//! - Average salary across companies (without revealing individual salaries)
//! - Disease prevalence across hospitals (without exposing patient data)
//! - Traffic statistics across ISPs (without revealing user browsing)

use rand::Rng;

/// Exercise 1: Split a value into N additive shares.
///
/// Generate N random numbers that sum to `value`.
///
/// Algorithm:
/// 1. Generate N-1 random values in range [-range, range]
/// 2. The Nth share = value - sum of first N-1 shares
///
/// Hints:
/// - Use `rand::rng().random_range(-range..=range)` for each random share
/// - The last share absorbs the difference
pub fn split_into_shares(value: i64, n: usize, range: i64) -> Vec<i64> {
    todo!("Split a value into N additive shares")
}

/// Exercise 2: Verify that shares sum to the original value.
///
/// Hints:
/// - Sum all shares and compare to expected value
pub fn verify_shares(shares: &[i64], expected: i64) -> bool {
    todo!("Verify shares sum to the expected value")
}

/// Exercise 3: Simulate a secure sum protocol.
///
/// Given private values from N parties:
/// 1. Each party splits their value into N shares
/// 2. Party i collects the i-th share from every party
/// 3. Each party sums their collected shares
/// 4. The global sum = sum of all parties' local sums
///
/// Returns the global sum without any party seeing another's value.
///
/// Hints:
/// - Create a 2D grid: shares[party][share_index]
/// - Each party's local sum = sum of column i across all parties
/// - Global sum = sum of all local sums
pub fn secure_sum(private_values: &[i64]) -> i64 {
    todo!("Simulate secure sum across N parties")
}

/// Exercise 4: Compute secure average.
///
/// Use secure_sum to compute the total, then divide by the number of parties.
///
/// Returns the average as f64.
pub fn secure_average(private_values: &[i64]) -> f64 {
    todo!("Compute secure average across N parties")
}

/// Exercise 5: Add Laplace noise to a secure sum for differential privacy.
///
/// Combine secure aggregation with differential privacy:
/// 1. Compute the secure sum
/// 2. Add Laplace noise calibrated to sensitivity = 1 (each party changes sum by 1 unit)
///
/// Hints:
/// - Use secure_sum to get the true sum
/// - Sample noise from Laplace(0, 1/epsilon)
/// - Add noise and round to nearest integer
pub fn secure_private_sum(private_values: &[i64], epsilon: f64) -> i64 {
    todo!("Combine secure sum with differential privacy")
}

/// Exercise 6: Detect if a share set has been tampered with.
///
/// Given the original value, number of parties, and the shares,
/// verify that:
/// 1. There are exactly N shares
/// 2. The shares sum to the original value
///
/// Returns Ok(()) or Err with description of the problem.
pub fn validate_share_integrity(
    shares: &[i64],
    original_value: i64,
    n: usize,
) -> Result<(), String> {
    todo!("Validate share integrity")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_shares_sum() {
        let shares = split_into_shares(100, 5, 200);
        assert_eq!(shares.len(), 5);
        let sum: i64 = shares.iter().sum();
        assert_eq!(sum, 100);
    }

    #[test]
    fn test_split_into_shares_single() {
        let shares = split_into_shares(42, 1, 100);
        assert_eq!(shares, vec![42]);
    }

    #[test]
    fn test_verify_shares_correct() {
        let shares = split_into_shares(100, 5, 200);
        assert!(verify_shares(&shares, 100));
    }

    #[test]
    fn test_verify_shares_incorrect() {
        let shares = vec![10, 20, 30];
        assert!(!verify_shares(&shares, 100));
    }

    #[test]
    fn test_secure_sum_basic() {
        let values = vec![10, 20, 30, 40];
        let result = secure_sum(&values);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_secure_sum_negative() {
        let values = vec![50, -20, 30];
        let result = secure_sum(&values);
        assert_eq!(result, 60);
    }

    #[test]
    fn test_secure_sum_single_party() {
        let values = vec![42];
        let result = secure_sum(&values);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_secure_average() {
        let values = vec![10, 20, 30, 40];
        let avg = secure_average(&values);
        assert!((avg - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_secure_private_sum_close() {
        let values = vec![100, 200, 300];
        let true_sum = 600;
        let result = secure_private_sum(&values, 1.0);
        // Should be close to true_sum (within noise)
        assert!(
            (result - true_sum).abs() < 50,
            "Private sum {} should be near {}",
            result,
            true_sum
        );
    }

    #[test]
    fn test_validate_share_integrity_ok() {
        let shares = split_into_shares(100, 5, 200);
        assert!(validate_share_integrity(&shares, 100, 5).is_ok());
    }

    #[test]
    fn test_validate_share_integrity_wrong_count() {
        let shares = vec![50, 50];
        let result = validate_share_integrity(&shares, 100, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_share_integrity_wrong_sum() {
        let shares = vec![10, 20, 30, 40, 5];
        let result = validate_share_integrity(&shares, 100, 5);
        assert!(result.is_err());
    }
}
