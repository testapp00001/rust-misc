//! # Exercise 04: Redis Metrics
//!
//! ## Learning Objective
//! Learn how to instrument Redis operations with dedicated metrics. Redis is
//! the critical path in flash sale systems, so its performance must be
//! monitored independently from application-level metrics.
//!
//! ## Flash Sale Context
//! Redis latency directly impacts purchase response time. If Redis operations
//! slow from 1ms to 10ms, your entire flash sale degrades. Separate Redis
//! metrics let you distinguish "the app is slow" from "Redis is slow."
//!
//! ## Instructions
//! 1. Implement `RedisMetrics::new()` to register Redis-specific metrics
//! 2. Implement `record_operation` to track operation latency by type
//! 3. Implement `record_error` to count errors by operation type
//! 4. Implement `get_pool_size_gauge` to expose the pool size gauge
//!
//! ## Hints
//! - Use `HistogramVec` with label `operation` for latency
//! - Use `IntCounterVec` with label `operation` for errors
//! - Use `IntGauge` (not Vec) for pool size since it is a single value
//! - Typical operations: "get", "set", "decr", "eval", "ping"

use prometheus::{HistogramVec, IntCounterVec, IntGauge, Registry};
use std::time::Duration;

/// Redis operation types.
#[derive(Debug, Clone, Copy)]
pub enum RedisOperation {
    Get,
    Set,
    Decr,
    Eval,
    Ping,
}

impl RedisOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            RedisOperation::Get => "get",
            RedisOperation::Set => "set",
            RedisOperation::Decr => "decr",
            RedisOperation::Eval => "eval",
            RedisOperation::Ping => "ping",
        }
    }
}

/// Prometheus metrics for Redis operations.
pub struct RedisMetrics {
    /// Histogram for Redis operation latency, labeled by operation type.
    pub operation_latency: HistogramVec,
    /// Counter for Redis errors, labeled by operation type.
    pub errors: IntCounterVec,
    /// Gauge for current Redis connection pool size.
    pub pool_size: IntGauge,
    /// The registry holding all metrics.
    pub registry: Registry,
}

impl RedisMetrics {
    /// Create and register all Redis metrics.
    ///
    /// Registers:
    /// - `redis_operation_latency_seconds` histogram with label `operation`
    /// - `redis_errors_total` counter with label `operation`
    /// - `redis_connection_pool_size` gauge
    ///
    /// # Returns
    /// A new `RedisMetrics` instance.
    pub fn new() -> Self {
        // TODO: Create a Registry
        // TODO: Create HistogramVec for operation_latency with label "operation"
        // TODO: Create IntCounterVec for errors with label "operation"
        // TODO: Create IntGauge for pool_size
        // TODO: Register all metrics
        todo!("Implement Redis metrics creation")
    }

    /// Record a Redis operation's latency.
    ///
    /// # Arguments
    /// * `op` - The Redis operation type
    /// * `duration` - How long the operation took
    pub fn record_operation(&self, op: RedisOperation, duration: Duration) {
        // TODO: Observe the duration on the appropriate histogram bucket
        todo!("Implement operation latency recording")
    }

    /// Record a Redis error for the given operation.
    ///
    /// # Arguments
    /// * `op` - The operation that failed
    pub fn record_error(&self, op: RedisOperation) {
        // TODO: Increment the error counter for the given operation
        todo!("Implement error recording")
    }

    /// Get a reference to the pool size gauge for external updates.
    ///
    /// # Returns
    /// A reference to the `IntGauge` for pool size.
    pub fn get_pool_size_gauge(&self) -> &IntGauge {
        // TODO: Return a reference to the pool_size gauge
        todo!("Implement pool size gauge accessor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Encoder;

    #[test]
    fn test_redis_metrics_creation() {
        let metrics = RedisMetrics::new();
        // Labeled metrics don't appear in gather() until a label combo is accessed.
        metrics.record_operation(RedisOperation::Get, Duration::from_millis(1));
        metrics.record_error(RedisOperation::Get);
        metrics.get_pool_size_gauge().set(1);
        let families = metrics.registry.gather();
        assert_eq!(families.len(), 3, "Should have 3 metric families");
    }

    #[test]
    fn test_record_operation_latency() {
        let metrics = RedisMetrics::new();
        metrics.record_operation(RedisOperation::Get, Duration::from_millis(2));
        metrics.record_operation(RedisOperation::Get, Duration::from_millis(5));
        metrics.record_operation(RedisOperation::Set, Duration::from_millis(3));

        let families = metrics.registry.gather();
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&families, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("get"), "Should contain 'get' operation");
        assert!(output.contains("set"), "Should contain 'set' operation");
    }

    #[test]
    fn test_record_error() {
        let metrics = RedisMetrics::new();
        metrics.record_error(RedisOperation::Get);
        metrics.record_error(RedisOperation::Eval);
        metrics.record_error(RedisOperation::Eval);

        let families = metrics.registry.gather();
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&families, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("get"), "Should contain 'get' label");
        assert!(output.contains("eval"), "Should contain 'eval' label");
    }

    #[test]
    fn test_pool_size_gauge() {
        let metrics = RedisMetrics::new();
        let gauge = metrics.get_pool_size_gauge();
        gauge.set(16);
        assert_eq!(gauge.get(), 16);
        gauge.dec();
        assert_eq!(gauge.get(), 15);
    }

    #[test]
    fn test_operation_labels() {
        assert_eq!(RedisOperation::Get.as_str(), "get");
        assert_eq!(RedisOperation::Set.as_str(), "set");
        assert_eq!(RedisOperation::Decr.as_str(), "decr");
        assert_eq!(RedisOperation::Eval.as_str(), "eval");
        assert_eq!(RedisOperation::Ping.as_str(), "ping");
    }
}
