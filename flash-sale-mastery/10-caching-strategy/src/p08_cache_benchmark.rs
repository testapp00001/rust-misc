//! # Exercise 08: Cache Benchmark
//!
//! ## Learning Objective
//! Measure and compare the performance of cache hits, cache misses (with
//! simulated DB load), and mixed workloads. Understand the latency and
//! throughput characteristics of different caching scenarios.
//!
//! ## Flash Sale Context
//! During a flash sale, the difference between a cache hit (~1 microsecond)
//! and a cache miss (~1 millisecond for DB) is three orders of magnitude.
//! Understanding these numbers helps you size your cache, set TTLs, and
//! decide when to warm vs. lazy-load. This exercise provides benchmarking
//! tools to measure these differences.
//!
//! ## Instructions
//! 1. Implement `BenchmarkResult` with latency and throughput metrics
//! 2. Implement `CacheBenchmark::new` to create a benchmark harness
//! 3. Implement `CacheBenchmark::populate` to pre-fill the cache
//! 4. Implement `benchmark_cache_hit` to measure hit latency
//! 5. Implement `benchmark_cache_miss` to measure miss + load latency
//! 6. Implement `benchmark_mixed` to measure a realistic workload
//!
//! ## Hints
//! - Use `std::time::Instant` for timing
//! - Run multiple iterations and compute averages
//! - Sort latencies to compute p99

use std::time::{Duration, Instant};

use dashmap::DashMap;

/// Results from a benchmark run.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the benchmark operation.
    pub operation: String,
    /// Number of iterations run.
    pub iterations: usize,
    /// Average latency per operation in microseconds.
    pub avg_latency_us: u64,
    /// 99th percentile latency in microseconds.
    pub p99_latency_us: u64,
    /// Minimum latency in microseconds.
    pub min_latency_us: u64,
    /// Maximum latency in microseconds.
    pub max_latency_us: u64,
    /// Operations per second (throughput).
    pub throughput_ops_per_sec: u64,
}

/// Cache benchmark harness.
pub struct CacheBenchmark {
    cache: DashMap<String, serde_json::Value>,
    /// Simulated DB latency in microseconds.
    db_latency_us: u64,
}

impl CacheBenchmark {
    /// Create a new benchmark harness.
    ///
    /// # Arguments
    /// * `db_latency_us` - Simulated database latency in microseconds
    pub fn new(db_latency_us: u64) -> Self {
        // TODO: Initialize with an empty DashMap and the given DB latency
        todo!("Implement CacheBenchmark::new")
    }

    /// Pre-populate the cache with the given number of entries.
    ///
    /// # Arguments
    /// * `count` - Number of entries to add to the cache
    pub fn populate(&self, count: usize) {
        // TODO: Insert `count` entries into the cache
        // TODO: Use keys like "key:0", "key:1", etc.
        todo!("Implement CacheBenchmark::populate")
    }

    /// Benchmark cache hit performance.
    ///
    /// Measures the time to read from a pre-populated cache entry.
    ///
    /// # Arguments
    /// * `iterations` - Number of read operations to perform
    pub fn benchmark_cache_hit(&self, iterations: usize) -> BenchmarkResult {
        // TODO: For each iteration, read a key from the cache
        // TODO: Record the latency of each read
        // TODO: Compute avg, p99, min, max, and throughput
        todo!("Implement CacheBenchmark::benchmark_cache_hit")
    }

    /// Benchmark cache miss performance.
    ///
    /// Measures the time to miss the cache and load from a simulated DB.
    ///
    /// # Arguments
    /// * `iterations` - Number of miss-and-load operations to perform
    pub fn benchmark_cache_miss(&self, iterations: usize) -> BenchmarkResult {
        // TODO: For each iteration:
        // TODO:   1. Look up a key that's NOT in the cache
        // TODO:   2. Simulate DB load (sleep for db_latency_us)
        // TODO:   3. Store the result in cache
        // TODO: Record latencies and compute statistics
        todo!("Implement CacheBenchmark::benchmark_cache_miss")
    }

    /// Benchmark a mixed workload with the given hit ratio.
    ///
    /// # Arguments
    /// * `hit_ratio` - Fraction of requests that should be cache hits (0.0 to 1.0)
    /// * `iterations` - Total number of operations to perform
    pub fn benchmark_mixed(
        &self,
        hit_ratio: f64,
        iterations: usize,
    ) -> BenchmarkResult {
        // TODO: For each iteration:
        // TODO:   - If random value < hit_ratio, do a cache hit
        // TODO:   - Otherwise, do a cache miss + load
        // TODO: Record latencies and compute statistics
        todo!("Implement CacheBenchmark::benchmark_mixed")
    }
}

/// Helper to compute percentile from a sorted list of latencies.
fn percentile(sorted: &[u64], p: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((p / 100.0) * sorted.len() as f64) as usize;
    let idx = idx.min(sorted.len() - 1);
    sorted[idx]
}

/// Helper to compute a BenchmarkResult from raw latencies.
fn compute_result(
    operation: String,
    iterations: usize,
    latencies: &mut [u64],
    total_duration: Duration,
) -> BenchmarkResult {
    latencies.sort_unstable();
    let avg = if iterations > 0 {
        latencies.iter().sum::<u64>() / iterations as u64
    } else {
        0
    };
    let throughput = if total_duration.as_secs_f64() > 0.0 {
        (iterations as f64 / total_duration.as_secs_f64()) as u64
    } else {
        0
    };

    BenchmarkResult {
        operation,
        iterations,
        avg_latency_us: avg,
        p99_latency_us: percentile(latencies, 99.0),
        min_latency_us: latencies.first().copied().unwrap_or(0),
        max_latency_us: latencies.last().copied().unwrap_or(0),
        throughput_ops_per_sec: throughput,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_cache_hit() {
        let bench = CacheBenchmark::new(1000); // 1ms simulated DB
        bench.populate(100);

        let result = bench.benchmark_cache_hit(1000);
        assert_eq!(result.iterations, 1000);
        assert!(
            result.avg_latency_us < 100,
            "Cache hit should be < 100us, got {}us",
            result.avg_latency_us
        );
        assert!(result.throughput_ops_per_sec > 0);
    }

    #[test]
    fn test_benchmark_cache_miss() {
        let bench = CacheBenchmark::new(500); // 500us simulated DB
        bench.populate(0); // Empty cache

        let result = bench.benchmark_cache_miss(100);
        assert_eq!(result.iterations, 100);
        // Miss should be at least as slow as the DB latency
        assert!(
            result.avg_latency_us >= 400,
            "Cache miss avg should be >= 400us, got {}us",
            result.avg_latency_us
        );
    }

    #[test]
    fn test_benchmark_mixed() {
        let bench = CacheBenchmark::new(1000);
        bench.populate(50); // 50 entries in cache

        let result = bench.benchmark_mixed(0.8, 200);
        assert_eq!(result.iterations, 200);
        // Mixed should be between pure hit and pure miss
        assert!(result.avg_latency_us > 0);
    }

    #[test]
    fn test_percentile_calculation() {
        let data: Vec<u64> = (1..=100).collect();
        // 0-based indexing: index 50 = value 51
        assert_eq!(percentile(&data, 50.0), 51);
        assert_eq!(percentile(&data, 99.0), 100);
        assert_eq!(percentile(&data, 0.0), 1);
    }

    #[test]
    fn test_hit_faster_than_miss() {
        let bench = CacheBenchmark::new(1000);
        bench.populate(100);

        let hit_result = bench.benchmark_cache_hit(500);

        // Clear cache for miss test
        bench.cache.clear();
        let miss_result = bench.benchmark_cache_miss(50);

        assert!(
            hit_result.avg_latency_us < miss_result.avg_latency_us,
            "Hit ({}) should be faster than miss ({})",
            hit_result.avg_latency_us,
            miss_result.avg_latency_us
        );
    }
}
