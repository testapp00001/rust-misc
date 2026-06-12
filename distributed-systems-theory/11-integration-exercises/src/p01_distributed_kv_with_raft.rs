//! # Exercise: Distributed KV Store with Raft Consensus
//!
//! ## Theory
//!
//! Raft is a consensus algorithm designed to be understandable and practical.
//! It ensures that a cluster of nodes agrees on a shared state even in the
//! presence of failures. The key components are:
//!
//! - **Leader election**: One node is elected leader; others are followers.
//! - **Log replication**: The leader replicates its log to followers.
//! - **Safety**: Once a log entry is committed, it will not be overwritten.
//!
//! Each log entry contains a command, the term when it was received, and its
//! index in the log. The leader commits an entry when a majority of nodes
//! have replicated it.
//!
//! ## Proof / Intuition
//!
//! Safety property: If two logs contain an entry with the same index and term,
//! then the logs are identical up to that index.
//!
//! This is maintained because:
//! 1. A leader only appends to its log (never overwrites)
//! 2. Followers only accept entries from leaders
//! 3. An entry is committed only when replicated to a majority
//!
//! Since a majority requires overlap between any two majorities, once committed,
//! the entry must be in the leader's log, and future leaders must include it.
//!
//! ## Implementation Task
//!
//! 1. Implement `RaftNode` with state management (Follower/Candidate/Leader)
//! 2. Implement log entry appending and commitment
//! 3. Implement state machine application (apply committed entries)
//! 4. Support Put, Get, Delete operations through the Raft log
//! 5. Verify data consistency across nodes after leader change
//!
//! ## Verification
//!
//! - Test put/get through the leader
//! - Test that data survives a leader change
//! - Test multiple operations maintain consistency

use std::collections::HashMap;

/// Node identifier type.
pub type NodeId = u64;

/// The state of a Raft node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

/// A command that can be executed on the state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Put(String, String),
    Get(String),
    Delete(String),
}

impl Command {
    /// Serialize the command to a string for log storage.
    pub fn to_string_repr(&self) -> String {
        match self {
            Command::Put(k, v) => format!("PUT {} {}", k, v),
            Command::Get(k) => format!("GET {}", k),
            Command::Delete(k) => format!("DELETE {}", k),
        }
    }

    /// Deserialize a command from a string.
    pub fn from_string_repr(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(3, ' ').collect();
        match parts[0] {
            "PUT" if parts.len() >= 3 => Some(Command::Put(
                parts[1].to_string(),
                parts[2].to_string(),
            )),
            "GET" if parts.len() >= 2 => Some(Command::Get(parts[1].to_string())),
            "DELETE" if parts.len() >= 2 => Some(Command::Delete(parts[1].to_string())),
            _ => None,
        }
    }
}

/// A log entry in the Raft log.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The term when the entry was received.
    pub term: u64,
    /// The index of the entry in the log.
    pub index: usize,
    /// The command to execute.
    pub command: Command,
}

/// A Raft node participating in consensus.
pub struct RaftNode {
    /// Current state of the node.
    pub state: RaftState,
    /// Current term.
    pub term: u64,
    /// The replicated log.
    pub log: Vec<LogEntry>,
    /// Index of the highest entry known to be committed.
    pub commit_index: usize,
    /// Index of the highest entry applied to the state machine.
    pub last_applied: usize,
    /// This node's ID.
    pub id: NodeId,
    /// ID of the current leader.
    pub leader_id: Option<NodeId>,
    /// IDs of peer nodes.
    pub peers: Vec<NodeId>,
    /// Votes received in the current election term.
    pub votes_received: Vec<NodeId>,
    /// The key-value state machine.
    pub state_machine: HashMap<String, String>,
    /// Which node this node voted for in the current term.
    pub voted_for: Option<NodeId>,
}

impl RaftNode {
    /// Create a new Raft node.
    pub fn new(id: NodeId, peers: Vec<NodeId>) -> Self {
        Self {
            state: RaftState::Follower,
            term: 0,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            id,
            leader_id: None,
            peers,
            votes_received: Vec::new(),
            state_machine: HashMap::new(),
            voted_for: None,
        }
    }

    /// Transition this node to leader state.
    pub fn become_leader(&mut self) {
        self.state = RaftState::Leader;
        self.leader_id = Some(self.id);
        self.votes_received.clear();
    }

    /// Append an entry to the log. Returns the index of the new entry.
    pub fn append_entry(&mut self, command: Command) -> usize {
        let index = self.log.len();
        let entry = LogEntry {
            term: self.term,
            index,
            command,
        };
        self.log.push(entry);
        index
    }

    /// Commit all entries up to and including the commit index.
    /// This applies them to the state machine.
    pub fn commit_entries(&mut self) {
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            let cmd = self.log[self.last_applied - 1].command.clone();
            self.apply_to_state_machine(&cmd);
        }
    }

    /// Apply a command to the state machine.
    fn apply_to_state_machine(&mut self, command: &Command) {
        match command {
            Command::Put(key, value) => {
                self.state_machine.insert(key.clone(), value.clone());
            }
            Command::Delete(key) => {
                self.state_machine.remove(key);
            }
            Command::Get(_) => {
                // Get commands don't modify state
            }
        }
    }

    /// Get a value from the state machine.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.state_machine.get(key).map(|s| s.as_str())
    }

    /// Put a key-value pair through the Raft log.
    pub fn put(&mut self, key: &str, value: &str) {
        let cmd = Command::Put(key.to_string(), value.to_string());
        self.append_entry(cmd);
    }

    /// Delete a key through the Raft log.
    pub fn delete(&mut self, key: &str) {
        let cmd = Command::Delete(key.to_string());
        self.append_entry(cmd);
    }

    /// Check if this node is the leader.
    pub fn is_leader(&self) -> bool {
        self.state == RaftState::Leader
    }

    /// Get the current term.
    pub fn term(&self) -> u64 {
        self.term
    }

    /// Get the log length.
    pub fn log_len(&self) -> usize {
        self.log.len()
    }
}

/// Simulate a Raft cluster with leader election and log replication.
pub struct RaftCluster {
    pub nodes: Vec<RaftNode>,
}

impl RaftCluster {
    /// Create a new Raft cluster with the given number of nodes.
    pub fn new(num_nodes: usize) -> Self {
        let mut nodes = Vec::new();
        for i in 0..num_nodes {
            let peers: Vec<NodeId> = (0..num_nodes)
                .filter(|&j| j != i)
                .map(|j| j as u64)
                .collect();
            nodes.push(RaftNode::new(i as u64, peers));
        }
        Self { nodes }
    }

    /// Elect a node as leader.
    pub fn elect_leader(&mut self, leader_id: NodeId) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == leader_id) {
            node.become_leader();
        }
    }

    /// Replicate the leader's log to all followers and commit.
    pub fn replicate_and_commit(&mut self) {
        let leader_id = self.nodes.iter().find(|n| n.is_leader()).map(|n| n.id);

        if let Some(lid) = leader_id {
            let leader_log = self.nodes.iter().find(|n| n.id == lid).unwrap().log.clone();
            let leader_commit = self.nodes.iter().find(|n| n.id == lid).unwrap().commit_index;

            // Replicate to followers
            for node in &mut self.nodes {
                if node.id != lid && node.state != RaftState::Leader {
                    node.log = leader_log.clone();
                    node.commit_index = leader_commit;
                }
            }

            // Leader commits
            if let Some(leader) = self.nodes.iter_mut().find(|n| n.id == lid) {
                leader.commit_index = leader.log.len();
                leader.commit_entries();
            }

            // Followers commit
            for node in &mut self.nodes {
                if node.id != lid {
                    node.commit_index = node.log.len();
                    node.commit_entries();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_get_through_leader() {
        let mut cluster = RaftCluster::new(3);
        cluster.elect_leader(0);

        // Put a value through the leader
        let leader = &mut cluster.nodes[0];
        leader.put("name", "Alice");
        leader.commit_index = leader.log.len();
        leader.commit_entries();

        assert_eq!(leader.get("name"), Some("Alice"));
    }

    #[test]
    fn test_data_survives_leader_change() {
        let mut cluster = RaftCluster::new(3);

        // First leader puts data
        cluster.elect_leader(0);
        cluster.nodes[0].put("key1", "value1");
        cluster.replicate_and_commit();

        // New leader elected
        cluster.elect_leader(1);

        // The new leader should have the same state
        assert_eq!(cluster.nodes[1].get("key1"), Some("value1"));
    }

    #[test]
    fn test_multiple_operations() {
        let mut cluster = RaftCluster::new(3);
        cluster.elect_leader(0);

        // Perform multiple operations
        {
            let leader = &mut cluster.nodes[0];
            leader.put("a", "1");
            leader.put("b", "2");
            leader.put("c", "3");
            leader.delete("b");
            leader.commit_index = leader.log.len();
            leader.commit_entries();
        }

        assert_eq!(cluster.nodes[0].get("a"), Some("1"));
        assert_eq!(cluster.nodes[0].get("b"), None);
        assert_eq!(cluster.nodes[0].get("c"), Some("3"));
    }

    #[test]
    fn test_consensus_across_nodes() {
        let mut cluster = RaftCluster::new(5);
        cluster.elect_leader(0);

        // Put data and replicate
        cluster.nodes[0].put("shared", "data");
        cluster.replicate_and_commit();

        // All nodes should have the same state
        for node in &cluster.nodes {
            assert_eq!(node.get("shared"), Some("data"));
        }
    }
}
