/// Problem: Benchmarking
///
/// Master benchmarking in Rust.
///
/// Key Concepts:
/// - Micro-benchmarking
/// - Macro-benchmarking
/// - Statistical analysis
/// - Benchmark frameworks
/// - Performance regression

use std::time::Instant;

/// Problem 1: Basic benchmark
/// Benchmark a function
pub fn basic_benchmark<F>(name: &str, iterations: u32, f: F) -> std::time::Duration
where
    F: Fn(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    start.elapsed()
}

/// Problem 2: Benchmark with warmup
/// Warmup before benchmark
pub fn benchmark_with_warmup<F>(name: &str, warmup: u32, iterations: u32, f: F) -> std::time::Duration
where
    F: Fn(),
{
    // Warmup
    for _ in 0..warmup {
        f();
    }

    // Benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    start.elapsed()
}

/// Problem 3: Benchmark with statistics
/// Collect statistics
pub fn benchmark_with_stats<F>(name: &str, iterations: u32, f: F) -> (std::time::Duration, std::time::Duration, std::time::Duration)
where
    F: Fn(),
{
    let mut durations = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        durations.push(start.elapsed());
    }

    let min = durations.iter().min().unwrap();
    let max = durations.iter().max().unwrap();
    let avg = durations.iter().sum::<std::time::Duration>() / iterations;

    (*min, *max, avg)
}

/// Problem 4: Benchmark comparison
/// Compare two implementations
pub fn benchmark_comparison<F1, F2>(name1: &str, f1: F1, name2: &str, f2: F2, iterations: u32) -> (std::time::Duration, std::time::Duration)
where
    F1: Fn(),
    F2: Fn(),
{
    let start1 = Instant::now();
    for _ in 0..iterations {
        f1();
    }
    let duration1 = start1.elapsed();

    let start2 = Instant::now();
    for _ in 0..iterations {
        f2();
    }
    let duration2 = start2.elapsed();

    (duration1, duration2)
}

/// Problem 5: Benchmark with memory
/// Track memory usage
pub fn benchmark_with_memory<F, T>(name: &str, f: F) -> (T, usize, std::time::Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    let size = std::mem::size_of_val(&result);
    (result, size, duration)
}

/// Problem 6: Benchmark with throughput
/// Measure throughput
pub fn benchmark_throughput<F>(name: &str, iterations: u32, f: F) -> f64
where
    F: Fn(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let duration = start.elapsed();
    iterations as f64 / duration.as_secs_f64()
}

/// Problem 7: Benchmark with latency
/// Measure latency
pub fn benchmark_latency<F>(name: &str, iterations: u32, f: F) -> Vec<std::time::Duration>
where
    F: Fn(),
{
    let mut latencies = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        latencies.push(start.elapsed());
    }
    latencies
}

/// Problem 8: Benchmark with percentile
/// Calculate percentiles
pub fn benchmark_percentile<F>(name: &str, iterations: u32, f: F) -> (std::time::Duration, std::time::Duration, std::time::Duration)
where
    F: Fn(),
{
    let mut durations = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        durations.push(start.elapsed());
    }

    durations.sort();
    let p50 = durations[iterations as usize / 2];
    let p95 = durations[(iterations as f64 * 0.95) as usize];
    let p99 = durations[(iterations as f64 * 0.99) as usize];

    (p50, p95, p99)
}

/// Problem 9: Benchmark with regression detection
/// Detect performance regression
pub fn benchmark_regression<F>(name: &str, baseline: std::time::Duration, iterations: u32, f: F) -> (std::time::Duration, bool)
where
    F: Fn(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let duration = start.elapsed();
    let avg = duration / iterations;
    let regression = avg > baseline * 2; // 2x slower is regression
    (avg, regression)
}

/// Problem 10: Benchmark with flamegraph data
/// Generate flamegraph data
pub fn benchmark_flamegraph<F>(name: &str, f: F) -> Vec<(String, u64)>
where
    F: Fn(),
{
    let start = Instant::now();
    f();
    let duration = start.elapsed();
    vec![(name.to_string(), duration.as_nanos() as u64)]
}

/// Problem 11: Benchmark with cache effects
/// Account for cache effects
pub fn benchmark_with_cache<F>(name: &str, iterations: u32, f: F) -> (std::time::Duration, std::time::Duration)
where
    F: Fn(),
{
    // Cold cache
    let start = Instant::now();
    f();
    let cold = start.elapsed();

    // Warm cache
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let warm = start.elapsed() / iterations;

    (cold, warm)
}

/// Problem 12: Benchmark with allocation tracking
/// Track allocations
pub fn benchmark_allocations<F>(name: &str, f: F) -> (usize, std::time::Duration)
where
    F: Fn(),
{
    let start = Instant::now();
    f();
    let duration = start.elapsed();
    let allocations = 0; // Simulated
    (allocations, duration)
}

/// Problem 13: Benchmark with concurrency
/// Benchmark concurrent execution
pub fn benchmark_concurrent<F>(name: &str, threads: u32, f: F) -> std::time::Duration
where
    F: Fn() + Send + Sync + 'static,
{
    let f = std::sync::Arc::new(f);
    let start = Instant::now();
    let mut handles = vec![];

    for _ in 0..threads {
        let f = f.clone();
        handles.push(std::thread::spawn(move || f()));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

/// Problem 14: Benchmark with I/O
/// Benchmark I/O operations
pub fn benchmark_io<F>(name: &str, iterations: u32, f: F) -> (std::time::Duration, u64)
where
    F: Fn(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let duration = start.elapsed();
    let bytes = 0; // Simulated
    (duration, bytes)
}

/// Problem 15: Benchmark with report
/// Generate benchmark report
pub fn benchmark_report<F>(name: &str, iterations: u32, f: F) -> String
where
    F: Fn(),
{
    let (min, max, avg) = benchmark_with_stats(name, iterations, f);
    format!(
        "Benchmark '{}': min={:?}, max={:?}, avg={:?}, iterations={}",
        name, min, max, avg, iterations
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_benchmark() {
        let duration = basic_benchmark("test", 1000, || {
            let _ = 1 + 1;
        });
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn test_benchmark_with_warmup() {
        let duration = benchmark_with_warmup("test", 100, 1000, || {
            let _ = 1 + 1;
        });
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn test_benchmark_with_stats() {
        let (min, max, avg) = benchmark_with_stats("test", 100, || {
            let _ = 1 + 1;
        });
        let _ = (min, max, avg);
    }

    #[test]
    fn test_benchmark_comparison() {
        let (d1, d2) = benchmark_comparison(
            "test1",
            || { let _ = 1 + 1; },
            "test2",
            || { let _ = 2 + 2; },
            1000,
        );
        assert!(d1.as_nanos() > 0);
        assert!(d2.as_nanos() > 0);
    }

    #[test]
    fn test_benchmark_with_memory() {
        let (result, size, duration) = benchmark_with_memory("test", || 42);
        assert_eq!(result, 42);
        assert!(size > 0);
        let _ = duration;
    }

    #[test]
    fn test_benchmark_throughput() {
        let throughput = benchmark_throughput("test", 1000, || {
            let _ = 1 + 1;
        });
        assert!(throughput > 0.0);
    }

    #[test]
    fn test_benchmark_latency() {
        let latencies = benchmark_latency("test", 100, || {
            let _ = 1 + 1;
        });
        assert_eq!(latencies.len(), 100);
    }

    #[test]
    fn test_benchmark_percentile() {
        let (p50, p95, p99) = benchmark_percentile("test", 100, || {
            let _ = 1 + 1;
        });
        let _ = (p50, p95, p99);
    }

    #[test]
    fn test_benchmark_regression() {
        let (avg, regression) = benchmark_regression(
            "test",
            std::time::Duration::from_nanos(1),
            100,
            || { let _ = 1 + 1; },
        );
        assert!(avg.as_nanos() > 0);
    }

    #[test]
    fn test_benchmark_flamegraph() {
        let data = benchmark_flamegraph("test", || {
            let _ = 1 + 1;
        });
        assert!(data.len() > 0);
    }

    #[test]
    fn test_benchmark_with_cache() {
        let (cold, warm) = benchmark_with_cache("test", 100, || {
            let _ = 1 + 1;
        });
        assert!(cold.as_nanos() > 0);
        assert!(warm.as_nanos() > 0);
    }

    #[test]
    fn test_benchmark_allocations() {
        let (allocations, duration) = benchmark_allocations("test", || {
            let _ = 1 + 1;
        });
        assert_eq!(allocations, 0);
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn test_benchmark_concurrent() {
        let duration = benchmark_concurrent("test", 4, || {
            let _ = 1 + 1;
        });
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn test_benchmark_io() {
        let (duration, bytes) = benchmark_io("test", 100, || {
            let _ = 1 + 1;
        });
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn test_benchmark_report() {
        let report = benchmark_report("test", 100, || {
            let _ = 1 + 1;
        });
        assert!(report.contains("test"));
    }
}
