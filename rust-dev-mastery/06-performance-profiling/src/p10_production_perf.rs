//! # Production Performance Monitoring
//!
//! In production, you need continuous performance monitoring. This module covers
//! latency tracking, throughput measurement, percentile computation, and
//! regression detection.
//!
//! ## Key Concepts
//! - **Latency percentiles**: P50, P95, P99 tell you more than averages
//! - **Throughput metrics**: Operations per second over time windows
//! - **Regression detection**: Automatically flagging performance degradation
//! - **Histogram-based tracking**: Efficient percentile computation with histograms

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A high-performance histogram for tracking latency distributions.
/// Uses logarithmic bucketing for good resolution across orders of magnitude.
pub struct LatencyHistogram {
    buckets: Vec<u64>,
    bucket_boundaries: Vec<f64>, // In microseconds
    total_count: u64,
    total_sum_us: f64,
}

impl LatencyHistogram {
    /// Creates a histogram with buckets from 1us to ~10s.
    pub fn new() -> Self {
        // Logarithmic buckets: 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, ...
        let mut boundaries = Vec::new();
        let bases = [1.0, 2.0, 5.0];
        let mut exp = 0.0;
        while exp <= 7.0 {
            // Up to ~10 seconds
            for &base in &bases {
                let value = base * 10f64.powf(exp);
                if value <= 10_000_000.0 {
                    // 10 seconds in microseconds
                    boundaries.push(value);
                }
            }
            exp += 1.0;
        }
        boundaries.sort_by(|a, b| a.partial_cmp(b).unwrap());
        boundaries.dedup_by(|a, b| (*a - *b).abs() < 0.001);

        LatencyHistogram {
            buckets: vec![0; boundaries.len() + 1],
            bucket_boundaries: boundaries,
            total_count: 0,
            total_sum_us: 0.0,
        }
    }

    /// Records a latency measurement.
    pub fn record(&mut self, duration: Duration) {
        let us = duration.as_secs_f64() * 1_000_000.0;
        self.total_count += 1;
        self.total_sum_us += us;

        let bucket = self.bucket_boundaries.partition_point(|&b| b < us);
        self.buckets[bucket] += 1;
    }

    /// Computes a percentile (0.0 to 1.0).
    pub fn percentile(&self, p: f64) -> Duration {
        assert!((0.0..=1.0).contains(&p));
        let target = (self.total_count as f64 * p) as u64;
        let mut cumulative = 0u64;

        for (i, &count) in self.buckets.iter().enumerate() {
            cumulative += count;
            if cumulative >= target {
                let us = if i < self.bucket_boundaries.len() {
                    self.bucket_boundaries[i]
                } else {
                    self.bucket_boundaries.last().copied().unwrap_or(0.0) * 2.0
                };
                return Duration::from_micros(us as u64);
            }
        }

        Duration::ZERO
    }

    pub fn count(&self) -> u64 {
        self.total_count
    }

    pub fn mean(&self) -> Duration {
        if self.total_count == 0 {
            return Duration::ZERO;
        }
        Duration::from_micros((self.total_sum_us / self.total_count as f64) as u64)
    }

    pub fn report(&self) -> HistogramReport {
        HistogramReport {
            count: self.total_count,
            mean: self.mean(),
            p50: self.percentile(0.5),
            p90: self.percentile(0.9),
            p95: self.percentile(0.95),
            p99: self.percentile(0.99),
            p999: self.percentile(0.999),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HistogramReport {
    pub count: u64,
    pub mean: Duration,
    pub p50: Duration,
    pub p90: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub p999: Duration,
}

impl std::fmt::Display for HistogramReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Latency Report ({} samples):", self.count)?;
        writeln!(f, "  Mean:  {:?}", self.mean)?;
        writeln!(f, "  P50:   {:?}", self.p50)?;
        writeln!(f, "  P90:   {:?}", self.p90)?;
        writeln!(f, "  P95:   {:?}", self.p95)?;
        writeln!(f, "  P99:   {:?}", self.p99)?;
        writeln!(f, "  P99.9: {:?}", self.p999)?;
        Ok(())
    }
}

/// Tracks throughput over sliding time windows.
pub struct ThroughputTracker {
    windows: Vec<TimeWindow>,
    window_duration: Duration,
}

struct TimeWindow {
    start: Instant,
    count: u64,
}

impl ThroughputTracker {
    pub fn new(window_duration: Duration) -> Self {
        ThroughputTracker {
            windows: Vec::new(),
            window_duration,
        }
    }

    pub fn record(&mut self) {
        let now = Instant::now();

        // Clean up expired windows
        self.windows
            .retain(|w| now - w.start < self.window_duration * 3);

        if self.windows.is_empty() || now - self.windows.last().unwrap().start >= self.window_duration {
            self.windows.push(TimeWindow {
                start: now,
                count: 1,
            });
        } else {
            self.windows.last_mut().unwrap().count += 1;
        }
    }

    /// Returns the throughput (ops/sec) for the most recent window.
    pub fn current_throughput(&self) -> f64 {
        self.windows.last().map_or(0.0, |w| {
            let elapsed = Instant::now() - w.start;
            let elapsed_secs = elapsed.as_secs_f64();
            if elapsed_secs > 0.0 {
                w.count as f64 / elapsed_secs
            } else {
                0.0
            }
        })
    }
}

/// A named metrics collector for tracking multiple metrics.
pub struct MetricsCollector {
    histograms: HashMap<String, LatencyHistogram>,
    counters: HashMap<String, u64>,
    gauges: HashMap<String, f64>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        MetricsCollector {
            histograms: HashMap::new(),
            counters: HashMap::new(),
            gauges: HashMap::new(),
        }
    }

    pub fn record_latency(&mut self, name: &str, duration: Duration) {
        self.histograms
            .entry(name.to_string())
            .or_insert_with(LatencyHistogram::new)
            .record(duration);
    }

    pub fn increment_counter(&mut self, name: &str) {
        *self.counters.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn increment_counter_by(&mut self, name: &str, amount: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += amount;
    }

    pub fn set_gauge(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters.get(name).copied().unwrap_or(0)
    }

    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.get(name).copied()
    }

    pub fn get_histogram_report(&self, name: &str) -> Option<HistogramReport> {
        self.histograms.get(name).map(|h| h.report())
    }

    pub fn report(&self) -> String {
        let mut report = String::from("Metrics Report:\n");

        report.push_str("\nCounters:\n");
        let mut counters: Vec<_> = self.counters.iter().collect();
        counters.sort_by_key(|(k, _)| k.clone());
        for (name, value) in counters {
            report.push_str(&format!("  {name}: {value}\n"));
        }

        report.push_str("\nGauges:\n");
        let mut gauges: Vec<_> = self.gauges.iter().collect();
        gauges.sort_by_key(|(k, _)| k.clone());
        for (name, value) in gauges {
            report.push_str(&format!("  {name}: {value:.2}\n"));
        }

        report.push_str("\nLatency Histograms:\n");
        let mut histograms: Vec<_> = self.histograms.iter().collect();
        histograms.sort_by_key(|(k, _)| k.clone());
        for (name, hist) in histograms {
            report.push_str(&format!("  {name}: {}\n", hist.report()));
        }

        report
    }
}

/// A performance baseline that can be used to detect regressions.
pub struct PerformanceBaseline {
    measurements: Vec<f64>, // In microseconds
    max_samples: usize,
}

impl PerformanceBaseline {
    pub fn new(max_samples: usize) -> Self {
        PerformanceBaseline {
            measurements: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record(&mut self, duration: Duration) {
        let us = duration.as_secs_f64() * 1_000_000.0;
        if self.measurements.len() >= self.max_samples {
            self.measurements.remove(0);
        }
        self.measurements.push(us);
    }

    pub fn mean(&self) -> f64 {
        if self.measurements.is_empty() {
            return 0.0;
        }
        self.measurements.iter().sum::<f64>() / self.measurements.len() as f64
    }

    pub fn stddev(&self) -> f64 {
        if self.measurements.len() < 2 {
            return 0.0;
        }
        let mean = self.mean();
        let variance: f64 = self
            .measurements
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / (self.measurements.len() - 1) as f64;
        variance.sqrt()
    }

    /// Checks if a new measurement is a regression (more than N standard deviations above mean).
    pub fn is_regression(&self, duration: Duration, threshold_stdevs: f64) -> bool {
        let us = duration.as_secs_f64() * 1_000_000.0;
        let mean = self.mean();
        let stddev = self.stddev();
        if stddev == 0.0 {
            return false;
        }
        us > mean + threshold_stdevs * stddev
    }

    pub fn sample_count(&self) -> usize {
        self.measurements.len()
    }
}

/// A scoped timer that records its duration to a metrics collector on drop.
pub struct ScopedTimer<'a> {
    collector: &'a mut MetricsCollector,
    name: String,
    start: Instant,
}

impl<'a> ScopedTimer<'a> {
    pub fn new(collector: &'a mut MetricsCollector, name: &str) -> Self {
        ScopedTimer {
            collector,
            name: name.to_string(),
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for ScopedTimer<'a> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        self.collector.record_latency(&self.name, elapsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_histogram_basic() {
        let mut hist = LatencyHistogram::new();

        for i in 1..=100 {
            hist.record(Duration::from_micros(i));
        }

        assert_eq!(hist.count(), 100);
        let report = hist.report();
        assert!(report.p50 > Duration::ZERO);
        assert!(report.p99 >= report.p50);
    }

    #[test]
    fn test_latency_histogram_percentiles() {
        let mut hist = LatencyHistogram::new();

        // Record 100 measurements: 1ms, 2ms, ..., 100ms
        for i in 1..=100 {
            hist.record(Duration::from_millis(i));
        }

        let p50 = hist.percentile(0.5);
        let p99 = hist.percentile(0.99);
        assert!(p50 <= p99);
    }

    #[test]
    fn test_latency_histogram_report() {
        let mut hist = LatencyHistogram::new();
        hist.record(Duration::from_millis(10));
        hist.record(Duration::from_millis(20));
        hist.record(Duration::from_millis(30));

        let report = hist.report();
        assert_eq!(report.count, 3);
        assert!(report.mean > Duration::ZERO);
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();

        collector.increment_counter("requests");
        collector.increment_counter("requests");
        collector.increment_counter_by("bytes_sent", 1024);
        collector.set_gauge("cpu_usage", 0.75);
        collector.record_latency("api_call", Duration::from_millis(50));

        assert_eq!(collector.get_counter("requests"), 2);
        assert_eq!(collector.get_counter("bytes_sent"), 1024);
        assert_eq!(collector.get_gauge("cpu_usage"), Some(0.75));
        assert!(collector.get_histogram_report("api_call").is_some());
        assert_eq!(collector.get_counter("nonexistent"), 0);
    }

    #[test]
    fn test_metrics_collector_report() {
        let mut collector = MetricsCollector::new();
        collector.increment_counter("requests");
        collector.set_gauge("memory_mb", 512.5);
        collector.record_latency("db_query", Duration::from_millis(10));

        let report = collector.report();
        assert!(report.contains("Counters"));
        assert!(report.contains("Gauges"));
        assert!(report.contains("Latency Histograms"));
    }

    #[test]
    fn test_performance_baseline() {
        let mut baseline = PerformanceBaseline::new(100);

        // Record some measurements with variance
        for i in 0..50 {
            baseline.record(Duration::from_millis(10 + (i % 5) as u64));
        }

        let mean = baseline.mean();
        assert!(mean > 9000.0 && mean < 13000.0); // ~10-12ms in microseconds
        assert!(baseline.stddev() > 0.0);

        // Normal measurement
        assert!(!baseline.is_regression(Duration::from_millis(12), 3.0));

        // Anomalous measurement (way above mean)
        assert!(baseline.is_regression(Duration::from_millis(100), 3.0));
    }

    #[test]
    fn test_performance_baseline_variance() {
        let mut baseline = PerformanceBaseline::new(100);

        baseline.record(Duration::from_millis(10));
        baseline.record(Duration::from_millis(20));
        baseline.record(Duration::from_millis(30));

        let mean = baseline.mean();
        assert!((mean - 20000.0).abs() < 1.0); // ~20ms in microseconds
        assert!(baseline.stddev() > 0.0);
    }

    #[test]
    fn test_scoped_timer() {
        let mut collector = MetricsCollector::new();

        {
            let _timer = ScopedTimer::new(&mut collector, "operation");
            std::thread::sleep(Duration::from_millis(10));
        }

        let report = collector.get_histogram_report("operation").unwrap();
        assert_eq!(report.count, 1);
        assert!(report.mean >= Duration::from_millis(8));
    }

    #[test]
    fn test_scoped_timer_multiple() {
        let mut collector = MetricsCollector::new();

        for _ in 0..5 {
            let _timer = ScopedTimer::new(&mut collector, "batch");
            std::thread::sleep(Duration::from_millis(5));
        }

        let report = collector.get_histogram_report("batch").unwrap();
        assert_eq!(report.count, 5);
    }

    #[test]
    fn test_throughput_tracker() {
        let mut tracker = ThroughputTracker::new(Duration::from_secs(1));

        for _ in 0..100 {
            tracker.record();
        }

        let throughput = tracker.current_throughput();
        assert!(throughput > 0.0);
    }

    #[test]
    fn test_histogram_display() {
        let mut hist = LatencyHistogram::new();
        hist.record(Duration::from_millis(5));
        hist.record(Duration::from_millis(10));

        let report = hist.report();
        let display = format!("{report}");
        assert!(display.contains("Latency Report"));
        assert!(display.contains("P50"));
    }
}
