//! # Solution 07: Counter Benchmark
//!
//! Complete implementation of counter performance benchmarks.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};

/// Results from a benchmark run.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub approach: String,
    pub operations: u64,
    pub elapsed: Duration,
    pub ops_per_sec: f64,
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
/// Measures raw CPU throughput of `AtomicI64::fetch_sub` without any
/// network overhead. This is the theoretical upper bound.
pub fn benchmark_local_counter(iterations: u64) -> BenchmarkResult {
    let counter = AtomicI64::new(iterations as i64);

    let start = Instant::now();
    for _ in 0..iterations {
        counter.fetch_sub(1, Ordering::SeqCst);
    }
    let elapsed = start.elapsed();

    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
    let avg_latency = elapsed / iterations as u32;

    BenchmarkResult {
        approach: "Local AtomicI64".to_string(),
        operations: iterations,
        elapsed,
        ops_per_sec,
        avg_latency,
    }
}

/// Benchmark Redis DECR approach.
///
/// Each operation is a single DECR command (one network round-trip).
pub async fn benchmark_redis_decr(
    pool: &deadpool_redis::Pool,
    key: &str,
    iterations: u64,
) -> Result<BenchmarkResult, String> {
    use deadpool_redis::redis::cmd;

    let mut conn = pool.get().await.map_err(|e| format!("Conn: {e}"))?;

    // Initialize counter
    cmd("SET")
        .arg(key)
        .arg(iterations as i64)
        .query_async::<()>(&mut conn)
        .await
        .map_err(|e| format!("SET: {e}"))?;

    let start = Instant::now();
    for _ in 0..iterations {
        let _: i64 = cmd("DECR")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("DECR: {e}"))?;
    }
    let elapsed = start.elapsed();

    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
    let avg_latency = elapsed / iterations as u32;

    Ok(BenchmarkResult {
        approach: "Redis DECR".to_string(),
        operations: iterations,
        elapsed,
        ops_per_sec,
        avg_latency,
    })
}

/// Benchmark Redis CAS approach.
///
/// Each operation uses WATCH/MULTI/EXEC (multiple round-trips).
/// Under single-threaded access, there should be no retries.
pub async fn benchmark_redis_cas(
    pool: &deadpool_redis::Pool,
    key: &str,
    iterations: u64,
) -> Result<BenchmarkResult, String> {
    use deadpool_redis::redis::{cmd, Pipeline, Value};

    let mut conn = pool.get().await.map_err(|e| format!("Conn: {e}"))?;

    cmd("SET")
        .arg(key)
        .arg(iterations as i64)
        .query_async::<()>(&mut conn)
        .await
        .map_err(|e| format!("SET: {e}"))?;

    let start = Instant::now();
    for _ in 0..iterations {
        // CAS loop (should not retry under single-threaded access)
        loop {
            cmd("WATCH")
                .arg(key)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| format!("WATCH: {e}"))?;

            let current: i64 = cmd("GET")
                .arg(key)
                .query_async(&mut conn)
                .await
                .map_err(|e| format!("GET: {e}"))?;

            let mut pipe = Pipeline::new();
            pipe.atomic();
            pipe.set(key, current - 1);
            let result: Value = pipe
                .query_async(&mut conn)
                .await
                .map_err(|e| format!("EXEC: {e}"))?;

            if !matches!(result, Value::Nil) {
                break;
            }
            // Retry on conflict
        }
    }
    let elapsed = start.elapsed();

    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
    let avg_latency = elapsed / iterations as u32;

    Ok(BenchmarkResult {
        approach: "Redis CAS (WATCH/MULTI/EXEC)".to_string(),
        operations: iterations,
        elapsed,
        ops_per_sec,
        avg_latency,
    })
}

/// Run all benchmarks and compare the approaches.
pub async fn compare_approaches(iterations: u64) -> Result<Vec<BenchmarkResult>, String> {
    use deadpool_redis::{Config, Runtime};

    let mut results = Vec::new();

    // Local counter benchmark (no Redis needed)
    results.push(benchmark_local_counter(iterations));

    // Redis benchmarks
    let cfg = Config::from_url("redis://127.0.0.1:6379");
    let pool = cfg
        .create_pool(Some(Runtime::Tokio1))
        .map_err(|e| format!("Pool: {e}"))?;

    // Use smaller iteration count for Redis to keep tests fast
    let redis_iterations = std::cmp::min(iterations, 500);

    results.push(
        benchmark_redis_decr(&pool, "bench:decr", redis_iterations)
            .await?,
    );

    results.push(
        benchmark_redis_cas(&pool, "bench:cas", redis_iterations)
            .await?,
    );

    // Cleanup
    let mut conn = pool.get().await.map_err(|e| format!("Conn: {e}"))?;
    use deadpool_redis::redis::cmd;
    let _: Result<(), _> = cmd("DEL")
        .arg("bench:decr")
        .arg("bench:cas")
        .query_async(&mut conn)
        .await;

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn get_pool() -> Result<deadpool_redis::Pool, String> {
        let cfg = Config::from_url(REDIS_URL);
        cfg.create_pool(Some(Runtime::Tokio1))
            .map_err(|e| format!("Pool error: {e}"))
    }

    #[test]
    fn test_local_counter_benchmark() {
        let result = benchmark_local_counter(10_000);
        assert_eq!(result.operations, 10_000);
        assert!(result.ops_per_sec > 0.0, "Should have positive ops/sec");
        assert!(result.elapsed > Duration::ZERO, "Should take some time");
    }

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
        let mut conn = pool.get().await.unwrap();
        let _: Result<(), _> = deadpool_redis::redis::cmd("DEL")
            .arg(key)
            .query_async(&mut conn)
            .await;
    }

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
