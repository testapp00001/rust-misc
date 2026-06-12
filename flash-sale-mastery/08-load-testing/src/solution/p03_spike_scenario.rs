//! # Solution 03: Spike Scenario
//!
//! Complete implementation of an instant spike load test scenario.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
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
    pub fn spike(target_rps: u64, duration: Duration) -> Self {
        Self {
            target_rps,
            duration,
            target_url: "http://127.0.0.1:19876/test".to_string(),
        }
    }

    /// Execute the spike scenario.
    pub async fn execute(&self) -> Result<ScenarioResult, ScenarioError> {
        if self.target_rps == 0 {
            return Err(ScenarioError::InvalidConfig(
                "target_rps must be > 0".to_string(),
            ));
        }

        let start = tokio::time::Instant::now();
        let deadline = start + self.duration;
        let shutdown = Arc::new(AtomicBool::new(false));
        let total_requests = Arc::new(AtomicU64::new(0));

        // Record requests per second in 1-second buckets for peak calculation.
        let bucket_requests = Arc::new(AtomicU64::new(0));
        let peak_rps = Arc::new(tokio::sync::Mutex::new(0.0f64));

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .connect_timeout(Duration::from_millis(200))
            .build()
            .map_err(|e| ScenarioError::ExecutionFailed(e.to_string()))?;

        // Spawn all tasks at once (instant spike).
        let mut handles = Vec::with_capacity(self.target_rps as usize);
        for _ in 0..self.target_rps {
            let client = client.clone();
            let shutdown = shutdown.clone();
            let total_requests = total_requests.clone();
            let bucket_requests = bucket_requests.clone();
            let url = self.target_url.clone();

            handles.push(tokio::spawn(async move {
                while !shutdown.load(Ordering::Relaxed) && tokio::time::Instant::now() < deadline {
                    let _ = client.get(&url).send().await;
                    total_requests.fetch_add(1, Ordering::Relaxed);
                    bucket_requests.fetch_add(1, Ordering::Relaxed);
                }
            }));
        }

        // Track peak RPS by sampling every second.
        let peak_rps_clone = peak_rps.clone();
        let bucket_clone = bucket_requests.clone();
        let shutdown_clone = shutdown.clone();
        let tracker = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            interval.tick().await; // skip first immediate tick
            while !shutdown_clone.load(Ordering::Relaxed) {
                interval.tick().await;
                let count = bucket_clone.swap(0, Ordering::Relaxed);
                let mut peak = peak_rps_clone.lock().await;
                if count as f64 > *peak {
                    *peak = count as f64;
                }
            }
        });

        // Wait for the duration.
        tokio::time::sleep(self.duration).await;

        // Signal shutdown and wait for all tasks.
        shutdown.store(true, Ordering::Relaxed);
        for h in handles {
            let _ = h.await;
        }
        let _ = tracker.await;

        let actual_duration = start.elapsed();
        let total = total_requests.load(Ordering::Relaxed);
        let peak = *peak_rps.lock().await;
        // If peak tracking didn't capture a full second, estimate from total.
        let peak_rps_val = if peak > 0.0 {
            peak
        } else if actual_duration.as_secs_f64() > 0.0 {
            total as f64 / actual_duration.as_secs_f64()
        } else {
            0.0
        };

        Ok(ScenarioResult {
            total_requests: total,
            peak_rps: peak_rps_val,
            actual_duration,
        })
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
        let scenario = Scenario::spike(10, Duration::from_millis(500));
        let result = scenario.execute().await;
        assert!(result.is_ok(), "Spike scenario should execute without error");
        let r = result.unwrap();
        assert!(r.total_requests > 0, "Should have sent requests");
        assert!(r.peak_rps > 0.0, "Peak RPS should be positive");
    }
}
