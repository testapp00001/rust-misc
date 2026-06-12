//! # Exercise 04: Concurrent Deduplication
//!
//! ## Learning Objective
//! Handle the case where multiple identical requests arrive at the exact same
//! time. A simple check-then-act pattern has a race condition: two requests
//! both check, both see "new", and both proceed. This exercise uses Redis
//! SET NX to create a three-state deduplicator that safely handles concurrency.
//!
//! ## Flash Sale Context
//! A user's browser sends the purchase request, and 50ms later a retry fires.
//! Both requests hit the API simultaneously. With naive idempotency (check
//! then set), both could pass. The `ConcurrentDeduplicator` uses a single
//! atomic Redis operation to ensure exactly one request proceeds.
//!
//! ## Instructions
//! 1. Implement `ConcurrentDeduplicator::new()` with a Redis pool
//! 2. Implement `try_acquire()` using SET NX with three states:
//!    - `Proceed`: Key was newly set, this request should be processed
//!    - `InProgress`: Key exists with "processing" marker, wait and retry
//!    - `Completed(cached)`: Key has a result, return the cached value
//! 3. Implement `mark_completed()` to store the final result
//! 4. Implement `mark_failed()` to release the lock on error
//!
//! ## Hints
//! - Use SET NX with a short TTL for the initial lock (e.g., 5 seconds)
//! - The value stored is "processing" initially, then the serialized result
//! - On GET, check if the value is "processing" (InProgress) or a result (Completed)
//! - Use `tokio::time::sleep` to implement wait-and-retry for InProgress

use deadpool_redis::{Config, Pool, Runtime};
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
///
/// The deduplicator ensures that when N identical requests arrive simultaneously,
/// exactly one of them gets to proceed (the one whose SET NX succeeds). The
/// others either wait for the result or receive the cached result.
pub struct ConcurrentDeduplicator {
    pool: Pool,
    lock_ttl_seconds: u64,
}

impl ConcurrentDeduplicator {
    /// Create a new deduplicator.
    ///
    /// # Arguments
    /// * `pool` - Redis connection pool
    /// * `lock_ttl_seconds` - How long a "processing" lock lasts before auto-releasing
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
    ///
    /// # Arguments
    /// * `key` - The idempotency key
    pub async fn try_acquire(&self, key: &str) -> Result<DedupResult<CachedResult>, DedupError> {
        // TODO: Attempt SET NX with value "processing" and the lock TTL
        // TODO: If SET NX succeeds -> return Proceed
        // TODO: If SET NX fails (key exists) -> GET the value
        //   - If value is "processing" -> return InProgress
        //   - Otherwise -> deserialize as CachedResult and return Completed
        todo!("Implement try_acquire with SET NX")
    }

    /// Mark a request as completed and store its result.
    ///
    /// Overwrites the "processing" marker with the actual result.
    /// The key retains the original remaining TTL.
    ///
    /// # Arguments
    /// * `key` - The idempotency key
    /// * `result` - The result to cache
    /// * `ttl_seconds` - How long to keep the cached result
    pub async fn mark_completed(
        &self,
        key: &str,
        result: &CachedResult,
        ttl_seconds: u64,
    ) -> Result<(), DedupError> {
        // TODO: Serialize result to JSON
        // TODO: SET the key with the serialized result and the given TTL (overwrites "processing")
        todo!("Implement mark_completed")
    }

    /// Mark a request as failed and release the lock.
    ///
    /// Deletes the key so another request can acquire it.
    pub async fn mark_failed(&self, key: &str) -> Result<(), DedupError> {
        // TODO: DEL the key to release the lock
        todo!("Implement mark_failed")
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

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    fn test_key(suffix: &str) -> String {
        format!("dedup:test:p04:{}:{}", std::process::id(), suffix)
    }

    async fn try_create() -> Result<ConcurrentDeduplicator, String> {
        ConcurrentDeduplicator::from_url(REDIS_URL, 5)
            .map_err(|e| format!("Could not create deduplicator: {e}"))
    }

    /// Test that the first acquirer gets Proceed.
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

    /// Test that a concurrent request while processing gets InProgress.
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

        // First request acquires
        let r1 = dedup.try_acquire(&key).await.expect("first acquire");
        assert_eq!(r1, DedupResult::Proceed);

        // Second request while first is processing
        let r2 = dedup.try_acquire(&key).await.expect("second acquire");
        assert_eq!(r2, DedupResult::InProgress);
    }

    /// Test that after completion, requests get the cached result.
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

        // Acquire and complete
        dedup.try_acquire(&key).await.expect("acquire");
        let cached = CachedResult {
            status_code: 200,
            body: r#"{"voucher":"V-789"}"#.to_string(),
        };
        dedup
            .mark_completed(&key, &cached, 60)
            .await
            .expect("mark_completed");

        // Next request gets the cached result
        let r = dedup.try_acquire(&key).await.expect("after complete");
        match r {
            DedupResult::Completed(result) => {
                assert_eq!(result.status_code, 200);
                assert_eq!(result.body, r#"{"voucher":"V-789"}"#);
            }
            other => panic!("Expected Completed, got: {other:?}"),
        }
    }

    /// Test that mark_failed releases the lock.
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

        // Acquire, then fail
        dedup.try_acquire(&key).await.expect("acquire");
        dedup.mark_failed(&key).await.expect("mark_failed");

        // Next request should be able to proceed
        let r = dedup.try_acquire(&key).await.expect("after fail");
        assert_eq!(r, DedupResult::Proceed);
    }

    /// Test concurrent access with many simultaneous requests.
    #[tokio::test]
    async fn test_many_concurrent_requests() {
        let dedup = match try_create().await {
            Ok(d) => d,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let key = test_key("many_concurrent");

        // Spawn 50 concurrent requests
        let mut handles = Vec::new();
        for _ in 0..50 {
            let dedup_ref = &dedup;
            let key_owned = key.clone();
            handles.push(
                async move { dedup_ref.try_acquire(&key_owned).await }
            );
        }

        let results: Vec<DedupResult<CachedResult>> = futures::future::join_all(handles)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("All requests should succeed");

        let proceed_count = results.iter().filter(|r| **r == DedupResult::Proceed).count();
        let in_progress_count = results
            .iter()
            .filter(|r| **r == DedupResult::InProgress)
            .count();

        // Exactly one should get Proceed
        assert_eq!(proceed_count, 1, "Exactly one request should proceed");
        // The rest should be InProgress
        assert_eq!(
            in_progress_count,
            49,
            "The rest should be InProgress"
        );
    }
}
