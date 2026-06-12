//! Persistent storage engine for the message queue.
//!
//! Messages are stored in append-only segment files with a sparse index for
//! efficient offset-based lookups. A write-ahead log (WAL) provides durability
//! guarantees when enabled.
//!
//! Sub-modules provide standalone, reusable building blocks:
//! - `wal`: Write-ahead log with CRC32 integrity checks
//! - `segment`: Segment-based log storage with append-only semantics
//! - `index`: Sparse offset index for fast lookups within segments

pub mod wal;
pub mod segment;
pub mod index;

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tracing::warn;

/// A single message stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredMessage {
    /// Monotonically increasing offset within the partition.
    pub offset: u64,
    /// Timestamp (millis since Unix epoch) when the message was produced.
    pub timestamp_ms: u64,
    /// The message key (optional, used for partitioning affinity).
    pub key: Option<Vec<u8>>,
    /// The message payload.
    pub value: Vec<u8>,
    /// Arbitrary headers attached by the producer.
    pub headers: Vec<(Vec<u8>, Vec<u8>)>,
}

/// Index entry mapping an offset to a byte position within a segment file.
#[derive(Debug, Clone, Copy)]
struct IndexEntry {
    offset: u64,
    byte_position: u64,
}

/// A single segment file on disk. Each segment is an append-only log of
/// messages with a sparse index for random access by offset.
struct Segment {
    /// Base offset of this segment (the offset of its first message).
    base_offset: u64,
    /// Next offset to be written.
    next_offset: u64,
    /// Current byte position in the segment file.
    current_position: u64,
    /// Sparse index: maps offsets to byte positions.
    index: Vec<IndexEntry>,
    /// The data file writer.
    writer: BufWriter<File>,
    /// Path to the segment data file.
    path: PathBuf,
    /// Maximum number of bytes before rolling to a new segment.
    max_bytes: u64,
    /// Bytes written to this segment so far.
    bytes_written: u64,
}

impl Segment {
    fn open(
        dir: &Path,
        base_offset: u64,
        max_bytes: u64,
    ) -> io::Result<Self> {
        let filename = format!("seg-{:020}.log", base_offset);
        let path = dir.join(&filename);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Segment {
            base_offset,
            next_offset: base_offset,
            current_position: 0,
            index: Vec::new(),
            writer: BufWriter::with_capacity(64 * 1024, file),
            path,
            max_bytes,
            bytes_written: 0,
        })
    }

    /// Append a message and return its offset.
    fn append(&mut self, message: &StoredMessage) -> io::Result<u64> {
        // Record index entry every 64 messages (sparse index).
        if self.next_offset % 64 == 0 || self.index.is_empty() {
            self.index.push(IndexEntry {
                offset: self.next_offset,
                byte_position: self.current_position,
            });
        }

        let serialized = bincode::serialize(message)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Write length prefix + payload.
        let len = serialized.len() as u32;
        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(&serialized)?;

        let offset = self.next_offset;
        self.next_offset += 1;
        self.current_position += 4 + serialized.len() as u64;
        self.bytes_written += 4 + serialized.len() as u64;

        Ok(offset)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    /// Read a single message at the given offset. Returns `None` if the
    /// offset is not within this segment.
    fn read_at(&mut self, offset: u64) -> io::Result<Option<StoredMessage>> {
        if offset < self.base_offset || offset >= self.next_offset {
            return Ok(None);
        }

        // Flush the writer so that buffered data is visible to the new file
        // handle we are about to open.
        self.writer.flush()?;

        let file = File::open(&self.path)?;
        let mut reader = io::BufReader::new(file);

        // Find the byte position from the index.
        let byte_pos = self.find_byte_position(offset);
        reader.seek(SeekFrom::Start(byte_pos))?;

        // Read the message at this offset.
        read_message_at_offset(&mut reader, offset, self.base_offset)
    }

    /// Find the byte position for an offset using the sparse index.
    fn find_byte_position(&self, offset: u64) -> u64 {
        // Binary search for the index entry with the largest offset <= target.
        let idx = match self.index.binary_search_by(|entry| entry.offset.cmp(&offset)) {
            Ok(i) => i,
            Err(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
        };

        self.index[idx].byte_position
    }

    /// Check if this segment has exceeded its maximum byte capacity.
    fn is_full(&self) -> bool {
        self.bytes_written >= self.max_bytes
    }

    fn base_offset(&self) -> u64 {
        self.base_offset
    }

    fn next_offset(&self) -> u64 {
        self.next_offset
    }

    fn message_count(&self) -> u64 {
        self.next_offset - self.base_offset
    }
}

/// Read a single message starting from the given byte position in a reader.
fn read_message_at_offset(
    reader: &mut impl io::Read,
    target_offset: u64,
    base_offset: u64,
) -> io::Result<Option<StoredMessage>> {
    let mut current_offset = base_offset;

    loop {
        // Read length prefix.
        let mut len_buf = [0u8; 4];
        match reader.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        reader.read_exact(&mut payload)?;

        if current_offset == target_offset {
            let message: StoredMessage = bincode::deserialize(&payload)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            return Ok(Some(message));
        }

        current_offset += 1;
    }
}

/// The storage engine manages segments for all partitions.
pub struct StorageEngine {
    /// Root data directory.
    data_dir: PathBuf,
    /// Maximum bytes per segment.
    segment_max_bytes: u64,
    /// Whether the WAL is enabled.
    wal_enabled: bool,
    /// Per-partition segments. Key: "topic/partition".
    partitions: Mutex<HashMap<String, Vec<Segment>>>,
    /// WAL writer (shared across partitions when enabled).
    wal: Mutex<Option<BufWriter<File>>>,
}

impl StorageEngine {
    /// Create a new storage engine. Creates the data directory if it does not
    /// exist.
    pub fn new(data_dir: &str, segment_max_bytes: u64, wal_enabled: bool) -> Self {
        let data_dir = PathBuf::from(data_dir);

        // Ensure the data directory exists.
        if let Err(e) = fs::create_dir_all(&data_dir) {
            warn!(dir = %data_dir.display(), error = %e, "Failed to create data directory");
        }

        // Open WAL if enabled.
        let wal = if wal_enabled {
            let wal_path = data_dir.join("wal.log");
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&wal_path)
            {
                Ok(file) => Some(BufWriter::with_capacity(64 * 1024, file)),
                Err(e) => {
                    warn!(error = %e, "Failed to open WAL, disabling");
                    None
                }
            }
        } else {
            None
        };

        StorageEngine {
            data_dir,
            segment_max_bytes,
            wal_enabled,
            partitions: Mutex::new(HashMap::new()),
            wal: Mutex::new(wal),
        }
    }

    /// Ensure a partition directory exists and has at least one segment.
    pub fn ensure_partition(
        &self,
        topic: &str,
        partition: u32,
    ) -> crate::error::Result<()> {
        let mut partitions = self
            .partitions
            .lock()
            .map_err(|e| crate::error::MqError::Storage(format!("Lock poisoned: {}", e)))?;

        let key = format!("{}/{}", topic, partition);
        if partitions.contains_key(&key) {
            return Ok(());
        }

        let partition_dir = self.data_dir.join(topic).join(partition.to_string());
        fs::create_dir_all(&partition_dir).map_err(|e| {
            crate::error::MqError::Storage(format!(
                "Failed to create partition dir: {}",
                e
            ))
        })?;

        let segment = Segment::open(&partition_dir, 0, self.segment_max_bytes)
            .map_err(|e| {
                crate::error::MqError::Storage(format!(
                    "Failed to open segment: {}",
                    e
                ))
            })?;

        partitions.insert(key, vec![segment]);
        Ok(())
    }

    /// Append a message to a topic/partition and return its offset.
    pub fn append(
        &self,
        topic: &str,
        partition: u32,
        message: &StoredMessage,
    ) -> crate::error::Result<u64> {
        // Write to WAL first if enabled.
        if self.wal_enabled {
            self.write_to_wal(topic, partition, message)?;
        }

        let mut partitions = self
            .partitions
            .lock()
            .map_err(|e| crate::error::MqError::Storage(format!("Lock poisoned: {}", e)))?;

        let key = format!("{}/{}", topic, partition);
        let segments = partitions.get_mut(&key).ok_or_else(|| {
            crate::error::MqError::PartitionNotFound {
                topic: topic.to_string(),
                partition,
            }
        })?;

        let last = segments.last_mut().unwrap();

        // Roll to a new segment if the current one is full.
        if last.is_full() {
            let new_base = last.next_offset();
            let partition_dir = self.data_dir.join(topic).join(partition.to_string());
            let new_segment =
                Segment::open(&partition_dir, new_base, self.segment_max_bytes)
                    .map_err(|e| {
                        crate::error::MqError::Storage(format!(
                            "Failed to create new segment: {}",
                            e
                        ))
                    })?;
            segments.push(new_segment);
        }

        let offset = segments.last_mut().unwrap().append(message).map_err(|e| {
            crate::error::MqError::Storage(format!("Failed to append: {}", e))
        })?;

        Ok(offset)
    }

    /// Read a single message by offset.
    pub fn read(
        &self,
        topic: &str,
        partition: u32,
        offset: u64,
    ) -> crate::error::Result<Option<StoredMessage>> {
        let mut partitions = self
            .partitions
            .lock()
            .map_err(|e| crate::error::MqError::Storage(format!("Lock poisoned: {}", e)))?;

        let key = format!("{}/{}", topic, partition);
        let segments = partitions.get_mut(&key).ok_or_else(|| {
            crate::error::MqError::PartitionNotFound {
                topic: topic.to_string(),
                partition,
            }
        })?;

        // Search segments in reverse order (newest first).
        for segment in segments.iter_mut().rev() {
            if offset >= segment.base_offset() && offset < segment.next_offset() {
                return segment.read_at(offset).map_err(|e| {
                    crate::error::MqError::Storage(format!(
                        "Failed to read offset {}: {}",
                        offset, e
                    ))
                });
            }
        }

        Ok(None)
    }

    /// Read a batch of messages starting at `offset`, up to `max_count`
    /// messages or `max_bytes` total bytes.
    pub fn read_batch(
        &self,
        topic: &str,
        partition: u32,
        offset: u64,
        max_count: usize,
        max_bytes: usize,
    ) -> crate::error::Result<Vec<StoredMessage>> {
        let mut messages = Vec::with_capacity(max_count);
        let mut total_bytes = 0usize;
        let mut current_offset = offset;

        for _ in 0..max_count {
            match self.read(topic, partition, current_offset)? {
                Some(msg) => {
                    let msg_size = bincode::serialized_size(&msg).unwrap_or(0) as usize;
                    if total_bytes + msg_size > max_bytes && !messages.is_empty() {
                        break;
                    }
                    total_bytes += msg_size;
                    messages.push(msg);
                    current_offset += 1;
                }
                None => break,
            }
        }

        Ok(messages)
    }

    /// Return the high watermark (next offset to be written) for a partition.
    pub fn high_watermark(
        &self,
        topic: &str,
        partition: u32,
    ) -> crate::error::Result<u64> {
        let partitions = self
            .partitions
            .lock()
            .map_err(|e| crate::error::MqError::Storage(format!("Lock poisoned: {}", e)))?;

        let key = format!("{}/{}", topic, partition);
        let segments = partitions.get(&key).ok_or_else(|| {
            crate::error::MqError::PartitionNotFound {
                topic: topic.to_string(),
                partition,
            }
        })?;

        Ok(segments.last().map_or(0, |s| s.next_offset()))
    }

    /// Return the total message count for a partition.
    pub fn message_count(
        &self,
        topic: &str,
        partition: u32,
    ) -> crate::error::Result<u64> {
        let partitions = self
            .partitions
            .lock()
            .map_err(|e| crate::error::MqError::Storage(format!("Lock poisoned: {}", e)))?;

        let key = format!("{}/{}", topic, partition);
        let segments = partitions.get(&key).ok_or_else(|| {
            crate::error::MqError::PartitionNotFound {
                topic: topic.to_string(),
                partition,
            }
        })?;

        Ok(segments.iter().map(|s| s.message_count()).sum())
    }

    /// Flush all open segments and the WAL to disk.
    pub fn flush(&self) {
        if let Ok(mut partitions) = self.partitions.lock() {
            for segments in partitions.values_mut() {
                for segment in segments.iter_mut() {
                    if let Err(e) = segment.flush() {
                        warn!(error = %e, "Failed to flush segment");
                    }
                }
            }
        }

        if let Ok(mut wal) = self.wal.lock() {
            if let Some(ref mut writer) = *wal {
                if let Err(e) = writer.flush() {
                    warn!(error = %e, "Failed to flush WAL");
                }
            }
        }
    }

    /// Write a message to the WAL for crash recovery.
    fn write_to_wal(
        &self,
        topic: &str,
        partition: u32,
        message: &StoredMessage,
    ) -> crate::error::Result<()> {
        let mut wal = self
            .wal
            .lock()
            .map_err(|e| crate::error::MqError::Storage(format!("Lock poisoned: {}", e)))?;

        let writer = wal.as_mut().ok_or_else(|| {
            crate::error::MqError::Storage("WAL not available".to_string())
        })?;

        // WAL entry: [topic_len][topic][partition][msg_len][msg]
        let topic_bytes = topic.as_bytes();
        let topic_len = topic_bytes.len() as u32;
        writer.write_all(&topic_len.to_le_bytes()).map_err(|e| {
            crate::error::MqError::Storage(format!("WAL write failed: {}", e))
        })?;
        writer.write_all(topic_bytes).map_err(|e| {
            crate::error::MqError::Storage(format!("WAL write failed: {}", e))
        })?;
        writer
            .write_all(&partition.to_le_bytes())
            .map_err(|e| {
                crate::error::MqError::Storage(format!("WAL write failed: {}", e))
            })?;

        let serialized = bincode::serialize(message).map_err(|e| {
            crate::error::MqError::Serialization(format!(
                "WAL serialization failed: {}",
                e
            ))
        })?;
        let msg_len = serialized.len() as u32;
        writer.write_all(&msg_len.to_le_bytes()).map_err(|e| {
            crate::error::MqError::Storage(format!("WAL write failed: {}", e))
        })?;
        writer.write_all(&serialized).map_err(|e| {
            crate::error::MqError::Storage(format!("WAL write failed: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_storage() -> StorageEngine {
        let dir = tempfile::tempdir().unwrap();
        StorageEngine::new(&dir.path().to_string_lossy(), 4096, false)
    }

    fn test_storage_with_wal() -> StorageEngine {
        let dir = tempfile::tempdir().unwrap();
        StorageEngine::new(&dir.path().to_string_lossy(), 4096, true)
    }

    fn make_message(offset: u64, payload: &str) -> StoredMessage {
        StoredMessage {
            offset,
            timestamp_ms: 1_000_000 + offset,
            key: None,
            value: payload.as_bytes().to_vec(),
            headers: vec![],
        }
    }

    #[test]
    fn append_and_read_single_message() {
        let storage = test_storage();
        storage.ensure_partition("topic1", 0).unwrap();

        let msg = make_message(0, "hello");
        let offset = storage.append("topic1", 0, &msg).unwrap();
        assert_eq!(offset, 0);

        let read = storage.read("topic1", 0, 0).unwrap().unwrap();
        assert_eq!(read.value, b"hello");
    }

    #[test]
    fn append_multiple_messages() {
        let storage = test_storage();
        storage.ensure_partition("topic1", 0).unwrap();

        for i in 0..100 {
            let msg = make_message(i, &format!("message-{}", i));
            let offset = storage.append("topic1", 0, &msg).unwrap();
            assert_eq!(offset, i);
        }

        let hw = storage.high_watermark("topic1", 0).unwrap();
        assert_eq!(hw, 100);

        let count = storage.message_count("topic1", 0).unwrap();
        assert_eq!(count, 100);

        // Read a message from the middle.
        let read = storage.read("topic1", 0, 50).unwrap().unwrap();
        assert_eq!(read.value, b"message-50");
    }

    #[test]
    fn read_batch() {
        let storage = test_storage();
        storage.ensure_partition("topic1", 0).unwrap();

        for i in 0..10 {
            let msg = make_message(i, &format!("msg-{}", i));
            storage.append("topic1", 0, &msg).unwrap();
        }

        let batch = storage
            .read_batch("topic1", 0, 0, 5, 1024 * 1024)
            .unwrap();
        assert_eq!(batch.len(), 5);
        assert_eq!(batch[0].value, b"msg-0");
        assert_eq!(batch[4].value, b"msg-4");
    }

    #[test]
    fn read_batch_respects_byte_limit() {
        let storage = test_storage();
        storage.ensure_partition("topic1", 0).unwrap();

        for i in 0..10 {
            // Each message is ~100 bytes.
            let payload = "x".repeat(100);
            let msg = make_message(i, &payload);
            storage.append("topic1", 0, &msg).unwrap();
        }

        // Set a tight byte limit.
        let batch = storage
            .read_batch("topic1", 0, 0, 10, 250)
            .unwrap();
        // Should return only a few messages.
        assert!(batch.len() < 10);
        assert!(!batch.is_empty());
    }

    #[test]
    fn read_nonexistent_offset() {
        let storage = test_storage();
        storage.ensure_partition("topic1", 0).unwrap();

        let msg = make_message(0, "hello");
        storage.append("topic1", 0, &msg).unwrap();

        let read = storage.read("topic1", 0, 999).unwrap();
        assert!(read.is_none());
    }

    #[test]
    fn partition_not_found() {
        let storage = test_storage();
        let result = storage.read("nonexistent", 0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn segment_rolling() {
        // Use a very small segment size so rolling happens.
        let dir = tempfile::tempdir().unwrap();
        let storage =
            StorageEngine::new(&dir.path().to_string_lossy(), 200, false);
        storage.ensure_partition("topic1", 0).unwrap();

        for i in 0..50 {
            let payload = "x".repeat(50);
            let msg = make_message(i, &payload);
            storage.append("topic1", 0, &msg).unwrap();
        }

        // All messages should still be readable.
        for i in 0..50 {
            let read = storage.read("topic1", 0, i).unwrap();
            assert!(read.is_some(), "offset {} should be readable", i);
        }
    }

    #[test]
    fn wal_enabled_append() {
        let storage = test_storage_with_wal();
        storage.ensure_partition("topic1", 0).unwrap();

        let msg = make_message(0, "wal-test");
        let offset = storage.append("topic1", 0, &msg).unwrap();
        assert_eq!(offset, 0);

        let read = storage.read("topic1", 0, 0).unwrap().unwrap();
        assert_eq!(read.value, b"wal-test");
    }

    #[test]
    fn high_watermark_empty_partition() {
        let storage = test_storage();
        storage.ensure_partition("topic1", 0).unwrap();
        assert_eq!(storage.high_watermark("topic1", 0).unwrap(), 0);
    }

    #[test]
    fn flush_does_not_panic() {
        let storage = test_storage();
        storage.ensure_partition("topic1", 0).unwrap();
        let msg = make_message(0, "data");
        storage.append("topic1", 0, &msg).unwrap();
        storage.flush();
    }

    #[test]
    fn multiple_partitions() {
        let storage = test_storage();
        storage.ensure_partition("topic1", 0).unwrap();
        storage.ensure_partition("topic1", 1).unwrap();
        storage.ensure_partition("topic1", 2).unwrap();

        let msg0 = make_message(0, "p0");
        let msg1 = make_message(0, "p1");
        let msg2 = make_message(0, "p2");

        storage.append("topic1", 0, &msg0).unwrap();
        storage.append("topic1", 1, &msg1).unwrap();
        storage.append("topic1", 2, &msg2).unwrap();

        assert_eq!(
            storage.read("topic1", 0, 0).unwrap().unwrap().value,
            b"p0"
        );
        assert_eq!(
            storage.read("topic1", 1, 0).unwrap().unwrap().value,
            b"p1"
        );
        assert_eq!(
            storage.read("topic1", 2, 0).unwrap().unwrap().value,
            b"p2"
        );
    }
}
