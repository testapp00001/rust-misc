//! # Solution 03: Database Idempotency Store
//!
//! Complete implementation of a durable idempotency store using SQLite.

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
    pub async fn new(pool: SqlitePool) -> Result<Self, DbIdempotencyStoreError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS idempotency_keys (
                key TEXT PRIMARY KEY,
                result TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    /// Check if an idempotency key exists and atomically insert it if not.
    pub async fn check_and_store(
        &self,
        key: &str,
        ttl_seconds: u64,
    ) -> Result<IdempotencyStatus<CachedResult>, DbIdempotencyStoreError> {
        // First, try to clean up expired keys for this specific key
        sqlx::query("DELETE FROM idempotency_keys WHERE key = ?1 AND expires_at < datetime('now')")
            .bind(key)
            .execute(&self.pool)
            .await?;

        // Try to insert a new row. If the key already exists (and is not expired),
        // the unique constraint will fail.
        let insert_result = sqlx::query(
            "INSERT INTO idempotency_keys (key, result, expires_at)
             VALUES (?1, NULL, datetime('now', ?2 || ' seconds'))
             ON CONFLICT(key) DO NOTHING",
        )
        .bind(key)
        .bind(ttl_seconds.to_string())
        .execute(&self.pool)
        .await?;

        if insert_result.rows_affected() > 0 {
            // Insert succeeded -- this is a new key
            Ok(IdempotencyStatus::FirstTime)
        } else {
            // Insert did nothing (conflict) -- key exists, fetch the result
            let row = sqlx::query("SELECT result FROM idempotency_keys WHERE key = ?1")
                .bind(key)
                .fetch_one(&self.pool)
                .await?;

            let result_text: Option<String> = row.get("result");
            match result_text {
                Some(text) if !text.is_empty() => {
                    let cached: CachedResult = serde_json::from_str(&text)?;
                    Ok(IdempotencyStatus::Duplicate(cached))
                }
                _ => {
                    // Key exists but no result stored yet (another request is processing)
                    // Return a "processing" indicator
                    Ok(IdempotencyStatus::Duplicate(CachedResult {
                        status_code: 202,
                        body: r#"{"status":"processing"}"#.to_string(),
                    }))
                }
            }
        }
    }

    /// Store the result for an idempotency key after processing.
    pub async fn store_result(
        &self,
        key: &str,
        result: &CachedResult,
    ) -> Result<(), DbIdempotencyStoreError> {
        let serialized = serde_json::to_string(result)?;

        sqlx::query("UPDATE idempotency_keys SET result = ?1 WHERE key = ?2")
            .bind(&serialized)
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
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

    #[tokio::test]
    async fn test_first_time_request() {
        let store = create_store().await;
        let status = store
            .check_and_store("key-001", 3600)
            .await
            .expect("check_and_store should succeed");
        assert_eq!(status, IdempotencyStatus::FirstTime);
    }

    #[tokio::test]
    async fn test_duplicate_returns_cached() {
        let store = create_store().await;

        let status1 = store
            .check_and_store("key-002", 3600)
            .await
            .expect("First check");
        assert_eq!(status1, IdempotencyStatus::FirstTime);

        let cached = CachedResult {
            status_code: 201,
            body: r#"{"voucher":"V-456"}"#.to_string(),
        };
        store
            .store_result("key-002", &cached)
            .await
            .expect("store_result");

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

    #[tokio::test]
    async fn test_persistence_across_operations() {
        let store = create_store().await;

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
