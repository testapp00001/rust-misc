//! # Exercise 03: Database Idempotency Store
//!
//! ## Learning Objective
//! Implement a durable idempotency store using a relational database's unique
//! constraint. Unlike Redis (which is volatile), a database store survives
//! restarts and provides ACID guarantees. The trade-off is higher latency.
//!
//! ## Flash Sale Context
//! Redis is the fast first-check, but what if Redis loses data on a restart
//! mid-sale? The database is the authoritative record. If Redis says "new" but
//! the DB says "duplicate", the DB wins. This exercise builds the durable layer.
//!
//! ## Instructions
//! 1. Create an in-memory SQLite database with the idempotency_keys table
//! 2. Implement `check_and_store` using INSERT with unique constraint
//! 3. Handle the unique constraint violation as a "duplicate" signal
//! 4. Implement `store_result` to update the cached result after processing
//!
//! ## Hints
//! - Use `sqlx::SqlitePool::connect("sqlite::memory:")` for testing
//! - The table schema: `key TEXT PRIMARY KEY, result TEXT, created_at TEXT, expires_at TEXT`
//! - `INSERT ... ON CONFLICT DO NOTHING` returns 0 rows affected for duplicates
//! - Alternatively, try INSERT first and catch the constraint violation
//! - Use `sqlx::query!` or `sqlx::query` for SQL execution

use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// Status returned when checking an idempotency key.
#[derive(Debug, Clone, PartialEq)]
pub enum IdempotencyStatus<T> {
    /// This is the first time we've seen this key.
    FirstTime,
    /// This key was seen before. The cached result is returned.
    Duplicate(T),
}

/// Result cached for an idempotency key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CachedResult {
    pub status_code: u16,
    pub body: String,
}

/// Database-based durable idempotency store.
pub struct DbIdempotencyStore {
    pool: SqlitePool,
}

impl DbIdempotencyStore {
    /// Create a new store and initialize the schema.
    ///
    /// Creates the `idempotency_keys` table if it doesn't exist.
    pub async fn new(pool: SqlitePool) -> Result<Self, DbIdempotencyStoreError> {
        // TODO: Execute CREATE TABLE IF NOT EXISTS for idempotency_keys
        // Schema: key TEXT PRIMARY KEY, result TEXT, created_at TEXT, expires_at TEXT
        todo!("Initialize the idempotency_keys table")
    }

    /// Check if an idempotency key exists and atomically insert it if not.
    ///
    /// Uses the database's unique constraint on the key column to ensure
    /// atomicity. If the INSERT succeeds, this is a new key. If it fails
    /// with a unique constraint violation, the key already exists.
    ///
    /// # Arguments
    /// * `key` - The idempotency key
    /// * `ttl_seconds` - How long to remember this key
    ///
    /// # Returns
    /// `FirstTime` if the key was inserted, `Duplicate(cached)` if it existed.
    pub async fn check_and_store(
        &self,
        key: &str,
        ttl_seconds: u64,
    ) -> Result<IdempotencyStatus<CachedResult>, DbIdempotencyStoreError> {
        // TODO: Try to INSERT a new row with the key
        //   - Use a placeholder result (NULL or empty) for first-time
        //   - Set created_at to current time, expires_at to now + ttl
        // TODO: If INSERT succeeds -> return FirstTime
        // TODO: If INSERT fails with unique constraint -> SELECT the existing result
        //   - If the row is expired, delete it and retry
        //   - Otherwise, deserialize and return Duplicate(result)
        todo!("Implement check_and_store with INSERT ON CONFLICT")
    }

    /// Store the result for an idempotency key after processing.
    ///
    /// Updates the cached result for a key that was previously inserted
    /// by `check_and_store`.
    pub async fn store_result(
        &self,
        key: &str,
        result: &CachedResult,
    ) -> Result<(), DbIdempotencyStoreError> {
        // TODO: UPDATE the row for this key with the serialized result
        todo!("Implement store_result UPDATE")
    }
}

/// Errors from the database idempotency store.
#[derive(Debug, thiserror::Error)]
pub enum DbIdempotencyStoreError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_store() -> DbIdempotencyStore {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create SQLite pool");
        DbIdempotencyStore::new(pool)
            .await
            .expect("Failed to initialize store")
    }

    /// Test that a first-time key is recognized as new.
    #[tokio::test]
    async fn test_first_time_request() {
        let store = create_store().await;
        let status = store
            .check_and_store("key-001", 3600)
            .await
            .expect("check_and_store should succeed");
        assert_eq!(status, IdempotencyStatus::FirstTime);
    }

    /// Test that a duplicate request returns the cached result.
    #[tokio::test]
    async fn test_duplicate_returns_cached() {
        let store = create_store().await;

        // First request
        let status1 = store
            .check_and_store("key-002", 3600)
            .await
            .expect("First check");
        assert_eq!(status1, IdempotencyStatus::FirstTime);

        // Store a result
        let cached = CachedResult {
            status_code: 201,
            body: r#"{"voucher":"V-456"}"#.to_string(),
        };
        store
            .store_result("key-002", &cached)
            .await
            .expect("store_result");

        // Second request
        let status2 = store
            .check_and_store("key-002", 3600)
            .await
            .expect("Second check");
        match status2 {
            IdempotencyStatus::Duplicate(result) => {
                assert_eq!(result.status_code, 201);
                assert_eq!(result.body, r#"{"voucher":"V-456"}"#);
            }
            other => panic!("Expected Duplicate, got: {other:?}"),
        }
    }

    /// Test that different keys are independent.
    #[tokio::test]
    async fn test_different_keys_independent() {
        let store = create_store().await;

        let s1 = store
            .check_and_store("key-a", 3600)
            .await
            .expect("key-a first");
        let s2 = store
            .check_and_store("key-b", 3600)
            .await
            .expect("key-b first");

        assert_eq!(s1, IdempotencyStatus::FirstTime);
        assert_eq!(s2, IdempotencyStatus::FirstTime);
    }

    /// Test that the store persists across multiple operations (simulating restart).
    #[tokio::test]
    async fn test_persistence_across_operations() {
        let store = create_store().await;

        // Insert and store result
        store
            .check_and_store("persist-key", 3600)
            .await
            .expect("insert");
        let cached = CachedResult {
            status_code: 200,
            body: "persisted".to_string(),
        };
        store
            .store_result("persist-key", &cached)
            .await
            .expect("store");

        // Simulate a "restart" by creating a new store with the same pool
        // (In a real scenario, you'd reconnect to the same database)
        let status = store
            .check_and_store("persist-key", 3600)
            .await
            .expect("re-check");
        match status {
            IdempotencyStatus::Duplicate(result) => {
                assert_eq!(result.body, "persisted");
            }
            other => panic!("Expected Duplicate after persistence, got: {other:?}"),
        }
    }
}
