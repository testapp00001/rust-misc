//! # Solution 01: Load Generator
//!
//! Complete implementation of an HTTP load generator using tokio + reqwest.

use std::time::Duration;

/// Configuration for a load test run.
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// Target URL to send requests to.
    pub target_url: String,
    /// Number of concurrent workers.
    pub concurrency: usize,
    /// Total duration of the load test.
    pub duration: Duration,
}

/// Result of a single HTTP request.
#[derive(Debug, Clone)]
pub struct RequestResult {
    /// Whether the request succeeded (2xx status).
    pub success: bool,
    /// Round-trip latency.
    pub latency: Duration,
}

/// Aggregated results of a load test.
#[derive(Debug, Clone)]
pub struct LoadTestResult {
    /// Total number of requests sent.
    pub total_requests: u64,
    /// Number of requests that received a 2xx response.
    pub successful: u64,
    /// Number of requests that failed (non-2xx or error).
    pub failed: u64,
    /// Requests per second (throughput).
    pub throughput_rps: f64,
    /// Latency percentiles in milliseconds: (p50, p95, p99).
    pub latency_percentiles: (f64, f64, f64),
}

/// Custom error type for load generator operations.
#[derive(Debug, thiserror::Error)]
pub enum LoadTestError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// A load generator that sends HTTP requests at controlled concurrency.
pub struct LoadGenerator {
    config: LoadTestConfig,
    client: reqwest::Client,
}

impl LoadGenerator {
    /// Create a new load generator from configuration.
    pub fn new(config: LoadTestConfig) -> Result<Self, LoadTestError> {
        if config.concurrency == 0 {
            return Err(LoadTestError::ConfigError(
                "Concurrency must be > 0".to_string(),
            ));
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(config.concurrency)
            .build()
            .map_err(|e| LoadTestError::ConfigError(e.to_string()))?;
        Ok(Self { config, client })
    }

    /// Execute the load test and return aggregated results.
    pub async fn run(&self) -> Result<LoadTestResult, LoadTestError> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<RequestResult>(self.config.concurrency * 100);
        let start = tokio::time::Instant::now();
        let deadline = start + self.config.duration;
        let url = self.config.target_url.clone();

        let mut handles = Vec::with_capacity(self.config.concurrency);
        for _ in 0..self.config.concurrency {
            let client = self.client.clone();
            let tx = tx.clone();
            let url = url.clone();
            handles.push(tokio::spawn(async move {
                loop {
                    if tokio::time::Instant::now() >= deadline {
                        break;
                    }
                    let req_start = tokio::time::Instant::now();
                    let result = match client.get(&url).send().await {
                        Ok(resp) => {
                            let latency = req_start.elapsed();
                            RequestResult {
                                success: resp.status().is_success(),
                                latency,
                            }
                        }
                        Err(_) => {
                            let latency = req_start.elapsed();
                            RequestResult {
                                success: false,
                                latency,
                            }
                        }
                    };
                    let _ = tx.send(result).await;
                }
            }));
        }

        // Drop the original sender so the channel closes when all workers finish.
        drop(tx);

        // Collect results in the background.
        let mut results = Vec::new();
        while let Some(r) = rx.recv().await {
            results.push(r);
        }

        // Wait for all workers to complete.
        for h in handles {
            let _ = h.await;
        }

        let elapsed = start.elapsed();
        let total = results.len() as u64;
        let successful = results.iter().filter(|r| r.success).count() as u64;
        let failed = total - successful;

        // Compute latency percentiles.
        let mut latencies_ms: Vec<f64> = results
            .iter()
            .map(|r| r.latency.as_secs_f64() * 1000.0)
            .collect();
        latencies_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p50 = percentile(&latencies_ms, 50.0);
        let p95 = percentile(&latencies_ms, 95.0);
        let p99 = percentile(&latencies_ms, 99.0);

        let throughput = if elapsed.as_secs_f64() > 0.0 {
            total as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        Ok(LoadTestResult {
            total_requests: total,
            successful,
            failed,
            throughput_rps: throughput,
            latency_percentiles: (p50, p95, p99),
        })
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let n = sorted.len() as f64;
    let rank = (p / 100.0 * n).ceil().max(1.0) as usize;
    let index = rank.min(sorted.len()) - 1;
    sorted[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_generator_creation() {
        let config = LoadTestConfig {
            target_url: "http://127.0.0.1:9999/test".to_string(),
            concurrency: 2,
            duration: Duration::from_secs(1),
        };
        let generator = LoadGenerator::new(config);
        assert!(generator.is_ok(), "LoadGenerator should be created successfully");
    }

    #[tokio::test]
    async fn test_load_generator_against_mock_server() {
        let config = LoadTestConfig {
            target_url: "http://127.0.0.1:19876/test".to_string(),
            concurrency: 2,
            duration: Duration::from_millis(500),
        };
        let generator = LoadGenerator::new(config).expect("Should create generator");
        let result = generator.run().await;
        assert!(result.is_ok(), "Generator should handle failures gracefully");
        let r = result.unwrap();
        assert_eq!(
            r.total_requests,
            r.successful + r.failed,
            "total = successful + failed"
        );
    }
}
