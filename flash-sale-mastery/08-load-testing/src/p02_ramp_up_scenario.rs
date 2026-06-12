//! # Exercise 02: Ramp-Up Scenario
//!
//! ## Learning Objective
//! Implement a gradual ramp-up load test that increases concurrency from zero
//! to a target level over a configurable duration.
//!
//! ## Flash Sale Context
//! Real flash sales do not start at full traffic. Users discover the sale
//! through social media, email, and the homepage over several minutes. A
//! ramp-up scenario simulates this gradual increase, helping you find the
//! inflection point where latency starts to degrade.
//!
//! ## Instructions
//! 1. Define a `Scenario` struct representing a load test scenario
//! 2. Implement `ramp_up(target_rps, ramp_duration, hold_duration)` to create
//!    a scenario that ramps from 0 to `target_rps` over `ramp_duration`, then
//!    holds at `target_rps` for `hold_duration`
//! 3. Implement `execute()` to run the scenario, spawning tasks gradually
//! 4. Each task should send HTTP requests at the appropriate rate
//!
//! ## Hints
//! - Use `tokio::time::sleep` to pace the spawning of new tasks
//! - Calculate the delay between new task spawns: ramp_duration / target_rps
//! - Use `tokio::sync::broadcast` or an `AtomicBool` to signal shutdown
//! - Track when each task was spawned to compute the current target rate

use std::time::Duration;

/// Result from executing a load test scenario.
#[derive(Debug, Clone)]
pub struct ScenarioResult {
    /// Total requests sent during the scenario.
    pub total_requests: u64,
    /// Requests per second at the end of the scenario.
    pub final_rps: f64,
    /// Duration the scenario actually ran.
    pub actual_duration: Duration,
}

/// Custom error type for scenario operations.
#[derive(Debug, thiserror::Error)]
pub enum ScenarioError {
    #[error("Scenario execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// A load test scenario with configurable shape.
pub struct Scenario {
    /// Target requests per second at peak.
    pub target_rps: u64,
    /// Duration to ramp from 0 to target_rps.
    pub ramp_duration: Duration,
    /// Duration to hold at target_rps after ramp completes.
    pub hold_duration: Duration,
    /// Target URL for requests.
    pub target_url: String,
}

impl Scenario {
    /// Create a ramp-up scenario.
    ///
    /// # Arguments
    /// * `target_rps` - Peak requests per second to reach
    /// * `ramp_duration` - Time to ramp from 0 to target_rps
    /// * `hold_duration` - Time to sustain target_rps after ramp completes
    ///
    /// # Returns
    /// A `Scenario` configured for gradual ramp-up.
    pub fn ramp_up(target_rps: u64, ramp_duration: Duration, hold_duration: Duration) -> Self {
        // TODO: Create and return a Scenario with the given parameters
        // TODO: Set a default target_url or accept one as parameter
        todo!("Implement ramp_up scenario creation")
    }

    /// Execute the ramp-up scenario.
    ///
    /// Gradually spawns tasks over `ramp_duration`, then holds steady for
    /// `hold_duration`. Returns aggregated results.
    ///
    /// # Returns
    /// A `ScenarioResult` with the total requests sent and final throughput.
    pub async fn execute(&self) -> Result<ScenarioResult, ScenarioError> {
        // TODO: Track the start time
        // TODO: Calculate delay between spawning new tasks: ramp_duration / target_rps
        // TODO: In a loop, spawn a new task every `delay` interval
        // TODO: Each task sends requests in a loop
        // TODO: After ramp_duration, hold for hold_duration
        // TODO: Signal all tasks to stop and collect results
        todo!("Implement ramp-up execution")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramp_up_config() {
        let scenario = Scenario::ramp_up(
            100,
            Duration::from_secs(10),
            Duration::from_secs(5),
        );
        assert_eq!(scenario.target_rps, 100);
        assert_eq!(scenario.ramp_duration, Duration::from_secs(10));
        assert_eq!(scenario.hold_duration, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_ramp_up_rate_matches_config() {
        // Verify that the ramp-up rate is approximately correct.
        let scenario = Scenario::ramp_up(
            10,
            Duration::from_secs(2),
            Duration::from_millis(500),
        );
        let result = scenario.execute().await;
        assert!(result.is_ok(), "Ramp-up scenario should execute without error");
        let r = result.unwrap();
        // Should have sent some requests during the ramp + hold period
        assert!(r.total_requests > 0, "Should have sent at least some requests");
        // Duration should be approximately ramp + hold
        let expected_duration = Duration::from_secs(2) + Duration::from_millis(500);
        let diff = if r.actual_duration > expected_duration {
            r.actual_duration - expected_duration
        } else {
            expected_duration - r.actual_duration
        };
        assert!(
            diff < Duration::from_millis(500),
            "Actual duration should be close to expected"
        );
    }
}
