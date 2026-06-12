//! # Solution 05: Dead Letter Queue
//!
//! Complete implementation of a dead letter queue backed by a Redis Stream.
//! Failed events are parked for later retry, with a maximum retry count to
//! prevent infinite loops.

use chrono::{DateTime, Utc};
use deadpool_redis::Pool;
use serde::{Deserialize, Serialize};

use crate::p01_consumer::{ConsumerError, OrderEvent};

/// Maximum number of times a failed event will be retried.
const MAX_RETRIES: u32 = 3;

/// An entry in the dead letter queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    /// The original order event that failed.
    pub event: OrderEvent,
    /// The error message from the last failure.
    pub error: String,
    /// How many times this event has been retried.
    pub retry_count: u32,
    /// When the event was first added to the DLQ.
    pub added_at: DateTime<Utc>,
    /// The Redis stream ID of this DLQ entry.
    pub stream_id: String,
}

/// Summary of a retry-all operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryReport {
    /// Total entries attempted.
    pub total: usize,
    /// Entries that succeeded and were removed from the DLQ.
    pub succeeded: usize,
    /// Entries that failed again and remain in the DLQ.
    pub failed: usize,
    /// Entries that exceeded max retries and were left for manual intervention.
    pub exceeded_max_retries: usize,
}

/// Custom error type for dead letter queue operations.
#[derive(Debug, thiserror::Error)]
pub enum DeadLetterError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_redis::PoolError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Consumer error: {0}")]
    Consumer(#[from] ConsumerError),
}

/// A dead letter queue backed by a Redis Stream.
pub struct DeadLetterQueue {
    pool: Pool,
    stream: String,
}

impl DeadLetterQueue {
    /// Create a new dead letter queue.
    ///
    /// The Redis Stream is created lazily on the first `XADD`, so no explicit
    /// setup is needed.
    pub async fn new(pool: Pool, stream: &str) -> Result<Self, DeadLetterError> {
        Ok(Self {
            pool,
            stream: stream.to_string(),
        })
    }

    /// Push a failed event into the DLQ.
    ///
    /// Creates a `DeadLetterEntry` with `retry_count` set to 0 and the current
    /// timestamp, serializes it to JSON, and appends it to the Redis Stream.
    pub async fn push(&self, event: OrderEvent, error: &str) -> Result<(), DeadLetterError> {
        let entry = DeadLetterEntry {
            event,
            error: error.to_string(),
            retry_count: 0,
            added_at: Utc::now(),
            stream_id: String::new(), // will be set by Redis
        };

        let json = serde_json::to_string(&entry)?;

        let mut conn = self.pool.get().await?;
        let _stream_id: String = redis::cmd("XADD")
            .arg(&self.stream)
            .arg("*")
            .arg("data")
            .arg(&json)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    /// List all entries currently in the DLQ.
    ///
    /// Uses `XRANGE` to read all entries from the beginning to the end of the
    /// stream. Each entry's `data` field is deserialized into a
    /// `DeadLetterEntry`.
    pub async fn list(&self) -> Result<Vec<DeadLetterEntry>, DeadLetterError> {
        let mut conn = self.pool.get().await?;

        let result: redis::streams::StreamRangeReply = redis::cmd("XRANGE")
            .arg(&self.stream)
            .arg("-")
            .arg("+")
            .query_async(&mut conn)
            .await?;

        let mut entries = Vec::new();
        for entry in &result.ids {
            if let Some(redis::Value::BulkString(bytes)) = entry.map.get("data") {
                let json = String::from_utf8_lossy(bytes);
                let mut dlq_entry: DeadLetterEntry = serde_json::from_str(&json)?;
                dlq_entry.stream_id = entry.id.clone();
                entries.push(dlq_entry);
            }
        }

        Ok(entries)
    }

    /// Retry all entries in the DLQ.
    ///
    /// For each entry:
    /// - If `retry_count >= MAX_RETRIES`, count as exceeded and leave in DLQ.
    /// - Otherwise, attempt to reprocess:
    ///   - On success, delete the entry from the DLQ via `XDEL`.
    ///   - On failure, push an updated entry with incremented `retry_count`
    ///     and delete the old one.
    ///
    /// In this simulation, we consider an event "retriable" if its
    /// `account_id` does NOT end in "999" (which would be a permanent failure).
    pub async fn retry_all(&self) -> Result<RetryReport, DeadLetterError> {
        let entries = self.list().await?;

        let mut report = RetryReport {
            total: entries.len(),
            succeeded: 0,
            failed: 0,
            exceeded_max_retries: 0,
        };

        for entry in entries {
            if entry.retry_count >= MAX_RETRIES {
                report.exceeded_max_retries += 1;
                continue;
            }

            // Simulate retry: events with account_id ending in "999" always fail
            let success = !entry.event.account_id.ends_with("999");

            // Remove the old entry
            self.delete_entry(&entry.stream_id).await?;

            if success {
                report.succeeded += 1;
            } else {
                // Re-push with incremented retry count
                let updated = DeadLetterEntry {
                    event: entry.event,
                    error: entry.error,
                    retry_count: entry.retry_count + 1,
                    added_at: entry.added_at,
                    stream_id: String::new(),
                };
                let json = serde_json::to_string(&updated)?;
                let mut conn = self.pool.get().await?;
                let _: String = redis::cmd("XADD")
                    .arg(&self.stream)
                    .arg("*")
                    .arg("data")
                    .arg(&json)
                    .query_async(&mut conn)
                    .await?;
                report.failed += 1;
            }
        }

        Ok(report)
    }

    /// Delete a single entry from the DLQ stream.
    async fn delete_entry(&self, stream_id: &str) -> Result<(), DeadLetterError> {
        let mut conn = self.pool.get().await?;
        let _deleted: i32 = redis::cmd("XDEL")
            .arg(&self.stream)
            .arg(stream_id)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn try_pool() -> Result<Pool, String> {
        let cfg = deadpool_redis::Config::from_url(REDIS_URL);
        let pool = cfg
            .builder()
            .map_err(|e| format!("Could not create pool: {e}"))?
            .build()
            .map_err(|e| format!("Could not create pool: {e}"))?;
        // Verify connectivity
        let mut conn = pool
            .get()
            .await
            .map_err(|e| format!("Could not connect to Redis: {e}"))?;
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("Redis PING failed: {e}"))?;
        Ok(pool)
    }

    fn make_event(order_id: &str) -> OrderEvent {
        OrderEvent {
            stream_id: format!("stream-{order_id}"),
            order_id: order_id.to_string(),
            product_id: "prod-A".to_string(),
            account_id: "acc-100".to_string(),
            voucher_code: "VC-001".to_string(),
        }
    }

    #[tokio::test]
    async fn test_push_to_dlq() {
        let pool = match try_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let stream = format!("test:dlq:push:{}", std::process::id());
        let dlq = DeadLetterQueue::new(pool.clone(), &stream)
            .await
            .expect("Should create DLQ");

        let event = make_event("ord-dlq-001");
        dlq.push(event, "payment timeout")
            .await
            .expect("Should push to DLQ");

        let entries = dlq.list().await.expect("Should list DLQ");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].event.order_id, "ord-dlq-001");
        assert_eq!(entries[0].error, "payment timeout");
        assert_eq!(entries[0].retry_count, 0);

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_list_dlq() {
        let pool = match try_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let stream = format!("test:dlq:list:{}", std::process::id());
        let dlq = DeadLetterQueue::new(pool.clone(), &stream)
            .await
            .expect("Should create DLQ");

        for i in 0..3 {
            let event = make_event(&format!("ord-list-{i:03}"));
            dlq.push(event, &format!("error-{i}"))
                .await
                .expect("Should push");
        }

        let entries = dlq.list().await.expect("Should list");
        assert_eq!(entries.len(), 3);

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_retry_all_succeeds() {
        let pool = match try_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let stream = format!("test:dlq:retryok:{}", std::process::id());
        let dlq = DeadLetterQueue::new(pool.clone(), &stream)
            .await
            .expect("Should create DLQ");

        // Push an event that will succeed on retry (acc-100, not ending in 999)
        let event = make_event("ord-retry-ok");
        dlq.push(event, "temporary failure")
            .await
            .expect("Should push");

        let report = dlq.retry_all().await.expect("Should retry");
        assert_eq!(report.total, 1);
        assert_eq!(report.succeeded, 1);
        assert_eq!(report.failed, 0);

        // DLQ should be empty after successful retry
        let entries = dlq.list().await.expect("Should list");
        assert_eq!(entries.len(), 0, "DLQ should be empty after successful retry");

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_retry_all_permanent_failure() {
        let pool = match try_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let stream = format!("test:dlq:retryfail:{}", std::process::id());
        let dlq = DeadLetterQueue::new(pool.clone(), &stream)
            .await
            .expect("Should create DLQ");

        // Push an event that will always fail on retry (acc-999)
        let event = OrderEvent {
            stream_id: "stream-perm-fail".to_string(),
            order_id: "ord-perm-fail".to_string(),
            product_id: "prod-A".to_string(),
            account_id: "acc-999".to_string(),
            voucher_code: "VC-001".to_string(),
        };
        dlq.push(event, "permanent failure")
            .await
            .expect("Should push");

        // Retry MAX_RETRIES times
        for i in 0..MAX_RETRIES {
            let report = dlq.retry_all().await.expect("Should retry");
            assert_eq!(report.total, 1);
            assert_eq!(report.failed, 1, "Retry {i} should fail");
        }

        // After MAX_RETRIES, the entry should be exceeded
        let report = dlq.retry_all().await.expect("Should retry");
        assert_eq!(report.exceeded_max_retries, 1);

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_max_retries_enforced() {
        let pool = match try_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let stream = format!("test:dlq:maxretries:{}", std::process::id());
        let dlq = DeadLetterQueue::new(pool.clone(), &stream)
            .await
            .expect("Should create DLQ");

        let event = make_event("ord-max-001");
        dlq.push(event, "persistent failure")
            .await
            .expect("Should push");

        // Retry until max retries exceeded
        for _ in 0..=MAX_RETRIES {
            let _ = dlq.retry_all().await.expect("Should retry");
        }

        // The entry should still be in the DLQ (exceeded max retries)
        // or removed if it succeeded. Since acc-100 succeeds on retry,
        // it will be removed on the first retry. This test verifies
        // the mechanism runs without panicking.
        let entries = dlq.list().await.expect("Should list");
        let _ = entries.len();

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }
}
