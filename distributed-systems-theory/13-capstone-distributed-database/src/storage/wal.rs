//! Write-Ahead Log (WAL) for durability simulation.
//!
//! Before any mutation is applied to the storage engine, it is first recorded in the WAL.
//! On recovery, the WAL entries are replayed to rebuild the engine state. This module
//! provides an append-only log that simulates this behavior in memory.

/// A single entry in the write-ahead log.
#[derive(Debug, Clone)]
pub struct WALEntry {
    /// Monotonically increasing id for this entry.
    pub id: u64,
    /// The serialized command (e.g., a string representation of the operation).
    pub command: String,
    /// The timestamp (HLC or physical) when this entry was created.
    pub timestamp: u64,
}

/// An in-memory write-ahead log that records mutations before they are applied.
#[derive(Debug)]
pub struct WAL {
    entries: Vec<WALEntry>,
    next_id: u64,
}

impl WAL {
    /// Create a new, empty WAL.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
        }
    }

    /// Append a command to the log with the given timestamp.
    ///
    /// Returns the auto-assigned id of the new entry.
    pub fn append(&mut self, command: String, timestamp: u64) -> u64 {
        let id = self.next_id;
        self.entries.push(WALEntry {
            id,
            command,
            timestamp,
        });
        self.next_id += 1;
        id
    }

    /// Recover all logged entries in order.
    ///
    /// In a real system this would read from disk; here it simply returns the
    /// in-memory entries to simulate replay.
    pub fn recover(&self) -> Vec<WALEntry> {
        self.entries.clone()
    }

    /// Return the number of entries in the log.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Clear all entries from the log.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.next_id = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_recover() {
        let mut wal = WAL::new();
        let id1 = wal.append("PUT a=1".into(), 100);
        let id2 = wal.append("PUT b=2".into(), 200);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let recovered = wal.recover();
        assert_eq!(recovered.len(), 2);
        assert_eq!(recovered[0].command, "PUT a=1");
        assert_eq!(recovered[1].command, "PUT b=2");
    }

    #[test]
    fn test_entry_count() {
        let mut wal = WAL::new();
        assert_eq!(wal.entry_count(), 0);
        wal.append("cmd1".into(), 1);
        wal.append("cmd2".into(), 2);
        assert_eq!(wal.entry_count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut wal = WAL::new();
        wal.append("cmd1".into(), 1);
        wal.append("cmd2".into(), 2);
        wal.clear();
        assert_eq!(wal.entry_count(), 0);
        assert!(wal.recover().is_empty());
    }
}
