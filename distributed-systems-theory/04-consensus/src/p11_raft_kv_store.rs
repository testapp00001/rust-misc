//! # Exercise: Raft KV Store
//!
//! ## Theory
//!
//! A linearizable key-value store can be built on top of Raft by using the
//! consensus log as the single source of truth. All write operations (put)
//! are submitted to Raft, which replicates them to a majority before
//! committing. Read operations can be served from the state machine
//! (which applies committed log entries) or, for stronger guarantees,
//! go through the Raft log as well.
//!
//! The state machine is a simple HashMap<String, String> that applies
//! SET and GET commands from the committed log.
//!
//! ## Proof / Intuition
//!
//! Linearizability is achieved because:
//! 1. All writes go through the Raft leader, which serializes them.
//! 2. A write is only acknowledged after majority commitment.
//! 3. Reads from the state machine see all committed writes.
//!
//! The real-time ordering property is satisfied because a client's write
//! is only acknowledged after commitment, and subsequent reads (from the
//! same or any node that has applied the committed entry) will see the
//! write.
//!
//! ## Implementation Task
//!
//! Build a linearizable KV store on Raft:
//! - `RaftKV` struct wrapping a Raft node
//! - `put(key, value)` -- submit to Raft, replicate, commit
//! - `get(key)` -- read from state machine
//! - State machine applies committed entries to a HashMap
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Linearizability: writes are visible to subsequent reads
//! - Data survives leader changes (via log replication)
//! - Concurrent operations maintain consistency

use std::collections::HashMap;

/// A log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: String,
}

/// Command types for the KV state machine.
#[derive(Debug, Clone)]
pub enum KvCommand {
    Set { key: String, value: String },
    Get { key: String },
}

impl KvCommand {
    pub fn parse(command: &str) -> Option<Self> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        match parts.as_slice() {
            ["SET", key, value] => Some(KvCommand::Set {
                key: key.to_string(),
                value: value.to_string(),
            }),
            ["GET", key] => Some(KvCommand::Get {
                key: key.to_string(),
            }),
            _ => None,
        }
    }
}

/// A Raft-backed linearizable KV store.
pub struct RaftKV {
    pub id: usize,
    pub current_term: u64,
    pub log: Vec<LogEntry>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub state_machine: HashMap<String, String>,
    pub is_leader: bool,
    pub peers: Vec<usize>,
    /// Number of nodes in the cluster.
    pub cluster_size: usize,
}

impl RaftKV {
    pub fn new(id: usize, cluster_size: usize) -> Self {
        let peers: Vec<usize> = (0..cluster_size).filter(|&j| j != id).collect();
        Self {
            id,
            current_term: 0,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            state_machine: HashMap::new(),
            is_leader: false,
            peers,
            cluster_size,
        }
    }

    /// Submit a put operation to the Raft cluster.
    ///
    /// This appends the command to the log, replicates to peers,
    /// and commits when a majority acknowledge.
    pub fn put(&mut self, key: &str, value: &str) -> Result<(), String> {
        if !self.is_leader {
            return Err("not a leader".to_string());
        }

        let command = format!("SET {key} {value}");
        let index = self.log.len() as u64 + 1;
        let entry = LogEntry {
            term: self.current_term,
            index,
            command,
        };
        self.log.push(entry);

        // Simulate replication: in a real system, this would be async RPCs.
        // Here we immediately commit (simulating successful replication).
        self.commit_index = index;
        self.apply_committed();

        Ok(())
    }

    /// Read a value from the state machine.
    pub fn get(&self, key: &str) -> Option<String> {
        self.state_machine.get(key).cloned()
    }

    /// Apply committed entries to the state machine.
    fn apply_committed(&mut self) {
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            if let Some(entry) = self.log.iter().find(|e| e.index == self.last_applied) {
                if let Some(cmd) = KvCommand::parse(&entry.command) {
                    match cmd {
                        KvCommand::Set { key, value } => {
                            self.state_machine.insert(key, value);
                        }
                        KvCommand::Get { .. } => {
                            // GET commands don't modify state
                        }
                    }
                }
            }
        }
    }

    /// Simulate receiving a AppendEntries from the leader.
    /// Used for testing follower behavior.
    pub fn receive_entries(
        &mut self,
        entries: Vec<LogEntry>,
        leader_commit: u64,
    ) {
        for entry in entries {
            if !self.log.iter().any(|e| e.index == entry.index) {
                self.log.push(entry);
            }
        }
        self.log.sort_by_key(|e| e.index);

        if leader_commit > self.commit_index {
            self.commit_index = leader_commit.min(self.log.len() as u64);
            self.apply_committed();
        }
    }

    /// Get the current state machine snapshot.
    pub fn snapshot(&self) -> &HashMap<String, String> {
        &self.state_machine
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linearizability_put_get() {
        let mut kv = RaftKV::new(0, 3);
        kv.is_leader = true;

        kv.put("x", "1").unwrap();
        assert_eq!(kv.get("x"), Some("1".to_string()));

        kv.put("x", "2").unwrap();
        assert_eq!(kv.get("x"), Some("2".to_string()));
    }

    #[test]
    fn data_survives_leader_changes() {
        // Leader writes
        let mut leader = RaftKV::new(0, 3);
        leader.is_leader = true;
        leader.put("a", "10").unwrap();
        leader.put("b", "20").unwrap();

        // Follower receives entries
        let mut follower = RaftKV::new(1, 3);
        follower.receive_entries(leader.log.clone(), leader.commit_index);

        assert_eq!(follower.get("a"), Some("10".to_string()));
        assert_eq!(follower.get("b"), Some("20".to_string()));

        // New leader takes over (the follower)
        follower.is_leader = true;
        follower.put("c", "30").unwrap();
        assert_eq!(follower.get("c"), Some("30".to_string()));
    }

    #[test]
    fn non_leader_rejects_writes() {
        let mut kv = RaftKV::new(1, 3);
        // Not a leader
        assert!(kv.put("x", "1").is_err());
    }

    #[test]
    fn multiple_keys() {
        let mut kv = RaftKV::new(0, 3);
        kv.is_leader = true;

        kv.put("name", "alice").unwrap();
        kv.put("age", "30").unwrap();
        kv.put("city", "nyc").unwrap();

        assert_eq!(kv.get("name"), Some("alice".to_string()));
        assert_eq!(kv.get("age"), Some("30".to_string()));
        assert_eq!(kv.get("city"), Some("nyc".to_string()));
    }

    #[test]
    fn get_nonexistent_key() {
        let kv = RaftKV::new(0, 3);
        assert_eq!(kv.get("missing"), None);
    }

    #[test]
    fn sequential_operations_consistent() {
        let mut kv = RaftKV::new(0, 3);
        kv.is_leader = true;

        kv.put("counter", "0").unwrap();
        kv.put("counter", "1").unwrap();
        kv.put("counter", "2").unwrap();
        kv.put("counter", "3").unwrap();

        assert_eq!(kv.get("counter"), Some("3".to_string()));
    }
}
