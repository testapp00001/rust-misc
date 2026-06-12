//! Segment-based log storage.
//!
//! Each partition is backed by an ordered list of immutable (or append-only)
//! segment files. Segments are rolled when they exceed a configurable byte
//! limit. Old segments can be truncated for retention policy enforcement.
//!
//! On-disk format per record:
//! ```text
//! [offset: u64 LE] [timestamp_ms: u64 LE] [data_len: u32 LE] [data: [u8]]
//! ```
//!
//! This module provides `Segment` (an in-memory representation of a single
//! segment) and `SegmentLog` (a ordered collection of segments for one
//! partition).

use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{MqError, Result};

/// A single record stored within a segment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredRecord {
    /// Monotonically increasing offset within the partition.
    pub offset: u64,
    /// Milliseconds since Unix epoch.
    pub timestamp_ms: u64,
    /// The record payload.
    pub data: Vec<u8>,
}

/// The on-disk binary layout of a record: 8 + 8 + 4 + data.len() bytes.
const RECORD_HEADER_SIZE: usize = 20; // 8 (offset) + 8 (timestamp) + 4 (length)

/// An append-only segment file.
///
/// Segments are the fundamental unit of storage. Each segment covers a
/// contiguous range of offsets starting from `base_offset`. When the
/// segment reaches `max_bytes`, it is sealed and a new segment is created.
pub struct Segment {
    /// The first offset in this segment.
    pub base_offset: u64,
    /// Path to the segment data file on disk.
    path: PathBuf,
    /// In-memory buffer of records (written to disk on flush).
    records: Vec<StoredRecord>,
    /// Current size in bytes (approximation based on record sizes).
    size_bytes: usize,
    /// Maximum byte capacity before rolling.
    max_bytes: usize,
    /// Next offset to be assigned.
    next_offset: u64,
}

impl Segment {
    /// Create a new in-memory segment. The data file is not created until
    /// `flush()` is called.
    pub fn new(dir: impl AsRef<Path>, base_offset: u64, max_bytes: usize) -> Self {
        let path = dir.as_ref().join(format!("{:020}.seg", base_offset));
        Segment {
            base_offset,
            path,
            records: Vec::new(),
            size_bytes: 0,
            max_bytes,
            next_offset: base_offset,
        }
    }

    /// Append a record to the segment.
    ///
    /// Returns `true` if the record was accepted, or `false` if the
    /// segment is full and a new segment should be created.
    pub fn append(&mut self, offset: u64, data: Vec<u8>) -> bool {
        let record_size = RECORD_HEADER_SIZE + data.len();
        if self.size_bytes + record_size > self.max_bytes {
            return false; // Segment is full
        }

        let record = StoredRecord {
            offset,
            timestamp_ms: epoch_millis(),
            data,
        };

        self.size_bytes += record_size;
        self.next_offset = offset + 1;
        self.records.push(record);
        true
    }

    /// Read records starting from `offset`, up to `max_bytes` total.
    ///
    /// Returns records in offset order. Records before the requested
    /// offset are skipped; reading stops when the byte budget is exhausted
    /// or there are no more records.
    pub fn read(&self, offset: u64, max_bytes: usize) -> Vec<&StoredRecord> {
        let mut result = Vec::new();
        let mut bytes = 0;

        for record in &self.records {
            if record.offset < offset {
                continue;
            }
            let record_size = RECORD_HEADER_SIZE + record.data.len();
            if bytes + record_size > max_bytes && !result.is_empty() {
                break;
            }
            bytes += record_size;
            result.push(record);
        }

        result
    }

    /// Read a single record by exact offset.
    pub fn read_at(&self, offset: u64) -> Option<&StoredRecord> {
        self.records.iter().find(|r| r.offset == offset)
    }

    /// Persist all in-memory records to the segment file on disk.
    pub fn flush(&self) -> Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| MqError::Storage(format!(
                "Failed to create segment file '{}': {}",
                self.path.display(), e
            )))?;

        let mut writer = BufWriter::with_capacity(64 * 1024, file);

        for record in &self.records {
            writer.write_all(&record.offset.to_le_bytes())
                .map_err(|e| MqError::Storage(format!("Segment write failed: {}", e)))?;
            writer.write_all(&record.timestamp_ms.to_le_bytes())
                .map_err(|e| MqError::Storage(format!("Segment write failed: {}", e)))?;
            let data_len = record.data.len() as u32;
            writer.write_all(&data_len.to_le_bytes())
                .map_err(|e| MqError::Storage(format!("Segment write failed: {}", e)))?;
            writer.write_all(&record.data)
                .map_err(|e| MqError::Storage(format!("Segment write failed: {}", e)))?;
        }

        writer.flush()
            .map_err(|e| MqError::Storage(format!("Segment flush failed: {}", e)))?;

        Ok(())
    }

    /// Load a segment from its on-disk file.
    pub fn load(
        path: impl AsRef<Path>,
        base_offset: u64,
        max_bytes: usize,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let file = File::open(&path).map_err(|e| MqError::Storage(format!(
            "Failed to open segment '{}': {}", path.display(), e
        )))?;
        let mut reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut size_bytes = 0usize;

        loop {
            // Read offset (8 bytes).
            let mut offset_buf = [0u8; 8];
            match reader.read_exact(&mut offset_buf) {
                Ok(()) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(MqError::Storage(format!(
                    "Failed to read segment offset: {}", e
                ))),
            }
            let offset = u64::from_le_bytes(offset_buf);

            // Read timestamp (8 bytes).
            let mut ts_buf = [0u8; 8];
            reader.read_exact(&mut ts_buf)
                .map_err(|e| MqError::Storage(format!(
                    "Failed to read segment timestamp: {}", e
                )))?;
            let timestamp_ms = u64::from_le_bytes(ts_buf);

            // Read data length (4 bytes).
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf)
                .map_err(|e| MqError::Storage(format!(
                    "Failed to read segment data length: {}", e
                )))?;
            let data_len = u32::from_le_bytes(len_buf) as usize;

            // Read data.
            let mut data = vec![0u8; data_len];
            reader.read_exact(&mut data)
                .map_err(|e| MqError::Storage(format!(
                    "Failed to read segment data: {}", e
                )))?;

            let record_size = RECORD_HEADER_SIZE + data_len;
            size_bytes += record_size;

            records.push(StoredRecord {
                offset,
                timestamp_ms,
                data,
            });
        }

        let next_offset = records.last().map_or(base_offset, |r| r.offset + 1);

        Ok(Segment {
            base_offset,
            path,
            records,
            size_bytes,
            max_bytes,
            next_offset,
        })
    }

    /// Returns `true` if the segment cannot accept another minimum-sized
    /// record (a record with empty data, which costs `RECORD_HEADER_SIZE`
    /// bytes).
    pub fn is_full(&self) -> bool {
        self.size_bytes + RECORD_HEADER_SIZE > self.max_bytes
    }

    /// Number of records in this segment.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Byte size of all records (serialized form approximation).
    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    /// The next offset that will be assigned on append.
    pub fn next_offset(&self) -> u64 {
        self.next_offset
    }

    /// The first offset in this segment.
    pub fn base_offset(&self) -> u64 {
        self.base_offset
    }

    /// Path to the segment file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check whether an offset falls within this segment's range.
    pub fn contains_offset(&self, offset: u64) -> bool {
        offset >= self.base_offset && offset < self.next_offset
    }
}

/// A log composed of an ordered list of segments for a single partition.
///
/// The `SegmentLog` manages segment creation, rolling, flushing, and
/// reading across the full offset range.
pub struct SegmentLog {
    /// The directory containing all segment files for this partition.
    dir: PathBuf,
    /// Ordered list of segments, from oldest to newest.
    segments: Vec<Segment>,
    /// Maximum bytes per segment before rolling.
    max_bytes_per_segment: usize,
}

impl SegmentLog {
    /// Create a new empty segment log in the given directory.
    pub fn new(dir: impl AsRef<Path>, max_bytes_per_segment: usize) -> Self {
        let dir = dir.as_ref().to_path_buf();
        SegmentLog {
            dir,
            segments: Vec::new(),
            max_bytes_per_segment,
        }
    }

    /// Load an existing segment log from disk by reading all `.seg` files
    /// in the directory.
    pub fn load(dir: impl AsRef<Path>, max_bytes_per_segment: usize) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir).map_err(|e| MqError::Storage(format!(
            "Failed to create segment dir: {}", e
        )))?;

        let mut segments = Vec::new();

        let mut entries: Vec<_> = fs::read_dir(&dir)
            .map_err(|e| MqError::Storage(format!(
                "Failed to read segment dir: {}", e
            )))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map_or(false, |ext| ext == "seg")
            })
            .collect();

        // Sort by filename (which encodes the base offset).
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let segment = Segment::load(&entry.path(), 0, max_bytes_per_segment)?;
            segments.push(segment);
        }

        Ok(SegmentLog {
            dir,
            segments,
            max_bytes_per_segment,
        })
    }

    /// Append a record, automatically rolling to a new segment if needed.
    pub fn append(&mut self, offset: u64, data: Vec<u8>) -> Result<()> {
        // Create the first segment if empty.
        if self.segments.is_empty() {
            self.segments.push(Segment::new(&self.dir, offset, self.max_bytes_per_segment));
        }

        let last = self.segments.last_mut().unwrap();

        if last.is_full() {
            let new_base = last.next_offset();
            let new_segment = Segment::new(&self.dir, new_base, self.max_bytes_per_segment);
            self.segments.push(new_segment);
        }

        let accepted = self.segments.last_mut().unwrap().append(offset, data);
        if !accepted {
            return Err(MqError::Storage(
                "Failed to append to segment after roll".to_string(),
            ));
        }

        Ok(())
    }

    /// Read records starting from `offset`, across all segments.
    pub fn read(&self, offset: u64, max_bytes: usize) -> Vec<&StoredRecord> {
        let mut result = Vec::new();
        let mut remaining = max_bytes;

        for segment in &self.segments {
            if !segment.contains_offset(offset) && segment.base_offset() >= offset {
                // This segment starts after the offset -- but we might still
                // need records from it if no earlier segment has them.
            }

            let records = segment.read(offset, remaining);
            let consumed: usize = records.iter()
                .map(|r| RECORD_HEADER_SIZE + r.data.len())
                .sum();

            result.extend(records);
            remaining = remaining.saturating_sub(consumed);

            if remaining == 0 {
                break;
            }
        }

        result
    }

    /// Flush all segments to disk.
    pub fn flush(&self) -> Result<()> {
        for segment in &self.segments {
            segment.flush()?;
        }
        Ok(())
    }

    /// Total number of records across all segments.
    pub fn total_records(&self) -> usize {
        self.segments.iter().map(|s| s.record_count()).sum()
    }

    /// Next offset that would be written.
    pub fn next_offset(&self) -> u64 {
        self.segments.last().map_or(0, |s| s.next_offset())
    }

    /// Number of segments.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
}

/// Get current time as milliseconds since Unix epoch.
fn epoch_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    // --- Segment tests ---

    #[test]
    fn segment_append_and_read() {
        let dir = temp_dir();
        let mut seg = Segment::new(dir.path(), 0, 1024 * 1024);

        assert!(seg.append(0, b"hello".to_vec()));
        assert!(seg.append(1, b"world".to_vec()));

        assert_eq!(seg.record_count(), 2);
        assert_eq!(seg.next_offset(), 2);

        let records = seg.read(0, 1024);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].data, b"hello");
        assert_eq!(records[1].data, b"world");
    }

    #[test]
    fn segment_full_rejects_append() {
        let dir = temp_dir();
        // Very small segment: RECORD_HEADER_SIZE(20) + 5 = 25 bytes minimum
        let mut seg = Segment::new(dir.path(), 0, 30);

        assert!(seg.append(0, b"aaa".to_vec())); // 23 bytes, fits
        assert!(!seg.append(1, b"bbb".to_vec())); // 23 more, exceeds 30
        assert!(seg.is_full());
    }

    #[test]
    fn segment_read_from_offset() {
        let dir = temp_dir();
        let mut seg = Segment::new(dir.path(), 10, 1024 * 1024);

        for i in 10..20 {
            seg.append(i, format!("msg-{}", i).into_bytes());
        }

        let records = seg.read(15, 1024);
        assert_eq!(records.len(), 5);
        assert_eq!(records[0].offset, 15);
        assert_eq!(records[4].offset, 19);
    }

    #[test]
    fn segment_read_at_exact_offset() {
        let dir = temp_dir();
        let mut seg = Segment::new(dir.path(), 0, 1024 * 1024);

        seg.append(0, b"a".to_vec());
        seg.append(1, b"b".to_vec());
        seg.append(2, b"c".to_vec());

        let record = seg.read_at(1).unwrap();
        assert_eq!(record.data, b"b");
        assert!(seg.read_at(99).is_none());
    }

    #[test]
    fn segment_flush_and_load() {
        let dir = temp_dir();
        {
            let mut seg = Segment::new(dir.path(), 0, 1024 * 1024);
            seg.append(0, b"persist-me".to_vec());
            seg.append(1, b"also-me".to_vec());
            seg.flush().unwrap();
        }

        let path = dir.path().join(format!("{:020}.seg", 0));
        let loaded = Segment::load(path, 0, 1024 * 1024).unwrap();
        assert_eq!(loaded.record_count(), 2);
        assert_eq!(loaded.read_at(0).unwrap().data, b"persist-me");
        assert_eq!(loaded.read_at(1).unwrap().data, b"also-me");
    }

    #[test]
    fn segment_contains_offset() {
        let dir = temp_dir();
        let mut seg = Segment::new(dir.path(), 100, 1024);
        // Add records so the segment has a meaningful range.
        for i in 100..120 {
            seg.append(i, b"data".to_vec());
        }
        assert!(seg.contains_offset(100));
        assert!(seg.contains_offset(110));
        assert!(seg.contains_offset(119));
        // After last record
        assert!(!seg.contains_offset(120));
        // Before base offset
        assert!(!seg.contains_offset(99));
    }

    // --- SegmentLog tests ---

    #[test]
    fn segment_log_append_and_read() {
        let dir = temp_dir();
        let mut log = SegmentLog::new(dir.path(), 1024 * 1024);

        for i in 0..100 {
            log.append(i, format!("msg-{}", i).into_bytes()).unwrap();
        }

        assert_eq!(log.total_records(), 100);
        assert_eq!(log.next_offset(), 100);

        let records = log.read(50, 1024 * 1024);
        assert!(!records.is_empty());
        assert_eq!(records[0].offset, 50);
    }

    #[test]
    fn segment_log_rolls() {
        let dir = temp_dir();
        // Very small segments to force rolling.
        let mut log = SegmentLog::new(dir.path(), 100);

        for i in 0..50 {
            log.append(i, format!("message-{}", i).into_bytes()).unwrap();
        }

        assert!(log.segment_count() > 1, "Expected multiple segments");
        assert_eq!(log.total_records(), 50);

        // All records should be readable.
        for i in 0..50 {
            let records = log.read(i, 1024);
            assert!(
                records.iter().any(|r| r.offset == i),
                "Offset {} not found", i
            );
        }
    }

    #[test]
    fn segment_log_flush() {
        let dir = temp_dir();
        let mut log = SegmentLog::new(dir.path(), 1024 * 1024);
        log.append(0, b"test".to_vec()).unwrap();
        log.flush().unwrap();
        // No panic = success.
    }

    #[test]
    fn segment_log_empty() {
        let dir = temp_dir();
        let log = SegmentLog::new(dir.path(), 1024 * 1024);
        assert_eq!(log.total_records(), 0);
        assert_eq!(log.next_offset(), 0);
        assert_eq!(log.segment_count(), 0);
    }

    #[test]
    fn stored_record_serialization_roundtrip() {
        let record = StoredRecord {
            offset: 42,
            timestamp_ms: 1_700_000_000_000,
            data: b"hello, world".to_vec(),
        };
        let encoded = bincode::serialize(&record).unwrap();
        let decoded: StoredRecord = bincode::deserialize(&encoded).unwrap();
        assert_eq!(record, decoded);
    }
}
