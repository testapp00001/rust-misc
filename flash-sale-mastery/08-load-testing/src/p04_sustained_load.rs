//! # Exercise 04: Sustained Load
//!
//! ## Learning Objective
//! Implement a sustained load test that holds a steady request rate for an
//! extended period to detect memory leaks, connection pool exhaustion, and
//! other time-dependent degradation.
//!
//! ## Flash Sale Context
//! A flash sale might last 30 minutes to an hour. Systems that perform well
//! for 60 seconds may degrade over time as connection pools fill up, memory
//! accumulates, or caches evict useful entries. Sustained load tests reveal
//! these slow-building problems.
//!
//! ## Instructions
//! 1. Implement `sustained(target_rps, duration)` to create a sustained scenario
//! 2. Implement `execute()` to maintain a steady request rate for the duration
//! 3. Track throughput at regular intervals to verify stability
//! 4. Return results including throughput samples over time
//!
//! ## Hints
//! - Use a token bucket or interval-based approach to maintain steady rate
//! - Sample throughput every N seconds to detect drift
//! - Use `tokio::time::interval` for consistent request pacing
//! - Track min/max throughput across samples to verify stability

use std::time::Duration;

/// Result from executing a load test scenario.
#[derive(Debug, Clone)]
pub struct ScenarioResult {
    /// Total requests sent during the scenario.
    pub total_requests: u64,
    /// Average requests per second over the entire run.
    pub avg_rps: f64,
    /// Duration the scenario actually ran.
    pub actual_duration: Duration,
    /// Throughput samples taken every second (rps values).
    pub throughput_samples: Vec<f64>,
}

/// Custom error type for scenario operations.
#[derive(Debug, thiserror::Error)]
pub enum ScenarioError {
    #[error("Scenario execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// A load test scenario that maintains a steady load level.
pub struct Scenario {
    /// Target requests per second.
    pub target_rps: u64,
    /// Duration to sustain the load.
    pub duration: Duration,
    /// Target URL for requests.
    pub target_url: String,
}

impl Scenario {
    /// Create a sustained load scenario.
    ///
    /// # Arguments
    /// * `target_rps` - Requests per second to maintain
    /// * `duration` - How long to sustain the load
    ///
    /// # Returns
    /// A `Scenario` configured for sustained load.
    pub fn sustained(target_rps: u64, duration: Duration) -> Self {
        // TODO: Create and return a Scenario with the given parameters
        todo!("Implement sustained scenario creation")
    }

    /// Execute the sustained load scenario.
    ///
    /// Maintains a steady request rate for the configured duration, collecting
    /// throughput samples every second.
    ///
    /// # Returns
    /// A `ScenarioResult` with total requests, average RPS, and throughput samples.
    pub async fn execute(&self) -> Result<ScenarioResult, ScenarioError> {
        // TODO: Use tokio::time::interval to pace requests
        // TODO: Spawn worker tasks that send requests at the target rate
        // TODO: Sample throughput every second
        // TODO: Run for the configured duration
        // TODO: Collect and return results
        todo!("Implement sustained load execution")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sustained_config() {
        let scenario = Scenario::sustained(500, Duration::from_secs(30));
        assert_eq!(scenario.target_rps, 500);
        assert_eq!(scenario.duration, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_stable_throughput() {
        // Verify that a sustained scenario maintains approximately stable throughput.
        let scenario = Scenario::sustained(10, Duration::from_secs(3));
        let result = scenario.execute().await;
        assert!(result.is_ok(), "Sustained scenario should execute without error");
        let r = result.unwrap();
        assert!(r.total_requests > 0, "Should have sent requests");
        assert!(
            !r.throughput_samples.is_empty(),
            "Should have throughput samples"
        );
        // Average RPS should be in a reasonable range
        assert!(r.avg_rps > 0.0, "Average RPS should be positive");
    }
}
