//! # Solution 04: Redis Metrics
//!
//! Complete implementation of Redis-specific performance metrics.

use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, Registry};
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
    pub operation_latency: HistogramVec,
    pub errors: IntCounterVec,
    pub pool_size: IntGauge,
    pub registry: Registry,
}

impl RedisMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        let operation_latency = HistogramVec::new(
            HistogramOpts::new(
                "redis_operation_latency_seconds",
                "Redis operation latency in seconds",
            )
            .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5]),
            &["operation"],
        )
        .expect("failed to create operation_latency histogram");

        let errors = IntCounterVec::new(
            Opts::new("redis_errors_total", "Total Redis errors by operation"),
            &["operation"],
        )
        .expect("failed to create errors counter");

        let pool_size = IntGauge::new(
            "redis_connection_pool_size",
            "Current Redis connection pool size",
        )
        .expect("failed to create pool_size gauge");

        registry
            .register(Box::new(operation_latency.clone()))
            .expect("failed to register operation_latency");
        registry
            .register(Box::new(errors.clone()))
            .expect("failed to register errors");
        registry
            .register(Box::new(pool_size.clone()))
            .expect("failed to register pool_size");

        RedisMetrics {
            operation_latency,
            errors,
            pool_size,
            registry,
        }
    }

    pub fn record_operation(&self, op: RedisOperation, duration: Duration) {
        self.operation_latency
            .with_label_values(&[op.as_str()])
            .observe(duration.as_secs_f64());
    }

    pub fn record_error(&self, op: RedisOperation) {
        self.errors.with_label_values(&[op.as_str()]).inc();
    }

    pub fn get_pool_size_gauge(&self) -> &IntGauge {
        &self.pool_size
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
