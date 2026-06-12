//! # Exercise 05: Consumer Groups
//!
//! ## Learning Objective
//! Use Redis Streams consumer groups (XREADGROUP, XACK) to distribute event
//! processing across multiple workers. Consumer groups ensure each event is
//! delivered to exactly one consumer within the group, while the Pending
//! Entry List (PEL) enables crash recovery.
//!
//! ## Flash Sale Context
//! After a flash sale generates thousands of events per second, multiple
//! worker processes need to consume them for order fulfillment, analytics,
//! and notifications. Consumer groups provide load balancing: each worker
//! gets a disjoint subset of events. If a worker crashes, unacknowledged
//! events are reassigned to other workers via the PEL.
//!
//! ## Instructions
//! 1. Implement `create_consumer_group` using XGROUP CREATE
//! 2. Implement `consume_events` using XREADGROUP to read events for a consumer
//! 3. Implement `acknowledge` using XACK to mark events as processed
//! 4. Implement `pending_events` using XPENDING to inspect the PEL
//! 5. Implement `claim_pending` using XCLAIM to recover stuck events
//!
//! ## Hints
//! - XGROUP CREATE stream_key group_name 0 MKSTREAM (creates stream if needed)
//! - XREADGROUP GROUP group_name consumer_name COUNT n STREAMS stream_key >
//!   The `>` means "only new, undelivered messages"
//! - XREADGROUP with `0` instead of `>` reads the consumer's pending entries
//! - XACK stream_key group_name entry_id marks an entry as processed
//! - XPENDING stream_key group_name shows pending entry summary

use redis::aio::MultiplexedConnection as RedisConnection;
use redis::RedisError;

use crate::p01_event_design::FlashSaleEvent;

/// Error type for consumer group operations.
#[derive(Debug, thiserror::Error)]
pub enum ConsumerError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Group already exists")]
    GroupAlreadyExists,
}

/// Create a consumer group on a stream.
///
/// If the stream does not exist, it will be created (MKSTREAM).
///
/// # Arguments
/// * `conn` - Redis connection
/// * `stream_key` - The stream key
/// * `group_name` - The consumer group name
pub async fn create_consumer_group(
    conn: &mut RedisConnection,
    stream_key: &str,
    group_name: &str,
) -> Result<(), ConsumerError> {
    // TODO: Use XGROUP CREATE stream_key group_name 0 MKSTREAM
    //       Handle the "BUSYGROUP" error (group already exists) gracefully
    todo!("Implement create_consumer_group")
}

/// Consume events from a stream as part of a consumer group.
///
/// Uses XREADGROUP to read undelivered events (the `>` marker).
///
/// # Arguments
/// * `conn` - Redis connection
/// * `stream_key` - The stream key
/// * `group_name` - The consumer group name
/// * `consumer_name` - This consumer's name (e.g., "worker-1")
/// * `count` - Maximum events to read
///
/// # Returns
/// A vector of (stream_id, FlashSaleEvent) tuples.
pub async fn consume_events(
    conn: &mut RedisConnection,
    stream_key: &str,
    group_name: &str,
    consumer_name: &str,
    count: usize,
) -> Result<Vec<(String, FlashSaleEvent)>, ConsumerError> {
    // TODO: Use XREADGROUP GROUP group_name consumer_name COUNT count STREAMS stream_key >
    // TODO: Parse entries and deserialize events
    todo!("Implement consume_events")
}

/// Acknowledge that an event has been processed.
///
/// This removes the event from the consumer's Pending Entry List (PEL).
///
/// # Arguments
/// * `conn` - Redis connection
/// * `stream_key` - The stream key
/// * `group_name` - The consumer group name
/// * `event_id` - The stream entry ID to acknowledge
pub async fn acknowledge(
    conn: &mut RedisConnection,
    stream_key: &str,
    group_name: &str,
    event_id: &str,
) -> Result<(), ConsumerError> {
    // TODO: Use XACK stream_key group_name event_id
    todo!("Implement acknowledge")
}

/// Read this consumer's pending (unacknowledged) events.
///
/// These are events that were delivered but not yet acknowledged.
/// Useful for crash recovery: a restarted consumer can read its own
/// pending events and retry processing.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `stream_key` - The stream key
/// * `group_name` - The consumer group name
/// * `consumer_name` - This consumer's name
/// * `count` - Maximum events to read
pub async fn pending_events(
    conn: &mut RedisConnection,
    stream_key: &str,
    group_name: &str,
    consumer_name: &str,
    count: usize,
) -> Result<Vec<(String, FlashSaleEvent)>, ConsumerError> {
    // TODO: Use XREADGROUP GROUP group_name consumer_name COUNT count STREAMS stream_key 0
    //       (note: 0 instead of > reads pending entries)
    todo!("Implement pending_events")
}

/// Claim pending events from another consumer that has been idle too long.
///
/// This is the crash recovery mechanism: if a consumer dies without
/// acknowledging events, another consumer can claim them after a timeout.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `stream_key` - The stream key
/// * `group_name` - The consumer group name
/// * `consumer_name` - The consumer claiming the events
/// * `min_idle_ms` - Minimum idle time in milliseconds before an event can be claimed
/// * `event_ids` - The stream entry IDs to claim
pub async fn claim_pending(
    conn: &mut RedisConnection,
    stream_key: &str,
    group_name: &str,
    consumer_name: &str,
    min_idle_ms: u64,
    event_ids: &[&str],
) -> Result<Vec<(String, FlashSaleEvent)>, ConsumerError> {
    // TODO: Use XCLAIM stream_key group_name consumer_name min_idle_ms event_id1 event_id2 ...
    // TODO: Parse the response similar to XREADGROUP
    todo!("Implement claim_pending")
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

    fn key(suffix: &str) -> String {
        format!("test:cg:{}:{}", suffix, std::process::id())
    }

    async fn publish_test_events(
        conn: &mut RedisConnection,
        stream_key: &str,
        count: usize,
    ) -> Vec<String> {
        let mut ids = Vec::new();
        for i in 0..count {
            let event = FlashSaleEvent::new(
                "product:1001",
                EventPayload::StockDecremented {
                    account_id: format!("user:{i}"),
                    quantity: 1,
                    remaining: 100 - i as u32,
                },
            );
            let json = serde_json::to_string(&event).unwrap();
            let id: String = redis::cmd("XADD")
                .arg(stream_key)
                .arg("*")
                .arg("event")
                .arg(&json)
                .query_async(conn)
                .await
                .unwrap();
            ids.push(id);
        }
        ids
    }

    #[tokio::test]
    async fn test_create_group_and_consume() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let stream = key("consume");
        let group = "test-group";
        let _: Result<(), _> = redis::cmd("DEL").arg(&stream).query_async(&mut conn).await;

        publish_test_events(&mut conn, &stream, 3).await;

        create_consumer_group(&mut conn, &stream, group)
            .await
            .expect("create group failed");

        let entries = consume_events(&mut conn, &stream, group, "worker-1", 10)
            .await
            .expect("consume failed");
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].1.aggregate_id, "product:1001");

        let _: Result<(), _> = redis::cmd("DEL").arg(&stream).query_async(&mut conn).await;
    }

    #[tokio::test]
    async fn test_acknowledgment() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let stream = key("ack");
        let group = "test-group-ack";
        let _: Result<(), _> = redis::cmd("DEL").arg(&stream).query_async(&mut conn).await;

        let ids = publish_test_events(&mut conn, &stream, 3).await;
        create_consumer_group(&mut conn, &stream, group)
            .await
            .expect("create group");

        // Consume all 3
        let entries = consume_events(&mut conn, &stream, group, "worker-1", 10)
            .await
            .expect("consume");
        assert_eq!(entries.len(), 3);

        // Acknowledge the first one
        acknowledge(&mut conn, &stream, group, &ids[0])
            .await
            .expect("ack failed");

        // Reading pending should now show 2 (unacknowledged)
        let pending = pending_events(&mut conn, &stream, group, "worker-1", 10)
            .await
            .expect("pending failed");
        assert_eq!(pending.len(), 2);

        let _: Result<(), _> = redis::cmd("DEL").arg(&stream).query_async(&mut conn).await;
    }

    #[tokio::test]
    async fn test_pel_recovery_after_crash() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let stream = key("pel");
        let group = "test-group-pel";
        let _: Result<(), _> = redis::cmd("DEL").arg(&stream).query_async(&mut conn).await;

        let ids = publish_test_events(&mut conn, &stream, 5).await;
        create_consumer_group(&mut conn, &stream, group)
            .await
            .expect("create group");

        // "worker-1" consumes 3 events but crashes (never acknowledges)
        let _ = consume_events(&mut conn, &stream, group, "worker-1", 3)
            .await
            .expect("consume");

        // "worker-2" comes along and claims worker-1's pending events
        let claimed = claim_pending(
            &mut conn,
            &stream,
            group,
            "worker-2",
            0, // min_idle_ms = 0 for testing (immediate claim)
            &ids[0..3],
        )
        .await
        .expect("claim failed");

        assert_eq!(claimed.len(), 3, "Should claim all 3 pending events");

        // worker-2 should now have 3 pending events
        let pending = pending_events(&mut conn, &stream, group, "worker-2", 10)
            .await
            .expect("pending failed");
        assert_eq!(pending.len(), 3);

        let _: Result<(), _> = redis::cmd("DEL").arg(&stream).query_async(&mut conn).await;
    }
}
