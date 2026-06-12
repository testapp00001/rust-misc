//! # Exercise 04: Redis Streams
//!
//! ## Learning Objective
//! Use Redis Streams (XADD) to publish events to a durable, append-only log
//! that multiple consumers can read independently. Redis Streams combine the
//! simplicity of Redis lists with the durability and fan-out of message brokers.
//!
//! ## Flash Sale Context
//! During a flash sale, every stock change must be published to a stream so
//! that downstream services (analytics, reconciliation, notifications) can
//! consume them independently. Redis Streams provide exactly-once semantics
//! within a consumer group and at-least-once across groups.
//!
//! ## Instructions
//! 1. Implement `publish_event` to XADD an event to a Redis stream
//! 2. Implement `read_events` to XREAD events starting from a given ID
//! 3. Events should be serialized as JSON in the stream entry fields
//! 4. Return the Redis-generated stream ID for each published event
//!
//! ## Hints
//! - XADD format: `XADD stream_key * field1 value1 field2 value2 ...`
//! - Use `*` as the ID to let Redis auto-generate the stream entry ID
//! - XREAD with COUNT and BLOCK options for efficient reading
//! - Each stream entry should have a single field "event" containing the JSON

use redis::aio::MultiplexedConnection as RedisConnection;
use redis::RedisError;

use crate::p01_event_design::FlashSaleEvent;

/// Error type for Redis stream operations.
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

/// Publish an event to a Redis stream using XADD.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `stream_key` - The stream key (e.g., "flash-sale:events")
/// * `event` - The event to publish
///
/// # Returns
/// The Redis-generated stream entry ID (e.g., "1234567890-0").
pub async fn publish_event(
    conn: &mut RedisConnection,
    stream_key: &str,
    event: &FlashSaleEvent,
) -> Result<String, StreamError> {
    // TODO: Serialize the event to JSON
    // TODO: Use redis::cmd("XADD").arg(stream_key).arg("*").arg("event").arg(&json)
    // TODO: Query async and return the stream ID
    todo!("Implement publish_event")
}

/// Read events from a Redis stream starting after the given ID.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `stream_key` - The stream key to read from
/// * `last_id` - Start reading after this ID (use "0" to read from the beginning)
/// * `count` - Maximum number of entries to read
///
/// # Returns
/// A vector of (stream_id, FlashSaleEvent) tuples.
pub async fn read_events(
    conn: &mut RedisConnection,
    stream_key: &str,
    last_id: &str,
    count: usize,
) -> Result<Vec<(String, FlashSaleEvent)>, StreamError> {
    // TODO: Use redis::cmd("XREAD").arg("COUNT").arg(count).arg("STREAMS").arg(stream_key).arg(last_id)
    // TODO: Parse the response and deserialize each entry's "event" field
    todo!("Implement read_events")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p01_event_design::EventPayload;

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn get_conn() -> Option<RedisConnection> {
        let client = redis::Client::open(REDIS_URL).ok()?;
        client.get_multiplexed_async_connection().await.ok()
    }

    fn stream_key(suffix: &str) -> String {
        format!("test:events:{}:{}", suffix, std::process::id())
    }

    #[tokio::test]
    async fn test_publish_and_read_back() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = stream_key("pub_read");

        // Clean up
        let _: Result<(), _> = redis::cmd("DEL").arg(&key).query_async(&mut conn).await;

        let event = FlashSaleEvent::new(
            "product:1001",
            EventPayload::StockDecremented {
                account_id: "user:42".to_string(),
                quantity: 1,
                remaining: 99,
            },
        );

        let stream_id = publish_event(&mut conn, &key, &event)
            .await
            .expect("publish should succeed");
        assert!(!stream_id.is_empty(), "Stream ID should not be empty");

        let entries = read_events(&mut conn, &key, "0", 10)
            .await
            .expect("read should succeed");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, stream_id);
        assert_eq!(entries[0].1.aggregate_id, "product:1001");

        // Clean up
        let _: Result<(), _> = redis::cmd("DEL").arg(&key).query_async(&mut conn).await;
    }

    #[tokio::test]
    async fn test_id_based_ordering() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = stream_key("ordering");

        let _: Result<(), _> = redis::cmd("DEL").arg(&key).query_async(&mut conn).await;

        let mut ids = Vec::new();
        for i in 0..5 {
            let event = FlashSaleEvent::new(
                "product:1001",
                EventPayload::StockDecremented {
                    account_id: format!("user:{i}"),
                    quantity: 1,
                    remaining: 100 - i,
                },
            );
            let sid = publish_event(&mut conn, &key, &event)
                .await
                .expect("publish failed");
            ids.push(sid);
        }

        // IDs should be in ascending order
        for window in ids.windows(2) {
            assert!(window[0] < window[1], "Stream IDs should be ascending");
        }

        let entries = read_events(&mut conn, &key, "0", 100)
            .await
            .expect("read failed");
        assert_eq!(entries.len(), 5);

        let _: Result<(), _> = redis::cmd("DEL").arg(&key).query_async(&mut conn).await;
    }

    #[tokio::test]
    async fn test_range_query() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = stream_key("range");

        let _: Result<(), _> = redis::cmd("DEL").arg(&key).query_async(&mut conn).await;

        let mut ids = Vec::new();
        for i in 0..5 {
            let event = FlashSaleEvent::new(
                "product:1001",
                EventPayload::StockDecremented {
                    account_id: format!("user:{i}"),
                    quantity: 1,
                    remaining: 100 - i as u32,
                },
            );
            let sid = publish_event(&mut conn, &key, &event)
                .await
                .expect("publish failed");
            ids.push(sid);
        }

        // Read starting after the second entry
        let after_id = &ids[1];
        let entries = read_events(&mut conn, &key, after_id, 100)
            .await
            .expect("read failed");
        // Should get entries 2, 3, 4 (the ones after ids[1])
        assert_eq!(entries.len(), 3);

        let _: Result<(), _> = redis::cmd("DEL").arg(&key).query_async(&mut conn).await;
    }
}
