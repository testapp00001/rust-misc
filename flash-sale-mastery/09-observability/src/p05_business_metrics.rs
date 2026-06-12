//! # Exercise 05: Business Metrics
//!
//! ## Learning Objective
//! Learn how to define business-level metrics that translate technical
//! measurements into business insights. While system metrics tell you "CPU is
//! at 80%," business metrics tell you "we sold 10,000 units in 30 seconds."
//!
//! ## Flash Sale Context
//! Stakeholders care about units sold, revenue, and conversion rates -- not
//! request latency percentiles. Business metrics bridge the gap between
//! engineering dashboards and boardroom presentations.
//!
//! ## Instructions
//! 1. Implement `BusinessMetrics::new()` to register business metrics
//! 2. Implement `record_sale` to track units sold and revenue
//! 3. Implement `set_active_users` to update the active users gauge
//! 4. Implement `update_conversion_rate` to set the conversion rate gauge
//!
//! ## Hints
//! - Use `IntCounterVec` with `product_id` label for products sold
//! - Use `Counter` (not Int) for revenue since it may be fractional
//! - Use `IntGauge` for active_users (single value, no labels)
//! - Use `Gauge` for conversion_rate (fractional percentage)
//! - Be careful about label cardinality: product_id is bounded, user_id is not

use prometheus::{Counter, Gauge, IntCounterVec, IntGauge, Registry};

/// Business metrics for flash sale monitoring.
pub struct BusinessMetrics {
    /// Counter for products sold, labeled by product_id.
    pub products_sold: IntCounterVec,
    /// Counter for total revenue (in currency units, e.g., dollars).
    pub revenue: Counter,
    /// Gauge for currently active users.
    pub active_users: IntGauge,
    /// Gauge for the current conversion rate (0.0 to 1.0).
    pub conversion_rate: Gauge,
    /// The registry holding all metrics.
    pub registry: Registry,
}

impl BusinessMetrics {
    /// Create and register all business metrics.
    ///
    /// Registers:
    /// - `flash_sale_products_sold_total` counter with label `product_id`
    /// - `flash_sale_revenue_total` counter
    /// - `flash_sale_active_users` gauge
    /// - `flash_sale_conversion_rate` gauge
    ///
    /// # Returns
    /// A new `BusinessMetrics` instance.
    pub fn new() -> Self {
        // TODO: Create a Registry
        // TODO: Create IntCounterVec for products_sold with label "product_id"
        // TODO: Create Counter for revenue
        // TODO: Create IntGauge for active_users
        // TODO: Create Gauge for conversion_rate
        // TODO: Register all metrics
        todo!("Implement business metrics creation")
    }

    /// Record a sale: increment products sold and add to revenue.
    ///
    /// # Arguments
    /// * `product_id` - The product that was sold
    /// * `amount` - The revenue amount for this sale
    pub fn record_sale(&self, product_id: &str, amount: f64) {
        // TODO: Increment the products_sold counter for the product_id
        // TODO: Add the amount to the revenue counter
        todo!("Implement sale recording")
    }

    /// Set the number of currently active users.
    ///
    /// # Arguments
    /// * `count` - The number of active users
    pub fn set_active_users(&self, count: i64) {
        // TODO: Set the active_users gauge
        todo!("Implement active users setting")
    }

    /// Update the conversion rate (0.0 to 1.0).
    ///
    /// # Arguments
    /// * `rate` - The conversion rate as a fraction (0.5 = 50%)
    pub fn update_conversion_rate(&self, rate: f64) {
        // TODO: Set the conversion_rate gauge
        todo!("Implement conversion rate update")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Encoder;

    #[test]
    fn test_business_metrics_creation() {
        let metrics = BusinessMetrics::new();
        // Labeled metrics don't appear in gather() until a label combo is accessed.
        metrics.record_sale("test", 1.0);
        metrics.set_active_users(1);
        metrics.update_conversion_rate(0.1);
        let families = metrics.registry.gather();
        assert_eq!(families.len(), 4, "Should have 4 metric families");
    }

    #[test]
    fn test_record_sale() {
        let metrics = BusinessMetrics::new();
        metrics.record_sale("PROD-001", 29.99);
        metrics.record_sale("PROD-001", 29.99);
        metrics.record_sale("PROD-002", 49.99);

        let families = metrics.registry.gather();
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&families, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("PROD-001"), "Should contain product label");
        assert!(output.contains("PROD-002"), "Should contain product label");
    }

    #[test]
    fn test_revenue_accumulates() {
        let metrics = BusinessMetrics::new();
        metrics.record_sale("PROD-001", 10.0);
        metrics.record_sale("PROD-001", 20.0);

        // Revenue counter should reflect total
        let families = metrics.registry.gather();
        let revenue_family = families
            .iter()
            .find(|f| f.get_name() == "flash_sale_revenue_total")
            .expect("Should have revenue metric");
        let sample = revenue_family.get_metric()[0].get_counter();
        assert!((sample.get_value() - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_active_users() {
        let metrics = BusinessMetrics::new();
        metrics.set_active_users(50000);
        let families = metrics.registry.gather();
        let family = families
            .iter()
            .find(|f| f.get_name() == "flash_sale_active_users")
            .expect("Should have active_users metric");
        assert_eq!(family.get_metric()[0].get_gauge().get_value() as i64, 50000);
    }

    #[test]
    fn test_conversion_rate() {
        let metrics = BusinessMetrics::new();
        metrics.update_conversion_rate(0.15);
        let families = metrics.registry.gather();
        let family = families
            .iter()
            .find(|f| f.get_name() == "flash_sale_conversion_rate")
            .expect("Should have conversion_rate metric");
        assert!((family.get_metric()[0].get_gauge().get_value() - 0.15).abs() < 0.001);
    }
}
