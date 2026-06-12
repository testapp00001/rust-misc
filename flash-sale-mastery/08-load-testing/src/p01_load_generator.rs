//! # Exercise 01: Load Generator
//!
//! ## Learning Objective
//! Build a custom HTTP load generator that sends requests at a controlled rate
//! and collects latency and throughput metrics.
//!
//! ## Flash Sale Context
//! Before a sale goes live, you need to verify the system can handle the
//! expected traffic. A load generator simulates thousands of concurrent users
//! hitting the flash sale endpoint, measuring how the system responds.
//!
//! ## Instructions
//! 1. Define a `LoadGenerator` struct with target URL, concurrency level, and duration
//! 2. Implement `new(config)` to create a generator from a `LoadTestConfig`
//! 3. Implement `run()` to execute the load test and return a `LoadTestResult`
//! 4. Each worker task should send HTTP GET requests and record latency
//! 5. Aggregate all worker results into the final `LoadTestResult`
//!
//! ## Hints
//! - Use `reqwest::Client` with connection pooling for realistic HTTP behavior
//! - Use `tokio::time::Instant` for latency measurement
//! - Use `tokio::sync::mpsc` to collect results from worker tasks
//! - Track time with `tokio::time::timeout` to enforce the test duration

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
    ///
    /// # Arguments
    /// * `config` - Load test configuration specifying URL, concurrency, and duration
    ///
    /// # Returns
    /// A new `LoadGenerator` ready to execute.
    pub fn new(config: LoadTestConfig) -> Result<Self, LoadTestError> {
        // TODO: Create a reqwest::Client with appropriate settings
        // TODO: Return the LoadGenerator
        todo!("Implement load generator creation")
    }

    /// Execute the load test and return aggregated results.
    ///
    /// Spawns `concurrency` worker tasks, each sending requests for the
    /// configured duration. Results are collected and aggregated.
    ///
    /// # Returns
    /// A `LoadTestResult` with total requests, success/failure counts,
    /// throughput, and latency percentiles.
    pub async fn run(&self) -> Result<LoadTestResult, LoadTestError> {
        // TODO: Create a channel to collect RequestResults from workers
        // TODO: Spawn `self.config.concurrency` worker tasks
        // TODO: Each worker sends requests in a loop until duration expires
        // TODO: Collect all results and compute LoadTestResult
        todo!("Implement load test execution")
    }
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
        // This test verifies the load generator works against an HTTP endpoint.
        // The student's implementation should handle connection errors gracefully.
        let config = LoadTestConfig {
            target_url: "http://127.0.0.1:19876/test".to_string(),
            concurrency: 2,
            duration: Duration::from_millis(500),
        };
        let generator = LoadGenerator::new(config).expect("Should create generator");
        let result = generator.run().await;
        // The mock server is not running, so requests will fail, but the generator
        // should still produce a result without panicking.
        assert!(result.is_ok(), "Generator should handle failures gracefully");
        let r = result.unwrap();
        assert_eq!(
            r.total_requests,
            r.successful + r.failed,
            "total = successful + failed"
        );
    }
}
