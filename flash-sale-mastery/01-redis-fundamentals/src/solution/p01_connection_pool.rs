//! # Solution 01: Redis Connection Pool
//!
//! Complete implementation of Redis connection pool management using deadpool-redis.

use deadpool_redis::{Config, Pool};
use deadpool_redis::Connection as RedisConnection;

/// Custom error type for connection pool operations.
#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("Failed to create pool: {0}")]
    CreationFailed(String),

    #[error("Failed to acquire connection: {0}")]
    ConnectionFailed(String),

    #[error("Pool is exhausted, no connections available")]
    PoolExhausted,
}

/// Create a new Redis connection pool with up to 16 connections.
pub fn create_pool(url: &str) -> Result<Pool, PoolError> {
    let cfg = Config::from_url(url);
    let pool = cfg
        .builder()
        .map_err(|e| PoolError::CreationFailed(e.to_string()))?
        .max_size(16)
        .build()
        .map_err(|e| PoolError::CreationFailed(e.to_string()))?;
    Ok(pool)
}

/// Acquire a connection from the pool.
pub async fn get_connection(pool: &Pool) -> Result<RedisConnection, PoolError> {
    pool.get()
        .await
        .map_err(|e| PoolError::ConnectionFailed(e.to_string()))
}

/// Get pool status: (available_connections, max_size).
pub fn pool_status(pool: &Pool) -> (usize, usize) {
    let status = pool.status();
    (status.available, status.max_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn try_connect() -> Result<Pool, String> {
        create_pool(REDIS_URL).map_err(|e| format!("Could not create pool: {e}"))
    }

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let (available, max_size) = pool_status(&pool);
        assert!(max_size > 0, "Pool max size should be positive");
        assert!(available <= max_size, "Available should not exceed max size");
    }

    #[tokio::test]
    async fn test_connection_acquisition() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = match get_connection(&pool).await {
            Ok(c) => c,
            Err(_) => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let result: String = deadpool_redis::redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .expect("PING should succeed");
        assert_eq!(result, "PONG");
    }

    #[tokio::test]
    async fn test_pool_status_after_use() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let (before, max_size) = pool_status(&pool);
        let _conn = match get_connection(&pool).await {
            Ok(c) => c,
            Err(_) => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let (after, _) = pool_status(&pool);
        assert!(before <= max_size);
        assert!(after <= max_size);
    }
}
