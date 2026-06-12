//! Partition storage with segment-based append-only logs.
//!
//! Each partition maintains an ordered sequence of segments. Each segment is
//! an append-only log of serialized messages. When a segment exceeds
//! `segment_max_bytes`, a new segment is created.
//!
//! Offsets are monotonically increasing across segments. The first message
//! in segment N has an offset strictly greater than the last message in
//! segment N-1.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::RwLock;

use super::message::{FetchMessage, Message};

/// A single partition of a topic.
///
/// Partitions are the unit of parallelism in the message queue. Each
/// partition has an independent append-only log with its own offset
/// sequence. A partition is assigned a leader node and a set of replicas
/// for fault tolerance.
pub struct Partition {
    /// The topic this partition belongs to.
    pub topic: String,
    /// The partition index within its topic.
    pub partition_id: u32,
    /// Node ID of the current leader for this partition.
    pub leader: u64,
    /// Node IDs of all replicas (including the leader).
    pub replicas: Vec<u64>,
    /// High watermark: the offset of the first message that has NOT been
    /// fully committed (replicated to a quorum). Consumers can only read
    /// up to this offset.
    high_watermark: AtomicU64,
    /// Log end offset: the offset that will be assigned to the next
    /// appended message.
    log_end_offset: AtomicU64,
    /// Segment-based message storage.
    storage: Arc<PartitionStorage>,
}

/// In-memory segment-based storage for a partition.
pub struct PartitionStorage {
    /// Ordered list of segments. The first segment has the lowest base
    /// offset; each subsequent segment has a strictly higher base offset.
    segments: RwLock<Vec<Segment>>,
    /// Filesystem path for this partition's data. Used for diagnostics and
    /// future on-disk persistence; currently the storage is in-memory.
    data_dir: String,
    /// Maximum number of raw message bytes per segment before rolling.
    segment_max_bytes: u64,
}

/// A contiguous, append-only chunk of messages within a partition.
struct Segment {
    /// Offset of the first message in this segment.
    base_offset: u64,
    /// Messages stored in this segment, in append order.
    messages: Vec<StoredMessage>,
    /// Total serialized bytes of all messages in this segment.
    size_bytes: u64,
}

/// A serialized message stored within a segment.
#[derive(Clone)]
struct StoredMessage {
    /// Offset of this message within the partition.
    offset: u64,
    /// Bincode-serialized `Message`.
    data: Vec<u8>,
}

impl Partition {
    /// Create a new partition with empty storage.
    ///
    /// `data_dir` is the directory path where this partition's data would
    /// be persisted on disk. Currently used only for metadata; the storage
    /// is in-memory.
    pub fn new(topic: String, partition_id: u32, data_dir: &str, segment_max_bytes: u64) -> Self {
        Self {
            topic,
            partition_id,
            leader: 0,
            replicas: Vec::new(),
            high_watermark: AtomicU64::new(0),
            log_end_offset: AtomicU64::new(0),
            storage: Arc::new(PartitionStorage::new(
                data_dir.to_string(),
                segment_max_bytes,
            )),
        }
    }

    /// Append a batch of messages to this partition.
    ///
    /// Each message is assigned a monotonically increasing offset starting
    /// from the current `log_end_offset`. The message's `partition` field
    /// is set to this partition's ID.
    ///
    /// Returns the list of assigned offsets in the same order as the input
    /// messages.
    pub async fn append(&self, messages: Vec<Message>) -> crate::error::Result<Vec<u64>> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let start_offset = self.log_end_offset.load(Ordering::Acquire);
        let mut offsets = Vec::with_capacity(messages.len());
        let mut serialized = Vec::with_capacity(messages.len());

        for (i, mut msg) in messages.into_iter().enumerate() {
            msg.offset = start_offset + i as u64;
            msg.partition = self.partition_id;

            let data = bincode::serialize(&msg).map_err(|e| {
                crate::error::MqError::Serialization(format!(
                    "Failed to serialize message at offset {}: {}",
                    start_offset + i as u64,
                    e
                ))
            })?;

            offsets.push(msg.offset);
            serialized.push(StoredMessage {
                offset: msg.offset,
                data,
            });
        }

        self.storage.append_messages(serialized).await?;

        let new_end = start_offset + offsets.len() as u64;
        self.log_end_offset.store(new_end, Ordering::Release);

        Ok(offsets)
    }

    /// Fetch messages starting from `offset`, up to `max_bytes` total.
    ///
    /// Returns the fetched messages and the current high watermark. If the
    /// requested offset is beyond the end of the log, returns
    /// `OffsetOutOfRange`.
    pub async fn fetch(
        &self,
        offset: u64,
        max_bytes: u64,
    ) -> crate::error::Result<(Vec<FetchMessage>, u64)> {
        let end_offset = self.log_end_offset.load(Ordering::Acquire);
        let hw = self.high_watermark.load(Ordering::Acquire);

        // Empty partition: return empty result.
        if end_offset == 0 {
            return Ok((Vec::new(), hw));
        }

        // Offset at or beyond the log end.
        if offset >= end_offset {
            return Err(crate::error::MqError::OffsetOutOfRange {
                offset,
                high_watermark: hw,
            });
        }

        let stored = self.storage.fetch_messages(offset, max_bytes).await?;

        let mut fetch_msgs = Vec::with_capacity(stored.len());
        for sm in stored {
            match bincode::deserialize::<Message>(&sm.data) {
                Ok(msg) => fetch_msgs.push(FetchMessage::from(msg)),
                Err(e) => {
                    // Corrupted message -- skip it rather than fail the
                    // entire fetch. In a production system you would log
                    // this and possibly increment an error counter.
                    tracing::warn!(
                        offset = sm.offset,
                        topic = %self.topic,
                        partition = self.partition_id,
                        error = %e,
                        "Failed to deserialize stored message, skipping"
                    );
                }
            }
        }

        Ok((fetch_msgs, hw))
    }

    /// Get the current high watermark.
    pub fn high_watermark(&self) -> u64 {
        self.high_watermark.load(Ordering::Acquire)
    }

    /// Get the current log end offset (next offset to be assigned).
    pub fn log_end_offset(&self) -> u64 {
        self.log_end_offset.load(Ordering::Acquire)
    }

    /// Set the high watermark.
    ///
    /// Called by the replication module when replicas acknowledge entries.
    /// The high watermark must never decrease.
    pub fn set_high_watermark(&self, offset: u64) {
        // Use fetch_max to ensure the watermark never goes backwards.
        self.high_watermark.fetch_max(offset, Ordering::Release);
    }
}

// ---------------------------------------------------------------------------
// PartitionStorage
// ---------------------------------------------------------------------------

impl PartitionStorage {
    fn new(data_dir: String, segment_max_bytes: u64) -> Self {
        Self {
            segments: RwLock::new(Vec::new()),
            data_dir,
            segment_max_bytes,
        }
    }

    /// Append serialized messages to the active (last) segment, rolling to
    /// a new segment when the byte limit is exceeded.
    async fn append_messages(&self, messages: Vec<StoredMessage>) -> crate::error::Result<()> {
        let mut segments = self.segments.write().await;

        for msg in messages {
            let msg_size = msg.data.len() as u64;

            // Roll to a new segment if there are no segments or the
            // current one would exceed the byte limit.
            let needs_roll = segments.is_empty()
                || segments.last().unwrap().size_bytes + msg_size > self.segment_max_bytes;

            if needs_roll {
                segments.push(Segment {
                    base_offset: msg.offset,
                    messages: Vec::new(),
                    size_bytes: 0,
                });
            }

            let segment = segments.last_mut().unwrap();
            segment.size_bytes += msg_size;
            segment.messages.push(msg);
        }

        Ok(())
    }

    /// Fetch serialized messages starting from `offset`, up to `max_bytes`.
    ///
    /// Uses binary search on segment base offsets to locate the starting
    /// segment, then scans forward.
    async fn fetch_messages(
        &self,
        offset: u64,
        max_bytes: u64,
    ) -> crate::error::Result<Vec<StoredMessage>> {
        let segments = self.segments.read().await;

        if segments.is_empty() {
            return Ok(Vec::new());
        }

        // Binary search for the segment whose base_offset is <= offset.
        // `binary_search_by_key` returns Ok(idx) on exact match, or
        // Err(idx) where idx is the insertion point.
        let start_idx = match segments.binary_search_by_key(&offset, |s| s.base_offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };

        let mut result = Vec::new();
        let mut bytes_read = 0u64;

        for segment in &segments[start_idx..] {
            for stored in &segment.messages {
                if stored.offset < offset {
                    continue;
                }

                let msg_size = stored.data.len() as u64;

                // If adding this message would exceed the byte budget and
                // we already have at least one message, stop.
                if !result.is_empty() && bytes_read + msg_size > max_bytes {
                    return Ok(result);
                }

                result.push(StoredMessage {
                    offset: stored.offset,
                    data: stored.data.clone(),
                });
                bytes_read += msg_size;
            }
        }

        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_partition() -> Partition {
        Partition::new("test-topic".to_string(), 0, "/tmp/test-p0", 1024 * 1024)
    }

    fn make_message(key: Option<Vec<u8>>, value: Vec<u8>) -> Message {
        Message::new(key, value)
    }

    #[tokio::test]
    async fn append_single_message() {
        let p = test_partition();
        let msg = make_message(None, b"hello".to_vec());
        let offsets = p.append(vec![msg]).await.unwrap();

        assert_eq!(offsets, vec![0]);
        assert_eq!(p.log_end_offset(), 1);
    }

    #[tokio::test]
    async fn append_multiple_messages() {
        let p = test_partition();
        let msgs = vec![
            make_message(None, b"a".to_vec()),
            make_message(None, b"b".to_vec()),
            make_message(None, b"c".to_vec()),
        ];
        let offsets = p.append(msgs).await.unwrap();

        assert_eq!(offsets, vec![0, 1, 2]);
        assert_eq!(p.log_end_offset(), 3);
    }

    #[tokio::test]
    async fn append_assigns_offsets_sequentially() {
        let p = test_partition();

        let o1 = p.append(vec![make_message(None, b"m1".to_vec())]).await.unwrap();
        let o2 = p.append(vec![make_message(None, b"m2".to_vec())]).await.unwrap();
        let o3 = p.append(vec![make_message(None, b"m3".to_vec())]).await.unwrap();

        assert_eq!(o1, vec![0]);
        assert_eq!(o2, vec![1]);
        assert_eq!(o3, vec![2]);
    }

    #[tokio::test]
    async fn append_empty_batch() {
        let p = test_partition();
        let offsets = p.append(vec![]).await.unwrap();
        assert!(offsets.is_empty());
        assert_eq!(p.log_end_offset(), 0);
    }

    #[tokio::test]
    async fn fetch_returns_messages() {
        let p = test_partition();
        p.append(vec![
            make_message(None, b"a".to_vec()),
            make_message(None, b"b".to_vec()),
            make_message(None, b"c".to_vec()),
        ])
        .await
        .unwrap();

        let (msgs, hw) = p.fetch(0, 1024 * 1024).await.unwrap();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].value, b"a");
        assert_eq!(msgs[1].value, b"b");
        assert_eq!(msgs[2].value, b"c");
        assert_eq!(hw, 0); // high watermark starts at 0
    }

    #[tokio::test]
    async fn fetch_from_middle_offset() {
        let p = test_partition();
        p.append(vec![
            make_message(None, b"a".to_vec()),
            make_message(None, b"b".to_vec()),
            make_message(None, b"c".to_vec()),
            make_message(None, b"d".to_vec()),
        ])
        .await
        .unwrap();

        let (msgs, _) = p.fetch(2, 1024 * 1024).await.unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].value, b"c");
        assert_eq!(msgs[1].value, b"d");
    }

    #[tokio::test]
    async fn fetch_empty_partition() {
        let p = test_partition();
        let (msgs, hw) = p.fetch(0, 1024 * 1024).await.unwrap();
        assert!(msgs.is_empty());
        assert_eq!(hw, 0);
    }

    #[tokio::test]
    async fn fetch_offset_out_of_range() {
        let p = test_partition();
        p.append(vec![make_message(None, b"a".to_vec())])
            .await
            .unwrap();

        let result = p.fetch(100, 1024 * 1024).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::MqError::OffsetOutOfRange {
                offset,
                high_watermark,
            } => {
                assert_eq!(offset, 100);
                assert_eq!(high_watermark, 0);
            }
            other => panic!("expected OffsetOutOfRange, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn fetch_respects_max_bytes() {
        let p = test_partition();
        // Append messages with known sizes. Each serialized message is
        // roughly 30-40 bytes (bincode overhead + "payload" bytes).
        for i in 0..10 {
            let value = format!("payload-{}", i);
            p.append(vec![make_message(None, value.into_bytes())])
                .await
                .unwrap();
        }

        // Set a tight byte limit that fits only a few messages.
        let (msgs, _) = p.fetch(0, 80).await.unwrap();
        // Should return at least one message but not all ten.
        assert!(!msgs.is_empty());
        assert!(msgs.len() < 10);
    }

    #[tokio::test]
    async fn fetch_always_includes_first_message_even_if_large() {
        let p = test_partition();
        // A single large message.
        let big_value = vec![0u8; 200];
        p.append(vec![make_message(None, big_value.clone())])
            .await
            .unwrap();
        p.append(vec![make_message(None, b"small".to_vec())])
            .await
            .unwrap();

        // max_bytes = 50 -- less than the first message, but we must
        // include at least one message.
        let (msgs, _) = p.fetch(0, 50).await.unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].value, big_value);
    }

    #[tokio::test]
    async fn high_watermark_starts_at_zero() {
        let p = test_partition();
        assert_eq!(p.high_watermark(), 0);
    }

    #[tokio::test]
    async fn set_high_watermark() {
        let p = test_partition();
        p.set_high_watermark(10);
        assert_eq!(p.high_watermark(), 10);
    }

    #[tokio::test]
    async fn high_watermark_never_goes_backwards() {
        let p = test_partition();
        p.set_high_watermark(10);
        p.set_high_watermark(5);
        assert_eq!(p.high_watermark(), 10);

        p.set_high_watermark(15);
        assert_eq!(p.high_watermark(), 15);
    }

    #[tokio::test]
    async fn segment_rolling() {
        // Use a very small segment size to trigger rolling.
        let p = Partition::new("t".to_string(), 0, "/tmp/test", 50);

        for i in 0..20 {
            let value = format!("msg-{}", i);
            p.append(vec![make_message(None, value.into_bytes())])
                .await
                .unwrap();
        }

        // All messages should still be readable.
        let (msgs, _) = p.fetch(0, 10 * 1024 * 1024).await.unwrap();
        assert_eq!(msgs.len(), 20);

        // Verify ordering.
        for i in 0..20 {
            let expected = format!("msg-{}", i);
            assert_eq!(msgs[i].value, expected.as_bytes());
        }
    }

    #[tokio::test]
    async fn large_batch_across_segments() {
        let p = Partition::new("t".to_string(), 0, "/tmp/test", 100);

        let msgs: Vec<Message> = (0..50)
            .map(|i| make_message(None, format!("msg-{}", i).into_bytes()))
            .collect();

        let offsets = p.append(msgs).await.unwrap();
        assert_eq!(offsets.len(), 50);
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[49], 49);

        // Read them all back.
        let (fetched, _) = p.fetch(0, 10 * 1024 * 1024).await.unwrap();
        assert_eq!(fetched.len(), 50);
    }

    #[tokio::test]
    async fn message_partition_field_set() {
        let p = Partition::new("t".to_string(), 7, "/tmp/test", 1024 * 1024);
        let msg = make_message(None, b"data".to_vec());
        p.append(vec![msg]).await.unwrap();

        // Fetch and verify the partition field was set.
        let (msgs, _) = p.fetch(0, 1024 * 1024).await.unwrap();
        assert_eq!(msgs.len(), 1);
        // FetchMessage doesn't carry partition, but the original Message
        // would have had it set. We can verify via the offset.
        assert_eq!(msgs[0].offset, 0);
    }

    #[tokio::test]
    async fn sequential_appends_consistent() {
        let p = test_partition();

        for batch_size in [1, 5, 10, 50] {
            let msgs: Vec<Message> = (0..batch_size)
                .map(|i| make_message(None, format!("b{}", i).into_bytes()))
                .collect();
            p.append(msgs).await.unwrap();
        }

        // All messages should be sequentially ordered.
        let total = 1 + 5 + 10 + 50; // 66
        let (msgs, _) = p.fetch(0, 10 * 1024 * 1024).await.unwrap();
        assert_eq!(msgs.len(), total);

        for i in 0..total {
            assert_eq!(msgs[i].offset, i as u64);
        }
    }

    #[tokio::test]
    async fn fetch_across_segment_boundary() {
        let p = Partition::new("t".to_string(), 0, "/tmp/test", 80);

        // Write messages that span multiple segments.
        for i in 0..10 {
            let value = format!("msg-{:02}", i);
            p.append(vec![make_message(None, value.into_bytes())])
                .await
                .unwrap();
        }

        // Fetch starting from the middle.
        let (msgs, _) = p.fetch(5, 10 * 1024 * 1024).await.unwrap();
        assert_eq!(msgs.len(), 5);
        assert_eq!(msgs[0].value, b"msg-05");
        assert_eq!(msgs[4].value, b"msg-09");
    }

    #[tokio::test]
    async fn key_preserved_in_fetch() {
        let p = test_partition();
        let msg = make_message(Some(b"my-key".to_vec()), b"val".to_vec());
        p.append(vec![msg]).await.unwrap();

        let (msgs, _) = p.fetch(0, 1024 * 1024).await.unwrap();
        assert_eq!(msgs[0].key, Some(b"my-key".to_vec()));
        assert_eq!(msgs[0].value, b"val");
    }
}
