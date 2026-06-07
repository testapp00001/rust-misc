//! # Profiling Tools
//!
//! Effective performance optimization starts with measurement. This module covers
//! the tools and workflows for profiling Rust applications, including `perf`,
//! `cargo-flamegraph`, and general profiling methodology.
//!
//! ## Key Concepts
//! - **perf**: Linux kernel profiler; samples CPU activity at high frequency
//! - **cargo-flamegraph**: Generates flame graphs from perf data
//! - **Profiling workflow**: Measure -> Identify -> Optimize -> Verify
//! - **Sampling vs tracing**: Sampling has low overhead; tracing captures every call

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A simple in-process profiler for measuring code sections.
/// Records wall-clock time for named sections and computes statistics.
pub struct Profiler {
    sections: HashMap<String, Vec<Duration>>,
    active: HashMap<String, Instant>,
}

impl Profiler {
    pub fn new() -> Self {
        Profiler {
            sections: HashMap::new(),
            active: HashMap::new(),
        }
    }

    /// Starts timing a named section.
    pub fn start(&mut self, name: &str) {
        self.active.insert(name.to_string(), Instant::now());
    }

    /// Stops timing a named section and records the duration.
    pub fn stop(&mut self, name: &str) -> Option<Duration> {
        if let Some(start) = self.active.remove(name) {
            let elapsed = start.elapsed();
            self.sections
                .entry(name.to_string())
                .or_default()
                .push(elapsed);
            Some(elapsed)
        } else {
            None
        }
    }

    /// Gets statistics for a named section.
    pub fn stats(&self, name: &str) -> Option<ProfileStats> {
        self.sections.get(name).map(|durations| {
            let count = durations.len();
            let total: Duration = durations.iter().sum();
            let mean = total / count as u32;
            let min = *durations.iter().min().unwrap();
            let max = *durations.iter().max().unwrap();

            let mut sorted = durations.clone();
            sorted.sort();
            let p50 = sorted[count / 2];
            let p95 = sorted[((count as f64 * 0.95) as usize).min(count - 1)];
            let p99 = sorted[((count as f64 * 0.99) as usize).min(count - 1)];

            ProfileStats {
                name: name.to_string(),
                count,
                total,
                mean,
                min,
                max,
                p50,
                p95,
                p99,
            }
        })
    }

    /// Returns all recorded section names.
    pub fn sections(&self) -> Vec<&str> {
        self.sections.keys().map(|s| s.as_str()).collect()
    }

    /// Prints a summary of all profiled sections.
    pub fn report(&self) -> String {
        let mut report = String::from("Profile Report:\n");
        report.push_str(&format!("{:<30} {:>8} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}\n",
            "Section", "Count", "Total(ms)", "Mean(ms)", "Min(ms)", "Max(ms)", "P95(ms)", "P99(ms)"));
        report.push_str(&"-".repeat(100));
        report.push('\n');

        let mut sections: Vec<_> = self.sections.keys().collect();
        sections.sort();

        for name in sections {
            if let Some(stats) = self.stats(name) {
                report.push_str(&format!(
                    "{:<30} {:>8} {:>10.3} {:>10.3} {:>10.3} {:>10.3} {:>10.3} {:>10.3}\n",
                    stats.name,
                    stats.count,
                    stats.total.as_secs_f64() * 1000.0,
                    stats.mean.as_secs_f64() * 1000.0,
                    stats.min.as_secs_f64() * 1000.0,
                    stats.max.as_secs_f64() * 1000.0,
                    stats.p95.as_secs_f64() * 1000.0,
                    stats.p99.as_secs_f64() * 1000.0,
                ));
            }
        }
        report
    }
}

#[derive(Debug, Clone)]
pub struct ProfileStats {
    pub name: String,
    pub count: usize,
    pub total: Duration,
    pub mean: Duration,
    pub min: Duration,
    pub max: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

/// RAII guard for profiling a scope. Automatically records the duration
/// when dropped.
pub struct ProfileGuard<'a> {
    profiler: &'a mut Profiler,
    name: String,
    start: Instant,
}

impl<'a> ProfileGuard<'a> {
    pub fn new(profiler: &'a mut Profiler, name: &str) -> Self {
        profiler.start(name);
        ProfileGuard {
            profiler,
            name: name.to_string(),
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for ProfileGuard<'a> {
    fn drop(&mut self) {
        let _ = self.profiler.stop(&self.name);
    }
}

/// Counts operations per second for a given workload.
pub struct ThroughputCounter {
    count: u64,
    start: Instant,
}

impl ThroughputCounter {
    pub fn new() -> Self {
        ThroughputCounter {
            count: 0,
            start: Instant::now(),
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn add(&mut self, n: u64) {
        self.count += n;
    }

    pub fn ops_per_second(&self) -> f64 {
        let elapsed = self.start.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.count as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Measures the allocation rate by tracking allocations.
/// Uses a simple counting approach for demonstration.
pub struct AllocationTracker {
    alloc_count: u64,
    dealloc_count: u64,
    bytes_allocated: u64,
    bytes_deallocated: u64,
}

impl AllocationTracker {
    pub fn new() -> Self {
        AllocationTracker {
            alloc_count: 0,
            dealloc_count: 0,
            bytes_allocated: 0,
            bytes_deallocated: 0,
        }
    }

    pub fn record_alloc(&mut self, size: usize) {
        self.alloc_count += 1;
        self.bytes_allocated += size as u64;
    }

    pub fn record_dealloc(&mut self, size: usize) {
        self.dealloc_count += 1;
        self.bytes_deallocated += size as u64;
    }

    pub fn net_allocations(&self) -> i64 {
        self.alloc_count as i64 - self.dealloc_count as i64
    }

    pub fn net_bytes(&self) -> i64 {
        self.bytes_allocated as i64 - self.bytes_deallocated as i64
    }

    pub fn report(&self) -> AllocationReport {
        AllocationReport {
            total_allocs: self.alloc_count,
            total_deallocs: self.dealloc_count,
            bytes_allocated: self.bytes_allocated,
            bytes_deallocated: self.bytes_deallocated,
            net_bytes: self.net_bytes(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AllocationReport {
    pub total_allocs: u64,
    pub total_deallocs: u64,
    pub bytes_allocated: u64,
    pub bytes_deallocated: u64,
    pub net_bytes: i64,
}

/// Wraps a function call and measures its allocation behavior.
pub fn measure_allocations<F, R>(f: F) -> (R, AllocationReport)
where
    F: FnOnce() -> R,
{
    let mut tracker = AllocationTracker::new();
    // In production, you'd use a custom allocator or dhat
    let result = f();
    // Simulate tracking (real implementation would intercept allocator)
    tracker.record_alloc(std::mem::size_of::<R>());
    (result, tracker.report())
}

/// A benchmark harness for quick performance measurements.
pub struct QuickBench {
    iterations: usize,
    warmup: usize,
}

impl QuickBench {
    pub fn new(iterations: usize, warmup: usize) -> Self {
        QuickBench { iterations, warmup }
    }

    /// Benchmarks a closure and returns timing statistics.
    pub fn run<F>(&self, name: &str, mut f: F) -> BenchResult
    where
        F: FnMut(),
    {
        // Warmup
        for _ in 0..self.warmup {
            f();
        }

        // Actual measurement
        let mut durations = Vec::with_capacity(self.iterations);
        for _ in 0..self.iterations {
            let start = Instant::now();
            f();
            durations.push(start.elapsed());
        }

        let total: Duration = durations.iter().sum();
        let mean = total / self.iterations as u32;
        let min = *durations.iter().min().unwrap();
        let max = *durations.iter().max().unwrap();

        BenchResult {
            name: name.to_string(),
            iterations: self.iterations,
            total,
            mean,
            min,
            max,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchResult {
    pub name: String,
    pub iterations: usize,
    pub total: Duration,
    pub mean: Duration,
    pub min: Duration,
    pub max: Duration,
}

impl std::fmt::Display for BenchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} iterations, mean={:?}, min={:?}, max={:?}",
            self.name, self.iterations, self.mean, self.min, self.max
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_basic() {
        let mut profiler = Profiler::new();

        profiler.start("section1");
        std::thread::sleep(Duration::from_millis(10));
        profiler.stop("section1");

        let stats = profiler.stats("section1").unwrap();
        assert_eq!(stats.count, 1);
        assert!(stats.mean >= Duration::from_millis(8));
    }

    #[test]
    fn test_profiler_multiple_measurements() {
        let mut profiler = Profiler::new();

        for _ in 0..5 {
            profiler.start("work");
            std::thread::sleep(Duration::from_millis(5));
            profiler.stop("work");
        }

        let stats = profiler.stats("work").unwrap();
        assert_eq!(stats.count, 5);
        assert!(stats.min <= stats.mean);
        assert!(stats.mean <= stats.max);
    }

    #[test]
    fn test_profiler_report() {
        let mut profiler = Profiler::new();

        profiler.start("fast");
        profiler.stop("fast");

        profiler.start("slow");
        std::thread::sleep(Duration::from_millis(10));
        profiler.stop("slow");

        let report = profiler.report();
        assert!(report.contains("fast"));
        assert!(report.contains("slow"));
    }

    #[test]
    fn test_profile_guard() {
        let mut profiler = Profiler::new();

        {
            let _guard = ProfileGuard::new(&mut profiler, "scoped");
            std::thread::sleep(Duration::from_millis(5));
        }

        let stats = profiler.stats("scoped").unwrap();
        assert_eq!(stats.count, 1);
    }

    #[test]
    fn test_throughput_counter() {
        let mut counter = ThroughputCounter::new();

        for _ in 0..1000 {
            counter.increment();
        }

        assert_eq!(counter.count(), 1000);
        assert!(counter.ops_per_second() > 0.0);
    }

    #[test]
    fn test_throughput_counter_batch() {
        let mut counter = ThroughputCounter::new();
        counter.add(500);
        assert_eq!(counter.count(), 500);
    }

    #[test]
    fn test_allocation_tracker() {
        let mut tracker = AllocationTracker::new();

        tracker.record_alloc(100);
        tracker.record_alloc(200);
        tracker.record_dealloc(100);

        assert_eq!(tracker.net_allocations(), 1);
        assert_eq!(tracker.net_bytes(), 200);

        let report = tracker.report();
        assert_eq!(report.total_allocs, 2);
        assert_eq!(report.total_deallocs, 1);
    }

    #[test]
    fn test_quick_bench() {
        let bench = QuickBench::new(100, 10);

        let result = bench.run("noop", || {
            std::hint::black_box(42);
        });

        assert_eq!(result.iterations, 100);
        assert!(result.mean < Duration::from_millis(1));
    }

    #[test]
    fn test_quick_bench_display() {
        let bench = QuickBench::new(10, 2);
        let result = bench.run("test", || {});
        let display = format!("{result}");
        assert!(display.contains("test"));
        assert!(display.contains("iterations"));
    }

    #[test]
    fn test_profiler_nonexistent_section() {
        let mut profiler = Profiler::new();
        assert!(profiler.stats("nonexistent").is_none());
        assert!(profiler.stop("nonexistent").is_none());
    }

    #[test]
    fn test_measure_allocations() {
        let (result, report) = measure_allocations(|| {
            let v: Vec<u8> = vec![1, 2, 3, 4, 5];
            v.len()
        });

        assert_eq!(result, 5);
        assert!(report.total_allocs > 0);
    }
}
