//! # Exercise 06: Redis Pipeline Operations
//!
//! ## Learning Objective
//! Learn to use Redis pipelining to batch multiple commands into a single
//! network round-trip, dramatically improving throughput for bulk operations.
//!
//! ## Flash Sale Context
//! When displaying a flash sale page, the system needs to fetch stock for
//! dozens of products. Sending each GET as a separate round-trip is wasteful.
//! Pipelining bundles all commands together: the client sends all requests at
//! once, and Redis processes them in order, returning all results together.
//! This reduces latency from N round-trips to 1.
//!
//! ## Instructions
//! 1. Implement `batch_get_stock` to fetch stock for multiple products in one pipeline
//! 2. Implement `batch_set_stock` to set stock for multiple products in one pipeline
//! 3. Implement `batch_increment` to increment multiple counters atomically
//!
//! ## Hints
//! - Create a pipeline with `redis::Pipeline::new()`
//! - Add commands with `pipeline.cmd("GET").arg(key)`
//! - Execute with `pipeline.query_async(&mut *conn)`
//! - Results come back as a Vec in the same order as commands

use deadpool_redis::Connection as RedisConnection;

/// Error type for pipeline operations.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

    #[error("Pipeline result mismatch: expected {expected} results, got {actual}")]
    ResultMismatch { expected: usize, actual: usize },
}

/// Fetch stock for multiple products in a single pipeline.
/// Returns a Vec of stock values in the same order as product_ids.
/// Missing keys return 0.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_ids` - List of product keys to fetch
pub async fn batch_get_stock(
    conn: &mut RedisConnection,
    product_ids: &[String],
) -> Result<Vec<i64>, PipelineError> {
    // TODO: Create a redis::Pipeline
    // TODO: Add a GET command for each product_id
    // TODO: Execute the pipeline and collect results
    // TODO: Handle Nil (missing) values as 0
    todo!("Implement batch_get_stock")
}

/// Set stock for multiple products in a single pipeline.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `items` - Slice of (product_id, stock_count) pairs
pub async fn batch_set_stock(
    conn: &mut RedisConnection,
    items: &[(String, i64)],
) -> Result<(), PipelineError> {
    // TODO: Create a pipeline with SET commands for each item
    // TODO: Execute the pipeline
    todo!("Implement batch_set_stock")
}

/// Increment multiple counters in a single pipeline.
/// Returns the new values after incrementing.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `keys` - Counter keys to increment
/// * `increment` - Amount to increment each by
pub async fn batch_increment(
    conn: &mut RedisConnection,
    keys: &[String],
    increment: i64,
) -> Result<Vec<i64>, PipelineError> {
    // TODO: Create a pipeline with INCRBY commands
    // TODO: Execute and return new values
    todo!("Implement batch_increment")
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn get_conn() -> Option<RedisConnection> {
        let cfg = Config::from_url(REDIS_URL);
        let pool = cfg.builder().ok()?.build().ok()?;
        pool.get().await.ok()
    }

    fn test_key(suffix: &str) -> String {
        format!("test:pipeline:{}:{}", suffix, std::process::id())
    }

    #[tokio::test]
    async fn test_batch_get_stock() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let prefix = test_key("batch_get");
        // Set up test data
        let items: Vec<(String, i64)> = (0..5)
            .map(|i| (format!("{prefix}:{i}"), (i + 1) * 10))
            .collect();
        batch_set_stock(&mut conn, &items).await.expect("batch set failed");

        let keys: Vec<String> = items.iter().map(|(k, _)| k.clone()).collect();
        let stocks = batch_get_stock(&mut conn, &keys).await.expect("batch get failed");
        assert_eq!(stocks.len(), 5);
        for (i, stock) in stocks.iter().enumerate() {
            assert_eq!(*stock, (i as i64 + 1) * 10);
        }
    }

    #[tokio::test]
    async fn test_batch_get_missing_keys() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let keys: Vec<String> = (0..3)
            .map(|i| format!("test:pipeline:missing:{i}:{}", std::process::id()))
            .collect();
        let stocks = batch_get_stock(&mut conn, &keys).await.expect("batch get failed");
        assert_eq!(stocks, vec![0, 0, 0], "Missing keys should return 0");
    }

    #[tokio::test]
    async fn test_batch_increment() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let prefix = test_key("batch_incr");
        let keys: Vec<String> = (0..4).map(|i| format!("{prefix}:{i}")).collect();
        // First increment from 0
        let vals = batch_increment(&mut conn, &keys, 5)
            .await
            .expect("batch incr failed");
        assert_eq!(vals, vec![5, 5, 5, 5]);
        // Second increment
        let vals = batch_increment(&mut conn, &keys, 3)
            .await
            .expect("batch incr failed");
        assert_eq!(vals, vec![8, 8, 8, 8]);
    }
}
