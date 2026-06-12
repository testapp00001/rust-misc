//! # Solution 05: Business Metrics
//!
//! Complete implementation of business-level metrics for flash sale monitoring.

use prometheus::{Counter, Gauge, IntCounterVec, IntGauge, Opts, Registry};

/// Business metrics for flash sale monitoring.
pub struct BusinessMetrics {
    pub products_sold: IntCounterVec,
    pub revenue: Counter,
    pub active_users: IntGauge,
    pub conversion_rate: Gauge,
    pub registry: Registry,
}

impl BusinessMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        let products_sold = IntCounterVec::new(
            Opts::new(
                "flash_sale_products_sold_total",
                "Total products sold by product ID",
            ),
            &["product_id"],
        )
        .expect("failed to create products_sold counter");

        let revenue = Counter::with_opts(Opts::new(
            "flash_sale_revenue_total",
            "Total revenue from flash sales",
        ))
        .expect("failed to create revenue counter");

        let active_users = IntGauge::new(
            "flash_sale_active_users",
            "Number of currently active users",
        )
        .expect("failed to create active_users gauge");

        let conversion_rate = Gauge::with_opts(Opts::new(
            "flash_sale_conversion_rate",
            "Current conversion rate (0.0 to 1.0)",
        ))
        .expect("failed to create conversion_rate gauge");

        registry
            .register(Box::new(products_sold.clone()))
            .expect("failed to register products_sold");
        registry
            .register(Box::new(revenue.clone()))
            .expect("failed to register revenue");
        registry
            .register(Box::new(active_users.clone()))
            .expect("failed to register active_users");
        registry
            .register(Box::new(conversion_rate.clone()))
            .expect("failed to register conversion_rate");

        BusinessMetrics {
            products_sold,
            revenue,
            active_users,
            conversion_rate,
            registry,
        }
    }

    pub fn record_sale(&self, product_id: &str, amount: f64) {
        self.products_sold.with_label_values(&[product_id]).inc();
        self.revenue.inc_by(amount);
    }

    pub fn set_active_users(&self, count: i64) {
        self.active_users.set(count);
    }

    pub fn update_conversion_rate(&self, rate: f64) {
        self.conversion_rate.set(rate);
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
