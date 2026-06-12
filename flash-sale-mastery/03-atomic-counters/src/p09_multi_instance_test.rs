//! # Exercise 09: Multi-Instance Consistency Test
//!
//! ## Learning Objective
//! Verify that a distributed atomic counter maintains consistency when
//! accessed by multiple simulated application instances (pods) concurrently.
//! This is the ultimate integration test for the patterns learned in this module.
//!
//! ## Flash Sale Context
//! In production Kubernetes, you might have 10-50 pods all serving flash
//! sale requests simultaneously. Each pod connects to the same Redis
//! cluster and decrements the same stock counter. This test simulates
//! that scenario and verifies the fundamental invariant:
//!
//! **Total successful decrements == Initial stock**
//!
//! If this invariant is violated, customers bought items that don't exist
//! (overselling) or stock was left unsold (underselling).
//!
//! ## Instructions
//! 1. Implement `SimulatedInstance` that represents one application pod
//! 2. Implement `run_multi_instance_test` that spawns N instances
//! 3. Each instance tries to decrement the shared counter M times
//! 4. Verify: sum of all successful decrements == initial stock
//! 5. Verify: final stock value == 0
//! 6. Use `DashMap` to track per-instance statistics
//!
//! ## Hints
//! - Each instance gets its own Redis connection (simulating a separate pod)
//! - Use `tokio::spawn` for concurrent instances
//! - Track per-instance success/failure counts with `DashMap`
//! - The test should pass with 10 instances and 1000 stock
//!
//! ## Trade-offs
//! This exercise doesn't have trade-offs -- it's a verification test.
//! The point is to prove that the distributed counter pattern works
//! correctly under realistic concurrent load.

use dashmap::DashMap;
use deadpool_redis::redis::cmd;
use deadpool_redis::{Config, Connection as RedisConnection, Pool, Runtime};
use std::sync::Arc;

/// Error type for multi-instance test operations.
#[derive(Debug, thiserror::Error)]
pub enum MultiInstanceError {
    #[error("Redis error: {0}")]
    RedisError(#[from] deadpool_redis::redis::RedisError),

    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool_redis::PoolError),

    #[error("Invariant violated: {0}")]
    InvariantViolation(String),
}

/// Statistics for a single simulated instance.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InstanceStats {
    /// Number of successful decrements.
    pub successes: u64,
    /// Number of failed decrements (stock exhausted).
    pub failures: u64,
    /// Number of errors (connection issues, etc.).
    pub errors: u64,
}

/// A simulated application instance (pod).
///
/// Each instance has its own Redis connection pool (simulating a
/// separate pod in Kubernetes) and attempts to decrement stock.
pub struct SimulatedInstance {
    /// Instance identifier (e.g., "pod-1").
    pub id: String,
    /// This instance's connection pool.
    pool: Pool,
}

impl SimulatedInstance {
    /// Create a new simulated instance.
    ///
    /// # Arguments
    /// * `id` - Instance identifier
    /// * `redis_url` - Redis connection URL
    pub async fn new(id: &str, redis_url: &str) -> Result<Self, MultiInstanceError> {
        // TODO: Create a connection pool for this instance
        // TODO: Return SimulatedInstance { id, pool }
        todo!("Create simulated instance with its own connection pool")
    }

    /// Attempt to decrement stock multiple times.
    ///
    /// # Arguments
    /// * `key` - Redis key for the stock counter
    /// * `attempts` - Number of decrement attempts
    ///
    /// # Returns
    /// InstanceStats with success/failure/error counts.
    pub async fn run(&self, key: &str, attempts: u64) -> InstanceStats {
        // TODO: Loop `attempts` times:
        //   1. Get a connection from this instance's pool
        //   2. DECR the key
        //   3. If result >= 0, count as success
        //   4. If result < 0, INCR to revert, count as failure
        //   5. If connection error, count as error
        // TODO: Return InstanceStats
        todo!("Implement instance decrement loop")
    }
}

/// Run the multi-instance test.
///
/// Spawns `num_instances` simulated pods, each trying to decrement
/// a shared counter `attempts_per_instance` times. Verifies that
/// the total successful decrements exactly equals the initial stock.
///
/// # Arguments
/// * `redis_url` - Redis connection URL
/// * `initial_stock` - Starting stock quantity
/// * `num_instances` - Number of simulated pods
/// * `attempts_per_instance` - Each pod tries this many decrements
pub async fn run_multi_instance_test(
    redis_url: &str,
    initial_stock: i64,
    num_instances: usize,
    attempts_per_instance: u64,
) -> Result<MultiInstanceTestReport, MultiInstanceError> {
    // TODO: Initialize stock in Redis
    // TODO: Create DashMap<String, InstanceStats> for tracking
    // TODO: Spawn `num_instances` tokio tasks, each creating a SimulatedInstance
    // TODO: Each task runs its instance and inserts stats into DashMap
    // TODO: Wait for all tasks to complete
    // TODO: Read final stock from Redis
    // TODO: Sum all successes and verify == initial_stock
    // TODO: Verify final stock == 0
    // TODO: Return MultiInstanceTestReport
    todo!("Implement multi-instance test")
}

/// Report from a multi-instance test.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiInstanceTestReport {
    /// Number of simulated instances.
    pub num_instances: usize,
    /// Initial stock.
    pub initial_stock: i64,
    /// Final stock value in Redis.
    pub final_stock: i64,
    /// Total successful decrements across all instances.
    pub total_successes: u64,
    /// Total failed decrements (stock exhausted).
    pub total_failures: u64,
    /// Total errors.
    pub total_errors: u64,
    /// Whether the invariant held: total_successes == initial_stock.
    pub invariant_held: bool,
    /// Per-instance statistics.
    pub per_instance: Vec<(String, InstanceStats)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const REDIS_URL: &str = "redis://127.0.0.1:6379";
    const TEST_KEY: &str = "test:multi:instance:counter";

    async fn try_connect() -> Result<Pool, String> {
        let cfg = Config::from_url(REDIS_URL);
        cfg.builder(Some(Runtime::Tokio1))
            .build()
            .map_err(|e| format!("Pool error: {e}"))
    }

    /// Test single instance can decrement correctly.
    #[tokio::test]
    async fn test_single_instance() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.unwrap();
        let _: Result<(), _> = cmd("DEL").arg(TEST_KEY).query_async(&mut conn).await;
        cmd("SET")
            .arg(TEST_KEY)
            .arg(10i64)
            .query_async::<_, ()>(&mut conn)
            .await
            .unwrap();

        let instance = SimulatedInstance::new("pod-0", REDIS_URL)
            .await
            .expect("Should create instance");
        let stats = instance.run(TEST_KEY, 20).await;

        assert_eq!(stats.successes, 10, "Should succeed exactly 10 times");
        assert_eq!(stats.failures, 10, "Should fail exactly 10 times");
        assert_eq!(stats.errors, 0, "Should have no errors");

        let _: Result<(), _> = cmd("DEL").arg(TEST_KEY).query_async(&mut conn).await;
    }

    /// Test 10 instances competing for 1000 items.
    /// Each instance tries 200 times. Total attempts = 2000, stock = 1000.
    #[tokio::test]
    async fn test_ten_instances_thousand_stock() {
        let report = match run_multi_instance_test(REDIS_URL, 1000, 10, 200).await {
            Ok(r) => r,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };

        // The fundamental invariant
        assert!(
            report.invariant_held,
            "INVIOLABLE: total_successes ({}) must == initial_stock ({})",
            report.total_successes,
            report.initial_stock
        );
        assert_eq!(report.final_stock, 0, "Stock should be exactly 0");
        assert_eq!(report.total_successes, 1000);
        assert_eq!(report.total_errors, 0, "Should have no errors");
    }

    /// Test with more instances than stock.
    #[tokio::test]
    async fn test_more_instances_than_stock() {
        let report = match run_multi_instance_test(REDIS_URL, 50, 20, 10).await {
            Ok(r) => r,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };

        assert!(report.invariant_held, "Invariant must hold");
        assert_eq!(report.final_stock, 0);
        assert_eq!(report.total_successes, 50);
    }
}
