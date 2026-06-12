//! # Solution 05: Consumer Groups
//!
//! Complete implementation of Redis Streams consumer groups with PEL recovery.

use redis::aio::MultiplexedConnection as RedisConnection;
use redis::RedisError;

use super::p01_event_design::FlashSaleEvent;

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

/// Helper: extract a string from a redis Value.
fn value_to_string(val: &redis::Value) -> Option<String> {
    match val {
        redis::Value::BulkString(bytes) => Some(String::from_utf8_lossy(bytes).to_string()),
        redis::Value::SimpleString(s) => Some(s.clone()),
        _ => None,
    }
}

/// Parse stream entries from a redis::Value response (shared helper).
fn parse_entries(value: redis::Value) -> Result<Vec<(String, FlashSaleEvent)>, ConsumerError> {
    let mut events = Vec::new();

    if let redis::Value::Nil = value {
        return Ok(events);
    }

    let streams = match &value {
        redis::Value::Array(streams) => streams,
        _ => return Ok(events),
    };

    for stream in streams {
        let entries = match stream {
            redis::Value::Array(parts) if parts.len() >= 2 => &parts[1],
            _ => continue,
        };

        let entry_list = match entries {
            redis::Value::Array(el) => el,
            _ => continue,
        };

        for entry in entry_list {
            let entry_parts = match entry {
                redis::Value::Array(ep) => ep,
                _ => continue,
            };

            if entry_parts.len() < 2 {
                continue;
            }

            let stream_id = match value_to_string(&entry_parts[0]) {
                Some(id) => id,
                None => continue,
            };

            let fields = match &entry_parts[1] {
                redis::Value::Array(f) => f,
                _ => continue,
            };

            let mut event_json: Option<String> = None;
            let mut i = 0;
            while i + 1 < fields.len() {
                if let Some(key) = value_to_string(&fields[i]) {
                    if key == "event" {
                        event_json = value_to_string(&fields[i + 1]);
                    }
                }
                i += 2;
            }

            if let Some(json) = event_json {
                match serde_json::from_str::<FlashSaleEvent>(&json) {
                    Ok(event) => events.push((stream_id, event)),
                    Err(e) => {
                        return Err(ConsumerError::Deserialization(e.to_string()));
                    }
                }
            }
        }
    }

    Ok(events)
}

/// Create a consumer group on a stream.
///
/// Handles BUSYGROUP error (group already exists) gracefully.
pub async fn create_consumer_group(
    conn: &mut RedisConnection,
    stream_key: &str,
    group_name: &str,
) -> Result<(), ConsumerError> {
    let result: Result<(), RedisError> = redis::cmd("XGROUP")
        .arg("CREATE")
        .arg(stream_key)
        .arg(group_name)
        .arg("0")
        .arg("MKSTREAM")
        .query_async(conn)
        .await;

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("BUSYGROUP") {
                Ok(())
            } else {
                Err(ConsumerError::Redis(e))
            }
        }
    }
}

/// Consume events from a stream as part of a consumer group.
///
/// Uses `>` to read only new, undelivered messages.
pub async fn consume_events(
    conn: &mut RedisConnection,
    stream_key: &str,
    group_name: &str,
    consumer_name: &str,
    count: usize,
) -> Result<Vec<(String, FlashSaleEvent)>, ConsumerError> {
    let result: redis::Value = redis::cmd("XREADGROUP")
        .arg("GROUP")
        .arg(group_name)
        .arg(consumer_name)
        .arg("COUNT")
        .arg(count)
        .arg("STREAMS")
        .arg(stream_key)
        .arg(">")
        .query_async(conn)
        .await?;

    parse_entries(result)
}

/// Acknowledge that an event has been processed.
pub async fn acknowledge(
    conn: &mut RedisConnection,
    stream_key: &str,
    group_name: &str,
    event_id: &str,
) -> Result<(), ConsumerError> {
    let _: i32 = redis::cmd("XACK")
        .arg(stream_key)
        .arg(group_name)
        .arg(event_id)
        .query_async(conn)
        .await?;

    Ok(())
}

/// Read this consumer's pending (unacknowledged) events.
///
/// Uses `0` instead of `>` to read the consumer's pending entry list.
pub async fn pending_events(
    conn: &mut RedisConnection,
    stream_key: &str,
    group_name: &str,
    consumer_name: &str,
    count: usize,
) -> Result<Vec<(String, FlashSaleEvent)>, ConsumerError> {
    let result: redis::Value = redis::cmd("XREADGROUP")
        .arg("GROUP")
        .arg(group_name)
        .arg(consumer_name)
        .arg("COUNT")
        .arg(count)
        .arg("STREAMS")
        .arg(stream_key)
        .arg("0")
        .query_async(conn)
        .await?;

    parse_entries(result)
}

/// Claim pending events from another consumer that has been idle too long.
pub async fn claim_pending(
    conn: &mut RedisConnection,
    stream_key: &str,
    group_name: &str,
    consumer_name: &str,
    min_idle_ms: u64,
    event_ids: &[&str],
) -> Result<Vec<(String, FlashSaleEvent)>, ConsumerError> {
    let mut cmd = redis::cmd("XCLAIM");
    cmd.arg(stream_key)
        .arg(group_name)
        .arg(consumer_name)
        .arg(min_idle_ms);

    for id in event_ids {
        cmd.arg(*id);
    }

    let result: redis::Value = cmd.query_async(conn).await?;
    parse_entries(result)
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
        let id_refs: Vec<&str> = ids[0..3].iter().map(|s| s.as_str()).collect();
        let claimed = claim_pending(
            &mut conn,
            &stream,
            group,
            "worker-2",
            0,
            &id_refs,
        )
        .await
        .expect("claim failed");

        assert_eq!(claimed.len(), 3, "Should claim all 3 pending events");

        let pending = pending_events(&mut conn, &stream, group, "worker-2", 10)
            .await
            .expect("pending failed");
        assert_eq!(pending.len(), 3);

        let _: Result<(), _> = redis::cmd("DEL").arg(&stream).query_async(&mut conn).await;
    }
}
