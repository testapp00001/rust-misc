//! # Exercise 03: Custom Metrics
//!
//! ## Learning Objective
//! Learn how to define and record Prometheus metrics for flash sale operations.
//! Metrics provide real-time numerical insight into system behavior, enabling
//! dashboards, alerts, and capacity planning.
//!
//! ## Flash Sale Context
//! During a flash sale you need to track: how many purchase attempts succeeded
//! vs. failed, current stock levels, and request latency distribution. These
//! metrics feed your Grafana dashboards and PagerDuty alerts.
//!
//! ## Instructions
//! 1. Implement `FlashSaleMetrics::new()` to register all metrics
//! 2. Implement `record_purchase` to increment the purchase_attempts counter
//! 3. Implement `record_latency` to observe request latency
//! 4. Implement `set_stock_level` to update the stock gauge
//!
//! ## Hints
//! - Use `prometheus::IntCounterVec` for counters with labels
//! - Use `prometheus::IntGaugeVec` for gauges with labels
//! - Use `prometheus::HistogramVec` for histograms with labels
//! - `prometheus::Registry::new()` creates a new metric registry

use prometheus::{HistogramVec, IntCounterVec, IntGaugeVec, Registry};
use std::time::Duration;

/// The result of a purchase attempt.
#[derive(Debug, Clone, Copy)]
pub enum PurchaseResult {
    /// Purchase completed successfully.
    Success,
    /// Product is out of stock.
    SoldOut,
    /// User already claimed this product.
    AlreadyClaimed,
    /// An error occurred during processing.
    Error,
}

impl PurchaseResult {
    /// Return the label string for this result.
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
    /// Counter for purchase attempts, labeled by result.
    pub purchase_attempts: IntCounterVec,
    /// Gauge for current stock levels, labeled by product_id.
    pub stock_level: IntGaugeVec,
    /// Histogram for request latency in seconds.
    pub request_latency: HistogramVec,
    /// The registry holding all metrics.
    pub registry: Registry,
}

impl FlashSaleMetrics {
    /// Create and register all flash sale metrics.
    ///
    /// Registers:
    /// - `flash_sale_purchase_attempts_total` counter with label `result`
    /// - `flash_sale_stock_level` gauge with label `product_id`
    /// - `flash_sale_request_latency_seconds` histogram with label `operation`
    ///
    /// # Returns
    /// A new `FlashSaleMetrics` instance with all metrics registered.
    pub fn new() -> Self {
        // TODO: Create a Registry
        // TODO: Create IntCounterVec for purchase_attempts with label "result"
        // TODO: Create IntGaugeVec for stock_level with label "product_id"
        // TODO: Create HistogramVec for request_latency with label "operation"
        // TODO: Register all metrics with the registry
        todo!("Implement metrics creation and registration")
    }

    /// Record a purchase attempt with the given result.
    ///
    /// # Arguments
    /// * `result` - The outcome of the purchase attempt
    pub fn record_purchase(&self, result: PurchaseResult) {
        // TODO: Increment the purchase_attempts counter with the result label
        todo!("Implement purchase recording")
    }

    /// Record a request latency observation.
    ///
    /// # Arguments
    /// * `operation` - The operation that was timed
    /// * `duration` - How long the operation took
    pub fn record_latency(&self, operation: &str, duration: Duration) {
        // TODO: Observe the duration on the request_latency histogram
        todo!("Implement latency recording")
    }

    /// Set the current stock level for a product.
    ///
    /// # Arguments
    /// * `product_id` - The product identifier
    /// * `level` - The current stock level
    pub fn set_stock_level(&self, product_id: &str, level: i64) {
        // TODO: Set the stock_level gauge for the given product_id
        todo!("Implement stock level setting")
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
        assert_eq!(
            metric_families.len(),
            3,
            "Should have 3 registered metric families"
        );
    }

    #[test]
    fn test_record_purchase_increments_counter() {
        let metrics = FlashSaleMetrics::new();
        metrics.record_purchase(PurchaseResult::Success);
        metrics.record_purchase(PurchaseResult::Success);
        metrics.record_purchase(PurchaseResult::SoldOut);

        // Gather and encode metrics to verify values
        let metric_families = metrics.registry.gather();
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(
            output.contains("success"),
            "Should contain 'success' label"
        );
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

        assert!(
            output.contains("purchase"),
            "Should contain 'purchase' operation"
        );
        assert!(
            output.contains("stock_check"),
            "Should contain 'stock_check' operation"
        );
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

        assert!(
            output.contains("PROD-001"),
            "Should contain product ID"
        );
    }

    #[test]
    fn test_purchase_result_labels() {
        assert_eq!(PurchaseResult::Success.as_str(), "success");
        assert_eq!(PurchaseResult::SoldOut.as_str(), "sold_out");
        assert_eq!(PurchaseResult::AlreadyClaimed.as_str(), "already_claimed");
        assert_eq!(PurchaseResult::Error.as_str(), "error");
    }
}
