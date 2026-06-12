//! # Solution 04: Concurrent Deduplication
//!
//! Complete implementation of a three-state deduplicator using Redis SET NX.

use deadpool_redis::{Config, Pool};
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};

/// Result of a deduplication attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum DedupResult<T> {
    /// No existing request found. This caller should proceed with processing.
    Proceed,
    /// Another request is currently processing. The caller should wait and retry.
    InProgress,
    /// A previous request already completed. Return this cached result.
    Completed(T),
}

/// The cached result stored for a completed request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CachedResult {
    pub status_code: u16,
    pub body: String,
}

/// Handles concurrent duplicate requests safely using Redis SET NX.
pub struct ConcurrentDeduplicator {
    pool: Pool,
    lock_ttl_seconds: u64,
}

impl ConcurrentDeduplicator {
    /// Create a new deduplicator.
    pub fn new(pool: Pool, lock_ttl_seconds: u64) -> Self {
        Self {
            pool,
            lock_ttl_seconds,
        }
    }

    /// Create a deduplicator connected to the given Redis URL.
    pub fn from_url(url: &str, lock_ttl_seconds: u64) -> Result<Self, DedupError> {
        let cfg = Config::from_url(url);
        let pool = cfg
            .builder()
            .map_err(|e| DedupError::Connection(e.to_string()))?
            .max_size(16)
            .build()
            .map_err(|e| DedupError::Connection(e.to_string()))?;
        Ok(Self {
            pool,
            lock_ttl_seconds,
        })
    }

    /// Try to acquire the right to process a request with this key.
    ///
    /// Uses SET NX atomically:
    /// - If the key doesn't exist: sets it to "processing" and returns `Proceed`
    /// - If the key exists with value "processing": returns `InProgress`
    /// - If the key exists with a JSON result: returns `Completed(result)`
    pub async fn try_acquire(&self, key: &str) -> Result<DedupResult<CachedResult>, DedupError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| DedupError::Connection(e.to_string()))?;

        // SET NX: returns true if key was set (new), false if key exists
        let was_set: bool = redis::cmd("SET")
            .arg(key)
            .arg("processing")
            .arg("NX")
            .arg("EX")
            .arg(self.lock_ttl_seconds)
            .query_async(&mut *conn)
            .await
            .map_err(|e| DedupError::Command(e.to_string()))?;

        if was_set {
            // Key was newly set -- this caller should proceed
            return Ok(DedupResult::Proceed);
        }

        // Key exists -- fetch its value to determine state
        let value: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| DedupError::Command(e.to_string()))?;

        match value {
            Some(v) if v == "processing" => Ok(DedupResult::InProgress),
            Some(v) => {
                let result: CachedResult = serde_json::from_str(&v)?;
                Ok(DedupResult::Completed(result))
            }
            None => {
                // Key expired between SET NX and GET -- retry as new
                Ok(DedupResult::Proceed)
            }
        }
    }

    /// Mark a request as completed and store its result.
    pub async fn mark_completed(
        &self,
        key: &str,
        result: &CachedResult,
        ttl_seconds: u64,
    ) -> Result<(), DedupError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| DedupError::Connection(e.to_string()))?;

        let serialized = serde_json::to_string(result)?;

        // Overwrite "processing" with the actual result, preserving TTL
        let _: () = redis::cmd("SET")
            .arg(key)
            .arg(&serialized)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut *conn)
            .await
            .map_err(|e| DedupError::Command(e.to_string()))?;

        Ok(())
    }

    /// Mark a request as failed and release the lock.
    pub async fn mark_failed(&self, key: &str) -> Result<(), DedupError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| DedupError::Connection(e.to_string()))?;

        let _: () = conn
            .del(key)
            .await
            .map_err(|e| DedupError::Command(e.to_string()))?;

        Ok(())
    }
}

/// Errors from the deduplicator.
#[derive(Debug, thiserror::Error)]
pub enum DedupError {
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
    use futures::future::join_all;

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    fn test_key(suffix: &str) -> String {
        format!("dedup:test:p04:{}:{}", std::process::id(), suffix)
    }

    async fn try_create() -> Result<ConcurrentDeduplicator, String> {
        ConcurrentDeduplicator::from_url(REDIS_URL, 5)
            .map_err(|e| format!("Could not create deduplicator: {e}"))
    }

    #[tokio::test]
    async fn test_first_acquirer_proceeds() {
        let dedup = match try_create().await {
            Ok(d) => d,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key = test_key("first");

        let result = dedup.try_acquire(&key).await.expect("try_acquire");
        assert_eq!(result, DedupResult::Proceed);
    }

    #[tokio::test]
    async fn test_concurrent_gets_in_progress() {
        let dedup = match try_create().await {
            Ok(d) => d,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key = test_key("concurrent");

        let r1 = dedup.try_acquire(&key).await.expect("first acquire");
        assert_eq!(r1, DedupResult::Proceed);

        let r2 = dedup.try_acquire(&key).await.expect("second acquire");
        assert_eq!(r2, DedupResult::InProgress);
    }

    #[tokio::test]
    async fn test_completed_returns_cached() {
        let dedup = match try_create().await {
            Ok(d) => d,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key = test_key("completed");

        dedup.try_acquire(&key).await.expect("acquire");
        let cached = CachedResult {
            status_code: 200,
            body: r#"{"voucher":"V-789"}"#.to_string(),
        };
        dedup
            .mark_completed(&key, &cached, 60)
            .await
            .expect("mark_completed");

        let r = dedup.try_acquire(&key).await.expect("after complete");
        match r {
            DedupResult::Completed(result) => {
                assert_eq!(result.status_code, 200);
                assert_eq!(result.body, r#"{"voucher":"V-789"}"#);
            }
            other => panic!("Expected Completed, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_failed_releases_lock() {
        let dedup = match try_create().await {
            Ok(d) => d,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key = test_key("failed");

        dedup.try_acquire(&key).await.expect("acquire");
        dedup.mark_failed(&key).await.expect("mark_failed");

        let r = dedup.try_acquire(&key).await.expect("after fail");
        assert_eq!(r, DedupResult::Proceed);
    }

    #[tokio::test]
    async fn test_many_concurrent_requests() {
        let _dedup = match try_create().await {
            Ok(d) => d,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key = test_key("many_concurrent");

        let mut handles = Vec::new();
        for _ in 0..50 {
            let key_owned = key.clone();
            // We need to share the dedup across tasks; use a reference via async block
            handles.push(async move {
                // We can't easily share &dedup across spawned tasks without Arc,
                // so we create a new dedup connection per task for this test
                let d = ConcurrentDeduplicator::from_url(REDIS_URL, 5).expect("create dedup");
                d.try_acquire(&key_owned).await
            });
        }

        let results: Vec<DedupResult<CachedResult>> = join_all(handles)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("All requests should succeed");

        let proceed_count = results.iter().filter(|r| **r == DedupResult::Proceed).count();
        let in_progress_count = results
            .iter()
            .filter(|r| **r == DedupResult::InProgress)
            .count();

        assert_eq!(proceed_count, 1, "Exactly one request should proceed");
        assert_eq!(in_progress_count, 49, "The rest should be InProgress");
    }
}
