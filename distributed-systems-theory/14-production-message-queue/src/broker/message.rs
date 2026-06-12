//! Message types for the broker layer.
//!
//! A `Message` is the fundamental unit of data in the queue. Messages carry
//! an optional key (used for partition affinity), a value (the payload), a
//! timestamp, arbitrary headers, and are assigned an offset and partition
//! number when appended to a partition log.
//!
//! `FetchMessage` is a lightweight projection returned to consumers, omitting
//! partition metadata that the consumer already knows.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single message stored in the broker.
///
/// Messages are immutable once appended to a partition. The `offset` and
/// `partition` fields are assigned by the partition on append; producers
/// should leave them at their default values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Optional key used for key-based partition routing. Messages with the
    /// same key are guaranteed to land on the same partition.
    pub key: Option<Vec<u8>>,
    /// The message payload.
    pub value: Vec<u8>,
    /// Server-side timestamp when the message was received.
    pub timestamp: DateTime<Utc>,
    /// Arbitrary key-value headers attached by the producer.
    pub headers: HashMap<String, Vec<u8>>,
    /// Monotonically increasing offset within the partition. Assigned on
    /// append; zero until then.
    pub offset: u64,
    /// Partition index this message belongs to. Assigned on append.
    pub partition: u32,
}

/// A batch of messages originating from a single producer, used for
/// idempotency tracking and efficient bulk transfers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatch {
    /// The messages in this batch.
    pub messages: Vec<Message>,
    /// Producer instance ID (UUID string).
    pub producer_id: String,
    /// Sequence number of the first message in this batch. Subsequent
    /// messages are at `base_sequence + 1`, `base_sequence + 2`, etc.
    pub base_sequence: u64,
}

/// A message returned from a `fetch` operation.
///
/// This is a consumer-facing projection that strips partition-level metadata
/// (the consumer already knows which partition it is reading from).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchMessage {
    /// Offset of this message within its partition.
    pub offset: u64,
    /// Optional message key.
    pub key: Option<Vec<u8>>,
    /// Message payload.
    pub value: Vec<u8>,
    /// Timestamp when the message was produced.
    pub timestamp: DateTime<Utc>,
    /// Arbitrary headers attached by the producer.
    pub headers: HashMap<String, Vec<u8>>,
}

impl Message {
    /// Create a new message with the given key and value.
    ///
    /// The timestamp is set to the current UTC time. Headers are empty.
    /// Offset and partition are set to zero and must be assigned by the
    /// partition before storage.
    pub fn new(key: Option<Vec<u8>>, value: Vec<u8>) -> Self {
        Self {
            key,
            value,
            timestamp: Utc::now(),
            headers: HashMap::new(),
            offset: 0,
            partition: 0,
        }
    }

    /// Calculate the approximate in-memory size of this message in bytes.
    ///
    /// This counts the key, value, and all header keys and values. It does
    /// not account for the overhead of the `HashMap` entries themselves, the
    /// `DateTime` timestamp, or the offset/partition fields. Use this for
    /// rough budgeting (e.g., segment roll decisions), not precise accounting.
    pub fn size(&self) -> usize {
        let key_size = self.key.as_ref().map_or(0, |k| k.len());
        let value_size = self.value.len();
        let headers_size: usize = self
            .headers
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum();
        key_size + value_size + headers_size
    }
}

impl From<Message> for FetchMessage {
    fn from(msg: Message) -> Self {
        Self {
            offset: msg.offset,
            key: msg.key,
            value: msg.value,
            timestamp: msg.timestamp,
            headers: msg.headers,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_message_defaults() {
        let msg = Message::new(Some(b"key".to_vec()), b"value".to_vec());
        assert_eq!(msg.key, Some(b"key".to_vec()));
        assert_eq!(msg.value, b"value");
        assert!(msg.headers.is_empty());
        assert_eq!(msg.offset, 0);
        assert_eq!(msg.partition, 0);
        // Timestamp should be recent (within the last second).
        let age = Utc::now()
            .signed_duration_since(msg.timestamp)
            .num_seconds();
        assert!(age >= 0 && age < 2);
    }

    #[test]
    fn new_message_no_key() {
        let msg = Message::new(None, b"payload".to_vec());
        assert!(msg.key.is_none());
        assert_eq!(msg.value, b"payload");
    }

    #[test]
    fn size_empty() {
        let msg = Message::new(None, Vec::new());
        assert_eq!(msg.size(), 0);
    }

    #[test]
    fn size_key_only() {
        let msg = Message::new(Some(b"abc".to_vec()), Vec::new());
        assert_eq!(msg.size(), 3);
    }

    #[test]
    fn size_value_only() {
        let msg = Message::new(None, b"hello".to_vec());
        assert_eq!(msg.size(), 5);
    }

    #[test]
    fn size_with_headers() {
        let mut msg = Message::new(None, Vec::new());
        msg.headers
            .insert("content-type".to_string(), b"application/json".to_vec());
        // "content-type" (12) + "application/json" (16) = 28
        assert_eq!(msg.size(), 28);
    }

    #[test]
    fn size_all_components() {
        let mut msg = Message::new(Some(b"k".to_vec()), b"v".to_vec());
        msg.headers.insert("h".to_string(), b"v".to_vec());
        // key=1 + value=1 + header_key=1 + header_value=1 = 4
        assert_eq!(msg.size(), 4);
    }

    #[test]
    fn into_fetch_message() {
        let mut msg = Message::new(Some(b"k".to_vec()), b"v".to_vec());
        msg.offset = 42;
        msg.partition = 3;
        msg.headers
            .insert("x".to_string(), b"y".to_vec());

        let fetch: FetchMessage = msg.clone().into();
        assert_eq!(fetch.offset, 42);
        assert_eq!(fetch.key, Some(b"k".to_vec()));
        assert_eq!(fetch.value, b"v");
        assert_eq!(fetch.headers.len(), 1);
        assert_eq!(fetch.headers.get("x").unwrap(), b"y");
    }

    #[test]
    fn message_serialization_roundtrip() {
        let mut msg = Message::new(Some(b"key".to_vec()), b"value".to_vec());
        msg.offset = 100;
        msg.partition = 2;
        msg.headers
            .insert("trace-id".to_string(), b"abc123".to_vec());

        let serialized = bincode::serialize(&msg).unwrap();
        let deserialized: Message = bincode::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.key, msg.key);
        assert_eq!(deserialized.value, msg.value);
        assert_eq!(deserialized.offset, 100);
        assert_eq!(deserialized.partition, 2);
        assert_eq!(deserialized.headers.len(), 1);
    }

    #[test]
    fn fetch_message_serialization_roundtrip() {
        let fetch = FetchMessage {
            offset: 42,
            key: Some(b"k".to_vec()),
            value: b"payload".to_vec(),
            timestamp: Utc::now(),
            headers: std::collections::HashMap::new(),
        };

        let serialized = bincode::serialize(&fetch).unwrap();
        let deserialized: FetchMessage = bincode::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.offset, 42);
        assert_eq!(deserialized.key, Some(b"k".to_vec()));
        assert_eq!(deserialized.value, b"payload");
    }

    #[test]
    fn batch_serialization_roundtrip() {
        let batch = MessageBatch {
            messages: vec![
                Message::new(None, b"m1".to_vec()),
                Message::new(Some(b"k".to_vec()), b"m2".to_vec()),
            ],
            producer_id: "prod-1".to_string(),
            base_sequence: 10,
        };

        let serialized = bincode::serialize(&batch).unwrap();
        let deserialized: MessageBatch = bincode::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.messages.len(), 2);
        assert_eq!(deserialized.producer_id, "prod-1");
        assert_eq!(deserialized.base_sequence, 10);
    }

    #[test]
    fn message_clone() {
        let msg = Message::new(Some(b"k".to_vec()), b"v".to_vec());
        let cloned = msg.clone();
        assert_eq!(msg.key, cloned.key);
        assert_eq!(msg.value, cloned.value);
    }
}
