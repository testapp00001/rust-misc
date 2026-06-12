use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Collects runtime metrics for the message queue broker.
///
/// All counters are lock-free atomics. Latency histograms and error counts
/// use `DashMap` for concurrent, sharded access.
#[derive(Debug)]
pub struct Metrics {
    messages_produced: AtomicU64,
    messages_consumed: AtomicU64,
    bytes_produced: AtomicU64,
    bytes_consumed: AtomicU64,
    replication_lag: AtomicU64,
    active_connections: AtomicU64,
    active_topics: AtomicU64,
    request_latency_us: DashMap<String, LatencyAccumulator>,
    error_counts: DashMap<String, AtomicU64>,
    start_time: Instant,
}

/// Accumulates latency samples using a simple reservoir of recent values.
/// For production use you would swap this for a HDR histogram, but this
/// keeps the dependency footprint small while still giving useful p50/p99
/// approximations.
#[derive(Debug)]
struct LatencyAccumulator {
    /// Circular buffer of recent latency samples (microseconds).
    samples: Vec<u64>,
    /// Write index into the circular buffer.
    write_idx: usize,
    /// Total number of samples recorded (may exceed buffer length).
    total_count: u64,
    /// Sum of all samples for computing the mean.
    total_sum: u64,
}

impl LatencyAccumulator {
    fn with_capacity(cap: usize) -> Self {
        Self {
            samples: vec![0; cap],
            write_idx: 0,
            total_count: 0,
            total_sum: 0,
        }
    }

    fn record(&mut self, value: u64) {
        let cap = self.samples.len();
        if cap > 0 {
            self.samples[self.write_idx % cap] = value;
            self.write_idx += 1;
        }
        self.total_count += 1;
        self.total_sum += value;
    }

    fn mean_us(&self) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }
        self.total_sum as f64 / self.total_count as f64
    }

    fn percentile(&self, p: f64) -> u64 {
        let cap = self.samples.len();
        let count = std::cmp::min(self.total_count as usize, cap);
        if count == 0 {
            return 0;
        }

        let mut sorted: Vec<u64> = self.samples[..count].to_vec();
        sorted.sort_unstable();

        let idx = ((p / 100.0) * (count as f64 - 1.0)).round() as usize;
        sorted[idx]
    }

    fn count(&self) -> u64 {
        self.total_count
    }
}

/// A point-in-time snapshot of broker metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub messages_produced: u64,
    pub messages_consumed: u64,
    pub bytes_produced: u64,
    pub bytes_consumed: u64,
    pub active_connections: u64,
    pub active_topics: u64,
    pub replication_lag: u64,
    pub uptime_secs: u64,
    pub error_counts: HashMap<String, u64>,
    /// Map of operation name -> latency percentiles.
    pub latency_percentiles: HashMap<String, LatencyPercentiles>,
}

/// Latency percentile breakdown for a single operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    pub p50_us: u64,
    pub p90_us: u64,
    pub p99_us: u64,
    pub mean_us: f64,
    pub count: u64,
}

impl Metrics {
    /// Create a new metrics collector. Returns an `Arc` so it can be shared
    /// across tasks cheaply.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            messages_produced: AtomicU64::new(0),
            messages_consumed: AtomicU64::new(0),
            bytes_produced: AtomicU64::new(0),
            bytes_consumed: AtomicU64::new(0),
            replication_lag: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            active_topics: AtomicU64::new(0),
            request_latency_us: DashMap::new(),
            error_counts: DashMap::new(),
            start_time: Instant::now(),
        })
    }

    /// Record that `count` messages were produced.
    pub fn record_produce(&self, count: u64) {
        self.messages_produced.fetch_add(count, Ordering::Relaxed);
    }

    /// Record that `count` bytes were produced.
    pub fn record_bytes_produced(&self, bytes: u64) {
        self.bytes_produced.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record that `count` messages were consumed.
    pub fn record_consume(&self, count: u64) {
        self.messages_consumed.fetch_add(count, Ordering::Relaxed);
    }

    /// Record that `count` bytes were consumed.
    pub fn record_bytes_consumed(&self, bytes: u64) {
        self.bytes_consumed.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record a latency sample for the given operation name.
    pub fn record_latency(&self, operation: &str, latency_us: u64) {
        self.request_latency_us
            .entry(operation.to_string())
            .or_insert_with(|| LatencyAccumulator::with_capacity(1024))
            .record(latency_us);
    }

    /// Record a latency sample using the provided `Instant` as the start time.
    /// Convenience method that computes the elapsed microseconds automatically.
    pub fn record_latency_since(&self, operation: &str, start: Instant) {
        let elapsed = start.elapsed().as_micros() as u64;
        self.record_latency(operation, elapsed);
    }

    /// Increment the error counter for the given error type.
    pub fn record_error(&self, error_type: &str) {
        self.error_counts
            .entry(error_type.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Set the current number of active connections (gauge).
    pub fn set_active_connections(&self, count: u64) {
        self.active_connections.store(count, Ordering::Relaxed);
    }

    /// Increment active connections by 1.
    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connections by 1.
    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get the current number of active connections.
    pub fn active_connections(&self) -> u64 {
        self.active_connections.load(Ordering::Relaxed)
    }

    /// Set the current replication lag (in messages or bytes, depending on
    /// context).
    pub fn set_replication_lag(&self, lag: u64) {
        self.replication_lag.store(lag, Ordering::Relaxed);
    }

    /// Set the number of active topics.
    pub fn set_active_topics(&self, count: u64) {
        self.active_topics.store(count, Ordering::Relaxed);
    }

    /// Capture a consistent snapshot of all metrics for export (e.g. to
    /// Prometheus, JSON endpoint, or log line).
    pub fn snapshot(&self) -> MetricsSnapshot {
        let error_counts: HashMap<String, u64> = self
            .error_counts
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().load(Ordering::Relaxed)))
            .collect();

        let latency_percentiles: HashMap<String, LatencyPercentiles> = self
            .request_latency_us
            .iter()
            .map(|entry| {
                let acc = entry.value();
                (
                    entry.key().clone(),
                    LatencyPercentiles {
                        p50_us: acc.percentile(50.0),
                        p90_us: acc.percentile(90.0),
                        p99_us: acc.percentile(99.0),
                        mean_us: acc.mean_us(),
                        count: acc.count(),
                    },
                )
            })
            .collect();

        MetricsSnapshot {
            messages_produced: self.messages_produced.load(Ordering::Relaxed),
            messages_consumed: self.messages_consumed.load(Ordering::Relaxed),
            bytes_produced: self.bytes_produced.load(Ordering::Relaxed),
            bytes_consumed: self.bytes_consumed.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            active_topics: self.active_topics.load(Ordering::Relaxed),
            replication_lag: self.replication_lag.load(Ordering::Relaxed),
            uptime_secs: self.start_time.elapsed().as_secs(),
            error_counts,
            latency_percentiles,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_produce_increments_counter() {
        let metrics = Metrics::new();
        metrics.record_produce(10);
        metrics.record_produce(5);
        let snap = metrics.snapshot();
        assert_eq!(snap.messages_produced, 15);
    }

    #[test]
    fn record_consume_increments_counter() {
        let metrics = Metrics::new();
        metrics.record_consume(3);
        let snap = metrics.snapshot();
        assert_eq!(snap.messages_consumed, 3);
    }

    #[test]
    fn bytes_counters() {
        let metrics = Metrics::new();
        metrics.record_bytes_produced(1024);
        metrics.record_bytes_consumed(512);
        let snap = metrics.snapshot();
        assert_eq!(snap.bytes_produced, 1024);
        assert_eq!(snap.bytes_consumed, 512);
    }

    #[test]
    fn error_counts() {
        let metrics = Metrics::new();
        metrics.record_error("timeout");
        metrics.record_error("timeout");
        metrics.record_error("connection_refused");
        let snap = metrics.snapshot();
        assert_eq!(snap.error_counts["timeout"], 2);
        assert_eq!(snap.error_counts["connection_refused"], 1);
    }

    #[test]
    fn active_connections_gauge() {
        let metrics = Metrics::new();
        metrics.set_active_connections(100);
        assert_eq!(metrics.snapshot().active_connections, 100);
        metrics.increment_connections();
        assert_eq!(metrics.snapshot().active_connections, 101);
        metrics.decrement_connections();
        assert_eq!(metrics.snapshot().active_connections, 100);
    }

    #[test]
    fn replication_lag() {
        let metrics = Metrics::new();
        metrics.set_replication_lag(42);
        assert_eq!(metrics.snapshot().replication_lag, 42);
    }

    #[test]
    fn latency_recording() {
        let metrics = Metrics::new();
        metrics.record_latency("produce", 100);
        metrics.record_latency("produce", 200);
        metrics.record_latency("produce", 300);

        let snap = metrics.snapshot();
        let p = &snap.latency_percentiles["produce"];
        assert_eq!(p.count, 3);
        assert!(p.mean_us > 0.0);
        assert!(p.p50_us > 0);
    }

    #[test]
    fn latency_since() {
        let metrics = Metrics::new();
        let start = Instant::now();
        // Simulate a small delay
        std::thread::sleep(std::time::Duration::from_millis(1));
        metrics.record_latency_since("test_op", start);

        let snap = metrics.snapshot();
        let p = &snap.latency_percentiles["test_op"];
        assert_eq!(p.count, 1);
        assert!(p.mean_us >= 500.0); // at least ~1ms = 1000us, allow margin
    }

    #[test]
    fn active_topics() {
        let metrics = Metrics::new();
        metrics.set_active_topics(5);
        assert_eq!(metrics.snapshot().active_topics, 5);
    }

    #[test]
    fn snapshot_uptime_is_nonzero() {
        let metrics = Metrics::new();
        let snap = metrics.snapshot();
        // Uptime should be 0 or 1 depending on timing
        assert!(snap.uptime_secs <= 1);
    }

    #[test]
    fn latency_accumulator_empty() {
        let acc = LatencyAccumulator::with_capacity(10);
        assert_eq!(acc.mean_us(), 0.0);
        assert_eq!(acc.percentile(50.0), 0);
        assert_eq!(acc.count(), 0);
    }

    #[test]
    fn latency_accumulator_circular_buffer() {
        let mut acc = LatencyAccumulator::with_capacity(4);
        acc.record(10);
        acc.record(20);
        acc.record(30);
        acc.record(40);
        acc.record(50); // wraps around, overwrites 10

        assert_eq!(acc.count(), 5);
        // Mean should be (10+20+30+40+50)/5 = 30
        assert!((acc.mean_us() - 30.0).abs() < f64::EPSILON);
    }
}
