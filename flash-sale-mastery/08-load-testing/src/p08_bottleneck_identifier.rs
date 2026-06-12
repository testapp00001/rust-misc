//! # Exercise 08: Bottleneck Identifier
//!
//! ## Learning Objective
//! Analyze load test metrics to identify the primary bottleneck in a flash sale
//! system, using decision logic based on CPU usage, latency patterns, and error
//! types.
//!
//! ## Flash Sale Context
//! After running a load test, you know the system breaks at 5,000 rps -- but
//! why? Is it CPU saturation on the API server? Redis becoming slow? The
//! database connection pool running out? Identifying the bottleneck tells you
//! exactly what to fix before the next sale.
//!
//! ## Instructions
//! 1. Define a `SystemMetrics` struct with CPU, memory, latency, and error data
//! 2. Define a `Bottleneck` enum with variants for each bottleneck type
//! 3. Implement `identify_bottleneck(metrics)` that analyzes metrics and returns
//!    the most likely bottleneck
//! 4. Use decision logic based on thresholds and patterns
//!
//! ## Hints
//! - High CPU + normal latency => CPU bound
//! - High latency + low CPU => I/O bound (network or disk)
//! - Connection timeout errors => Connection pool exhausted
//! - Redis-specific errors or high Redis latency => Redis saturated
//! - DB-specific errors or high DB latency => DB saturated
//! - Check multiple signals; the strongest signal wins

/// System metrics collected during a load test.
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Average CPU utilization (0.0 to 1.0).
    pub cpu_usage: f64,
    /// Average memory utilization (0.0 to 1.0).
    pub memory_usage: f64,
    /// Average request latency in milliseconds.
    pub avg_latency_ms: f64,
    /// p99 request latency in milliseconds.
    pub p99_latency_ms: f64,
    /// Number of connection timeout errors.
    pub connection_timeout_count: u64,
    /// Number of Redis errors.
    pub redis_error_count: u64,
    /// Number of database errors.
    pub db_error_count: u64,
    /// Average Redis command latency in milliseconds.
    pub redis_latency_ms: f64,
    /// Average database query latency in milliseconds.
    pub db_latency_ms: f64,
    /// Total requests sent.
    pub total_requests: u64,
    /// Total errors of any kind.
    pub total_errors: u64,
}

/// Identified bottleneck in the system.
#[derive(Debug, Clone, PartialEq)]
pub enum Bottleneck {
    /// CPU is the limiting factor.
    CpuBound,
    /// I/O (network or disk) is the limiting factor.
    IoBound,
    /// HTTP connection pool is exhausted.
    ConnectionPoolExhausted,
    /// Redis is saturated or slow.
    RedisSaturated,
    /// Database is saturated or slow.
    DbSaturated,
    /// No clear bottleneck detected; system is healthy.
    NoneDetected,
}

/// Custom error type for bottleneck identification.
#[derive(Debug, thiserror::Error)]
pub enum BottleneckError {
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
}

/// Identify the primary bottleneck from system metrics.
///
/// Uses decision logic based on multiple signals:
/// - CPU usage, memory usage, latency patterns
/// - Error types (connection timeouts, Redis errors, DB errors)
/// - Component-specific latencies
///
/// # Arguments
/// * `metrics` - System metrics collected during a load test
///
/// # Returns
/// The identified `Bottleneck` variant.
pub fn identify_bottleneck(metrics: &SystemMetrics) -> Result<Bottleneck, BottleneckError> {
    // TODO: Check for connection pool exhaustion (high timeout count)
    // TODO: Check for Redis saturation (high redis latency or errors)
    // TODO: Check for DB saturation (high db latency or errors)
    // TODO: Check for CPU-bound (high CPU usage)
    // TODO: Check for I/O-bound (high latency, low CPU)
    // TODO: Default to NoneDetected
    todo!("Implement bottleneck identification")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy_metrics() -> SystemMetrics {
        SystemMetrics {
            cpu_usage: 0.3,
            memory_usage: 0.4,
            avg_latency_ms: 50.0,
            p99_latency_ms: 200.0,
            connection_timeout_count: 0,
            redis_error_count: 0,
            db_error_count: 0,
            redis_latency_ms: 1.0,
            db_latency_ms: 5.0,
            total_requests: 10000,
            total_errors: 10,
        }
    }

    #[test]
    fn test_cpu_bound() {
        let mut metrics = healthy_metrics();
        metrics.cpu_usage = 0.95;
        metrics.avg_latency_ms = 100.0;
        let bottleneck = identify_bottleneck(&metrics).expect("Should identify");
        assert_eq!(bottleneck, Bottleneck::CpuBound);
    }

    #[test]
    fn test_io_bound() {
        let mut metrics = healthy_metrics();
        metrics.cpu_usage = 0.2;
        metrics.avg_latency_ms = 2000.0;
        metrics.p99_latency_ms = 5000.0;
        let bottleneck = identify_bottleneck(&metrics).expect("Should identify");
        assert_eq!(bottleneck, Bottleneck::IoBound);
    }

    #[test]
    fn test_connection_pool_exhausted() {
        let mut metrics = healthy_metrics();
        metrics.connection_timeout_count = 500;
        metrics.total_errors = 500;
        let bottleneck = identify_bottleneck(&metrics).expect("Should identify");
        assert_eq!(bottleneck, Bottleneck::ConnectionPoolExhausted);
    }

    #[test]
    fn test_redis_saturated() {
        let mut metrics = healthy_metrics();
        metrics.redis_latency_ms = 100.0;
        metrics.redis_error_count = 200;
        metrics.total_errors = 200;
        let bottleneck = identify_bottleneck(&metrics).expect("Should identify");
        assert_eq!(bottleneck, Bottleneck::RedisSaturated);
    }

    #[test]
    fn test_db_saturated() {
        let mut metrics = healthy_metrics();
        metrics.db_latency_ms = 500.0;
        metrics.db_error_count = 300;
        metrics.total_errors = 300;
        let bottleneck = identify_bottleneck(&metrics).expect("Should identify");
        assert_eq!(bottleneck, Bottleneck::DbSaturated);
    }

    #[test]
    fn test_no_bottleneck() {
        let metrics = healthy_metrics();
        let bottleneck = identify_bottleneck(&metrics).expect("Should identify");
        assert_eq!(bottleneck, Bottleneck::NoneDetected);
    }
}
