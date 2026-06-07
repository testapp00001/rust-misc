//! # Monitoring Setup for Rust Services
//!
//! Production Rust services need comprehensive monitoring: health checks,
//! metrics collection, distributed tracing, and alerting. This module covers
//! building monitoring infrastructure with Prometheus-compatible metrics,
//! health check endpoints, and dashboard configuration.
//!
//! ## Key Components:
//!
//! - **Health Checks**: Liveness and readiness probes
//! **Metrics**: Counters, gauges, histograms (Prometheus format)
//! - **Dashboards**: Grafana dashboard configuration
//! - **Alerting**: Alert rules for critical conditions
//!
//! ## Prometheus Metrics Naming:
//!
//! - Counters: `http_requests_total`, `errors_total`
//! - Gauges: `active_connections`, `memory_usage_bytes`
//! - Histograms: `http_request_duration_seconds`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Health check status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Result of a health check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub component: String,
    pub message: String,
    pub latency_ms: u64,
    pub metadata: HashMap<String, String>,
}

/// A health check that can be registered with the health checker.
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self) -> HealthCheckResult;
}

/// Aggregates multiple health checks into a single status.
pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }

    pub fn register(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }

    /// Run all health checks and return the aggregate status.
    pub fn check_all(&self) -> AggregateHealth {
        let results: Vec<HealthCheckResult> =
            self.checks.iter().map(|c| c.check()).collect();
        AggregateHealth::from_results(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateHealth {
    pub status: HealthStatus,
    pub checks: Vec<HealthCheckResult>,
    pub uptime_secs: u64,
}

impl AggregateHealth {
    pub fn from_results(results: Vec<HealthCheckResult>) -> Self {
        let status = if results.iter().any(|r| r.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else if results.iter().any(|r| r.status == HealthStatus::Degraded) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        Self {
            status,
            checks: results,
            uptime_secs: 0,
        }
    }

    /// Serialize to JSON for the health endpoint response.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get the HTTP status code for the health response.
    pub fn http_status_code(&self) -> u16 {
        match self.status {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200, // Still serving traffic
            HealthStatus::Unhealthy => 503,
        }
    }
}

/// A simple in-memory metrics registry compatible with Prometheus exposition format.
/// This is a teaching implementation; production code would use the `metrics` or
/// `prometheus` crate.
#[derive(Debug)]
pub struct MetricsRegistry {
    counters: HashMap<String, Arc<AtomicCounter>>,
    gauges: HashMap<String, Arc<AtomicGauge>>,
    histograms: HashMap<String, Arc<MutexHistogram>>,
}

#[derive(Debug)]
pub struct AtomicCounter {
    value: AtomicU64,
    labels: HashMap<String, String>,
}

impl AtomicCounter {
    pub fn new(labels: HashMap<String, String>) -> Self {
        Self {
            value: AtomicU64::new(0),
            labels,
        }
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct AtomicGauge {
    value: AtomicI64,
    labels: HashMap<String, String>,
}

impl AtomicGauge {
    pub fn new(labels: HashMap<String, String>) -> Self {
        Self {
            value: AtomicI64::new(0),
            labels,
        }
    }

    pub fn set(&self, v: i64) {
        self.value.store(v, Ordering::Relaxed);
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Simplified histogram using mutex for bucket updates.
#[derive(Debug)]
pub struct MutexHistogram {
    buckets: Vec<(f64, AtomicU64)>, // (upper_bound, count)
    sum: AtomicU64,                  // stored as bits of f64
    count: AtomicU64,
    labels: HashMap<String, String>,
}

impl MutexHistogram {
    pub fn new(buckets: Vec<f64>, labels: HashMap<String, String>) -> Self {
        let mut bucket_pairs: Vec<(f64, AtomicU64)> =
            buckets.into_iter().map(|b| (b, AtomicU64::new(0))).collect();
        bucket_pairs.push((f64::INFINITY, AtomicU64::new(0)));
        Self {
            buckets: bucket_pairs,
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
            labels,
        }
    }

    pub fn observe(&self, value: f64) {
        for (upper, count) in &self.buckets {
            if value <= *upper {
                count.fetch_add(1, Ordering::Relaxed);
            }
        }
        self.count.fetch_add(1, Ordering::Relaxed);
        // Store sum as bit representation for atomic update
        let bits = value.to_bits();
        self.sum.fetch_add(bits, Ordering::Relaxed);
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }

    pub fn register_counter(
        &mut self,
        name: &str,
        labels: HashMap<String, String>,
    ) -> Arc<AtomicCounter> {
        let counter = Arc::new(AtomicCounter::new(labels));
        self.counters.insert(name.to_string(), Arc::clone(&counter));
        counter
    }

    pub fn register_gauge(
        &mut self,
        name: &str,
        labels: HashMap<String, String>,
    ) -> Arc<AtomicGauge> {
        let gauge = Arc::new(AtomicGauge::new(labels));
        self.gauges.insert(name.to_string(), Arc::clone(&gauge));
        gauge
    }

    pub fn register_histogram(
        &mut self,
        name: &str,
        buckets: Vec<f64>,
        labels: HashMap<String, String>,
    ) -> Arc<MutexHistogram> {
        let histogram = Arc::new(MutexHistogram::new(buckets, labels));
        self.histograms.insert(name.to_string(), Arc::clone(&histogram));
        histogram
    }

    /// Export all metrics in Prometheus text exposition format.
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        for (name, counter) in &self.counters {
            output.push_str(&format!("# TYPE {} counter\n", name));
            let labels_str = format_labels(&counter.labels);
            output.push_str(&format!("{}{} {}\n", name, labels_str, counter.get()));
        }

        for (name, gauge) in &self.gauges {
            output.push_str(&format!("# TYPE {} gauge\n", name));
            let labels_str = format_labels(&gauge.labels);
            output.push_str(&format!("{}{} {}\n", name, labels_str, gauge.get()));
        }

        for (name, histogram) in &self.histograms {
            output.push_str(&format!("# TYPE {} histogram\n", name));
            for (upper, count) in &histogram.buckets {
                let le = if *upper == f64::INFINITY {
                    "+Inf".to_string()
                } else {
                    upper.to_string()
                };
                let mut labels = histogram.labels.clone();
                labels.insert("le".to_string(), le);
                output.push_str(&format!(
                    "{}_bucket{} {}\n",
                    name,
                    format_labels(&labels),
                    count.load(Ordering::Relaxed)
                ));
            }
            output.push_str(&format!(
                "{}_count{} {}\n",
                name,
                format_labels(&histogram.labels),
                histogram.count()
            ));
        }

        output
    }
}

fn format_labels(labels: &HashMap<String, String>) -> String {
    if labels.is_empty() {
        return String::new();
    }
    let pairs: Vec<String> = labels
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, v))
        .collect();
    format!("{{{}}}", pairs.join(","))
}

/// Grafana dashboard configuration for a Rust service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaDashboard {
    pub title: String,
    pub panels: Vec<GrafanaPanel>,
    pub refresh_interval: String,
    pub time_range: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaPanel {
    pub title: String,
    pub panel_type: PanelType,
    pub queries: Vec<String>,
    pub position: (u32, u32),  // (row, col)
    pub size: (u32, u32),       // (width, height)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    Graph,
    Gauge,
    Stat,
    Table,
    Heatmap,
}

impl GrafanaDashboard {
    /// Generate a standard Rust service dashboard.
    pub fn standard_service_dashboard(service_name: &str) -> Self {
        Self {
            title: format!("{} - Service Dashboard", service_name),
            panels: vec![
                GrafanaPanel {
                    title: "Request Rate".into(),
                    panel_type: PanelType::Graph,
                    queries: vec![format!(
                        "rate(http_requests_total{{service=\"{}\"}}[5m])", service_name
                    )],
                    position: (0, 0),
                    size: (12, 8),
                },
                GrafanaPanel {
                    title: "Error Rate".into(),
                    panel_type: PanelType::Graph,
                    queries: vec![format!(
                        "rate(http_requests_total{{service=\"{}\",status=~\"5..\"}}[5m])",
                        service_name
                    )],
                    position: (0, 12),
                    size: (12, 8),
                },
                GrafanaPanel {
                    title: "Latency P99".into(),
                    panel_type: PanelType::Graph,
                    queries: vec![format!(
                        "histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{{service=\"{}\"}}[5m]))",
                        service_name
                    )],
                    position: (8, 0),
                    size: (12, 8),
                },
                GrafanaPanel {
                    title: "Active Connections".into(),
                    panel_type: PanelType::Gauge,
                    queries: vec![format!(
                        "active_connections{{service=\"{}\"}}", service_name
                    )],
                    position: (8, 12),
                    size: (6, 8),
                },
                GrafanaPanel {
                    title: "Memory Usage".into(),
                    panel_type: PanelType::Graph,
                    queries: vec![
                        format!("process_resident_memory_bytes{{service=\"{}\"}}", service_name),
                        format!("go_memstats_heap_inuse_bytes{{service=\"{}\"}}", service_name),
                    ],
                    position: (16, 0),
                    size: (12, 8),
                },
            ],
            refresh_interval: "10s".into(),
            time_range: "1h".into(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Prometheus alert rules for a Rust service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub expr: String,
    pub duration: String,
    pub severity: AlertSeverity,
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

impl AlertRule {
    /// Standard alert rules for a Rust service.
    pub fn standard_rules(service_name: &str) -> Vec<Self> {
        vec![
            Self {
                name: format!("{}_high_error_rate", service_name),
                expr: format!(
                    "rate(http_requests_total{{service=\"{}\",status=~\"5..\"}}[5m]) > 0.05",
                    service_name
                ),
                duration: "5m".into(),
                severity: AlertSeverity::Critical,
                annotations: {
                    let mut m = HashMap::new();
                    m.insert("summary".into(), format!("High error rate on {}", service_name));
                    m.insert(
                        "description".into(),
                        "Error rate is above 5% for 5 minutes".into(),
                    );
                    m
                },
            },
            Self {
                name: format!("{}_high_latency", service_name),
                expr: format!(
                    "histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{{service=\"{}\"}}[5m])) > 1.0",
                    service_name
                ),
                duration: "5m".into(),
                severity: AlertSeverity::Warning,
                annotations: {
                    let mut m = HashMap::new();
                    m.insert("summary".into(), format!("High latency on {}", service_name));
                    m.insert(
                        "description".into(),
                        "P99 latency is above 1 second for 5 minutes".into(),
                    );
                    m
                },
            },
            Self {
                name: format!("{}_high_memory", service_name),
                expr: format!(
                    "process_resident_memory_bytes{{service=\"{}\"}} > 1073741824",
                    service_name
                ),
                duration: "10m".into(),
                severity: AlertSeverity::Warning,
                annotations: {
                    let mut m = HashMap::new();
                    m.insert("summary".into(), format!("High memory usage on {}", service_name));
                    m.insert(
                        "description".into(),
                        "Memory usage is above 1 GB for 10 minutes".into(),
                    );
                    m
                },
            },
        ]
    }
}

/// Latency tracker for measuring operation durations.
pub struct LatencyTracker {
    start: Instant,
}

impl LatencyTracker {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Finish tracking and return duration in milliseconds.
    pub fn finish(self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

/// SLI/SLO definitions for a Rust service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliSloDefinition {
    pub sli_name: String,
    pub sli_query: String,
    pub slo_target: f64, // e.g., 0.999 for 99.9%
    pub window: String,  // e.g., "30d"
    pub alert_threshold: f64,
}

impl SliSloDefinition {
    /// Define standard availability SLO.
    pub fn availability(service_name: &str, target: f64) -> Self {
        Self {
            sli_name: "availability".into(),
            sli_query: format!(
                "1 - (rate(http_requests_total{{service=\"{}\",status=~\"5..\"}}[30d]) / rate(http_requests_total{{service=\"{}\"}}[30d]))",
                service_name, service_name
            ),
            slo_target: target,
            window: "30d".into(),
            alert_threshold: target - 0.001,
        }
    }

    /// Define latency SLO.
    pub fn latency(service_name: &str, quantile: f64, target_ms: f64) -> Self {
        Self {
            sli_name: format!("latency_p{}", (quantile * 100.0) as u32),
            sli_query: format!(
                "histogram_quantile({}, rate(http_request_duration_seconds_bucket{{service=\"{}\"}}[30d]))",
                quantile, service_name
            ),
            slo_target: target_ms / 1000.0, // convert to seconds
            window: "30d".into(),
            alert_threshold: target_ms / 1000.0 * 1.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_checker_healthy() {
        struct TestCheck;
        impl HealthCheck for TestCheck {
            fn name(&self) -> &str { "test" }
            fn check(&self) -> HealthCheckResult {
                HealthCheckResult {
                    status: HealthStatus::Healthy,
                    component: "test".into(),
                    message: "OK".into(),
                    latency_ms: 5,
                    metadata: HashMap::new(),
                }
            }
        }

        let mut checker = HealthChecker::new();
        checker.register(Box::new(TestCheck));
        let result = checker.check_all();
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.http_status_code(), 200);
    }

    #[test]
    fn test_health_checker_unhealthy() {
        struct BadCheck;
        impl HealthCheck for BadCheck {
            fn name(&self) -> &str { "bad" }
            fn check(&self) -> HealthCheckResult {
                HealthCheckResult {
                    status: HealthStatus::Unhealthy,
                    component: "bad".into(),
                    message: "Connection refused".into(),
                    latency_ms: 5000,
                    metadata: HashMap::new(),
                }
            }
        }

        let mut checker = HealthChecker::new();
        checker.register(Box::new(BadCheck));
        let result = checker.check_all();
        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert_eq!(result.http_status_code(), 503);
    }

    #[test]
    fn test_health_checker_degraded() {
        struct OkCheck;
        impl HealthCheck for OkCheck {
            fn name(&self) -> &str { "ok" }
            fn check(&self) -> HealthCheckResult {
                HealthCheckResult {
                    status: HealthStatus::Healthy,
                    component: "ok".into(),
                    message: "OK".into(),
                    latency_ms: 5,
                    metadata: HashMap::new(),
                }
            }
        }

        struct DegradedCheck;
        impl HealthCheck for DegradedCheck {
            fn name(&self) -> &str { "degraded" }
            fn check(&self) -> HealthCheckResult {
                HealthCheckResult {
                    status: HealthStatus::Degraded,
                    component: "cache".into(),
                    message: "Slow response".into(),
                    latency_ms: 500,
                    metadata: HashMap::new(),
                }
            }
        }

        let mut checker = HealthChecker::new();
        checker.register(Box::new(OkCheck));
        checker.register(Box::new(DegradedCheck));
        let result = checker.check_all();
        assert_eq!(result.status, HealthStatus::Degraded);
        assert_eq!(result.checks.len(), 2);
    }

    #[test]
    fn test_health_check_json() {
        let result = AggregateHealth {
            status: HealthStatus::Healthy,
            checks: vec![],
            uptime_secs: 3600,
        };
        let json = result.to_json();
        assert!(json.contains("Healthy"));
        assert!(json.contains("3600"));
    }

    #[test]
    fn test_metrics_counter() {
        let counter = AtomicCounter::new(HashMap::new());
        assert_eq!(counter.get(), 0);
        counter.inc();
        assert_eq!(counter.get(), 1);
        counter.add(10);
        assert_eq!(counter.get(), 11);
    }

    #[test]
    fn test_metrics_gauge() {
        let gauge = AtomicGauge::new(HashMap::new());
        assert_eq!(gauge.get(), 0);
        gauge.set(42);
        assert_eq!(gauge.get(), 42);
        gauge.inc();
        assert_eq!(gauge.get(), 43);
        gauge.dec();
        assert_eq!(gauge.get(), 42);
    }

    #[test]
    fn test_metrics_histogram() {
        let histogram =
            MutexHistogram::new(vec![0.1, 0.5, 1.0, 5.0], HashMap::new());
        histogram.observe(0.05);
        histogram.observe(0.3);
        histogram.observe(0.8);
        histogram.observe(2.0);
        histogram.observe(10.0);

        assert_eq!(histogram.count(), 5);
    }

    #[test]
    fn test_metrics_registry_export() {
        let mut registry = MetricsRegistry::new();
        let counter = registry.register_counter(
            "http_requests_total",
            [("method".into(), "GET".into())].into(),
        );
        counter.inc();
        counter.inc();
        counter.inc();

        let gauge = registry.register_gauge(
            "active_connections",
            HashMap::new(),
        );
        gauge.set(5);

        let output = registry.export_prometheus();
        assert!(output.contains("# TYPE http_requests_total counter"));
        assert!(output.contains("http_requests_total{method=\"GET\"} 3"));
        assert!(output.contains("# TYPE active_connections gauge"));
        assert!(output.contains("active_connections 5"));
    }

    #[test]
    fn test_grafana_dashboard() {
        let dashboard = GrafanaDashboard::standard_service_dashboard("my-api");
        assert!(dashboard.title.contains("my-api"));
        assert!(!dashboard.panels.is_empty());
        assert_eq!(dashboard.refresh_interval, "10s");

        let json = dashboard.to_json();
        assert!(json.contains("Request Rate"));
        assert!(json.contains("Error Rate"));
    }

    #[test]
    fn test_alert_rules() {
        let rules = AlertRule::standard_rules("my-api");
        assert!(!rules.is_empty());

        let error_rule = rules.iter().find(|r| r.name.contains("error")).unwrap();
        assert!(error_rule.expr.contains("5.."));
        assert!(matches!(error_rule.severity, AlertSeverity::Critical));
    }

    #[test]
    fn test_latency_tracker() {
        let tracker = LatencyTracker::start();
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ms = tracker.finish();
        assert!(ms >= 8.0); // Allow some tolerance
    }

    #[test]
    fn test_slo_availability() {
        let slo = SliSloDefinition::availability("my-api", 0.999);
        assert_eq!(slo.slo_target, 0.999);
        assert!(slo.sli_query.contains("5.."));
        assert!(slo.alert_threshold < slo.slo_target);
    }

    #[test]
    fn test_slo_latency() {
        let slo = SliSloDefinition::latency("my-api", 0.99, 500.0);
        assert_eq!(slo.sli_name, "latency_p99");
        assert!((slo.slo_target - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_format_labels() {
        let labels: HashMap<String, String> = [
            ("method".into(), "GET".into()),
            ("path".into(), "/api".into()),
        ]
        .into();
        let formatted = format_labels(&labels);
        assert!(formatted.starts_with('{'));
        assert!(formatted.ends_with('}'));
        assert!(formatted.contains("method=\"GET\""));
        assert!(formatted.contains("path=\"/api\""));
    }

    #[test]
    fn test_format_labels_empty() {
        let labels = HashMap::new();
        assert_eq!(format_labels(&labels), "");
    }

    #[test]
    fn test_histogram_prometheus_export() {
        let mut registry = MetricsRegistry::new();
        let histogram = registry.register_histogram(
            "http_request_duration_seconds",
            vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0],
            HashMap::new(),
        );
        histogram.observe(0.005);
        histogram.observe(0.03);
        histogram.observe(0.08);

        let output = registry.export_prometheus();
        assert!(output.contains("http_request_duration_seconds_bucket{le=\"0.01\"}"));
        assert!(output.contains("http_request_duration_seconds_bucket{le=\"+Inf\"}"));
        assert!(output.contains("http_request_duration_seconds_count 3"));
    }
}
