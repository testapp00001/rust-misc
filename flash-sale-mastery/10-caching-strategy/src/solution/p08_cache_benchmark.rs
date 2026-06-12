//! # Solution 08: Cache Benchmark
//!
//! Complete implementation of cache performance benchmarking.

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
    pub cache: DashMap<String, serde_json::Value>,
    /// Simulated DB latency in microseconds.
    db_latency_us: u64,
}

impl CacheBenchmark {
    /// Create a new benchmark harness.
    pub fn new(db_latency_us: u64) -> Self {
        Self {
            cache: DashMap::new(),
            db_latency_us,
        }
    }

    /// Pre-populate the cache with the given number of entries.
    pub fn populate(&self, count: usize) {
        for i in 0..count {
            self.cache.insert(
                format!("key:{i}"),
                serde_json::json!({"value": i}),
            );
        }
    }

    /// Benchmark cache hit performance.
    pub fn benchmark_cache_hit(&self, iterations: usize) -> BenchmarkResult {
        let mut latencies = Vec::with_capacity(iterations);
        let total_start = Instant::now();

        for i in 0..iterations {
            let key = format!("key:{}", i % self.cache.len().max(1));
            let start = Instant::now();
            let _ = self.cache.get(&key);
            latencies.push(start.elapsed().as_micros() as u64);
        }

        let total_duration = total_start.elapsed();
        compute_result("cache_hit".to_string(), iterations, &mut latencies, total_duration)
    }

    /// Benchmark cache miss performance.
    pub fn benchmark_cache_miss(&self, iterations: usize) -> BenchmarkResult {
        let mut latencies = Vec::with_capacity(iterations);
        let total_start = Instant::now();

        for i in 0..iterations {
            let key = format!("miss:{i}");
            let start = Instant::now();

            // Cache miss -- simulate DB load
            let sleep_duration = Duration::from_micros(self.db_latency_us);
            // Use a spin-wait for sub-millisecond precision
            let spin_start = Instant::now();
            while spin_start.elapsed() < sleep_duration {
                std::hint::spin_loop();
            }

            // Store result in cache (simulating cache-aside)
            self.cache.insert(key, serde_json::json!({"from_db": true}));

            latencies.push(start.elapsed().as_micros() as u64);
        }

        let total_duration = total_start.elapsed();
        compute_result("cache_miss".to_string(), iterations, &mut latencies, total_duration)
    }

    /// Benchmark a mixed workload with the given hit ratio.
    pub fn benchmark_mixed(
        &self,
        hit_ratio: f64,
        iterations: usize,
    ) -> BenchmarkResult {
        let mut latencies = Vec::with_capacity(iterations);
        let total_start = Instant::now();
        let cache_size = self.cache.len();

        for i in 0..iterations {
            let is_hit = rand::random::<f64>() < hit_ratio;

            let start = Instant::now();
            if is_hit && cache_size > 0 {
                // Cache hit
                let key = format!("key:{}", i % cache_size);
                let _ = self.cache.get(&key);
            } else {
                // Cache miss -- simulate DB load
                let key = format!("mixed:{i}");
                let sleep_duration = Duration::from_micros(self.db_latency_us);
                let spin_start = Instant::now();
                while spin_start.elapsed() < sleep_duration {
                    std::hint::spin_loop();
                }
                self.cache.insert(key, serde_json::json!({"from_db": true}));
            }
            latencies.push(start.elapsed().as_micros() as u64);
        }

        let total_duration = total_start.elapsed();
        compute_result("mixed".to_string(), iterations, &mut latencies, total_duration)
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
