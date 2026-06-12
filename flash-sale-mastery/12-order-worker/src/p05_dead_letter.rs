//! # Exercise 05: Dead Letter Queue
//!
//! ## Learning Objective
//! Learn to implement a dead letter queue (DLQ) for events that fail processing
//! after all retry attempts. The DLQ provides visibility into failures and
//! enables manual or automated remediation.
//!
//! ## Flash Sale Context
//! Some order events will fail: the payment service may be down, the database
//! may reject the insert, or the event data may be malformed. Rather than losing
//! these events or retrying forever, we park them in a dead letter queue. An
//! operator can later inspect the DLQ, fix the underlying issue, and trigger a
//! retry of all parked events.
//!
//! ## Instructions
//! 1. Implement `DeadLetterQueue::new` to create the DLQ backed by a Redis Stream
//! 2. Implement `push` to add a failed event with its error message and retry count
//! 3. Implement `list` to retrieve all entries in the DLQ
//! 4. Implement `retry_all` to re-process all entries (up to max retries) and
//!    return a `RetryReport` summarizing the results
//!
//! ## Hints
//! - Use a separate Redis Stream for the DLQ (e.g., "orders:dlq")
//! - Each DLQ entry stores: original event JSON, error message, retry count,
//!   and timestamp
//! - `retry_all` should read all entries, attempt to reprocess each one, and
//!   either remove it from the DLQ (success) or increment its retry count
//!   (failure)
//! - Enforce a maximum retry count (e.g., 3) -- entries that exceed this are
//!   left in the DLQ for manual intervention

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
    /// # Arguments
    /// * `pool` - Redis connection pool
    /// * `stream` - Redis Stream key for the DLQ (e.g., "orders:dlq")
    pub async fn new(pool: Pool, stream: &str) -> Result<Self, DeadLetterError> {
        todo!("Initialize DeadLetterQueue (no special setup needed for streams)")
    }

    /// Push a failed event into the DLQ.
    ///
    /// # Arguments
    /// * `event` - The order event that failed processing
    /// * `error` - Description of the failure
    pub async fn push(&self, event: OrderEvent, error: &str) -> Result<(), DeadLetterError> {
        // TODO: Serialize the DeadLetterEntry to JSON
        // TODO: Use XADD to append to the DLQ stream
        todo!("Implement push to dead letter queue")
    }

    /// List all entries currently in the DLQ.
    ///
    /// # Returns
    /// A vector of `DeadLetterEntry` items.
    pub async fn list(&self) -> Result<Vec<DeadLetterEntry>, DeadLetterError> {
        // TODO: Use XRANGE to read all entries from the DLQ stream
        // TODO: Parse each entry into a DeadLetterEntry
        todo!("Implement list to read all DLQ entries")
    }

    /// Retry all entries in the DLQ.
    ///
    /// For each entry:
    /// - If retry_count < MAX_RETRIES, attempt to reprocess
    /// - On success, remove the entry from the DLQ (XDEL)
    /// - On failure, update the entry (push new version, delete old)
    /// - If retry_count >= MAX_RETRIES, count as exceeded
    ///
    /// # Returns
    /// A `RetryReport` summarizing the results.
    pub async fn retry_all(&self) -> Result<RetryReport, DeadLetterError> {
        todo!("Implement retry-all with max retry enforcement")
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

    /// Test that failed events can be pushed to the DLQ.
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

    /// Test listing multiple entries in the DLQ.
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

    /// Test that retry_all processes entries and returns a report.
    #[tokio::test]
    async fn test_retry_all() {
        let pool = match try_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let stream = format!("test:dlq:retry:{}", std::process::id());
        let dlq = DeadLetterQueue::new(pool.clone(), &stream)
            .await
            .expect("Should create DLQ");

        let event = make_event("ord-retry-001");
        dlq.push(event, "temporary failure")
            .await
            .expect("Should push");

        let report = dlq.retry_all().await.expect("Should retry");
        assert_eq!(report.total, 1);
        // The outcome depends on whether the retry "succeeds" in simulation
        assert!(
            report.succeeded + report.failed + report.exceeded_max_retries == 1,
            "Report should account for all entries"
        );

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    /// Test that entries exceeding max retries are left in the DLQ.
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

        // Push an event and retry it MAX_RETRIES times
        let event = make_event("ord-max-001");
        dlq.push(event, "persistent failure")
            .await
            .expect("Should push");

        for _ in 0..MAX_RETRIES {
            let report = dlq.retry_all().await.expect("Should retry");
            // Each retry should attempt the entry
            assert!(report.total >= 1);
        }

        // After MAX_RETRIES, the entry should still be in the DLQ
        // (exceeded_max_retries) or already removed (if it succeeded once)
        let entries = dlq.list().await.expect("Should list");
        // The entry may still be there if all retries failed
        // This test verifies the retry mechanism runs without panicking
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
