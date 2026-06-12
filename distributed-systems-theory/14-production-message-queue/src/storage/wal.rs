//! Write-Ahead Log (WAL) for crash-safe durability.
//!
//! Every message is first written to the WAL and flushed to disk before being
//! appended to the active segment. On startup, the WAL is replayed to
//! recover any writes that were not flushed to segments.
//!
//! Format on disk:
//! ```text
//! [entry_len: u32 BE] [WalEntry: bincode] [entry_len: u32 BE] [WalEntry: bincode] ...
//! ```
//! Each `WalEntry` carries a monotonically increasing sequence number, the
//! raw payload, and a CRC32 checksum for corruption detection.

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{MqError, Result};

/// A single entry in the write-ahead log.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalEntry {
    /// Monotonically increasing sequence number.
    pub sequence: u64,
    /// The raw payload bytes.
    pub data: Vec<u8>,
    /// CRC32 checksum of `data` for corruption detection.
    pub checksum: u32,
}

impl WalEntry {
    /// Verify the checksum matches the data.
    pub fn verify(&self) -> bool {
        crc32(&self.data) == self.checksum
    }
}

/// An append-only write-ahead log backed by a single file.
///
/// Entries are length-prefixed with a 4-byte big-endian length followed by a
/// bincode-serialized `WalEntry`. This allows efficient sequential writes
/// and reliable recovery via linear scan.
pub struct WriteAheadLog {
    /// Path to the WAL file on disk.
    path: PathBuf,
    /// Buffered writer for sequential appends.
    writer: BufWriter<File>,
    /// Next sequence number to assign.
    next_sequence: u64,
    /// Total bytes written (including length prefixes).
    bytes_written: u64,
}

impl WriteAheadLog {
    /// Open or create a WAL at the given path.
    ///
    /// If the file already exists, it is opened in append mode and the
    /// sequence counter is recovered from the last entry.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Recover sequence number from existing entries.
        let next_sequence = if path.exists() {
            match Self::read_all_from_path(&path) {
                Ok(entries) => {
                    entries.last().map_or(0, |e| e.sequence + 1)
                }
                Err(_) => 0,
            }
        } else {
            0
        };

        // Compute bytes written for existing data.
        let bytes_written = if path.exists() {
            fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| MqError::Storage(format!(
                "Failed to open WAL at '{}': {}", path.display(), e
            )))?;

        Ok(WriteAheadLog {
            path,
            writer: BufWriter::with_capacity(64 * 1024, file),
            next_sequence,
            bytes_written,
        })
    }

    /// Append a raw payload to the WAL and return its sequence number.
    ///
    /// The entry is serialized, length-prefixed, written, and flushed to
    /// ensure durability before returning.
    pub fn append(&mut self, data: &[u8]) -> Result<u64> {
        let entry = WalEntry {
            sequence: self.next_sequence,
            data: data.to_vec(),
            checksum: crc32(data),
        };

        let encoded = bincode::serialize(&entry)
            .map_err(|e| MqError::Storage(format!(
                "WAL serialization failed: {}", e
            )))?;

        let len = (encoded.len() as u32).to_be_bytes();

        self.writer.write_all(&len)
            .map_err(|e| MqError::Storage(format!("WAL write failed: {}", e)))?;
        self.writer.write_all(&encoded)
            .map_err(|e| MqError::Storage(format!("WAL write failed: {}", e)))?;
        self.writer.flush()
            .map_err(|e| MqError::Storage(format!("WAL flush failed: {}", e)))?;

        let seq = self.next_sequence;
        self.next_sequence += 1;
        self.bytes_written += 4 + encoded.len() as u64;

        Ok(seq)
    }

    /// Read all valid entries from the WAL file (without consuming the
    /// writer). Used during startup recovery.
    pub fn read_all(&self) -> Result<Vec<WalEntry>> {
        Self::read_all_from_path(&self.path)
    }

    /// Read all entries from a WAL file, skipping corrupt or truncated
    /// entries.
    fn read_all_from_path(path: &Path) -> Result<Vec<WalEntry>> {
        let data = fs::read(path)
            .map_err(|e| MqError::Storage(format!(
                "Failed to read WAL '{}': {}", path.display(), e
            )))?;

        let mut entries = Vec::new();
        let mut pos = 0;

        while pos + 4 <= data.len() {
            let len = u32::from_be_bytes([
                data[pos], data[pos + 1], data[pos + 2], data[pos + 3],
            ]) as usize;
            pos += 4;

            if pos + len > data.len() {
                // Truncated entry -- stop reading.
                break;
            }

            if let Ok(entry) = bincode::deserialize::<WalEntry>(&data[pos..pos + len]) {
                // Verify checksum before accepting the entry.
                if entry.verify() {
                    entries.push(entry);
                }
                // If checksum fails, we still skip forward (the entry is
                // corrupt but we don't want to infinite-loop).
            }
            // If deserialization fails, skip forward by the declared length.
            pos += len;
        }

        Ok(entries)
    }

    /// Truncate the WAL file, discarding all entries.
    ///
    /// This is used after a successful snapshot/compaction to reclaim space.
    pub fn truncate(&mut self) -> Result<()> {
        self.writer = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&self.path)
                .map_err(|e| MqError::Storage(format!(
                    "Failed to truncate WAL '{}': {}", self.path.display(), e
                )))?,
        );
        self.next_sequence = 0;
        self.bytes_written = 0;
        Ok(())
    }

    /// Total bytes written to the WAL (including length prefixes).
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Number of entries that will be recovered on the next `read_all`.
    pub fn entry_count(&self) -> u64 {
        self.next_sequence
    }

    /// Path to the WAL file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Compute a CRC32 checksum using the standard polynomial 0xEDB88320.
///
/// This is the same algorithm used by Ethernet, zlib, and many other
/// protocols. The implementation processes one byte at a time for
/// simplicity; a production system might use hardware acceleration.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_wal() -> (WriteAheadLog, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let wal = WriteAheadLog::open(dir.path().join("test.wal")).unwrap();
        (wal, dir)
    }

    #[test]
    fn append_and_read_roundtrip() {
        let (mut wal, _dir) = temp_wal();
        wal.append(b"hello").unwrap();
        wal.append(b"world").unwrap();

        let entries = wal.read_all().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence, 0);
        assert_eq!(entries[0].data, b"hello");
        assert_eq!(entries[1].sequence, 1);
        assert_eq!(entries[1].data, b"world");
    }

    #[test]
    fn sequence_numbers_monotonic() {
        let (mut wal, _dir) = temp_wal();
        let s0 = wal.append(b"a").unwrap();
        let s1 = wal.append(b"b").unwrap();
        let s2 = wal.append(b"c").unwrap();
        assert_eq!(s0, 0);
        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
    }

    #[test]
    fn checksum_valid() {
        let (mut wal, _dir) = temp_wal();
        wal.append(b"test-data").unwrap();
        let entries = wal.read_all().unwrap();
        assert!(entries[0].verify());
    }

    #[test]
    fn checksum_detects_corruption() {
        let entry = WalEntry {
            sequence: 0,
            data: b"original".to_vec(),
            checksum: crc32(b"original"),
        };
        assert!(entry.verify());

        let corrupted = WalEntry {
            sequence: 0,
            data: b"corruptd".to_vec(),
            checksum: crc32(b"original"),
        };
        assert!(!corrupted.verify());
    }

    #[test]
    fn bytes_written_tracking() {
        let (mut wal, _dir) = temp_wal();
        assert_eq!(wal.bytes_written(), 0);
        wal.append(b"hello").unwrap();
        assert!(wal.bytes_written() > 0);
        let before = wal.bytes_written();
        wal.append(b"world").unwrap();
        assert!(wal.bytes_written() > before);
    }

    #[test]
    fn entry_count() {
        let (mut wal, _dir) = temp_wal();
        assert_eq!(wal.entry_count(), 0);
        wal.append(b"one").unwrap();
        wal.append(b"two").unwrap();
        assert_eq!(wal.entry_count(), 2);
    }

    #[test]
    fn truncate_resets_state() {
        let (mut wal, _dir) = temp_wal();
        wal.append(b"data").unwrap();
        assert_eq!(wal.entry_count(), 1);
        wal.truncate().unwrap();
        assert_eq!(wal.entry_count(), 0);
        assert_eq!(wal.bytes_written(), 0);
        assert!(wal.read_all().unwrap().is_empty());
    }

    #[test]
    fn recovery_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("persist.wal");

        // Write entries and drop the WAL handle.
        {
            let mut wal = WriteAheadLog::open(&path).unwrap();
            wal.append(b"first").unwrap();
            wal.append(b"second").unwrap();
        }

        // Reopen and verify recovery.
        let wal = WriteAheadLog::open(&path).unwrap();
        let entries = wal.read_all().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].data, b"first");
        assert_eq!(entries[1].data, b"second");
        // Sequence counter should be recovered.
        assert_eq!(wal.entry_count(), 2);
    }

    #[test]
    fn empty_wal() {
        let (wal, _dir) = temp_wal();
        let entries = wal.read_all().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn large_payload() {
        let (mut wal, _dir) = temp_wal();
        let payload = vec![0xABu8; 100_000];
        wal.append(&payload).unwrap();
        let entries = wal.read_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].data.len(), 100_000);
        assert!(entries[0].verify());
    }

    #[test]
    fn crc32_known_values() {
        // CRC32 of empty string
        assert_eq!(crc32(b""), 0x0000_0000);
        // CRC32 of "123456789" is a standard test vector
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }
}
