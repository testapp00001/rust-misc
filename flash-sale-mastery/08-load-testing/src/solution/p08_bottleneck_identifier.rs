//! # Solution 08: Bottleneck Identifier
//!
//! Complete implementation of bottleneck identification from load test metrics.

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
/// Decision priority (highest confidence signal wins):
/// 1. Connection pool exhaustion: significant connection timeouts
/// 2. Redis saturation: high Redis latency or Redis errors
/// 3. Database saturation: high DB latency or DB errors
/// 4. CPU bound: high CPU usage with moderate latency
/// 5. I/O bound: high latency with low CPU usage
/// 6. None detected: everything looks healthy
pub fn identify_bottleneck(metrics: &SystemMetrics) -> Result<Bottleneck, BottleneckError> {
    let _error_rate = if metrics.total_requests > 0 {
        metrics.total_errors as f64 / metrics.total_requests as f64
    } else {
        0.0
    };

    // Threshold for "significant" error counts relative to total traffic.
    let significant_error_threshold = (metrics.total_requests as f64 * 0.01).max(10.0) as u64;

    // 1. Connection pool exhaustion: many connection timeouts.
    if metrics.connection_timeout_count > significant_error_threshold {
        return Ok(Bottleneck::ConnectionPoolExhausted);
    }

    // 2. Redis saturation: high Redis latency (> 50ms) or many Redis errors.
    if metrics.redis_latency_ms > 50.0 || metrics.redis_error_count > significant_error_threshold {
        return Ok(Bottleneck::RedisSaturated);
    }

    // 3. Database saturation: high DB latency (> 100ms) or many DB errors.
    if metrics.db_latency_ms > 100.0 || metrics.db_error_count > significant_error_threshold {
        return Ok(Bottleneck::DbSaturated);
    }

    // 4. CPU bound: high CPU usage (> 85%).
    if metrics.cpu_usage > 0.85 {
        return Ok(Bottleneck::CpuBound);
    }

    // 5. I/O bound: high latency but CPU is not the issue.
    if metrics.avg_latency_ms > 500.0 && metrics.cpu_usage < 0.5 {
        return Ok(Bottleneck::IoBound);
    }

    // 6. No clear bottleneck.
    Ok(Bottleneck::NoneDetected)
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
