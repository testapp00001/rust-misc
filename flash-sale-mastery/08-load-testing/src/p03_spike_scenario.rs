//! # Exercise 03: Spike Scenario
//!
//! ## Learning Objective
//! Implement an instant spike load test that jumps immediately to N concurrent
//! requests, testing how the system handles sudden traffic bursts.
//!
//! ## Flash Sale Context
//! When a flash sale link is shared on social media, traffic can jump from near
//! zero to thousands of requests per second in seconds. This tests circuit
//! breakers, admission control, and auto-scaling responses.
//!
//! ## Instructions
//! 1. Implement `spike(target_rps, duration)` to create a spike scenario
//! 2. Implement `execute()` to instantly spawn all tasks at once
//! 3. All tasks start sending requests simultaneously
//! 4. Tasks run for the configured duration, then stop
//!
//! ## Hints
//! - Use `tokio::task::JoinSet` or collect `JoinHandle`s for all spawned tasks
//! - Use a shared `AtomicBool` or `CancellationToken` to signal shutdown
//! - Start all tasks with `tokio::spawn` before awaiting any results
//! - Use `tokio::time::sleep(duration)` as the deadline for each task

use std::time::Duration;

/// Result from executing a load test scenario.
#[derive(Debug, Clone)]
pub struct ScenarioResult {
    /// Total requests sent during the scenario.
    pub total_requests: u64,
    /// Requests per second at peak.
    pub peak_rps: f64,
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

/// A load test scenario that applies instant spike load.
pub struct Scenario {
    /// Target requests per second.
    pub target_rps: u64,
    /// Duration to sustain the spike.
    pub duration: Duration,
    /// Target URL for requests.
    pub target_url: String,
}

impl Scenario {
    /// Create a spike scenario.
    ///
    /// # Arguments
    /// * `target_rps` - Requests per second to apply instantly
    /// * `duration` - How long to sustain the spike
    ///
    /// # Returns
    /// A `Scenario` configured for an instant spike.
    pub fn spike(target_rps: u64, duration: Duration) -> Self {
        // TODO: Create and return a Scenario with the given parameters
        todo!("Implement spike scenario creation")
    }

    /// Execute the spike scenario.
    ///
    /// Spawns all tasks simultaneously and runs for the configured duration.
    /// Returns aggregated results.
    ///
    /// # Returns
    /// A `ScenarioResult` with the total requests sent and peak throughput.
    pub async fn execute(&self) -> Result<ScenarioResult, ScenarioError> {
        // TODO: Spawn all tasks at once (up to target_rps tasks)
        // TODO: Each task sends requests in a tight loop until duration expires
        // TODO: Use a shared shutdown signal
        // TODO: Collect results from all tasks
        todo!("Implement spike execution")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spike_config() {
        let scenario = Scenario::spike(1000, Duration::from_secs(5));
        assert_eq!(scenario.target_rps, 1000);
        assert_eq!(scenario.duration, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_spike_instant_load() {
        // Verify that a spike scenario starts at full load immediately.
        let scenario = Scenario::spike(10, Duration::from_millis(500));
        let result = scenario.execute().await;
        assert!(result.is_ok(), "Spike scenario should execute without error");
        let r = result.unwrap();
        assert!(r.total_requests > 0, "Should have sent requests");
        // Peak RPS should be at or near the target
        assert!(r.peak_rps > 0.0, "Peak RPS should be positive");
    }
}
