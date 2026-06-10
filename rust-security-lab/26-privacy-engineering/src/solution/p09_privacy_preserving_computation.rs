//! # Lesson 09: Privacy-Preserving Computation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;

/// Split a value into N additive shares.
///
/// Generates N-1 random shares and the Nth share absorbs the difference.
pub fn split_into_shares(value: i64, n: usize, range: i64) -> Vec<i64> {
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![value];
    }

    let mut rng = rand::thread_rng();
    let mut shares = Vec::with_capacity(n);
    let mut sum = 0i64;

    for _ in 0..(n - 1) {
        let share: i64 = rng.gen_range(-range..=range);
        shares.push(share);
        sum += share;
    }

    // Last share makes the total equal to value
    shares.push(value - sum);
    shares
}

/// Verify that shares sum to the expected value.
pub fn verify_shares(shares: &[i64], expected: i64) -> bool {
    shares.iter().sum::<i64>() == expected
}

/// Simulate a secure sum protocol across N parties.
///
/// Each party splits their value into N shares and distributes them.
/// Each party sums the shares they receive. The global sum equals
/// the sum of all parties' local sums.
pub fn secure_sum(private_values: &[i64]) -> i64 {
    let n = private_values.len();
    if n == 0 {
        return 0;
    }

    // Each party generates their shares
    let all_shares: Vec<Vec<i64>> = private_values
        .iter()
        .map(|&val| split_into_shares(val, n, 1000))
        .collect();

    // Each party i collects the i-th share from every party and sums them
    let mut global_sum = 0i64;
    for i in 0..n {
        let local_sum: i64 = all_shares.iter().map(|shares| shares[i]).sum();
        global_sum += local_sum;
    }

    global_sum
}

/// Compute secure average across N parties.
pub fn secure_average(private_values: &[i64]) -> f64 {
    if private_values.is_empty() {
        return 0.0;
    }
    let total = secure_sum(private_values);
    total as f64 / private_values.len() as f64
}

/// Combine secure sum with differential privacy.
///
/// Adds Laplace noise to the secure sum for privacy protection.
pub fn secure_private_sum(private_values: &[i64], epsilon: f64) -> i64 {
    let true_sum = secure_sum(private_values);
    // Laplace noise with sensitivity = 1, scale = 1/epsilon
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen_range(-0.5f64..0.5f64);
    let noise = -(1.0 / epsilon) * u.signum() * (1.0 - 2.0 * u.abs()).ln();
    (true_sum as f64 + noise).round() as i64
}

/// Validate share integrity.
pub fn validate_share_integrity(
    shares: &[i64],
    original_value: i64,
    n: usize,
) -> Result<(), String> {
    if shares.len() != n {
        return Err(format!(
            "Expected {} shares, got {}",
            n,
            shares.len()
        ));
    }
    let sum: i64 = shares.iter().sum();
    if sum != original_value {
        return Err(format!(
            "Shares sum to {}, expected {}",
            sum, original_value
        ));
    }
    Ok(())
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
