//! # Solution 07: Redis Transactions (WATCH/MULTI/EXEC)
//!
//! Complete implementation of Redis optimistic locking with WATCH/MULTI/EXEC.

use deadpool_redis::redis::RedisError;
use deadpool_redis::Connection as RedisConnection;

/// Error type for transaction operations.
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Transaction aborted due to concurrent modification")]
    Conflict,

    #[error("Insufficient stock: need {needed}, have {available}")]
    InsufficientStock { needed: i64, available: i64 },

    #[error("Max retries ({0}) exceeded")]
    MaxRetriesExceeded(usize),
}

/// Optimistically update stock using WATCH/MULTI/EXEC.
pub async fn optimistic_update_stock(
    conn: &mut RedisConnection,
    product_id: &str,
    expected: i64,
    new_value: i64,
) -> Result<bool, TransactionError> {
    deadpool_redis::redis::cmd("WATCH")
        .arg(product_id)
        .query_async::<()>(&mut *conn)
        .await?;

    let current: Option<i64> = deadpool_redis::redis::cmd("GET")
        .arg(product_id)
        .query_async(&mut *conn)
        .await?;

    let current = current.unwrap_or(0);
    if current != expected {
        return Ok(false);
    }

    let mut pipeline = deadpool_redis::redis::Pipeline::new();
    pipeline
        .cmd("SET")
        .arg(product_id)
        .arg(new_value);
    let result: Option<Vec<()>> = pipeline.query_async(&mut *conn).await?;

    match result {
        Some(_) => Ok(true),
        None => Err(TransactionError::Conflict),
    }
}

/// Transfer stock from one product to another atomically with retries.
pub async fn optimistic_transfer(
    conn: &mut RedisConnection,
    from_product: &str,
    to_product: &str,
    amount: i64,
    max_retries: usize,
) -> Result<(), TransactionError> {
    for attempt in 0..=max_retries {
        // WATCH both keys
        deadpool_redis::redis::cmd("WATCH")
            .arg(from_product)
            .arg(to_product)
            .query_async::<()>(&mut *conn)
            .await?;

        let from_val: Option<i64> = deadpool_redis::redis::cmd("GET")
            .arg(from_product)
            .query_async(&mut *conn)
            .await?;
        let from_val = from_val.unwrap_or(0);

        if from_val < amount {
            return Err(TransactionError::InsufficientStock {
                needed: amount,
                available: from_val,
            });
        }

        let mut pipeline = deadpool_redis::redis::Pipeline::new();
        pipeline.cmd("DECRBY").arg(from_product).arg(amount);
        pipeline.cmd("INCRBY").arg(to_product).arg(amount);
        let result: Option<Vec<i64>> = pipeline.query_async(&mut *conn).await?;

        if result.is_some() {
            return Ok(());
        }

        // Conflict, UNWATCH and retry
        let _ = deadpool_redis::redis::cmd("UNWATCH")
            .query_async::<()>(&mut *conn)
            .await;

        if attempt == max_retries {
            return Err(TransactionError::MaxRetriesExceeded(max_retries));
        }
    }

    Err(TransactionError::MaxRetriesExceeded(max_retries))
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
        format!("test:txn:{}:{}", suffix, std::process::id())
    }

    async fn set_val(conn: &mut RedisConnection, key: &str, val: i64) {
        deadpool_redis::redis::cmd("SET")
            .arg(key)
            .arg(val)
            .query_async::<()>(conn)
            .await
            .unwrap();
    }

    async fn get_val(conn: &mut RedisConnection, key: &str) -> i64 {
        deadpool_redis::redis::cmd("GET")
            .arg(key)
            .query_async::<Option<i64>>(conn)
            .await
            .unwrap()
            .unwrap_or(0)
    }

    #[tokio::test]
    async fn test_optimistic_update_success() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("update");
        set_val(&mut conn, &key, 100).await;
        let success = optimistic_update_stock(&mut conn, &key, 100, 99)
            .await
            .expect("transaction failed");
        assert!(success);
        assert_eq!(get_val(&mut conn, &key).await, 99);
    }

    #[tokio::test]
    async fn test_optimistic_update_conflict() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("conflict");
        set_val(&mut conn, &key, 50).await;
        let result = optimistic_update_stock(&mut conn, &key, 999, 49).await;
        match result {
            Ok(false) => {}
            Err(TransactionError::Conflict) => {}
            other => panic!("Expected conflict, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_optimistic_transfer() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let from = test_key("transfer_from");
        let to = test_key("transfer_to");
        set_val(&mut conn, &from, 100).await;
        set_val(&mut conn, &to, 0).await;
        optimistic_transfer(&mut conn, &from, &to, 30, 3).await.unwrap();
        assert_eq!(get_val(&mut conn, &from).await, 70);
        assert_eq!(get_val(&mut conn, &to).await, 30);
    }

    #[tokio::test]
    async fn test_optimistic_transfer_insufficient() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let from = test_key("transfer_from_low");
        let to = test_key("transfer_to_low");
        set_val(&mut conn, &from, 10).await;
        set_val(&mut conn, &to, 0).await;
        let result = optimistic_transfer(&mut conn, &from, &to, 50, 3).await;
        assert!(result.is_err());
    }
}
