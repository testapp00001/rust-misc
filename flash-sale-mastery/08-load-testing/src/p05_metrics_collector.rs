//! # Exercise 05: Metrics Collector
//!
//! ## Learning Objective
//! Build a thread-safe metrics collector that records request results from
//! concurrent workers and computes aggregated statistics.
//!
//! ## Flash Sale Context
//! During a load test, dozens or hundreds of worker tasks send requests
//! concurrently. Each task reports its results, and the collector must
//! aggregate them without losing data or becoming a bottleneck itself.
//!
//! ## Instructions
//! 1. Implement `MetricsCollector` with thread-safe internal state
//! 2. Implement `record(request_result)` to accept results from any task
//! 3. Implement `snapshot()` to return a point-in-time view of metrics
//! 4. Implement `finalize()` to produce a complete `FinalReport`
//! 5. The report must include p50, p95, p99, p99.9 latency, error rate, and throughput
//!
//! ## Hints
//! - Use `std::sync::Mutex` or `tokio::sync::Mutex` for interior mutability
//! - Store raw latency values for accurate percentile calculation
//! - Track timestamps to compute throughput over time
//! - Use `AtomicU64` for simple counters if you want lock-free counting

use std::time::Duration;

/// A single request result to be recorded.
#[derive(Debug, Clone)]
pub struct RequestResult {
    /// Whether the request succeeded.
    pub success: bool,
    /// Round-trip latency.
    pub latency: Duration,
    /// When the request completed.
    pub timestamp: std::time::Instant,
}

/// A point-in-time snapshot of metrics.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Requests recorded so far.
    pub total_requests: u64,
    /// Successful requests so far.
    pub successful: u64,
    /// Failed requests so far.
    pub failed: u64,
    /// Current error rate (0.0 to 1.0).
    pub error_rate: f64,
}

/// Final aggregated report from a load test.
#[derive(Debug, Clone)]
pub struct FinalReport {
    /// Total requests recorded.
    pub total_requests: u64,
    /// Successful requests.
    pub successful: u64,
    /// Failed requests.
    pub failed: u64,
    /// Error rate (0.0 to 1.0).
    pub error_rate: f64,
    /// Throughput in requests per second.
    pub throughput_rps: f64,
    /// p50 latency in milliseconds.
    pub p50_ms: f64,
    /// p95 latency in milliseconds.
    pub p95_ms: f64,
    /// p99 latency in milliseconds.
    pub p99_ms: f64,
    /// p99.9 latency in milliseconds.
    pub p999_ms: f64,
}

/// Custom error type for metrics operations.
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("No data recorded")]
    NoData,

    #[error("Metrics computation error: {0}")]
    ComputationError(String),
}

/// Thread-safe metrics collector for load test results.
pub struct MetricsCollector {
    // TODO: Add fields for storing request results
    // Consider: Vec<RequestResult> behind a Mutex, or separate atomic counters
}

impl MetricsCollector {
    /// Create a new empty metrics collector.
    pub fn new() -> Self {
        // TODO: Initialize internal state
        todo!("Implement MetricsCollector creation")
    }

    /// Record a request result.
    ///
    /// This method is safe to call from multiple concurrent tasks.
    ///
    /// # Arguments
    /// * `result` - The request result to record
    pub fn record(&self, result: RequestResult) {
        // TODO: Store the result in a thread-safe manner
        todo!("Implement result recording")
    }

    /// Get a point-in-time snapshot of current metrics.
    ///
    /// # Returns
    /// A `MetricsSnapshot` with current counts and error rate.
    pub fn snapshot(&self) -> MetricsSnapshot {
        // TODO: Read current state and compute snapshot
        todo!("Implement snapshot")
    }

    /// Finalize the metrics and produce a complete report.
    ///
    /// Computes all latency percentiles and throughput from recorded data.
    ///
    /// # Returns
    /// A `FinalReport` with all aggregated metrics.
    ///
    /// # Errors
    /// Returns `MetricsError::NoData` if no results were recorded.
    pub fn finalize(&self) -> Result<FinalReport, MetricsError> {
        // TODO: Compute p50, p95, p99, p99.9 from stored latencies
        // TODO: Compute throughput from timestamps
        // TODO: Return FinalReport
        todo!("Implement finalize")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_record_and_snapshot() {
        let collector = MetricsCollector::new();
        collector.record(RequestResult {
            success: true,
            latency: Duration::from_millis(50),
            timestamp: Instant::now(),
        });
        collector.record(RequestResult {
            success: false,
            latency: Duration::from_millis(100),
            timestamp: Instant::now(),
        });
        let snap = collector.snapshot();
        assert_eq!(snap.total_requests, 2);
        assert_eq!(snap.successful, 1);
        assert_eq!(snap.failed, 1);
        assert!((snap.error_rate - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_finalize_percentiles() {
        let collector = MetricsCollector::new();
        let now = Instant::now();
        // Record 100 requests with known latencies: 1ms, 2ms, ..., 100ms
        for i in 1..=100 {
            collector.record(RequestResult {
                success: true,
                latency: Duration::from_millis(i),
                timestamp: now,
            });
        }
        let report = collector.finalize().expect("Should produce report");
        assert_eq!(report.total_requests, 100);
        assert_eq!(report.successful, 100);
        assert_eq!(report.failed, 0);
        // p50 should be around 50ms
        assert!(
            (report.p50_ms - 50.0).abs() < 2.0,
            "p50 should be ~50ms, got {}",
            report.p50_ms
        );
        // p95 should be around 95ms
        assert!(
            (report.p95_ms - 95.0).abs() < 2.0,
            "p95 should be ~95ms, got {}",
            report.p95_ms
        );
        // p99 should be around 99ms
        assert!(
            (report.p99_ms - 99.0).abs() < 2.0,
            "p99 should be ~99ms, got {}",
            report.p99_ms
        );
    }

    #[tokio::test]
    async fn test_concurrent_recording() {
        use std::sync::Arc;
        let collector = Arc::new(MetricsCollector::new());
        let mut handles = Vec::new();
        for _ in 0..10 {
            let c = Arc::clone(&collector);
            handles.push(tokio::spawn(async move {
                for _ in 0..100 {
                    c.record(RequestResult {
                        success: true,
                        latency: Duration::from_millis(10),
                        timestamp: Instant::now(),
                    });
                }
            }));
        }
        for h in handles {
            h.await.expect("Task should complete");
        }
        let snap = collector.snapshot();
        assert_eq!(snap.total_requests, 1000);
    }

    #[test]
    fn test_finalize_no_data() {
        let collector = MetricsCollector::new();
        let result = collector.finalize();
        assert!(result.is_err(), "Should error on empty data");
    }
}
