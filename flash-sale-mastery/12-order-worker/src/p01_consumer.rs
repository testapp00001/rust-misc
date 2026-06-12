//! # Exercise 01: Redis Streams Consumer
//!
//! ## Learning Objective
//! Learn how to consume messages from a Redis Stream using consumer groups.
//! Consumer groups provide load balancing across multiple workers and
//! crash recovery through the Pending Entry List (PEL).
//!
//! ## Flash Sale Context
//! The flash sale API (Module 11) writes order events to a Redis Stream via
//! `XADD`. This consumer reads those events using `XREADGROUP`, which delivers
//! each message to exactly one consumer in the group. If a consumer crashes
//! before acknowledging, the message stays in the PEL and can be reclaimed.
//!
//! ## Instructions
//! 1. Implement `OrderConsumer::new` to create the consumer and ensure the
//!    consumer group exists (use `XGROUP CREATE` with `MKSTREAM`)
//! 2. Implement `consume` to read a batch of events using `XREADGROUP`
//! 3. Implement `acknowledge` to confirm processing via `XACK`
//! 4. On startup, check the PEL for unacknowledged messages from a previous
//!    crash and re-deliver them
//!
//! ## Hints
//! - Use `redis::cmd("XGROUP").arg("CREATE")...` for group creation
//! - `XREADGROUP GROUP group consumer COUNT n BLOCK 5000 STREAMS key >`
//!   reads new messages; use `0` instead of `>` to read pending messages
//! - The `>` ID means "only undelivered messages"; `0` means "start from
//!   the beginning of my pending list"
//! - Handle the `BUSYGROUP` error gracefully (group already exists)

use deadpool_redis::Pool;
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
    /// If the group already exists, the `BUSYGROUP` error is silently ignored.
    /// On startup, this should also check the PEL for messages from a previous
    /// crash and re-deliver them.
    ///
    /// # Arguments
    /// * `pool` - Redis connection pool
    /// * `stream` - Stream key name (e.g., "orders:stream")
    /// * `group` - Consumer group name (e.g., "order-workers")
    /// * `consumer_name` - This consumer's unique name (e.g., "worker-1")
    pub async fn new(
        pool: Pool,
        stream: &str,
        group: &str,
        consumer_name: &str,
    ) -> Result<Self, ConsumerError> {
        // TODO: Acquire a connection from the pool
        // TODO: Create the consumer group with XGROUP CREATE stream group 0 MKSTREAM
        // TODO: Handle BUSYGROUP error (group already exists) gracefully
        // TODO: Return the OrderConsumer struct
        todo!("Implement consumer creation and group setup")
    }

    /// Read a batch of events from the stream.
    ///
    /// Uses `XREADGROUP` to read up to 10 messages with a 5-second block
    /// timeout. Messages are delivered at-least-once.
    ///
    /// # Returns
    /// A vector of `OrderEvent`s. An empty vector means no new messages.
    pub async fn consume(&mut self) -> Result<Vec<OrderEvent>, ConsumerError> {
        // TODO: Acquire a Redis connection
        // TODO: Run XREADGROUP GROUP {group} {consumer_name} COUNT 10 BLOCK 5000 STREAMS {stream} >
        // TODO: Parse the response into Vec<OrderEvent>
        // TODO: Return empty vec if no messages (timeout)
        todo!("Implement message consumption with XREADGROUP")
    }

    /// Acknowledge a message so it is removed from the PEL.
    ///
    /// # Arguments
    /// * `event_id` - The Redis stream entry ID to acknowledge
    pub async fn acknowledge(&mut self, event_id: &str) -> Result<(), ConsumerError> {
        // TODO: Acquire a Redis connection
        // TODO: Run XACK {stream} {group} {event_id}
        todo!("Implement message acknowledgement with XACK")
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

    /// Test that a consumer can be created and the group is set up.
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

        let _consumer = OrderConsumer::new(pool, &stream, &group, "test-worker")
            .await
            .expect("Should create consumer");

        // Clean up
        let cfg = deadpool_redis::Config::from_url(REDIS_URL);
        let pool = cfg
            .builder()
            .unwrap()
            .build()
            .unwrap();
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    /// Test consuming events that were added to the stream.
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

    /// Test acknowledging an event removes it from the PEL.
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
        // Pending should be empty (nil or empty array)
        match pending {
            redis::Value::Nil => {} // expected
            redis::Value::Array(ref arr) if arr.is_empty() => {} // expected
            other => panic!("Expected empty PEL after ack, got: {other:?}"),
        }

        // Clean up
        let _: () = redis::cmd("DEL")
            .arg(&stream)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    /// Test PEL recovery: messages pending from a "crashed" consumer are
    /// re-delivered when a new consumer joins the same group.
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
