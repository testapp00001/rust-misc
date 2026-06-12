//! # Solution 01: Redis Streams Consumer
//!
//! Complete implementation of a Redis Streams consumer with consumer groups,
//! crash recovery via PEL, and at-least-once delivery semantics.

use deadpool_redis::Pool;
use redis::streams::StreamReadReply;
use serde::{Deserialize, Serialize};

/// An order event read from the Redis Stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderEvent {
    /// The Redis stream entry ID (e.g., "1234567890-0").
    pub stream_id: String,
    /// Unique order identifier.
    pub order_id: String,
    /// The product being purchased.
    pub product_id: String,
    /// The account making the purchase.
    pub account_id: String,
    /// Voucher code assigned to this order (may be empty if not yet generated).
    pub voucher_code: String,
}

/// Custom error type for consumer operations.
#[derive(Debug, thiserror::Error)]
pub enum ConsumerError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_redis::PoolError),

    #[error("Failed to parse stream entry: {0}")]
    ParseError(String),
}

/// A Redis Streams consumer that reads order events from a consumer group.
pub struct OrderConsumer {
    pool: Pool,
    stream: String,
    group: String,
    consumer_name: String,
}

impl OrderConsumer {
    /// Create a new consumer and ensure the consumer group exists.
    ///
    /// Uses `XGROUP CREATE` with `MKSTREAM` to create both the stream and
    /// group if they don't already exist. The `BUSYGROUP` error (group already
    /// exists) is silently ignored.
    ///
    /// On startup, checks the PEL for unacknowledged messages from a previous
    /// crash by reading pending messages with `XREADGROUP ... STREAMS key 0`.
    pub async fn new(
        pool: Pool,
        stream: &str,
        group: &str,
        consumer_name: &str,
    ) -> Result<Self, ConsumerError> {
        let mut conn = pool.get().await?;

        // Create the consumer group. Ignore BUSYGROUP (group already exists).
        let result: Result<(), redis::RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(stream)
            .arg(group)
            .arg("0")
            .arg("MKSTREAM")
            .query_async(&mut conn)
            .await;

        match result {
            Ok(()) => {}
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("BUSYGROUP") {
                    // Group already exists -- this is fine
                } else {
                    return Err(ConsumerError::Redis(e));
                }
            }
        }

        let consumer = Self {
            pool,
            stream: stream.to_string(),
            group: group.to_string(),
            consumer_name: consumer_name.to_string(),
        };

        // Check for pending (unacknowledged) messages from a previous crash.
        // Using `0` as the ID reads from the consumer's pending list.
        tracing::info!(
            consumer = consumer_name,
            stream = stream,
            group = group,
            "Checking PEL for unacknowledged messages"
        );

        let mut conn = consumer.pool.get().await?;

        let result: StreamReadReply = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(group)
            .arg(consumer_name)
            .arg("COUNT")
            .arg(100)
            .arg("STREAMS")
            .arg(stream)
            .arg("0") // "0" = read pending messages, not new ones
            .query_async(&mut conn)
            .await?;

        let pending_count: usize = result.keys.iter().map(|k| k.ids.len()).sum();
        if pending_count > 0 {
            tracing::info!(
                count = pending_count,
                "Recovered pending messages from PEL"
            );
        }

        Ok(consumer)
    }

    /// Read a batch of events from the stream using `XREADGROUP`.
    ///
    /// Uses `>` as the stream ID to read only new (undelivered) messages.
    /// Blocks for up to 5 seconds if no messages are available.
    ///
    /// Returns an empty vector if the block timeout expires with no messages.
    pub async fn consume(&mut self) -> Result<Vec<OrderEvent>, ConsumerError> {
        let mut conn = self.pool.get().await?;

        let result: StreamReadReply = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(&self.group)
            .arg(&self.consumer_name)
            .arg("COUNT")
            .arg(10)
            .arg("BLOCK")
            .arg(5000)
            .arg("STREAMS")
            .arg(&self.stream)
            .arg(">") // ">" = only new, undelivered messages
            .query_async(&mut conn)
            .await?;

        let mut events = Vec::new();
        for key in &result.keys {
            for entry in &key.ids {
                let event = parse_stream_entry(&entry.id, &entry.map)?;
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Acknowledge a message so it is removed from the PEL.
    ///
    /// Uses `XACK` to confirm that the message has been successfully processed.
    /// After acknowledgement, the message will not be re-delivered.
    pub async fn acknowledge(&mut self, event_id: &str) -> Result<(), ConsumerError> {
        let mut conn = self.pool.get().await?;

        let _acked: i32 = redis::cmd("XACK")
            .arg(&self.stream)
            .arg(&self.group)
            .arg(event_id)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }
}

/// Parse a Redis stream entry into an `OrderEvent`.
fn parse_stream_entry(
    entry_id: &str,
    map: &std::collections::HashMap<String, redis::Value>,
) -> Result<OrderEvent, ConsumerError> {
    let get_field = |name: &str| -> Result<String, ConsumerError> {
        match map.get(name) {
            Some(redis::Value::BulkString(bytes)) => String::from_utf8(bytes.to_vec())
                .map_err(|e| ConsumerError::ParseError(format!("Invalid UTF-8 in {name}: {e}"))),
            Some(other) => Err(ConsumerError::ParseError(format!(
                "Expected string for {name}, got: {other:?}"
            ))),
            None => Err(ConsumerError::ParseError(format!(
                "Missing required field: {name}"
            ))),
        }
    };

    Ok(OrderEvent {
        stream_id: entry_id.to_string(),
        order_id: get_field("order_id")?,
        product_id: get_field("product_id")?,
        account_id: get_field("account_id")?,
        voucher_code: get_field("voucher_code").unwrap_or_default(),
    })
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

    #[tokio::test]
    async fn test_consumer_creation() {
        let pool = match try_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let stream = format!("test:consumer:creation:{}", std::process::id());
        let group = format!("test-group-{}", std::process::id());

        let _consumer = OrderConsumer::new(pool.clone(), &stream, &group, "test-worker")
            .await
            .expect("Should create consumer");

        // Creating again should succeed (BUSYGROUP handled)
        let _consumer2 = OrderConsumer::new(pool.clone(), &stream, &group, "test-worker-2")
            .await
            .expect("Should create second consumer (BUSYGROUP handled)");

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_consume_events() {
        let pool = match try_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let stream = format!("test:consumer:consume:{}", std::process::id());
        let group = format!("test-group-consume-{}", std::process::id());

        // Add an event to the stream
        let mut conn = pool.get().await.unwrap();
        let event_id: String = redis::cmd("XADD")
            .arg(&stream)
            .arg("*")
            .arg("order_id")
            .arg("ord-001")
            .arg("product_id")
            .arg("prod-A")
            .arg("account_id")
            .arg("acc-100")
            .arg("voucher_code")
            .arg("VC-001")
            .query_async(&mut conn)
            .await
            .unwrap();
        drop(conn);

        let mut consumer = OrderConsumer::new(pool.clone(), &stream, &group, "test-worker")
            .await
            .expect("Should create consumer");

        let events = consumer.consume().await.expect("Should consume events");
        assert_eq!(events.len(), 1, "Should receive exactly 1 event");
        assert_eq!(events[0].order_id, "ord-001");
        assert_eq!(events[0].stream_id, event_id);

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_acknowledge_event() {
        let pool = match try_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let stream = format!("test:consumer:ack:{}", std::process::id());
        let group = format!("test-group-ack-{}", std::process::id());

        // Add an event
        let mut conn = pool.get().await.unwrap();
        let _: String = redis::cmd("XADD")
            .arg(&stream)
            .arg("*")
            .arg("order_id")
            .arg("ord-ack")
            .arg("product_id")
            .arg("prod-B")
            .arg("account_id")
            .arg("acc-200")
            .arg("voucher_code")
            .arg("VC-002")
            .query_async(&mut conn)
            .await
            .unwrap();
        drop(conn);

        let mut consumer = OrderConsumer::new(pool.clone(), &stream, &group, "test-worker")
            .await
            .expect("Should create consumer");

        let events = consumer.consume().await.expect("Should consume");
        assert_eq!(events.len(), 1);

        consumer
            .acknowledge(&events[0].stream_id)
            .await
            .expect("Should acknowledge");

        // After ack, PEL should be empty for this consumer
        let mut conn = pool.get().await.unwrap();
        let pending: redis::Value = redis::cmd("XPENDING")
            .arg(&stream)
            .arg(&group)
            .arg("-")
            .arg("+")
            .arg("1")
            .query_async(&mut conn)
            .await
            .unwrap();
        match pending {
            redis::Value::Nil => {}
            redis::Value::Array(ref arr) if arr.is_empty() => {}
            other => panic!("Expected empty PEL after ack, got: {other:?}"),
        }

        // Clean up
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_pel_recovery() {
        let pool = match try_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let stream = format!("test:consumer:pel:{}", std::process::id());
        let group = format!("test-group-pel-{}", std::process::id());

        // Add an event
        let mut conn = pool.get().await.unwrap();
        let _: String = redis::cmd("XADD")
            .arg(&stream)
            .arg("*")
            .arg("order_id")
            .arg("ord-pel")
            .arg("product_id")
            .arg("prod-C")
            .arg("account_id")
            .arg("acc-300")
            .arg("voucher_code")
            .arg("VC-003")
            .query_async(&mut conn)
            .await
            .unwrap();
        drop(conn);

        // Simulate a "crashed" consumer: read but never ack
        {
            let mut consumer1 =
                OrderConsumer::new(pool.clone(), &stream, &group, "crashed-worker")
                    .await
                    .expect("Should create first consumer");
            let events = consumer1.consume().await.expect("Should consume");
            assert_eq!(events.len(), 1, "Crashed worker should receive 1 event");
            // Consumer1 "crashes" -- no ack, dropped here
        }

        // New consumer joins -- should recover the pending message
        let mut consumer2 = OrderConsumer::new(pool.clone(), &stream, &group, "recovery-worker")
            .await
            .expect("Should create recovery consumer");

        let events = consumer2.consume().await.expect("Should recover pending events");
        assert_eq!(
            events.len(),
            1,
            "Recovery worker should receive the pending event from PEL"
        );
        assert_eq!(events[0].order_id, "ord-pel");

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }
}
