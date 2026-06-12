//! Simplified Raft consensus implementation.
//!
//! This module provides a single-node simulation of the Raft consensus protocol.
//! A `RaftNode` maintains a replicated log, transitions between Follower, Candidate,
//! and Leader states, and applies committed entries to a deterministic state machine.
//!
//! In a real system, elections and log replication occur over the network. Here we
//! simulate the core logic synchronously for clarity.

use std::collections::HashSet;

use crate::consensus::state_machine::StateMachine;

/// Unique identifier for a node in the Raft cluster.
pub type NodeId = u64;

/// A command that can be proposed to the Raft log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Insert or update a key-value pair.
    Put(String, String),
    /// Read a value by key (for completeness; reads are typically served by the leader).
    Get(String),
    /// Remove a key-value pair.
    Delete(String),
}

/// A single entry in the Raft log.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The term when the entry was created.
    pub term: u64,
    /// The index of the entry in the log (1-based).
    pub index: u64,
    /// The command stored in this entry.
    pub command: Command,
}

/// The role a Raft node currently holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaftState {
    /// A follower that replicates the leader's log.
    Follower,
    /// A candidate that is attempting to win an election.
    Candidate,
    /// The leader that accepts client commands and replicates them.
    Leader,
}

/// A simplified Raft node that manages log replication and command application.
#[derive(Debug)]
pub struct RaftNode {
    /// The unique identifier of this node.
    pub id: NodeId,
    /// Current role of this node.
    pub state: RaftState,
    /// Current term (monotonically increasing).
    pub term: u64,
    /// The replicated log of entries.
    pub log: Vec<LogEntry>,
    /// Index of the highest log entry known to be committed.
    pub commit_index: u64,
    /// Index of the highest log entry applied to the state machine.
    pub last_applied: u64,
    /// The node id of the current leader, if known.
    pub leader_id: Option<NodeId>,
    /// Set of nodes that voted for this node in the current term.
    pub votes_received: HashSet<NodeId>,
    /// List of peer node ids (does not include self).
    pub peers: Vec<NodeId>,
    /// The node this node voted for in the current term, if any.
    pub voted_for: Option<NodeId>,
    /// The state machine that applies committed commands.
    state_machine: StateMachine,
}

impl RaftNode {
    /// Create a new Raft node initialized as a Follower with an empty log.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for this node.
    /// * `peers` - The node ids of all peers (not including this node).
    pub fn new(id: NodeId, peers: Vec<NodeId>) -> Self {
        Self {
            id,
            state: RaftState::Follower,
            term: 0,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            leader_id: None,
            votes_received: HashSet::new(),
            peers,
            voted_for: None,
            state_machine: StateMachine::new(),
        }
    }

    /// Transition this node to the Leader state.
    ///
    /// Resets vote tracking and records self as the leader.
    pub fn become_leader(&mut self) {
        self.state = RaftState::Leader;
        self.leader_id = Some(self.id);
        self.votes_received.clear();
    }

    /// Propose a new command to the log.
    ///
    /// Appends the command as a new log entry with the current term and returns
    /// the index of the new entry.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to propose.
    ///
    /// # Returns
    ///
    /// The 1-based index of the newly appended log entry.
    pub fn propose(&mut self, command: Command) -> Result<u64, String> {
        let index = self.log.len() as u64 + 1;
        let entry = LogEntry {
            term: self.term,
            index,
            command,
        };
        self.log.push(entry);
        Ok(index)
    }

    /// Simulate replicating the latest log entry to followers.
    ///
    /// In this simplified version, the entry is assumed to be replicated to a
    /// majority and the commit index is advanced accordingly.
    pub fn replicate_to_followers(&mut self) {
        // In a real implementation this would send AppendEntries RPCs.
        // Here we simply advance the commit index to the last log entry,
        // assuming a majority acknowledgment.
        if let Some(last) = self.log.last() {
            self.commit_index = last.index;
        }
    }

    /// Return the id of the current leader, if one is known.
    pub fn get_leader(&self) -> Option<NodeId> {
        self.leader_id
    }

    /// Apply all committed but not-yet-applied log entries to the state machine.
    pub fn apply_committed(&mut self) {
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            let idx = (self.last_applied - 1) as usize;
            if let Some(entry) = self.log.get(idx) {
                self.state_machine.apply(entry.command.clone());
            }
        }
    }

    /// Read a value from the state machine by key.
    pub fn get(&self, key: &str) -> Option<String> {
        self.state_machine.get(key)
    }

    /// Propose a Put command to the log.
    pub fn put(&mut self, key: String, value: String) -> Result<u64, String> {
        self.propose(Command::Put(key, value))
    }

    /// Propose a Delete command to the log.
    pub fn delete(&mut self, key: String) -> Result<u64, String> {
        self.propose(Command::Delete(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_node_is_follower() {
        let node = RaftNode::new(1, vec![2, 3]);
        assert_eq!(node.state, RaftState::Follower);
        assert_eq!(node.id, 1);
        assert_eq!(node.peers, vec![2, 3]);
        assert_eq!(node.term, 0);
    }

    #[test]
    fn test_become_leader() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.become_leader();
        assert_eq!(node.state, RaftState::Leader);
        assert_eq!(node.leader_id, Some(1));
    }

    #[test]
    fn test_propose_returns_incrementing_index() {
        let mut node = RaftNode::new(1, vec![]);
        let idx1 = node.propose(Command::Put("a".into(), "1".into())).unwrap();
        let idx2 = node.propose(Command::Put("b".into(), "2".into())).unwrap();
        assert_eq!(idx1, 1);
        assert_eq!(idx2, 2);
        assert_eq!(node.log.len(), 2);
    }

    #[test]
    fn test_replicate_and_apply() {
        let mut node = RaftNode::new(1, vec![]);
        node.propose(Command::Put("x".into(), "10".into())).unwrap();
        node.replicate_to_followers();
        assert_eq!(node.commit_index, 1);
        node.apply_committed();
        assert_eq!(node.last_applied, 1);
        assert_eq!(node.get("x"), Some("10".into()));
    }

    #[test]
    fn test_delete_command() {
        let mut node = RaftNode::new(1, vec![]);
        node.propose(Command::Put("key".into(), "val".into())).unwrap();
        node.propose(Command::Delete("key".into())).unwrap();
        node.replicate_to_followers();
        node.apply_committed();
        assert_eq!(node.get("key"), None);
    }

    #[test]
    fn test_put_and_delete_helpers() {
        let mut node = RaftNode::new(1, vec![]);
        node.put("a".into(), "1".into()).unwrap();
        node.delete("a".into()).unwrap();
        assert_eq!(node.log.len(), 2);
    }

    #[test]
    fn test_get_leader() {
        let mut node = RaftNode::new(1, vec![]);
        assert_eq!(node.get_leader(), None);
        node.become_leader();
        assert_eq!(node.get_leader(), Some(1));
    }
}
