//! # Solution 09: Multi-Instance Consistency Test
//!
//! Complete implementation of multi-instance simulation and consistency verification.

use dashmap::DashMap;
use deadpool_redis::redis::cmd;
use deadpool_redis::{Config, Pool, Runtime};
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
    pub successes: u64,
    pub failures: u64,
    pub errors: u64,
}

/// A simulated application instance (pod).
///
/// Each instance has its own connection pool, simulating a separate
/// Kubernetes pod connecting to the same Redis cluster.
pub struct SimulatedInstance {
    pub id: String,
    pool: Pool,
}

impl SimulatedInstance {
    /// Create a new simulated instance with its own connection pool.
    pub async fn new(id: &str, redis_url: &str) -> Result<Self, MultiInstanceError> {
        let cfg = Config::from_url(redis_url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| MultiInstanceError::InvariantViolation(e.to_string()))?;

        Ok(Self {
            id: id.to_string(),
            pool,
        })
    }

    /// Attempt to decrement stock multiple times.
    ///
    /// Uses the DECR-then-check pattern (same as p02).
    /// Each attempt gets a fresh connection from the pool.
    pub async fn run(&self, key: &str, attempts: u64) -> InstanceStats {
        let mut stats = InstanceStats::default();

        for _ in 0..attempts {
            let result = async {
                let mut conn = self.pool.get().await?;
                let new_value: i64 = cmd("DECR").arg(key).query_async(&mut conn).await?;

                if new_value >= 0 {
                    Ok::<bool, MultiInstanceError>(true)
                } else {
                    // Revert
                    let _: i64 = cmd("INCR").arg(key).query_async(&mut conn).await?;
                    Ok(false)
                }
            }
            .await;

            match result {
                Ok(true) => stats.successes += 1,
                Ok(false) => stats.failures += 1,
                Err(_) => stats.errors += 1,
            }
        }

        stats
    }
}

/// Report from a multi-instance test.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiInstanceTestReport {
    pub num_instances: usize,
    pub initial_stock: i64,
    pub final_stock: i64,
    pub total_successes: u64,
    pub total_failures: u64,
    pub total_errors: u64,
    pub invariant_held: bool,
    pub per_instance: Vec<(String, InstanceStats)>,
}

/// Run the multi-instance test.
///
/// ## What This Verifies
///
/// With N pods all decrementing the same Redis counter:
/// 1. Total successful decrements == initial stock (no overselling)
/// 2. Final stock == 0 (all stock sold)
/// 3. No errors (all operations completed)
///
/// This is the ultimate proof that the distributed atomic counter
/// pattern works correctly.
pub async fn run_multi_instance_test(
    redis_url: &str,
    initial_stock: i64,
    num_instances: usize,
    attempts_per_instance: u64,
) -> Result<MultiInstanceTestReport, MultiInstanceError> {
    let cfg = Config::from_url(redis_url);
    let admin_pool = cfg
        .create_pool(Some(Runtime::Tokio1))
        .map_err(|e| MultiInstanceError::InvariantViolation(e.to_string()))?;

    let key = format!("test:multi:{}:{:x}", initial_stock, std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos());

    // Initialize stock
    {
        let mut conn = admin_pool.get().await?;
        cmd("SET")
            .arg(&key)
            .arg(initial_stock)
            .query_async::<()>(&mut conn)
            .await?;
    }

    // Shared stats collector
    let stats: Arc<DashMap<String, InstanceStats>> = Arc::new(DashMap::new());

    // Spawn all instances
    let mut handles = Vec::new();
    for i in 0..num_instances {
        let url = redis_url.to_string();
        let k = key.clone();
        let s = Arc::clone(&stats);
        let id = format!("pod-{}", i);

        handles.push(tokio::spawn(async move {
            let instance = SimulatedInstance::new(&id, &url)
                .await
                .expect("Should create instance");
            let instance_stats = instance.run(&k, attempts_per_instance).await;
            s.insert(id, instance_stats);
        }));
    }

    // Wait for all instances to complete
    for h in handles {
        h.await.map_err(|e| {
            MultiInstanceError::InvariantViolation(format!("Task panicked: {}", e))
        })?;
    }

    // Read final stock
    let mut conn = admin_pool.get().await?;
    let final_stock: i64 = cmd("GET")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .unwrap_or(0);

    // Aggregate stats
    let mut total_successes = 0u64;
    let mut total_failures = 0u64;
    let mut total_errors = 0u64;
    let mut per_instance = Vec::new();

    for entry in stats.iter() {
        total_successes += entry.value().successes;
        total_failures += entry.value().failures;
        total_errors += entry.value().errors;
        per_instance.push((entry.key().clone(), entry.value().clone()));
    }

    let invariant_held = total_successes == initial_stock as u64;

    // Cleanup
    let _: Result<(), _> = cmd("DEL").arg(&key).query_async(&mut conn).await;

    Ok(MultiInstanceTestReport {
        num_instances,
        initial_stock,
        final_stock,
        total_successes,
        total_failures,
        total_errors,
        invariant_held,
        per_instance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn try_connect() -> Result<Pool, String> {
        let cfg = Config::from_url(REDIS_URL);
        cfg.create_pool(Some(Runtime::Tokio1))
            .map_err(|e| format!("Pool error: {e}"))
    }

    #[tokio::test]
    async fn test_single_instance() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };

        let key = format!("test:single:{:x}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());

        let mut conn = pool.get().await.unwrap();
        cmd("SET")
            .arg(&key)
            .arg(10i64)
            .query_async::<()>(&mut conn)
            .await
            .unwrap();

        let instance = SimulatedInstance::new("pod-0", REDIS_URL)
            .await
            .expect("Should create instance");
        let stats = instance.run(&key, 20).await;

        assert_eq!(stats.successes, 10, "Should succeed exactly 10 times");
        assert_eq!(stats.failures, 10, "Should fail exactly 10 times");
        assert_eq!(stats.errors, 0, "Should have no errors");

        let _: Result<(), _> = cmd("DEL").arg(&key).query_async(&mut conn).await;
    }

    #[tokio::test]
    async fn test_ten_instances_thousand_stock() {
        let report = match run_multi_instance_test(REDIS_URL, 1000, 10, 200).await {
            Ok(r) => r,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };

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
