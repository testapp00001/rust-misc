//! # Exercise: Timeout Tradeoffs in Failure Detection
//!
//! ## Theory
//!
//! Real systems use timeouts as a practical workaround for FLP impossibility. However,
//! choosing the right timeout value involves a fundamental trade-off:
//!
//! - **Short timeouts:** Detect failures quickly but generate false positives (suspecting
//!   correct processes that are merely slow).
//! - **Long timeouts:** Fewer false positives but slow failure detection, leading to
//!   longer unavailability during actual failures.
//!
//! ## Proof / Intuition
//!
//! The trade-off can be formalized:
//!
//! Let T be the timeout value, and let D be the actual message delay distribution.
//!
//! - **False Positive Rate (FPR):** P(delay > T) = integral from T to infinity of f(d) dd,
//!   where f(d) is the delay distribution PDF. As T decreases, FPR increases.
//!
//! - **Detection Latency:** For a crashed process, the detection time is approximately T
//!   (we must wait T after the last heartbeat before suspecting). As T increases,
//!   detection latency increases.
//!
//! The optimal T minimizes a cost function that weights FPR and detection latency:
//!   Cost(T) = alpha * FPR(T) + beta * DetectionLatency(T)
//!
//! In practice, adaptive timeouts (like TCP's RTO) adjust T based on observed delays.
//!
//! ## Implementation Task
//!
//! Implement a timeout analyzer:
//!
//! - `TimeoutAnalyzer` that simulates different timeout values.
//! - `SimulationConfig` with delay distribution parameters.
//! - `SimulationResult` with false positive rate, true positive rate, and detection latency.
//! - `analyze_timeout(timeout, config) -> SimulationResult`.
//!
//! ## Verification
//!
//! - Verify shorter timeout has more false positives.
//! - Verify longer timeout has higher detection latency.
//! - Verify there exists an optimal timeout that minimizes total cost.

use rand::Rng;

/// Configuration for the simulation.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Number of processes to simulate.
    pub num_processes: u32,
    /// Number of time steps to simulate.
    pub num_steps: u32,
    /// Mean message delay (in time steps).
    pub mean_delay: f64,
    /// Standard deviation of message delay.
    pub std_dev: f64,
    /// Number of processes that actually crash.
    pub num_crashed: u32,
    /// Seed for reproducible randomness.
    pub seed: u64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            num_processes: 10,
            num_steps: 1000,
            mean_delay: 50.0,
            std_dev: 20.0,
            num_crashed: 2,
            seed: 42,
        }
    }
}

/// Result of simulating a particular timeout value.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// The timeout value used.
    pub timeout: u64,
    /// False positive rate (alive processes incorrectly suspected).
    pub false_positive_rate: f64,
    /// True positive rate (crashed processes correctly suspected).
    pub true_positive_rate: f64,
    /// Average detection latency for crashed processes (in time steps).
    pub detection_latency: f64,
    /// Number of total suspicion events.
    pub total_suspicion_events: u32,
    /// Number of false positive events.
    pub false_positive_events: u32,
}

impl SimulationResult {
    /// Compute a weighted cost combining FPR and detection latency.
    pub fn cost(&self, alpha: f64, beta: f64) -> f64 {
        alpha * self.false_positive_rate + beta * self.detection_latency
    }
}

/// Generate a delay from a normal distribution, clamped to be non-negative.
fn generate_delay(mean: f64, std_dev: f64, rng: &mut impl Rng) -> u64 {
    let normal = rng.gen_range(-3.0..=3.0); // Approximate normal with uniform in [-3, 3] std.
    let delay = mean + normal * std_dev;
    delay.max(0.0) as u64
}

/// Simulate a single timeout value and return the result.
pub fn analyze_timeout(timeout: u64, config: &SimulationConfig) -> SimulationResult {
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let mut rng = SmallRng::seed_from_u64(config.seed);

    // Track last heartbeat time for each process.
    let mut last_heartbeat: Vec<u64> = vec![0; config.num_processes as usize];
    // Track if each process is crashed.
    let crashed: Vec<bool> = (0..config.num_processes)
        .map(|i| i < config.num_crashed)
        .collect();

    let mut false_positives = 0u32;
    let mut true_positives = 0u32;
    let mut total_suspicions = 0u32;
    let mut detection_latencies: Vec<u64> = Vec::new();
    let mut detected_crashes: Vec<bool> = vec![false; config.num_processes as usize];

    for step_u32 in 0..config.num_steps {
        let step = step_u32 as u64;
        // Generate heartbeats for alive processes.
        for i in 0..config.num_processes as usize {
            if crashed[i] {
                continue;
            }
            let delay = generate_delay(config.mean_delay, config.std_dev, &mut rng);
            if delay <= timeout {
                last_heartbeat[i] = step;
            }
        }

        // Check for suspects.
        for i in 0..config.num_processes as usize {
            let time_since_heartbeat = step.saturating_sub(last_heartbeat[i]);
            if time_since_heartbeat > timeout {
                total_suspicions += 1;
                if crashed[i] {
                    true_positives += 1;
                    if !detected_crashes[i] {
                        detected_crashes[i] = true;
                        detection_latencies.push(time_since_heartbeat);
                    }
                } else {
                    false_positives += 1;
                }
            }
        }
    }

    let alive_count = config.num_processes - config.num_crashed;
    let false_positive_rate = if alive_count > 0 {
        false_positives as f64 / (alive_count as f64 * config.num_steps as f64)
    } else {
        0.0
    };

    let true_positive_rate = if config.num_crashed > 0 {
        true_positives as f64 / (config.num_crashed as f64 * config.num_steps as f64)
    } else {
        0.0
    };

    let detection_latency = if detection_latencies.is_empty() {
        0.0
    } else {
        detection_latencies.iter().sum::<u64>() as f64 / detection_latencies.len() as f64
    };

    SimulationResult {
        timeout,
        false_positive_rate,
        true_positive_rate,
        detection_latency,
        total_suspicion_events: total_suspicions,
        false_positive_events: false_positives,
    }
}

/// Analyze a range of timeout values and return results.
pub fn sweep_timeouts(
    min_timeout: u64,
    max_timeout: u64,
    step: u64,
    config: &SimulationConfig,
) -> Vec<SimulationResult> {
    (min_timeout..=max_timeout)
        .step_by(step as usize)
        .map(|t| analyze_timeout(t, config))
        .collect()
}

/// Find the optimal timeout that minimizes cost.
pub fn find_optimal_timeout(
    min_timeout: u64,
    max_timeout: u64,
    step: u64,
    config: &SimulationConfig,
    alpha: f64,
    beta: f64,
) -> (u64, f64) {
    let results = sweep_timeouts(min_timeout, max_timeout, step, config);
    let mut best_timeout = min_timeout;
    let mut best_cost = f64::MAX;

    for result in &results {
        let cost = result.cost(alpha, beta);
        if cost < best_cost {
            best_cost = cost;
            best_timeout = result.timeout;
        }
    }

    (best_timeout, best_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorter_timeout_has_more_false_positives() {
        let config = SimulationConfig::default();
        let short_result = analyze_timeout(10, &config);
        let long_result = analyze_timeout(200, &config);

        assert!(
            short_result.false_positive_rate >= long_result.false_positive_rate,
            "Shorter timeout (FPR={}) should have >= FPR than longer timeout (FPR={})",
            short_result.false_positive_rate,
            long_result.false_positive_rate
        );
    }

    #[test]
    fn longer_timeout_has_higher_detection_latency() {
        let config = SimulationConfig::default();
        let short_result = analyze_timeout(5, &config);
        let long_result = analyze_timeout(200, &config);

        assert!(
            long_result.detection_latency >= short_result.detection_latency,
            "Longer timeout (latency={}) should have >= detection latency than shorter (latency={})",
            long_result.detection_latency,
            short_result.detection_latency
        );
    }

    #[test]
    fn sweep_shows_tradeoff_pattern() {
        let config = SimulationConfig::default();
        let results = sweep_timeouts(5, 200, 10, &config);

        // FPR should generally decrease as timeout increases.
        let first_fpr = results.first().unwrap().false_positive_rate;
        let last_fpr = results.last().unwrap().false_positive_rate;
        assert!(
            last_fpr <= first_fpr,
            "FPR should decrease with larger timeout: first={}, last={}",
            first_fpr,
            last_fpr
        );
    }

    #[test]
    fn optimal_timeout_exists() {
        let config = SimulationConfig::default();
        let (optimal, cost) = find_optimal_timeout(5, 200, 5, &config, 1.0, 0.01);
        assert!(optimal >= 5, "Optimal timeout should be within range");
        assert!(optimal <= 200, "Optimal timeout should be within range");
        assert!(cost < f64::MAX, "Cost should be finite");
    }

    #[test]
    fn result_cost_scales_with_weights() {
        let result = SimulationResult {
            timeout: 50,
            false_positive_rate: 0.1,
            true_positive_rate: 0.9,
            detection_latency: 60.0,
            total_suspicion_events: 100,
            false_positive_events: 50,
        };

        let cost1 = result.cost(1.0, 0.0);
        let cost2 = result.cost(0.0, 1.0);
        let cost3 = result.cost(1.0, 1.0);

        assert!(
            (cost1 - 0.1).abs() < 1e-10,
            "Cost with alpha=1 should equal FPR"
        );
        assert!(
            (cost2 - 60.0).abs() < 1e-10,
            "Cost with beta=1 should equal detection latency"
        );
        assert!(
            (cost3 - 60.1).abs() < 1e-10,
            "Cost with both weights should be sum"
        );
    }

    #[test]
    fn zero_timeout_has_maximum_false_positives() {
        let config = SimulationConfig::default();
        let result = analyze_timeout(0, &config);
        assert!(
            result.false_positive_rate > 0.0,
            "Zero timeout should produce false positives"
        );
    }
}
