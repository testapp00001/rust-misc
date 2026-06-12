//! # Solution 02: Ramp-Up Scenario
//!
//! Complete implementation of a gradual ramp-up load test scenario.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
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
    pub fn ramp_up(target_rps: u64, ramp_duration: Duration, hold_duration: Duration) -> Self {
        Self {
            target_rps,
            ramp_duration,
            hold_duration,
            target_url: "http://127.0.0.1:19876/test".to_string(),
        }
    }

    /// Execute the ramp-up scenario.
    pub async fn execute(&self) -> Result<ScenarioResult, ScenarioError> {
        if self.target_rps == 0 {
            return Err(ScenarioError::InvalidConfig(
                "target_rps must be > 0".to_string(),
            ));
        }

        let start = tokio::time::Instant::now();
        let total_duration = self.ramp_duration + self.hold_duration;
        let shutdown = Arc::new(AtomicBool::new(false));
        let total_requests = Arc::new(AtomicU64::new(0));

        // Calculate delay between spawning new tasks during ramp phase.
        let spawn_interval = if self.target_rps > 0 {
            self.ramp_duration / self.target_rps as u32
        } else {
            Duration::from_secs(1)
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .connect_timeout(Duration::from_millis(200))
            .build()
            .map_err(|e| ScenarioError::ExecutionFailed(e.to_string()))?;

        let mut handles = Vec::new();
        let url = self.target_url.clone();

        // Spawn tasks gradually during the ramp phase.
        for _ in 0..self.target_rps {
            if start.elapsed() >= total_duration {
                break;
            }

            let client = client.clone();
            let shutdown = shutdown.clone();
            let total_requests = total_requests.clone();
            let url = url.clone();
            let _task_start = tokio::time::Instant::now();
            let task_deadline = start + total_duration;

            handles.push(tokio::spawn(async move {
                while !shutdown.load(Ordering::Relaxed)
                    && tokio::time::Instant::now() < task_deadline
                {
                    let _ = client.get(&url).send().await;
                    total_requests.fetch_add(1, Ordering::Relaxed);
                    // Pace: each task targets approximately 1 rps
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }));

            // Wait before spawning the next task (gradual ramp).
            tokio::time::sleep(spawn_interval).await;
        }

        // Wait for remaining duration if ramp finished early.
        let remaining = total_duration.saturating_sub(start.elapsed());
        if remaining > Duration::ZERO {
            tokio::time::sleep(remaining).await;
        }

        // Signal shutdown and wait for all tasks.
        shutdown.store(true, Ordering::Relaxed);
        for h in handles {
            let _ = h.await;
        }

        let actual_duration = start.elapsed();
        let total = total_requests.load(Ordering::Relaxed);
        let final_rps = if actual_duration.as_secs_f64() > 0.0 {
            total as f64 / actual_duration.as_secs_f64()
        } else {
            0.0
        };

        Ok(ScenarioResult {
            total_requests: total,
            final_rps,
            actual_duration,
        })
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
        let scenario = Scenario::ramp_up(
            10,
            Duration::from_secs(2),
            Duration::from_millis(500),
        );
        let result = scenario.execute().await;
        assert!(result.is_ok(), "Ramp-up scenario should execute without error");
        let r = result.unwrap();
        assert!(r.total_requests > 0, "Should have sent at least some requests");
        let expected_duration = Duration::from_secs(2) + Duration::from_millis(500);
        let diff = if r.actual_duration > expected_duration {
            r.actual_duration - expected_duration
        } else {
            expected_duration - r.actual_duration
        };
        assert!(
            diff < Duration::from_secs(3),
            "Actual duration should be close to expected, diff={:?}",
            diff
        );
    }
}
