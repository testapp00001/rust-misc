//! # Lesson 8: Metrics Integration
//!
//! Metrics complement logging by providing aggregated numerical data.
//! This lesson covers counters, gauges, histograms, and how metrics
//! integrate with the tracing ecosystem.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Metric types
// ---------------------------------------------------------------------------

/// A counter that only goes up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counter {
    pub name: String,
    pub value: u64,
    pub labels: BTreeMap<String, String>,
}

impl Counter {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: 0,
            labels: BTreeMap::new(),
        }
    }

    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn add(&mut self, amount: u64) {
        self.value += amount;
    }

    pub fn get(&self) -> u64 {
        self.value
    }
}

/// A gauge that can go up and down.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gauge {
    pub name: String,
    pub value: f64,
    pub labels: BTreeMap<String, String>,
}

impl Gauge {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: 0.0,
            labels: BTreeMap::new(),
        }
    }

    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    pub fn set(&mut self, value: f64) {
        self.value = value;
    }

    pub fn increment(&mut self) {
        self.value += 1.0;
    }

    pub fn decrement(&mut self) {
        self.value -= 1.0;
    }

    pub fn add(&mut self, amount: f64) {
        self.value += amount;
    }

    pub fn get(&self) -> f64 {
        self.value
    }
}

/// A histogram for tracking distributions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    pub name: String,
    pub values: Vec<f64>,
    pub labels: BTreeMap<String, String>,
}

impl Histogram {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            values: Vec::new(),
            labels: BTreeMap::new(),
        }
    }

    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    pub fn record(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn sum(&self) -> f64 {
        self.values.iter().sum()
    }

    pub fn mean(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.sum() / self.values.len() as f64
        }
    }

    pub fn min(&self) -> f64 {
        self.values
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min)
    }

    pub fn max(&self) -> f64 {
        self.values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn percentile(&self, p: f64) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        let mut sorted = self.values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((p / 100.0) * (sorted.len() - 1) as f64) as usize;
        let idx = idx.min(sorted.len() - 1);
        sorted[idx]
    }
}

// ---------------------------------------------------------------------------
// Metrics registry
// ---------------------------------------------------------------------------

/// A registry that holds all metrics.
pub struct MetricsRegistry {
    counters: BTreeMap<String, Counter>,
    gauges: BTreeMap<String, Gauge>,
    histograms: BTreeMap<String, Histogram>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            counters: BTreeMap::new(),
            gauges: BTreeMap::new(),
            histograms: BTreeMap::new(),
        }
    }

    pub fn register_counter(&mut self, counter: Counter) {
        self.counters.insert(counter.name.clone(), counter);
    }

    pub fn register_gauge(&mut self, gauge: Gauge) {
        self.gauges.insert(gauge.name.clone(), gauge);
    }

    pub fn register_histogram(&mut self, histogram: Histogram) {
        self.histograms.insert(histogram.name.clone(), histogram);
    }

    pub fn counter(&mut self, name: &str) -> Option<&mut Counter> {
        self.counters.get_mut(name)
    }

    pub fn gauge(&mut self, name: &str) -> Option<&mut Gauge> {
        self.gauges.get_mut(name)
    }

    pub fn histogram(&mut self, name: &str) -> Option<&mut Histogram> {
        self.histograms.get_mut(name)
    }

    /// Export all metrics in a Prometheus-compatible format.
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        for counter in self.counters.values() {
            output.push_str(&format!("# TYPE {} counter\n", counter.name));
            if counter.labels.is_empty() {
                output.push_str(&format!("{} {}\n", counter.name, counter.value));
            } else {
                let labels = format_labels(&counter.labels);
                output.push_str(&format!("{}{{{}}} {}\n", counter.name, labels, counter.value));
            }
        }

        for gauge in self.gauges.values() {
            output.push_str(&format!("# TYPE {} gauge\n", gauge.name));
            if gauge.labels.is_empty() {
                output.push_str(&format!("{} {}\n", gauge.name, gauge.value));
            } else {
                let labels = format_labels(&gauge.labels);
                output.push_str(&format!("{}{{{}}} {}\n", gauge.name, labels, gauge.value));
            }
        }

        for histogram in self.histograms.values() {
            output.push_str(&format!("# TYPE {} histogram\n", histogram.name));
            output.push_str(&format!("{}_count {}\n", histogram.name, histogram.count()));
            output.push_str(&format!("{}_sum {}\n", histogram.name, histogram.sum()));
        }

        output
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn format_labels(labels: &BTreeMap<String, String>) -> String {
    labels
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, v))
        .collect::<Vec<_>>()
        .join(",")
}

// ---------------------------------------------------------------------------
// Metrics middleware pattern
// ---------------------------------------------------------------------------

/// Collects request metrics.
pub struct RequestMetrics {
    pub total_requests: Counter,
    pub active_requests: Gauge,
    pub request_duration: Histogram,
    pub errors: Counter,
}

impl RequestMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: Counter::new("http_requests_total"),
            active_requests: Gauge::new("http_active_requests"),
            request_duration: Histogram::new("http_request_duration_ms"),
            errors: Counter::new("http_errors_total"),
        }
    }

    pub fn record_request(&mut self, duration: Duration, status: u16) {
        self.total_requests.increment();
        self.request_duration
            .record(duration.as_secs_f64() * 1000.0);
        if status >= 400 {
            self.errors.increment();
        }
    }

    pub fn start_request(&mut self) {
        self.active_requests.increment();
    }

    pub fn end_request(&mut self) {
        self.active_requests.decrement();
    }
}

impl Default for RequestMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let mut counter = Counter::new("requests");
        assert_eq!(counter.get(), 0);

        counter.increment();
        assert_eq!(counter.get(), 1);

        counter.add(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_counter_with_labels() {
        let counter = Counter::new("requests")
            .with_label("method", "GET")
            .with_label("status", "200");
        assert_eq!(counter.labels.len(), 2);
    }

    #[test]
    fn test_gauge() {
        let mut gauge = Gauge::new("connections");
        assert_eq!(gauge.get(), 0.0);

        gauge.increment();
        assert_eq!(gauge.get(), 1.0);

        gauge.decrement();
        assert_eq!(gauge.get(), 0.0);

        gauge.set(42.5);
        assert_eq!(gauge.get(), 42.5);
    }

    #[test]
    fn test_gauge_add() {
        let mut gauge = Gauge::new("test");
        gauge.add(10.5);
        gauge.add(5.5);
        assert_eq!(gauge.get(), 16.0);
    }

    #[test]
    fn test_histogram_basic() {
        let mut hist = Histogram::new("latency");
        hist.record(10.0);
        hist.record(20.0);
        hist.record(30.0);

        assert_eq!(hist.count(), 3);
        assert_eq!(hist.sum(), 60.0);
        assert_eq!(hist.mean(), 20.0);
        assert_eq!(hist.min(), 10.0);
        assert_eq!(hist.max(), 30.0);
    }

    #[test]
    fn test_histogram_empty() {
        let hist = Histogram::new("empty");
        assert_eq!(hist.count(), 0);
        assert_eq!(hist.mean(), 0.0);
    }

    #[test]
    fn test_histogram_percentile() {
        let mut hist = Histogram::new("test");
        for i in 1..=100 {
            hist.record(i as f64);
        }
        assert_eq!(hist.percentile(50.0), 50.0);
        assert_eq!(hist.percentile(95.0), 95.0);
        assert_eq!(hist.percentile(99.0), 99.0);
    }

    #[test]
    fn test_metrics_registry() {
        let mut registry = MetricsRegistry::new();
        registry.register_counter(Counter::new("requests"));
        registry.register_gauge(Gauge::new("connections"));
        registry.register_histogram(Histogram::new("latency"));

        registry.counter("requests").unwrap().increment();
        registry.gauge("connections").unwrap().set(5.0);
        registry.histogram("latency").unwrap().record(100.0);

        assert_eq!(registry.counter("requests").unwrap().get(), 1);
        assert_eq!(registry.gauge("connections").unwrap().get(), 5.0);
        assert_eq!(registry.histogram("latency").unwrap().count(), 1);
    }

    #[test]
    fn test_prometheus_export() {
        let mut registry = MetricsRegistry::new();
        registry.register_counter(
            Counter::new("requests").with_label("method", "GET"),
        );
        registry.register_gauge(Gauge::new("connections"));
        registry.register_histogram(Histogram::new("latency"));

        registry.counter("requests").unwrap().increment();
        registry.gauge("connections").unwrap().set(5.0);
        registry.histogram("latency").unwrap().record(100.0);

        let output = registry.export_prometheus();
        assert!(output.contains("# TYPE requests counter"));
        assert!(output.contains("requests{method=\"GET\"} 1"));
        assert!(output.contains("# TYPE connections gauge"));
        assert!(output.contains("connections 5"));
        assert!(output.contains("# TYPE latency histogram"));
        assert!(output.contains("latency_count 1"));
    }

    #[test]
    fn test_request_metrics() {
        let mut metrics = RequestMetrics::new();

        metrics.start_request();
        metrics.record_request(Duration::from_millis(100), 200);
        metrics.end_request();

        metrics.start_request();
        metrics.record_request(Duration::from_millis(200), 500);
        metrics.end_request();

        assert_eq!(metrics.total_requests.get(), 2);
        assert_eq!(metrics.errors.get(), 1);
        assert_eq!(metrics.active_requests.get(), 0.0);
        assert_eq!(metrics.request_duration.count(), 2);
    }

    #[test]
    fn test_request_metrics_active() {
        let mut metrics = RequestMetrics::new();
        metrics.start_request();
        metrics.start_request();
        assert_eq!(metrics.active_requests.get(), 2.0);
        metrics.end_request();
        assert_eq!(metrics.active_requests.get(), 1.0);
    }

    #[test]
    fn test_format_labels() {
        let mut labels = BTreeMap::new();
        labels.insert("method".to_string(), "GET".to_string());
        labels.insert("path".to_string(), "/api".to_string());
        let formatted = format_labels(&labels);
        assert!(formatted.contains("method=\"GET\""));
        assert!(formatted.contains("path=\"/api\""));
    }

    #[test]
    fn test_histogram_with_labels() {
        let hist = Histogram::new("test").with_label("endpoint", "/api");
        assert_eq!(hist.labels.len(), 1);
    }
}
