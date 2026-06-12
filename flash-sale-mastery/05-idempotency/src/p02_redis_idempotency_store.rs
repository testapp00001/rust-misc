//! # Exercise 02: Redis Idempotency Store
//!
//! ## Learning Objective
//! Implement a fast, in-memory idempotency store using Redis SET NX (set if not
//! exists). This is the first line of defense against duplicate requests in a
//! flash sale system. SET NX is atomic, meaning concurrent requests for the same
//! key are safely serialized by Redis.
//!
//! ## Flash Sale Context
//! During a flash sale, 10,000 requests/second hit the API. Before processing
//! any purchase, we check Redis: "Have I seen this idempotency key before?"
//! If SET NX succeeds (key did not exist), this is a new request -- proceed.
//! If SET NX fails (key already exists), this is a duplicate -- return the
//! cached result. The TTL ensures keys don't linger forever.
//!
//! ## Instructions
//! 1. Connect to Redis using deadpool-redis
//! 2. Implement `check_and_store` using Redis SET NX with a TTL
//! 3. For first-time requests: store the key with a placeholder, return `FirstTime`
//! 4. For duplicates: fetch the cached result, return `Duplicate(cached_result)`
//! 5. Implement `store_result` to update the cached result after processing
//!
//! ## Hints
//! - `redis::cmd("SET").arg(key).arg(value).arg("NX").arg("EX").arg(ttl)` for SET NX with TTL
//! - Use `serde_json` to serialize/deserialize cached results
//! - SET NX returns `Some(())` on success (key was set) and `None` if key exists
//! - Store a placeholder value initially, then update with the real result

use deadpool_redis::{Config, Pool, Runtime};
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};

/// Status returned when checking an idempotency key.
#[derive(Debug, Clone, PartialEq)]
pub enum IdempotencyStatus<T> {
    /// This is the first time we've seen this key. The caller should proceed.
    FirstTime,
    /// This key was seen before. The cached result is returned.
    Duplicate(T),
}

/// Result cached for an idempotency key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedResult {
    pub status_code: u16,
    pub body: String,
}

/// Redis-based idempotency store.
pub struct IdempotencyStore {
    pool: Pool,
}

impl IdempotencyStore {
    /// Create a new store backed by a Redis connection pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Create a store with a connection to the given Redis URL.
    pub fn from_url(url: &str) -> Result<Self, IdempotencyStoreError> {
        let cfg = Config::from_url(url);
        let pool = cfg
            .builder()
            .map_err(|e| IdempotencyStoreError::Connection(e.to_string()))?
            .max_size(16)
            .build()
            .map_err(|e| IdempotencyStoreError::Connection(e.to_string()))?;
        Ok(Self { pool })
    }

    /// Check if an idempotency key has been seen before, and atomically mark it
    /// as seen if it hasn't.
    ///
    /// Uses Redis SET NX to atomically check-and-set. If the key already exists,
    /// returns the cached result. If not, sets the key with a placeholder and
    /// returns `FirstTime`.
    ///
    /// # Arguments
    /// * `key` - The idempotency key to check
    /// * `ttl_seconds` - How long to remember this key (in seconds)
    ///
    /// # Returns
    /// `FirstTime` if this is a new key, `Duplicate(cached)` if seen before.
    pub async fn check_and_store(
        &self,
        key: &str,
        ttl_seconds: u64,
    ) -> Result<IdempotencyStatus<CachedResult>, IdempotencyStoreError> {
        // TODO: Get a connection from the pool
        // TODO: Use SET NX to atomically attempt to set the key
        //   - If SET NX succeeds (returns true), this is a new key -> return FirstTime
        //   - If SET NX fails (returns false), the key exists -> fetch cached result
        // TODO: For duplicates, GET the cached value and deserialize it
        // TODO: Return the appropriate IdempotencyStatus
        todo!("Implement check_and_store with SET NX")
    }

    /// Store the result for an idempotency key after processing.
    ///
    /// Call this after successfully processing a request to cache the result.
    /// Future duplicate requests will receive this cached result.
    ///
    /// # Arguments
    /// * `key` - The idempotency key
    /// * `result` - The result to cache
    /// * `ttl_seconds` - Remaining TTL (should match the original TTL)
    pub async fn store_result(
        &self,
        key: &str,
        result: &CachedResult,
        ttl_seconds: u64,
    ) -> Result<(), IdempotencyStoreError> {
        // TODO: Serialize the result to JSON
        // TODO: SET the key with the serialized result and TTL (overwrite the placeholder)
        todo!("Implement store_result")
    }
}

/// Errors from idempotency store operations.
#[derive(Debug, thiserror::Error)]
pub enum IdempotencyStoreError {
    #[error("Redis connection error: {0}")]
    Connection(String),

    #[error("Redis command error: {0}")]
    Command(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    fn test_key(suffix: &str) -> String {
        format!("idempotency:test:p02:{}:{}", std::process::id(), suffix)
    }

    async fn try_create_store() -> Result<IdempotencyStore, String> {
        IdempotencyStore::from_url(REDIS_URL).map_err(|e| format!("Could not create store: {e}"))
    }

    /// Test that a first-time request is recognized as new.
    #[tokio::test]
    async fn test_first_time_request() {
        let store = match try_create_store().await {
            Ok(s) => s,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key = test_key("first_time");
        let status = store
            .check_and_store(&key, 60)
            .await
            .expect("check_and_store should succeed");
        assert_eq!(status, IdempotencyStatus::FirstTime);
    }

    /// Test that a duplicate request returns the cached result.
    #[tokio::test]
    async fn test_duplicate_returns_cached() {
        let store = match try_create_store().await {
            Ok(s) => s,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key = test_key("duplicate");

        // First request
        let status1 = store
            .check_and_store(&key, 60)
            .await
            .expect("First check should succeed");
        assert_eq!(status1, IdempotencyStatus::FirstTime);

        // Store a result
        let cached = CachedResult {
            status_code: 200,
            body: r#"{"voucher":"V-123"}"#.to_string(),
        };
        store
            .store_result(&key, &cached, 60)
            .await
            .expect("store_result should succeed");

        // Second request -- should get the cached result
        let status2 = store
            .check_and_store(&key, 60)
            .await
            .expect("Second check should succeed");
        match status2 {
            IdempotencyStatus::Duplicate(result) => {
                assert_eq!(result.status_code, 200);
                assert_eq!(result.body, r#"{"voucher":"V-123"}"#);
            }
            other => panic!("Expected Duplicate, got: {other:?}"),
        }
    }

    /// Test that different keys are treated independently.
    #[tokio::test]
    async fn test_different_keys_independent() {
        let store = match try_create_store().await {
            Ok(s) => s,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key_a = test_key("independent_a");
        let key_b = test_key("independent_b");

        let status_a = store
            .check_and_store(&key_a, 60)
            .await
            .expect("check key_a");
        let status_b = store
            .check_and_store(&key_b, 60)
            .await
            .expect("check key_b");

        assert_eq!(status_a, IdempotencyStatus::FirstTime);
        assert_eq!(status_b, IdempotencyStatus::FirstTime);
    }

    /// Test that keys expire after TTL.
    #[tokio::test]
    async fn test_ttl_expiration() {
        let store = match try_create_store().await {
            Ok(s) => s,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key = test_key("ttl_expire");

        // Store with a 1-second TTL
        let status1 = store
            .check_and_store(&key, 1)
            .await
            .expect("First check");
        assert_eq!(status1, IdempotencyStatus::FirstTime);

        // Wait for TTL to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Should be treated as a new request
        let status2 = store
            .check_and_store(&key, 60)
            .await
            .expect("Check after expiry");
        assert_eq!(
            status2,
            IdempotencyStatus::FirstTime,
            "Expired key should be treated as new"
        );
    }
}
