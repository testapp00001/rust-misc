//! # Solution 02: Redis String Operations
//!
//! Complete implementation of Redis STRING operations for flash sale stock management.

use deadpool_redis::redis::RedisError;
use deadpool_redis::Connection as RedisConnection;

/// Error type for string operations.
#[derive(Debug, thiserror::Error)]
pub enum StockError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Stock went below zero: current={current}")]
    StockExhausted { current: i64 },
}

/// Set the stock count for a product.
pub async fn set_stock(
    conn: &mut RedisConnection,
    product_id: &str,
    count: i64,
) -> Result<(), StockError> {
    deadpool_redis::redis::cmd("SET")
        .arg(product_id)
        .arg(count)
        .query_async::<()>(&mut *conn)
        .await?;
    Ok(())
}

/// Get the current stock count for a product. Returns 0 if key is missing.
pub async fn get_stock(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64, StockError> {
    let val: Option<i64> = deadpool_redis::redis::cmd("GET")
        .arg(product_id)
        .query_async(&mut *conn)
        .await?;
    Ok(val.unwrap_or(0))
}

/// Atomically decrement stock by 1. Errors if stock would go below zero.
pub async fn decrement_stock(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64, StockError> {
    // Check current stock first to prevent going below zero
    let current = get_stock(conn, product_id).await?;
    if current <= 0 {
        return Err(StockError::StockExhausted { current });
    }
    let new_val: i64 = deadpool_redis::redis::cmd("DECR")
        .arg(product_id)
        .query_async(&mut *conn)
        .await?;
    Ok(new_val)
}

/// Set stock only if the key does not already exist (SETNX).
pub async fn set_stock_if_absent(
    conn: &mut RedisConnection,
    product_id: &str,
    count: i64,
) -> Result<bool, StockError> {
    let result: bool = deadpool_redis::redis::cmd("SET")
        .arg(product_id)
        .arg(count)
        .arg("NX")
        .query_async(&mut *conn)
        .await?;
    Ok(result)
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
        format!("test:stock:{}:{}", suffix, std::process::id())
    }

    #[tokio::test]
    async fn test_set_and_get_stock() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("set_get");
        set_stock(&mut conn, &key, 100).await.expect("set_stock failed");
        let stock = get_stock(&mut conn, &key).await.expect("get_stock failed");
        assert_eq!(stock, 100);
    }

    #[tokio::test]
    async fn test_get_stock_missing_key_returns_zero() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("missing");
        let stock = get_stock(&mut conn, &key).await.expect("get_stock failed");
        assert_eq!(stock, 0);
    }

    #[tokio::test]
    async fn test_decrement_stock() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("decr");
        set_stock(&mut conn, &key, 3).await.expect("set_stock failed");
        assert_eq!(decrement_stock(&mut conn, &key).await.unwrap(), 2);
        assert_eq!(decrement_stock(&mut conn, &key).await.unwrap(), 1);
        assert_eq!(decrement_stock(&mut conn, &key).await.unwrap(), 0);
        assert!(decrement_stock(&mut conn, &key).await.is_err());
    }

    #[tokio::test]
    async fn test_set_stock_if_absent() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("setnx");
        assert!(set_stock_if_absent(&mut conn, &key, 50).await.unwrap());
        assert!(!set_stock_if_absent(&mut conn, &key, 99).await.unwrap());
        assert_eq!(get_stock(&mut conn, &key).await.unwrap(), 50);
    }
}
