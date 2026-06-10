//! # Lesson 01: Differential Privacy -- Adding Calibrated Noise (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;

/// Sample from Laplace(0, scale) distribution.
///
/// Uses inverse CDF method: U ~ Uniform(-0.5, 0.5) -> -b * sign(U) * ln(1 - 2|U|).
pub fn laplace_sample(scale: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen_range(-0.5f64..0.5f64);
    -scale * u.signum() * (1.0 - 2.0 * u.abs()).ln()
}

/// Apply the Laplace mechanism to a count query.
///
/// Sensitivity of COUNT = 1 (one person changes count by 1).
/// Noise scale = sensitivity / epsilon.
pub fn laplace_count(true_count: i64, epsilon: f64) -> i64 {
    let noise = laplace_sample(1.0 / epsilon);
    (true_count as f64 + noise).round().max(0.0) as i64
}

/// Apply the Laplace mechanism to a sum query.
///
/// Sensitivity = max_contribution (one person can change sum by at most this).
pub fn laplace_sum(true_sum: f64, max_contribution: f64, epsilon: f64) -> f64 {
    let noise = laplace_sample(max_contribution / epsilon);
    (true_sum + noise).max(0.0)
}

/// Privacy budget tracker.
pub struct PrivacyBudget {
    pub total_epsilon: f64,
    pub remaining_epsilon: f64,
}

impl PrivacyBudget {
    pub fn new(total_epsilon: f64) -> Self {
        Self {
            total_epsilon,
            remaining_epsilon: total_epsilon,
        }
    }

    pub fn can_query(&self, cost: f64) -> bool {
        self.remaining_epsilon >= cost
    }

    pub fn spend(&mut self, cost: f64) -> bool {
        if self.remaining_epsilon >= cost {
            self.remaining_epsilon -= cost;
            true
        } else {
            false
        }
    }
}

/// Execute a differentially private count query with budget enforcement.
pub fn private_count_query(budget: &mut PrivacyBudget, true_count: i64, epsilon: f64) -> Option<i64> {
    if budget.can_query(epsilon) {
        budget.spend(epsilon);
        Some(laplace_count(true_count, epsilon))
    } else {
        None
    }
}

/// Compute a differentially private mean.
///
/// Splits epsilon equally between sum and count queries.
pub fn private_mean(values: &[f64], max_val: f64, epsilon: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let half_eps = epsilon / 2.0;
    let true_sum: f64 = values.iter().sum();
    let true_count = values.len() as i64;

    let noisy_sum = laplace_sum(true_sum, max_val, half_eps);
    let noisy_count = laplace_count(true_count, half_eps);

    if noisy_count <= 0 {
        0.0
    } else {
        (noisy_sum / noisy_count as f64).clamp(0.0, max_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_laplace_sample_centered() {
        let samples: Vec<f64> = (0..10000).map(|_| laplace_sample(1.0)).collect();
        let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
        assert!(mean.abs() < 0.2, "Mean should be near 0, got {}", mean);
    }

    #[test]
    fn test_laplace_count_close() {
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
        let noisy_mean = private_mean(&values, 100.0, 1.0);
        assert!(
            (noisy_mean - 50.5).abs() < 30.0,
            "Noisy mean {} should be near 50.5",
            noisy_mean
        );
    }
}
