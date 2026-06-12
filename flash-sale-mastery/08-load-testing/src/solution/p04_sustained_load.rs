//! # Solution 04: Sustained Load
//!
//! Complete implementation of a sustained load test scenario.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
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
    pub fn sustained(target_rps: u64, duration: Duration) -> Self {
        Self {
            target_rps,
            duration,
            target_url: "http://127.0.0.1:19876/test".to_string(),
        }
    }

    /// Execute the sustained load scenario.
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

        // Per-second bucket for throughput sampling.
        let bucket_requests = Arc::new(AtomicU64::new(0));
        let throughput_samples = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        // Determine number of worker tasks and per-task delay to hit target_rps.
        let worker_count = self.target_rps.min(100); // cap workers, increase per-worker rate
        let requests_per_worker_per_sec = self.target_rps as f64 / worker_count as f64;
        let per_request_delay =
            Duration::from_secs_f64(1.0 / requests_per_worker_per_sec.max(1.0));

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .connect_timeout(Duration::from_millis(200))
            .build()
            .map_err(|e| ScenarioError::ExecutionFailed(e.to_string()))?;

        let mut handles = Vec::with_capacity(worker_count as usize);
        for _ in 0..worker_count {
            let client = client.clone();
            let shutdown = shutdown.clone();
            let total_requests = total_requests.clone();
            let bucket_requests = bucket_requests.clone();
            let url = self.target_url.clone();
            let delay = per_request_delay;

            handles.push(tokio::spawn(async move {
                while !shutdown.load(Ordering::Relaxed) && tokio::time::Instant::now() < deadline {
                    let _ = client.get(&url).send().await;
                    total_requests.fetch_add(1, Ordering::Relaxed);
                    bucket_requests.fetch_add(1, Ordering::Relaxed);
                    tokio::time::sleep(delay).await;
                }
            }));
        }

        // Throughput sampler: record every second.
        let samples_clone = throughput_samples.clone();
        let bucket_clone = bucket_requests.clone();
        let shutdown_clone = shutdown.clone();
        let sampler = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            interval.tick().await; // skip first immediate tick
            while !shutdown_clone.load(Ordering::Relaxed) {
                interval.tick().await;
                let count = bucket_clone.swap(0, Ordering::Relaxed);
                samples_clone.lock().await.push(count as f64);
            }
        });

        // Wait for the full duration.
        tokio::time::sleep(self.duration).await;

        // Shutdown.
        shutdown.store(true, Ordering::Relaxed);
        for h in handles {
            let _ = h.await;
        }
        let _ = sampler.await;

        let actual_duration = start.elapsed();
        let total = total_requests.load(Ordering::Relaxed);
        let avg_rps = if actual_duration.as_secs_f64() > 0.0 {
            total as f64 / actual_duration.as_secs_f64()
        } else {
            0.0
        };
        let samples = throughput_samples.lock().await.clone();

        Ok(ScenarioResult {
            total_requests: total,
            avg_rps,
            actual_duration,
            throughput_samples: samples,
        })
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
        let scenario = Scenario::sustained(10, Duration::from_secs(3));
        let result = scenario.execute().await;
        assert!(result.is_ok(), "Sustained scenario should execute without error");
        let r = result.unwrap();
        assert!(r.total_requests > 0, "Should have sent requests");
        assert!(
            !r.throughput_samples.is_empty(),
            "Should have throughput samples"
        );
        assert!(r.avg_rps > 0.0, "Average RPS should be positive");
    }
}
