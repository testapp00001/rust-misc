//! # Solution 04: Redis Streams
//!
//! Complete implementation of Redis Streams for event publishing and reading.

use redis::aio::MultiplexedConnection as RedisConnection;
use redis::RedisError;

use super::p01_event_design::FlashSaleEvent;

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
/// Returns the Redis-generated stream entry ID.
pub async fn publish_event(
    conn: &mut RedisConnection,
    stream_key: &str,
    event: &FlashSaleEvent,
) -> Result<String, StreamError> {
    let json = serde_json::to_string(event)
        .map_err(|e| StreamError::Serialization(e.to_string()))?;

    let stream_id: String = redis::cmd("XADD")
        .arg(stream_key)
        .arg("*")
        .arg("event")
        .arg(&json)
        .query_async(conn)
        .await?;

    Ok(stream_id)
}

/// Read events from a Redis stream starting after the given ID.
///
/// Returns a vector of (stream_id, FlashSaleEvent) tuples.
pub async fn read_events(
    conn: &mut RedisConnection,
    stream_key: &str,
    last_id: &str,
    count: usize,
) -> Result<Vec<(String, FlashSaleEvent)>, StreamError> {
    let result: redis::Value = redis::cmd("XREAD")
        .arg("COUNT")
        .arg(count)
        .arg("STREAMS")
        .arg(stream_key)
        .arg(last_id)
        .query_async(conn)
        .await?;

    parse_stream_response(result)
}

/// Parse the XREAD response into a vector of (stream_id, event) tuples.
fn parse_stream_response(value: redis::Value) -> Result<Vec<(String, FlashSaleEvent)>, StreamError> {
    let mut events = Vec::new();

    // XREAD returns: [[stream_key, [[id, [field, value, ...]], ...]]]
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

            let stream_id = match &entry_parts[0] {
                redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes).to_string(),
                redis::Value::SimpleString(s) => s.clone(),
                _ => continue,
            };

            let fields = match &entry_parts[1] {
                redis::Value::Array(f) => f,
                _ => continue,
            };

            // Find the "event" field value
            let mut event_json: Option<String> = None;
            let mut i = 0;
            while i + 1 < fields.len() {
                let key = match &fields[i] {
                    redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes).to_string(),
                    redis::Value::SimpleString(s) => s.clone(),
                    _ => {
                        i += 1;
                        continue;
                    }
                };
                if key == "event" {
                    match &fields[i + 1] {
                        redis::Value::BulkString(bytes) => {
                            event_json = Some(String::from_utf8_lossy(bytes).to_string());
                        }
                        redis::Value::SimpleString(s) => {
                            event_json = Some(s.clone());
                        }
                        _ => {}
                    }
                }
                i += 2;
            }

            if let Some(json) = event_json {
                match serde_json::from_str::<FlashSaleEvent>(&json) {
                    Ok(event) => events.push((stream_id, event)),
                    Err(e) => {
                        return Err(StreamError::Deserialization(e.to_string()));
                    }
                }
            }
        }
    }

    Ok(events)
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

        let after_id = &ids[1];
        let entries = read_events(&mut conn, &key, after_id, 100)
            .await
            .expect("read failed");
        assert_eq!(entries.len(), 3);

        let _: Result<(), _> = redis::cmd("DEL").arg(&key).query_async(&mut conn).await;
    }
}
