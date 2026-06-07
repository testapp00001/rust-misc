/// Problem: Profiling
///
/// Master profiling in Rust.
///
/// Key Concepts:
/// - Time measurement
/// - CPU profiling
/// - Memory profiling
/// - Benchmarking
/// - Flamegraphs

use std::time::Instant;

/// Problem 1: Basic timing
/// Measure execution time
pub fn basic_timing() -> std::time::Duration {
    let start = Instant::now();
    let mut sum: i64 = 0;
    for i in 0..1000000 {
        sum += i;
    }
    let _ = sum;
    start.elapsed()
}

/// Problem 2: Timing with iterations
/// Average time over iterations
pub fn timing_with_iterations(iterations: u32) -> std::time::Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = 1 + 1;
    }
    start.elapsed() / iterations
}

/// Problem 3: Memory usage
/// Track memory usage (simulated)
pub fn memory_usage() -> usize {
    let v = vec![0u8; 1024];
    v.len()
}

/// Problem 4: Allocation counting
/// Count allocations (simulated)
pub fn allocation_count() -> usize {
    let mut count = 0;
    for i in 0..100 {
        let _ = vec![i; 100];
        count += 1;
    }
    count
}

/// Problem 5: Profile with closure
/// Profile a closure
pub fn profile_closure<F, T>(name: &str, f: F) -> (T, std::time::Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// Problem 6: Profile with statistics
/// Collect timing statistics
pub fn profile_with_stats<F>(name: &str, iterations: u32, f: F) -> (std::time::Duration, std::time::Duration, std::time::Duration)
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

/// Problem 7: Profile with memory
/// Profile memory usage
pub fn profile_memory<F, T>(f: F) -> (T, usize)
where
    F: FnOnce() -> T,
{
    let result = f();
    let size = std::mem::size_of_val(&result);
    (result, size)
}

/// Problem 8: Profile with CPU cycles
/// Simulate CPU cycle counting
pub fn profile_cpu_cycles<F, T>(f: F) -> (T, u64)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let cycles = start.elapsed().as_nanos() as u64;
    (result, cycles)
}

/// Problem 9: Profile with cache misses
/// Simulate cache miss tracking
pub fn profile_cache<F, T>(f: F) -> (T, usize)
where
    F: FnOnce() -> T,
{
    let result = f();
    let cache_misses = 0; // Simulated
    (result, cache_misses)
}

/// Problem 10: Profile with I/O
/// Profile I/O operations
pub fn profile_io<F, T>(f: F) -> (T, std::time::Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let io_time = start.elapsed();
    (result, io_time)
}

/// Problem 11: Profile with thread count
/// Profile with multiple threads
pub fn profile_threads<F>(name: &str, thread_count: u32, f: F) -> std::time::Duration
where
    F: Fn() + Send + Sync + 'static,
{
    let start = Instant::now();
    let f = std::sync::Arc::new(f);
    let mut handles = vec![];

    for _ in 0..thread_count {
        let f = f.clone();
        handles.push(std::thread::spawn(move || f()));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

/// Problem 12: Profile with comparison
/// Compare two implementations
pub fn profile_comparison<F1, F2, T>(name1: &str, f1: F1, name2: &str, f2: F2) -> (std::time::Duration, std::time::Duration)
where
    F1: Fn() -> T,
    F2: Fn() -> T,
{
    let start1 = Instant::now();
    for _ in 0..1000 {
        f1();
    }
    let duration1 = start1.elapsed();

    let start2 = Instant::now();
    for _ in 0..1000 {
        f2();
    }
    let duration2 = start2.elapsed();

    (duration1, duration2)
}

/// Problem 13: Profile with threshold
/// Alert if execution exceeds threshold
pub fn profile_with_threshold<F, T>(name: &str, threshold: std::time::Duration, f: F) -> (T, bool)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    let exceeded = duration > threshold;
    (result, exceeded)
}

/// Problem 14: Profile with histogram
/// Create timing histogram
pub fn profile_histogram<F>(name: &str, iterations: u32, f: F) -> Vec<(std::time::Duration, u32)>
where
    F: Fn(),
{
    let mut durations = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        durations.push(start.elapsed());
    }

    let mut histogram = Vec::new();
    let bucket_size = std::time::Duration::from_micros(100);
    let mut current_bucket = std::time::Duration::ZERO;
    let mut count = 0;

    for duration in &durations {
        if *duration < current_bucket + bucket_size {
            count += 1;
        } else {
            if count > 0 {
                histogram.push((current_bucket, count));
            }
            current_bucket += bucket_size;
            count = 1;
        }
    }
    if count > 0 {
        histogram.push((current_bucket, count));
    }

    histogram
}

/// Problem 15: Profile with flamegraph data
/// Generate flamegraph data (simulated)
pub fn profile_flamegraph<F>(name: &str, f: F) -> Vec<(String, u64)>
where
    F: Fn(),
{
    let start = Instant::now();
    f();
    let duration = start.elapsed();

    vec![
        (name.to_string(), duration.as_nanos() as u64),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_timing() {
        let duration = basic_timing();
        let _ = duration; // Just ensure it doesn't panic
    }

    #[test]
    fn test_timing_with_iterations() {
        let duration = timing_with_iterations(1000);
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn test_memory_usage() {
        assert_eq!(memory_usage(), 1024);
    }

    #[test]
    fn test_allocation_count() {
        assert_eq!(allocation_count(), 100);
    }

    #[test]
    fn test_profile_closure() {
        let (result, duration) = profile_closure("test", || 42);
        assert_eq!(result, 42);
        let _ = duration;
    }

    #[test]
    fn test_profile_with_stats() {
        let (min, max, avg) = profile_with_stats("test", 10, || {
            let _ = 1 + 1;
        });
        let _ = (min, max, avg);
    }

    #[test]
    fn test_profile_memory() {
        let (result, size) = profile_memory(|| 42);
        assert_eq!(result, 42);
        assert!(size > 0);
    }

    #[test]
    fn test_profile_cpu_cycles() {
        let (result, cycles) = profile_cpu_cycles(|| 42);
        assert_eq!(result, 42);
        assert!(cycles > 0);
    }

    #[test]
    fn test_profile_cache() {
        let (result, cache_misses) = profile_cache(|| 42);
        assert_eq!(result, 42);
        assert_eq!(cache_misses, 0);
    }

    #[test]
    fn test_profile_io() {
        let (result, io_time) = profile_io(|| 42);
        assert_eq!(result, 42);
        let _ = io_time;
    }

    #[test]
    fn test_profile_threads() {
        let duration = profile_threads("test", 4, || {
            let _ = 1 + 1;
        });
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn test_profile_comparison() {
        let (d1, d2) = profile_comparison(
            "test1",
            || 1 + 1,
            "test2",
            || 2 + 2,
        );
        assert!(d1.as_nanos() > 0);
        assert!(d2.as_nanos() > 0);
    }

    #[test]
    fn test_profile_with_threshold() {
        let (result, exceeded) = profile_with_threshold(
            "test",
            std::time::Duration::from_secs(1),
            || 42,
        );
        assert_eq!(result, 42);
        assert!(!exceeded);
    }

    #[test]
    fn test_profile_histogram() {
        let histogram = profile_histogram("test", 10, || {
            let _ = 1 + 1;
        });
        assert!(histogram.len() > 0);
    }

    #[test]
    fn test_profile_flamegraph() {
        let data = profile_flamegraph("test", || {
            let _ = 1 + 1;
        });
        assert!(data.len() > 0);
    }
}
