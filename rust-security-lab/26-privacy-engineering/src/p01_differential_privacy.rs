//! # Lesson 01: Differential Privacy -- Adding Calibrated Noise
//!
//! ## What is Differential Privacy?
//!
//! Differential privacy (DP) is a mathematical framework that guarantees that the output
//! of a computation does not reveal whether any individual's data was included in the input.
//! The core idea: add carefully calibrated random noise so that the result is statistically
//! similar regardless of any single person's data.
//!
//! ## The Laplace Mechanism
//!
//! For a numeric query f(D) with sensitivity S (the max change one record can cause),
//! the Laplace mechanism adds noise drawn from Laplace(0, S/epsilon).
//!
//! ```
//! noisy_answer = true_answer + Laplace(0, sensitivity / epsilon)
//! ```
//!
//! ## Privacy Budget (Epsilon)
//!
//! - **epsilon** controls the privacy/accuracy tradeoff
//! - Small epsilon (e.g., 0.1) = strong privacy, noisy results
//! - Large epsilon (e.g., 10.0) = weak privacy, accurate results
//! - Each query "spends" epsilon from a total budget
//! - When budget is exhausted, no more queries allowed
//!
//! ## Attack: Identifying Individuals in Aggregate Data
//!
//! Without DP, an adversary can run queries like:
//!   count(patients with disease X AND zip=02138 AND age=45 AND gender=M)
//! If the count changes by 1 when they add/remove their target's record, they know
//! the target has the disease. Differential privacy prevents this.
//!
//! ## Sensitivity
//!
//! Global sensitivity = max over all neighboring datasets D, D' of |f(D) - f(D')|.
//! For COUNT: sensitivity = 1 (adding one record changes count by 1).
//! For SUM of bounded values [0, max_val]: sensitivity = max_val.
//! For MEAN: need to use more advanced mechanisms.

use rand::Rng;

/// Exercise 1: Sample from a Laplace distribution.
///
/// Laplace(0, b) has PDF: (1/2b) * exp(-|x|/b)
///
/// To sample: generate U ~ Uniform(-0.5, 0.5), then return -b * sign(U) * ln(1 - 2|U|).
///
/// Hints:
/// - Use `rand::thread_rng().gen_range(-0.5f64..0.5f64)` for uniform sample
/// - `f64::signum()` gives the sign, `f64::abs()` gives absolute value
/// - `f64::ln()` is the natural logarithm
pub fn laplace_sample(scale: f64) -> f64 {
    todo!("Sample from Laplace(0, scale)")
}

/// Exercise 2: Apply the Laplace mechanism to a count query.
///
/// Given a true count and privacy parameters, return a noisy count.
/// The sensitivity of a count query is 1.0 (adding one person changes count by 1).
///
/// Hints:
/// - Compute noise_scale = sensitivity / epsilon = 1.0 / epsilon
/// - Add Laplace noise to the true count
/// - Round to nearest integer and clamp to >= 0
pub fn laplace_count(true_count: i64, epsilon: f64) -> i64 {
    todo!("Add Laplace noise to a count query")
}

/// Exercise 3: Apply the Laplace mechanism to a sum query.
///
/// Given a true sum, the maximum value any individual can contribute,
/// and epsilon, return a noisy sum.
///
/// Hints:
/// - Sensitivity = max_contribution (one person can change sum by at most this)
/// - noise_scale = max_contribution / epsilon
/// - Add Laplace noise, round to nearest integer, clamp to >= 0
pub fn laplace_sum(true_sum: f64, max_contribution: f64, epsilon: f64) -> f64 {
    todo!("Add Laplace noise to a sum query")
}

/// Exercise 4: Check if a privacy budget allows a query.
///
/// A PrivacyBudget tracks remaining epsilon. Each query costs some epsilon.
/// If the remaining budget is less than the requested cost, the query is denied.
///
/// Implement the `can_query` and `spend` methods.
pub struct PrivacyBudget {
    /// Total epsilon allocated
    pub total_epsilon: f64,
    /// Epsilon remaining
    pub remaining_epsilon: f64,
}

impl PrivacyBudget {
    /// Create a new privacy budget with the given total epsilon.
    pub fn new(total_epsilon: f64) -> Self {
        todo!("Create a new PrivacyBudget")
    }

    /// Check if a query costing `cost` epsilon can be performed.
    pub fn can_query(&self, cost: f64) -> bool {
        todo!("Check if remaining budget >= cost")
    }

    /// Spend `cost` epsilon from the budget.
    /// Returns true if the spend succeeded, false if insufficient budget.
    pub fn spend(&mut self, cost: f64) -> bool {
        todo!("Spend epsilon from the budget")
    }
}

/// Exercise 5: Execute a query with budget enforcement.
///
/// If the budget allows, apply Laplace noise to the count and deduct epsilon.
/// If the budget is insufficient, return None.
///
/// Hints:
/// - Check `budget.can_query(epsilon)`
/// - If yes, `budget.spend(epsilon)` and return `Some(laplace_count(...))`
/// - If no, return `None`
pub fn private_count_query(budget: &mut PrivacyBudget, true_count: i64, epsilon: f64) -> Option<i64> {
    todo!("Execute a differentially private count query with budget check")
}

/// Exercise 6: Compute the mean with differential privacy.
///
/// Given a list of values, compute the noisy mean by:
/// 1. Computing noisy sum (sensitivity = max_val)
/// 2. Computing noisy count (sensitivity = 1)
/// 3. Dividing noisy sum by noisy count
/// 4. Clamp the result to [0, max_val]
///
/// Hints:
/// - Split epsilon equally: half for sum, half for count
/// - Use `laplace_sum` and `laplace_count`
/// - Handle the case where noisy count <= 0 (return 0.0)
pub fn private_mean(values: &[f64], max_val: f64, epsilon: f64) -> f64 {
    todo!("Compute a differentially private mean")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_laplace_sample_centered() {
        // With large scale, samples should center around 0
        let samples: Vec<f64> = (0..10000).map(|_| laplace_sample(1.0)).collect();
        let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
        assert!(mean.abs() < 0.2, "Mean should be near 0, got {}", mean);
    }

    #[test]
    fn test_laplace_count_close() {
        // With moderate epsilon, noisy count should be reasonably close
        let true_val = 100;
        let noisy = laplace_count(true_val, 1.0);
        assert!(
            (noisy - true_val).abs() < 50,
            "Noisy count {} should be within ~50 of {}",
            noisy,
            true_val
        );
    }

    #[test]
    fn test_laplace_count_non_negative() {
        // Even for small true counts, result should be >= 0
        for _ in 0..100 {
            let noisy = laplace_count(1, 0.5);
            assert!(noisy >= 0, "Noisy count should be >= 0, got {}", noisy);
        }
    }

    #[test]
    fn test_laplace_sum_range() {
        let true_sum = 500.0;
        let max_contrib = 100.0;
        let epsilon = 1.0;
        let noisy = laplace_sum(true_sum, max_contrib, epsilon);
        // Should be within a reasonable range
        assert!(
            (noisy - true_sum).abs() < 500.0,
            "Noisy sum {} should be near {}",
            noisy,
            true_sum
        );
    }

    #[test]
    fn test_privacy_budget_create() {
        let budget = PrivacyBudget::new(1.0);
        assert_eq!(budget.total_epsilon, 1.0);
        assert_eq!(budget.remaining_epsilon, 1.0);
    }

    #[test]
    fn test_privacy_budget_spend() {
        let mut budget = PrivacyBudget::new(1.0);
        assert!(budget.spend(0.3));
        assert!((budget.remaining_epsilon - 0.7).abs() < 1e-10);
        assert!(budget.spend(0.3));
        assert!(!budget.spend(0.5), "Should fail -- only 0.4 remaining");
    }

    #[test]
    fn test_privacy_budget_exhausted() {
        let mut budget = PrivacyBudget::new(0.5);
        assert!(budget.spend(0.5));
        assert!(!budget.can_query(0.01));
        assert!(!budget.spend(0.01));
    }

    #[test]
    fn test_private_count_query_with_budget() {
        let mut budget = PrivacyBudget::new(1.0);
        let result = private_count_query(&mut budget, 100, 0.5);
        assert!(result.is_some());
        let noisy = result.unwrap();
        assert!((noisy - 100).abs() < 50);
        assert!((budget.remaining_epsilon - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_private_count_query_budget_exhausted() {
        let mut budget = PrivacyBudget::new(0.1);
        let result = private_count_query(&mut budget, 100, 0.5);
        assert!(result.is_none(), "Should fail with insufficient budget");
        assert!((budget.remaining_epsilon - 0.1).abs() < 1e-10, "Budget should be unchanged");
    }

    #[test]
    fn test_private_mean_reasonable() {
        let values: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        // True mean is 50.5
        let noisy_mean = private_mean(&values, 100.0, 1.0);
        assert!(
            (noisy_mean - 50.5).abs() < 30.0,
            "Noisy mean {} should be near 50.5",
            noisy_mean
        );
    }
}
