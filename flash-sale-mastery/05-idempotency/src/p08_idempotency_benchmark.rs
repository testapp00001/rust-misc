//! # Exercise 08: Idempotency Benchmark
//!
//! ## Learning Objective
//! Measure and compare the performance of different idempotency strategies.
//! In a flash sale handling 10,000 requests/second, the idempotency check is
//! on the hot path. Understanding the latency and throughput characteristics
//! of each approach is critical for capacity planning.
//!
//! ## Flash Sale Context
//! Every purchase request must pass through the idempotency check before
//! processing. If the check takes 5ms, that's 5ms added to every request.
//! At 10K req/s, that's 50 seconds of cumulative latency per second. This
//! benchmark helps you choose the right strategy for your SLA.
//!
//! ## Instructions
//! 1. Implement the check functions for each storage backend
//! 2. Implement the benchmark functions that measure latency and throughput
//! 3. Run benchmarks with `cargo bench -p idempotency`
//! 4. Compare results across strategies
//!
//! ## Hints
//! - Use `criterion::Criterion` for benchmarks
//! - For async benchmarks, use `tokio::runtime::Runtime::block_on()`
//! - Pre-populate stores with data to test both hit and miss paths
//! - Use `BenchmarkId::new` to parameterize benchmarks

use criterion::{black_box, Criterion};

/// In-memory idempotency check using a HashMap.
///
/// This is the fastest possible check (no network, no disk) but provides
/// no durability and no sharing across processes.
pub fn check_in_memory(
    store: &std::collections::HashMap<String, String>,
    key: &str,
) -> bool {
    // TODO: Check if the key exists in the HashMap
    // Return true if this is a new key, false if duplicate
    todo!("Implement in-memory check")
}

/// Redis SET NX idempotency check.
///
/// Fast (sub-millisecond) and shared across processes, but volatile.
pub async fn check_redis(pool: &deadpool_redis::Pool, key: &str) -> Result<bool, String> {
    // TODO: Acquire a connection from the pool
    // TODO: Execute SET NX with a TTL
    // TODO: Return true if new, false if duplicate
    todo!("Implement Redis check")
}

/// SQLite unique constraint idempotency check.
///
/// Durable and ACID-compliant, but slower (1-5ms).
pub async fn check_sqlite(pool: &sqlx::SqlitePool, key: &str) -> Result<bool, String> {
    // TODO: Try to INSERT the key
    // TODO: If insert succeeds -> return true (new)
    // TODO: If unique constraint violation -> return false (duplicate)
    todo!("Implement SQLite check")
}

/// Set up an in-memory SQLite database for benchmarking.
pub async fn setup_sqlite() -> sqlx::SqlitePool {
    // TODO: Create an in-memory SQLite pool
    // TODO: Create the idempotency_keys table
    todo!("Implement SQLite setup")
}

/// Run all benchmarks. Called from the bench binary.
pub fn run_benchmarks(c: &mut Criterion) {
    bench_in_memory_latency(c);
    bench_redis_latency(c);
    bench_sqlite_latency(c);
    bench_concurrent_throughput(c);
    bench_hit_vs_miss(c);
}

/// Benchmark in-memory idempotency check latency.
fn bench_in_memory_latency(c: &mut Criterion) {
    // TODO: Create a pre-populated HashMap
    // TODO: Benchmark check_in_memory with both new and duplicate keys
    todo!("Implement in-memory latency benchmark")
}

/// Benchmark Redis SET NX latency.
fn bench_redis_latency(c: &mut Criterion) {
    // TODO: Create a Redis connection pool
    // TODO: Benchmark check_redis with both new and duplicate keys
    // TODO: Handle the case where Redis is unavailable (skip benchmark)
    todo!("Implement Redis latency benchmark")
}

/// Benchmark SQLite unique constraint latency.
fn bench_sqlite_latency(c: &mut Criterion) {
    // TODO: Create an in-memory SQLite pool
    // TODO: Benchmark check_sqlite with both new and duplicate keys
    todo!("Implement SQLite latency benchmark")
}

/// Benchmark throughput under concurrent load.
fn bench_concurrent_throughput(c: &mut Criterion) {
    // TODO: Create a benchmark group for "concurrent_throughput"
    // TODO: Vary the number of concurrent tasks (1, 10, 50, 100)
    // TODO: Measure how many idempotency checks can be performed per second
    todo!("Implement concurrent throughput benchmark")
}

/// Benchmark comparison: first check (miss) vs duplicate check (hit).
fn bench_hit_vs_miss(c: &mut Criterion) {
    // TODO: Compare the latency of checking a new key vs an existing key
    // TODO: For each backend (memory, Redis, SQLite)
    todo!("Implement hit vs miss benchmark")
}
