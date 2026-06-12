//! # Solution 03: Custom Metrics
//!
//! Complete implementation of Prometheus metrics for flash sale operations.

use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, IntGaugeVec, Opts, Registry};
use std::time::Duration;

/// The result of a purchase attempt.
#[derive(Debug, Clone, Copy)]
pub enum PurchaseResult {
    Success,
    SoldOut,
    AlreadyClaimed,
    Error,
}

impl PurchaseResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            PurchaseResult::Success => "success",
            PurchaseResult::SoldOut => "sold_out",
            PurchaseResult::AlreadyClaimed => "already_claimed",
            PurchaseResult::Error => "error",
        }
    }
}

/// Prometheus metrics for flash sale operations.
pub struct FlashSaleMetrics {
    pub purchase_attempts: IntCounterVec,
    pub stock_level: IntGaugeVec,
    pub request_latency: HistogramVec,
    pub registry: Registry,
}

impl FlashSaleMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        let purchase_attempts = IntCounterVec::new(
            Opts::new(
                "flash_sale_purchase_attempts_total",
                "Total number of purchase attempts",
            ),
            &["result"],
        )
        .expect("failed to create purchase_attempts counter");

        let stock_level = IntGaugeVec::new(
            Opts::new(
                "flash_sale_stock_level",
                "Current stock level per product",
            ),
            &["product_id"],
        )
        .expect("failed to create stock_level gauge");

        let request_latency = HistogramVec::new(
            HistogramOpts::new(
                "flash_sale_request_latency_seconds",
                "Request latency in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5]),
            &["operation"],
        )
        .expect("failed to create request_latency histogram");

        registry
            .register(Box::new(purchase_attempts.clone()))
            .expect("failed to register purchase_attempts");
        registry
            .register(Box::new(stock_level.clone()))
            .expect("failed to register stock_level");
        registry
            .register(Box::new(request_latency.clone()))
            .expect("failed to register request_latency");

        FlashSaleMetrics {
            purchase_attempts,
            stock_level,
            request_latency,
            registry,
        }
    }

    pub fn record_purchase(&self, result: PurchaseResult) {
        self.purchase_attempts
            .with_label_values(&[result.as_str()])
            .inc();
    }

    pub fn record_latency(&self, operation: &str, duration: Duration) {
        self.request_latency
            .with_label_values(&[operation])
            .observe(duration.as_secs_f64());
    }

    pub fn set_stock_level(&self, product_id: &str, level: i64) {
        self.stock_level
            .with_label_values(&[product_id])
            .set(level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Encoder;

    #[test]
    fn test_metrics_creation() {
        let metrics = FlashSaleMetrics::new();
        // Labeled metrics don't appear in gather() until a label combo is accessed,
        // so touch each metric to ensure it's registered and visible.
        metrics.record_purchase(PurchaseResult::Success);
        metrics.record_latency("test", Duration::from_millis(1));
        metrics.set_stock_level("test", 1);
        let metric_families = metrics.registry.gather();
        assert_eq!(metric_families.len(), 3, "Should have 3 registered metric families");
    }

    #[test]
    fn test_record_purchase_increments_counter() {
        let metrics = FlashSaleMetrics::new();
        metrics.record_purchase(PurchaseResult::Success);
        metrics.record_purchase(PurchaseResult::Success);
        metrics.record_purchase(PurchaseResult::SoldOut);

        let metric_families = metrics.registry.gather();
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("success"), "Should contain 'success' label");
        assert!(output.contains("sold_out"), "Should contain 'sold_out' label");
    }

    #[test]
    fn test_record_latency() {
        let metrics = FlashSaleMetrics::new();
        metrics.record_latency("purchase", Duration::from_millis(50));
        metrics.record_latency("purchase", Duration::from_millis(150));
        metrics.record_latency("stock_check", Duration::from_millis(10));

        let metric_families = metrics.registry.gather();
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("purchase"), "Should contain 'purchase' operation");
        assert!(output.contains("stock_check"), "Should contain 'stock_check' operation");
    }

    #[test]
    fn test_set_stock_level() {
        let metrics = FlashSaleMetrics::new();
        metrics.set_stock_level("PROD-001", 100);
        metrics.set_stock_level("PROD-002", 0);

        let metric_families = metrics.registry.gather();
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("PROD-001"), "Should contain product ID");
    }

    #[test]
    fn test_purchase_result_labels() {
        assert_eq!(PurchaseResult::Success.as_str(), "success");
        assert_eq!(PurchaseResult::SoldOut.as_str(), "sold_out");
        assert_eq!(PurchaseResult::AlreadyClaimed.as_str(), "already_claimed");
        assert_eq!(PurchaseResult::Error.as_str(), "error");
    }
}
