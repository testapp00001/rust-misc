//! # Exercise 07: Redis Transactions (WATCH/MULTI/EXEC)
//!
//! ## Learning Objective
//! Understand Redis optimistic locking using WATCH, MULTI, and EXEC to ensure
//! atomic read-modify-write operations without Lua scripting.
//!
//! ## Flash Sale Context
//! When updating stock, we need to read the current value, check it's positive,
//! and decrement it -- all atomically. If two requests read stock=1 simultaneously,
//! both might try to decrement, causing overselling. WATCH lets us detect when
//! another client modified the key between our read and write, causing the
//! transaction to abort and retry.
//!
//! ## Instructions
//! 1. Implement `optimistic_update_stock` using WATCH/MULTI/EXEC
//! 2. Implement `optimistic_transfer` to move stock between two keys atomically
//!
//! ## Hints
//! - WATCH a key before reading it
//! - After WATCH, use MULTI to start a transaction queue commands
//! - EXEC executes all queued commands atomically, returns None if WATCH failed
//! - If EXEC returns None, retry the whole operation

use deadpool_redis::Connection as RedisConnection;

/// Error type for transaction operations.
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

    #[error("Transaction aborted due to concurrent modification")]
    Conflict,

    #[error("Insufficient stock: need {needed}, have {available}")]
    InsufficientStock { needed: i64, available: i64 },

    #[error("Max retries ({0}) exceeded")]
    MaxRetriesExceeded(usize),
}

/// Optimistically update stock using WATCH/MULTI/EXEC.
///
/// Reads current stock, checks it's >= `expected`, then sets it to `new_value`.
/// If another client modified the key between WATCH and EXEC, the transaction
/// aborts and returns Err(Conflict).
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Stock key
/// * `expected` - Expected current value (transaction aborts if mismatch)
/// * `new_value` - New value to set
pub async fn optimistic_update_stock(
    conn: &mut RedisConnection,
    product_id: &str,
    expected: i64,
    new_value: i64,
) -> Result<bool, TransactionError> {
    // TODO: WATCH the key
    // TODO: GET the current value
    // TODO: If current != expected, return Ok(false) or Err(Conflict)
    // TODO: MULTI, SET new_value, EXEC
    // TODO: If EXEC returns None (watch failed), return Err(Conflict)
    todo!("Implement optimistic_update_stock")
}

/// Transfer stock from one product to another atomically.
/// Decrements source by `amount` and increments destination.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `from_product` - Source product key
/// * `to_product` - Destination product key
/// * `amount` - Amount to transfer
/// * `max_retries` - Maximum retry attempts on conflict
pub async fn optimistic_transfer(
    conn: &mut RedisConnection,
    from_product: &str,
    to_product: &str,
    amount: i64,
    max_retries: usize,
) -> Result<(), TransactionError> {
    // TODO: WATCH both keys
    // TODO: Read both values
    // TODO: Check source has enough stock
    // TODO: MULTI, DECRBY source, INCRBY dest, EXEC
    // TODO: Retry on conflict up to max_retries
    todo!("Implement optimistic_transfer")
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
        assert!(success, "Transaction should succeed with correct expected value");
        let val = get_val(&mut conn, &key).await;
        assert_eq!(val, 99);
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
        // Try to update with wrong expected value
        let result = optimistic_update_stock(&mut conn, &key, 999, 49).await;
        // Should either return Ok(false) or Err(Conflict) depending on impl
        match result {
            Ok(false) => {} // Expected: value mismatch detected
            Err(TransactionError::Conflict) => {} // Also acceptable
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
        optimistic_transfer(&mut conn, &from, &to, 30, 3)
            .await
            .expect("transfer failed");
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
        assert!(result.is_err(), "Should fail with insufficient stock");
    }
}
