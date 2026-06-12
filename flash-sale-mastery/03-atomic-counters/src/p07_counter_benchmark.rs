//! # Exercise 07: Counter Benchmark
//!
//! ## Learning Objective
//! Measure and compare the performance characteristics of different
//! counter approaches: throughput (ops/sec), latency, and behavior
//! under contention.
//!
//! ## Flash Sale Context
//! Choosing the right counter strategy for a flash sale requires
//! understanding the performance trade-offs. A local atomic counter
//! is fastest but doesn't scale. Redis DECR is fast but has the
//! oversell window. CAS is correct but slow under contention.
//! This exercise quantifies these differences.
//!
//! ## Instructions
//! 1. Implement `benchmark_local_counter` to measure in-memory throughput
//! 2. Implement `benchmark_redis_decr` to measure Redis DECR throughput
//! 3. Implement `benchmark_redis_cas` to measure CAS throughput
//! 4. Implement `compare_approaches` that runs all benchmarks and returns results
//!
//! ## Hints
//! - Use `std::time::Instant` for timing
//! - Run many iterations and divide by elapsed time for ops/sec
//! - For Redis benchmarks, use a connection pool to avoid connection overhead
//! - Compare single-threaded vs multi-threaded results
//!
//! ## Trade-offs (What You'll Discover)
//! - Local counter: ~100M ops/sec (no network, no contention at low thread count)
//! - Redis DECR: ~100K-500K ops/sec (network round-trip per operation)
//! - Redis CAS: ~10K-50K ops/sec (multiple round-trips + retries under contention)
//! - Redis Lua: ~200K-400K ops/sec (single round-trip, server-side logic)

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};

/// Results from a benchmark run.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the approach benchmarked.
    pub approach: String,
    /// Total operations completed.
    pub operations: u64,
    /// Total elapsed time.
    pub elapsed: Duration,
    /// Operations per second.
    pub ops_per_sec: f64,
    /// Average latency per operation.
    pub avg_latency: Duration,
}

impl std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} ops in {:?} ({:.0} ops/sec, avg latency {:?})",
            self.approach, self.operations, self.elapsed, self.ops_per_sec, self.avg_latency
        )
    }
}

/// Benchmark the local atomic counter approach.
///
/// Creates an `AtomicI64` counter and decrements it `iterations` times
/// from a single thread. Measures raw throughput without network overhead.
///
/// # Arguments
/// * `iterations` - Number of decrement operations to perform
///
/// # Returns
/// Benchmark results for the local counter.
pub fn benchmark_local_counter(iterations: u64) -> BenchmarkResult {
    // TODO: Create an AtomicI64 with initial value = iterations
    // TODO: Start timer
    // TODO: Loop `iterations` times, calling fetch_sub(1)
    // TODO: Stop timer
    // TODO: Calculate ops/sec and avg latency
    // TODO: Return BenchmarkResult
    todo!("Implement local counter benchmark")
}

/// Benchmark Redis DECR approach.
///
/// Uses a Redis connection pool and performs DECR operations.
/// Each operation is a single DECR command.
///
/// # Arguments
/// * `pool` - Redis connection pool
/// * `key` - Redis key to decrement
/// * `iterations` - Number of operations
pub async fn benchmark_redis_decr(
    pool: &deadpool_redis::Pool,
    key: &str,
    iterations: u64,
) -> Result<BenchmarkResult, String> {
    // TODO: Initialize the key with `iterations` value
    // TODO: Start timer
    // TODO: Loop `iterations` times:
    //   - Get connection from pool
    //   - DECR the key
    // TODO: Stop timer
    // TODO: Calculate and return BenchmarkResult
    todo!("Implement Redis DECR benchmark")
}

/// Benchmark Redis CAS approach.
///
/// Uses WATCH/MULTI/EXEC for each decrement.
///
/// # Arguments
/// * `pool` - Redis connection pool
/// * `key` - Redis key to decrement
/// * `iterations` - Number of operations
pub async fn benchmark_redis_cas(
    pool: &deadpool_redis::Pool,
    key: &str,
    iterations: u64,
) -> Result<BenchmarkResult, String> {
    // TODO: Initialize the key with `iterations` value
    // TODO: Start timer
    // TODO: Loop `iterations` times:
    //   - GET current value
    //   - WATCH + MULTI + SET(current-1) + EXEC
    //   - Retry if conflict
    // TODO: Stop timer
    // TODO: Calculate and return BenchmarkResult
    todo!("Implement Redis CAS benchmark")
}

/// Run all benchmarks and compare the approaches.
///
/// # Arguments
/// * `iterations` - Number of operations per benchmark
pub async fn compare_approaches(iterations: u64) -> Result<Vec<BenchmarkResult>, String> {
    // TODO: Run benchmark_local_counter
    // TODO: Create Redis pool
    // TODO: Run benchmark_redis_decr
    // TODO: Run benchmark_redis_cas
    // TODO: Return all results
    todo!("Implement benchmark comparison")
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn get_pool() -> Result<deadpool_redis::Pool, String> {
        let cfg = Config::from_url(REDIS_URL);
        cfg.builder(Some(Runtime::Tokio1))
            .build()
            .map_err(|e| format!("Pool error: {e}"))
    }

    /// Test local counter benchmark runs and produces valid results.
    #[test]
    fn test_local_counter_benchmark() {
        let result = benchmark_local_counter(10_000);
        assert_eq!(result.operations, 10_000);
        assert!(result.ops_per_sec > 0.0, "Should have positive ops/sec");
        assert!(result.elapsed > Duration::ZERO, "Should take some time");
    }

    /// Test Redis DECR benchmark runs.
    #[tokio::test]
    async fn test_redis_decr_benchmark() {
        let pool = match get_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key = "test:bench:decr";
        let result = benchmark_redis_decr(&pool, key, 100)
            .await
            .expect("Benchmark should run");
        assert_eq!(result.operations, 100);
        assert!(result.ops_per_sec > 0.0);

        // Cleanup
        let mut conn = pool.get().await.unwrap();
        let _: Result<(), _> = deadpool_redis::redis::cmd("DEL")
            .arg(key)
            .query_async(&mut conn)
            .await;
    }

    /// Test Redis CAS benchmark runs.
    #[tokio::test]
    async fn test_redis_cas_benchmark() {
        let pool = match get_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key = "test:bench:cas";
        let result = benchmark_redis_cas(&pool, key, 50)
            .await
            .expect("Benchmark should run");
        assert_eq!(result.operations, 50);
        assert!(result.ops_per_sec > 0.0);

        let mut conn = pool.get().await.unwrap();
        let _: Result<(), _> = deadpool_redis::redis::cmd("DEL")
            .arg(key)
            .query_async(&mut conn)
            .await;
    }

    /// Test full comparison runs.
    #[tokio::test]
    async fn test_compare_approaches() {
        let results = compare_approaches(100).await;
        match results {
            Ok(results) => {
                assert!(!results.is_empty(), "Should have benchmark results");
                for r in &results {
                    println!("{}", r);
                }
            }
            Err(msg) => {
                eprintln!("SKIP: {msg}");
            }
        }
    }
}
