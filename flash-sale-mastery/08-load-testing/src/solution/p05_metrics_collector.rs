//! # Solution 05: Metrics Collector
//!
//! Complete implementation of a thread-safe metrics collector for load tests.

use std::sync::Mutex;
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

/// Internal state of the metrics collector, protected by a Mutex.
struct CollectorState {
    results: Vec<RequestResult>,
    successful: u64,
    failed: u64,
}

/// Thread-safe metrics collector for load test results.
pub struct MetricsCollector {
    state: Mutex<CollectorState>,
}

impl MetricsCollector {
    /// Create a new empty metrics collector.
    pub fn new() -> Self {
        Self {
            state: Mutex::new(CollectorState {
                results: Vec::new(),
                successful: 0,
                failed: 0,
            }),
        }
    }

    /// Record a request result.
    pub fn record(&self, result: RequestResult) {
        let mut state = self.state.lock().expect("Lock poisoned");
        if result.success {
            state.successful += 1;
        } else {
            state.failed += 1;
        }
        state.results.push(result);
    }

    /// Get a point-in-time snapshot of current metrics.
    pub fn snapshot(&self) -> MetricsSnapshot {
        let state = self.state.lock().expect("Lock poisoned");
        let total = state.results.len() as u64;
        let successful = state.successful;
        let failed = state.failed;
        let error_rate = if total > 0 {
            failed as f64 / total as f64
        } else {
            0.0
        };
        MetricsSnapshot {
            total_requests: total,
            successful,
            failed,
            error_rate,
        }
    }

    /// Finalize the metrics and produce a complete report.
    pub fn finalize(&self) -> Result<FinalReport, MetricsError> {
        let state = self.state.lock().expect("Lock poisoned");
        if state.results.is_empty() {
            return Err(MetricsError::NoData);
        }

        let total = state.results.len() as u64;
        let successful = state.successful;
        let failed = state.failed;
        let error_rate = failed as f64 / total as f64;

        // Compute throughput from timestamps.
        let first = state.results.iter().map(|r| r.timestamp).min().unwrap();
        let last = state.results.iter().map(|r| r.timestamp).max().unwrap();
        let elapsed = last.duration_since(first).as_secs_f64();
        let throughput_rps = if elapsed > 0.0 {
            total as f64 / elapsed
        } else {
            total as f64 // all at the same instant
        };

        // Compute percentiles.
        let mut latencies_ms: Vec<f64> = state
            .results
            .iter()
            .map(|r| r.latency.as_secs_f64() * 1000.0)
            .collect();
        latencies_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p50_ms = percentile(&latencies_ms, 50.0);
        let p95_ms = percentile(&latencies_ms, 95.0);
        let p99_ms = percentile(&latencies_ms, 99.0);
        let p999_ms = percentile(&latencies_ms, 99.9);

        Ok(FinalReport {
            total_requests: total,
            successful,
            failed,
            error_rate,
            throughput_rps,
            p50_ms,
            p95_ms,
            p99_ms,
            p999_ms,
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
        assert!(
            (report.p50_ms - 50.0).abs() < 2.0,
            "p50 should be ~50ms, got {}",
            report.p50_ms
        );
        assert!(
            (report.p95_ms - 95.0).abs() < 2.0,
            "p95 should be ~95ms, got {}",
            report.p95_ms
        );
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
