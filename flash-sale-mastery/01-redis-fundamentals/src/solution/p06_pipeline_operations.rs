//! # Solution 06: Redis Pipeline Operations
//!
//! Complete implementation of Redis pipelining for batch operations.

use deadpool_redis::redis::RedisError;
use deadpool_redis::Connection as RedisConnection;

/// Error type for pipeline operations.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Pipeline result mismatch: expected {expected} results, got {actual}")]
    ResultMismatch { expected: usize, actual: usize },
}

/// Fetch stock for multiple products in a single pipeline.
pub async fn batch_get_stock(
    conn: &mut RedisConnection,
    product_ids: &[String],
) -> Result<Vec<i64>, PipelineError> {
    let mut pipeline = deadpool_redis::redis::Pipeline::new();
    for id in product_ids {
        pipeline.cmd("GET").arg(id.as_str());
    }
    let results: Vec<Option<i64>> = pipeline.query_async(&mut *conn).await?;
    Ok(results.into_iter().map(|v| v.unwrap_or(0)).collect())
}

/// Set stock for multiple products in a single pipeline.
pub async fn batch_set_stock(
    conn: &mut RedisConnection,
    items: &[(String, i64)],
) -> Result<(), PipelineError> {
    let mut pipeline = deadpool_redis::redis::Pipeline::new();
    for (key, value) in items {
        pipeline.cmd("SET").arg(key.as_str()).arg(value);
    }
    let _: Vec<()> = pipeline.query_async(&mut *conn).await?;
    Ok(())
}

/// Increment multiple counters in a single pipeline.
pub async fn batch_increment(
    conn: &mut RedisConnection,
    keys: &[String],
    increment: i64,
) -> Result<Vec<i64>, PipelineError> {
    let mut pipeline = deadpool_redis::redis::Pipeline::new();
    for key in keys {
        pipeline.cmd("INCRBY").arg(key.as_str()).arg(increment);
    }
    let results: Vec<i64> = pipeline.query_async(&mut *conn).await?;
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::Config;

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
        let stocks = batch_get_stock(&mut conn, &keys).await.unwrap();
        assert_eq!(stocks, vec![0, 0, 0]);
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
        let vals = batch_increment(&mut conn, &keys, 5).await.unwrap();
        assert_eq!(vals, vec![5, 5, 5, 5]);
        let vals = batch_increment(&mut conn, &keys, 3).await.unwrap();
        assert_eq!(vals, vec![8, 8, 8, 8]);
    }
}
