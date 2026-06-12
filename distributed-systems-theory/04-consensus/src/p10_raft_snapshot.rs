//! # Exercise: Raft Snapshot (Log Compaction)
//!
//! ## Theory
//!
//! In a long-running Raft cluster, the log grows without bound. Snapshotting
//! is the standard technique for log compaction: the state machine serializes
//! its current state into a snapshot, and all log entries up to a certain
//! index are discarded.
//!
//! The snapshot records:
//! - `last_included_index`: the last log index included in the snapshot
//! - `last_included_term`: the term of that entry
//! - `state`: the serialized state machine
//!
//! When a follower is far behind, the leader can send an InstallSnapshot RPC
//! instead of replaying thousands of log entries.
//!
//! ## Proof / Intuition
//!
//! Snapshotting preserves safety because:
//! 1. The snapshot captures the state machine at a known point.
//! 2. Any entries after the snapshot are replayed on top of it.
//! 3. The term of the last included entry is preserved for consistency checks.
//!
//! A follower that receives a snapshot replaces its entire log with the
//! snapshot and begins receiving new entries from that point.
//!
//! ## Implementation Task
//!
//! Implement Raft snapshotting:
//! - `Snapshot` struct with last_included_index, last_included_term, state
//! - `create_snapshot(up_to_index)` -- compact the log
//! - `install_snapshot(snapshot)` -- install on a follower
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Snapshot compacts the log
//! - Follower installs snapshot correctly
//! - State is preserved after snapshot

use std::collections::HashMap;

/// A snapshot of the Raft state machine.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub last_included_index: u64,
    pub last_included_term: u64,
    pub state: HashMap<String, String>,
}

/// A Raft node with snapshot support.
#[derive(Debug)]
pub struct SnapshotNode {
    pub id: usize,
    pub current_term: u64,
    pub log: Vec<LogEntry>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub state_machine: HashMap<String, String>,
    /// The most recent snapshot.
    pub snapshot: Option<Snapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: String,
}

impl SnapshotNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            current_term: 0,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            state_machine: HashMap::new(),
            snapshot: None,
        }
    }

    /// Append a log entry.
    pub fn append_entry(&mut self, command: &str) {
        let base = self.snapshot.as_ref().map(|s| s.last_included_index).unwrap_or(0);
        let index = base + self.log.len() as u64 + 1;
        self.log.push(LogEntry {
            term: self.current_term,
            index,
            command: command.to_string(),
        });
    }

    /// Apply committed entries to the state machine.
    pub fn apply_committed(&mut self) {
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            if let Some(entry) = self.log.iter().find(|e| e.index == self.last_applied) {
                let parts: Vec<&str> = entry.command.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "SET" {
                    self.state_machine
                        .insert(parts[1].to_string(), parts[2].to_string());
                }
            }
        }
    }

    /// Create a snapshot up to the given index.
    ///
    /// This compacts the log by removing all entries up to `up_to_index`.
    /// The snapshot captures the state machine at the current state.
    pub fn create_snapshot(&mut self, up_to_index: u64) -> Snapshot {
        // Apply any remaining committed entries first
        self.commit_index = up_to_index;
        self.apply_committed();

        let last_included_term = self
            .log
            .iter()
            .find(|e| e.index == up_to_index)
            .map(|e| e.term)
            .unwrap_or(0);

        let snapshot = Snapshot {
            last_included_index: up_to_index,
            last_included_term,
            state: self.state_machine.clone(),
        };

        // Compact the log: remove entries up to the snapshot index
        self.log
            .retain(|e| e.index > up_to_index);

        self.snapshot = Some(snapshot.clone());
        snapshot
    }

    /// Install a snapshot received from the leader.
    ///
    /// Replaces the state machine and discards log entries before the
    /// snapshot's last included index.
    pub fn install_snapshot(&mut self, snapshot: Snapshot) {
        self.last_applied = snapshot.last_included_index;
        self.commit_index = snapshot.last_included_index;

        // Discard log entries that are included in the snapshot
        let base_index = snapshot.last_included_index;
        self.log.retain(|e| e.index > base_index);

        self.state_machine = snapshot.state;
        self.snapshot = Some(Snapshot {
            last_included_index: snapshot.last_included_index,
            last_included_term: snapshot.last_included_term,
            state: self.state_machine.clone(),
        });
    }

    /// Get the effective base index (0 if no snapshot, otherwise
    /// snapshot.last_included_index).
    pub fn base_index(&self) -> u64 {
        self.snapshot
            .as_ref()
            .map(|s| s.last_included_index)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_compacts_log() {
        let mut node = SnapshotNode::new(0);
        node.current_term = 1;

        for i in 0..10 {
            node.append_entry(&format!("SET k{i} v{i}"));
        }
        node.commit_index = 10;
        node.apply_committed();

        let snapshot = node.create_snapshot(5);
        assert_eq!(snapshot.last_included_index, 5);
        // Log should only contain entries after index 5
        assert!(node.log.iter().all(|e| e.index > 5));
        assert_eq!(node.log.len(), 5);
    }

    #[test]
    fn follower_installs_snapshot() {
        let mut follower = SnapshotNode::new(1);
        follower.current_term = 1;
        follower.append_entry("SET old stale");
        follower.commit_index = 1;
        follower.apply_committed();

        let snapshot = Snapshot {
            last_included_index: 10,
            last_included_term: 1,
            state: {
                let mut m = HashMap::new();
                m.insert("x".to_string(), "42".to_string());
                m
            },
        };

        follower.install_snapshot(snapshot);
        assert_eq!(follower.state_machine.get("x"), Some(&"42".to_string()));
        assert_eq!(follower.last_applied, 10);
        assert!(follower.log.is_empty());
    }

    #[test]
    fn state_preserved_after_snapshot() {
        let mut node = SnapshotNode::new(0);
        node.current_term = 1;
        node.append_entry("SET a 1");
        node.append_entry("SET b 2");
        node.commit_index = 2;
        node.apply_committed();

        let snap = node.create_snapshot(2);
        assert_eq!(snap.state.get("a"), Some(&"1".to_string()));
        assert_eq!(snap.state.get("b"), Some(&"2".to_string()));
    }

    #[test]
    fn new_entries_after_snapshot() {
        let mut node = SnapshotNode::new(0);
        node.current_term = 1;
        for i in 0..5 {
            node.append_entry(&format!("SET k{i} v{i}"));
        }
        node.commit_index = 5;
        node.apply_committed();

        node.create_snapshot(5);

        // Add new entries after snapshot
        node.current_term = 2;
        node.append_entry("SET k_new new_val");
        node.commit_index = 6;
        node.apply_committed();

        assert_eq!(node.base_index(), 5);
        assert_eq!(
            node.state_machine.get("k_new"),
            Some(&"new_val".to_string())
        );
        assert_eq!(node.log.len(), 1);
    }
}
