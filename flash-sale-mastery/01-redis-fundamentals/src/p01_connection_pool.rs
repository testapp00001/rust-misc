//! # Exercise 01: Redis Connection Pool
//!
//! ## Learning Objective
//! Learn how to create and manage a Redis connection pool using `deadpool-redis`.
//! Connection pooling is essential for flash sale systems where thousands of
//! concurrent requests need fast Redis access without the overhead of creating
//! a new connection per request.
//!
//! ## Flash Sale Context
//! During a flash sale, the API layer may handle 10,000+ requests per second.
//! Each request needs to check stock, record claims, and update counters in Redis.
//! Creating a fresh TCP connection for each request would be catastrophic for
//! latency. A connection pool pre-establishes connections and lends them out
//! on demand.
//!
//! ## Instructions
//! 1. Implement `create_pool` to build a `deadpool_redis::Pool` from a Redis URL
//! 2. Implement `get_connection` to acquire a pooled connection
//! 3. Implement `pool_status` to report current pool health
//!
//! ## Hints
//! - Use `deadpool_redis::Config` to configure the pool
//! - The `Pool::builder()` method lets you set max size
//! - `pool.get()` returns a `Result<PooledConnection, _>`

use deadpool_redis::redis::cmd;
use deadpool_redis::{Config, Pool, Runtime};
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

/// Create a new Redis connection pool.
///
/// # Arguments
/// * `url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
///
/// # Returns
/// A configured `Pool` ready to serve connections.
pub fn create_pool(url: &str) -> Result<Pool, PoolError> {
    // TODO: Create a deadpool_redis Config from the URL
    // TODO: Build a Pool with a max size of 16 connections
    // TODO: Return the pool or a PoolError::CreationFailed
    todo!("Implement connection pool creation")
}

/// Acquire a connection from the pool.
///
/// # Arguments
/// * `pool` - Reference to the connection pool
///
/// # Returns
/// A pooled connection that auto-returns to the pool when dropped.
pub async fn get_connection(pool: &Pool) -> Result<RedisConnection, PoolError> {
    // TODO: Call pool.get() to acquire a connection
    // TODO: Map any error to PoolError::ConnectionFailed
    todo!("Implement connection acquisition")
}

/// Get pool status information.
///
/// # Arguments
/// * `pool` - Reference to the connection pool
///
/// # Returns
/// A tuple of (available_connections, max_size).
pub fn pool_status(pool: &Pool) -> (usize, usize) {
    // TODO: Use pool.status() to get current pool metrics
    // TODO: Return (available, max_size)
    todo!("Implement pool status reporting")
}

#[cfg(test)]
mod tests {
    use super::*;

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn try_connect() -> Result<Pool, String> {
        create_pool(REDIS_URL).map_err(|e| format!("Could not create pool: {e}"))
    }

    /// Test that a pool can be created from a valid Redis URL.
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
        assert!(
            available <= max_size,
            "Available should not exceed max size"
        );
    }

    /// Test that a connection can be acquired and used for a PING.
    #[tokio::test]
    async fn test_connection_acquisition() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = get_connection(&pool)
            .await
            .expect("Should acquire connection");
        let result: String = cmd("PING")
            .query_async(&mut *conn)
            .await
            .expect("PING should succeed");
        assert_eq!(result, "PONG", "Redis should respond with PONG");
    }

    /// Test that pool status reflects usage after acquiring a connection.
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
        let _conn = get_connection(&pool)
            .await
            .expect("Should acquire connection");
        let (after, _) = pool_status(&pool);
        // After acquiring, available should decrease (or stay same if recycled)
        // The key assertion is that status is reportable
        assert!(before <= max_size);
        assert!(after <= max_size);
    }
}
