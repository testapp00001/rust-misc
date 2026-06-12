//! # Solution 02: Redis Idempotency Store
//!
//! Complete implementation of a Redis-based idempotency store using SET NX.

use deadpool_redis::{Config, Pool};
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
    /// Uses Redis SET NX to atomically check-and-set.
    pub async fn check_and_store(
        &self,
        key: &str,
        ttl_seconds: u64,
    ) -> Result<IdempotencyStatus<CachedResult>, IdempotencyStoreError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| IdempotencyStoreError::Command(e.to_string()))?;

        // SET NX: returns true if key was set (new), false if key exists
        let was_set: bool = redis::cmd("SET")
            .arg(key)
            .arg("processing")
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut *conn)
            .await
            .map_err(|e| IdempotencyStoreError::Command(e.to_string()))?;

        if was_set {
            // Key was newly set -- this is a first-time request
            Ok(IdempotencyStatus::FirstTime)
        } else {
            // Key already exists -- fetch the cached value
            let value: Option<String> = conn
                .get(key)
                .await
                .map_err(|e| IdempotencyStoreError::Command(e.to_string()))?;

            match value {
                Some(v) if v == "processing" => {
                    // Still being processed by another request.
                    // Treat as duplicate (the other request will store the result).
                    // In a production system, you might want to wait/retry here.
                    Ok(IdempotencyStatus::Duplicate(CachedResult {
                        status_code: 202,
                        body: r#"{"status":"processing"}"#.to_string(),
                    }))
                }
                Some(v) => {
                    // Has a cached result
                    let result: CachedResult = serde_json::from_str(&v)?;
                    Ok(IdempotencyStatus::Duplicate(result))
                }
                None => {
                    // Key expired between SET NX and GET -- treat as new
                    // This is a rare race but possible with very short TTLs
                    Ok(IdempotencyStatus::FirstTime)
                }
            }
        }
    }

    /// Store the result for an idempotency key after processing.
    pub async fn store_result(
        &self,
        key: &str,
        result: &CachedResult,
        ttl_seconds: u64,
    ) -> Result<(), IdempotencyStoreError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| IdempotencyStoreError::Command(e.to_string()))?;

        let serialized = serde_json::to_string(result)?;

        // SET with TTL (overwrites the "processing" placeholder)
        let _: () = redis::cmd("SET")
            .arg(key)
            .arg(&serialized)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut *conn)
            .await
            .map_err(|e| IdempotencyStoreError::Command(e.to_string()))?;

        Ok(())
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

        let status1 = store
            .check_and_store(&key, 60)
            .await
            .expect("First check should succeed");
        assert_eq!(status1, IdempotencyStatus::FirstTime);

        let cached = CachedResult {
            status_code: 200,
            body: r#"{"voucher":"V-123"}"#.to_string(),
        };
        store
            .store_result(&key, &cached, 60)
            .await
            .expect("store_result should succeed");

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

        let status1 = store
            .check_and_store(&key, 1)
            .await
            .expect("First check");
        assert_eq!(status1, IdempotencyStatus::FirstTime);

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

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
