//! # Exercise: Raft Persistence
//!
//! ## Theory
//!
//! Raft requires that certain state survives crashes:
//!
//! - **current_term:** The latest term the node has seen.
//! - **voted_for:** The candidate the node voted for in the current term.
//! - **log:** The sequence of commands.
//!
//! These three pieces of state must be persisted to stable storage before
//! responding to RPCs. Without persistence, a node that crashes and restarts
//! could:
//! - Vote twice in the same term (violating the at-most-once vote invariant)
//! - Lose committed log entries
//! - Forget its current term, causing it to accept stale leaders
//!
//! ## Proof / Intuition
//!
//! The Raft paper proves that if all three pieces of state are persisted
//! before responding to RPCs, the safety guarantees of Raft are maintained
//! even after crashes. Specifically:
//!
//! 1. A node cannot vote for two different candidates in the same term
//!    (voted_for is persisted).
//! 2. Committed entries are never lost (log is persisted).
//! 3. Term numbers never go backward (current_term is persisted).
//!
//! Other state (commit_index, last_applied, state machine) can be
//! reconstructed or recomputed after a restart.
//!
//! ## Implementation Task
//!
//! Implement persistence for Raft:
//! - `PersistentState` struct for persistent fields
//! - `save_to_disk()` / `load_from_disk()` simulation
//! - Node restart: load state, rejoin cluster
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - State survives simulated restart
//! - Leader can be re-elected after crash
//! - Voted_for prevents double voting

use std::collections::HashMap;

/// Persistent state that must survive crashes.
#[derive(Debug, Clone)]
pub struct PersistentState {
    pub current_term: u64,
    pub voted_for: Option<usize>,
    pub log: Vec<LogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: String,
}

impl PersistentState {
    pub fn new() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
        }
    }

    /// Simulate saving to disk.
    pub fn save_to_disk(&self) -> DiskImage {
        DiskImage {
            current_term: self.current_term,
            voted_for: self.voted_for,
            log: self.log.clone(),
        }
    }

    /// Simulate loading from disk.
    pub fn load_from_disk(disk: &DiskImage) -> Self {
        Self {
            current_term: disk.current_term,
            voted_for: disk.voted_for,
            log: disk.log.clone(),
        }
    }

    /// Append a log entry.
    pub fn append(&mut self, command: &str, term: u64) {
        let index = self.log.len() as u64 + 1;
        self.log.push(LogEntry {
            term,
            index,
            command: command.to_string(),
        });
    }
}

impl Default for PersistentState {
    fn default() -> Self {
        Self::new()
    }
}

/// A simulated disk image.
#[derive(Debug, Clone)]
pub struct DiskImage {
    pub current_term: u64,
    pub voted_for: Option<usize>,
    pub log: Vec<LogEntry>,
}

/// A Raft node with persistence.
#[derive(Debug)]
pub struct PersistentRaftNode {
    pub id: usize,
    pub persistent: PersistentState,
    pub state: NodeState,
    pub commit_index: u64,
    pub last_applied: u64,
    pub state_machine: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Follower,
    Candidate,
    Leader,
}

impl PersistentRaftNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            persistent: PersistentState::new(),
            state: NodeState::Follower,
            commit_index: 0,
            last_applied: 0,
            state_machine: HashMap::new(),
        }
    }

    /// Save persistent state to disk (call before responding to RPCs).
    pub fn persist(&self) -> DiskImage {
        self.persistent.save_to_disk()
    }

    /// Crash and restart from disk.
    pub fn crash_and_restart(disk: &DiskImage, id: usize) -> Self {
        let persistent = PersistentState::load_from_disk(disk);
        Self {
            id,
            persistent,
            state: NodeState::Follower,
            commit_index: 0,
            last_applied: 0,
            state_machine: HashMap::new(),
        }
    }

    /// Start an election.
    pub fn start_election(&mut self) {
        self.state = NodeState::Candidate;
        self.persistent.current_term += 1;
        self.persistent.voted_for = Some(self.id);
    }

    /// Handle a vote request.
    pub fn handle_vote_request(
        &mut self,
        term: u64,
        candidate_id: usize,
        last_log_index: u64,
    ) -> bool {
        if term > self.persistent.current_term {
            self.persistent.current_term = term;
            self.state = NodeState::Follower;
            self.persistent.voted_for = None;
        }

        let mut granted = false;
        if term >= self.persistent.current_term {
            let can_vote = self.persistent.voted_for.is_none()
                || self.persistent.voted_for == Some(candidate_id);
            let log_ok = last_log_index >= self.persistent.log.len() as u64;
            if can_vote && log_ok {
                self.persistent.voted_for = Some(candidate_id);
                granted = true;
            }
        }
        granted
    }

    /// Append a log entry.
    pub fn append_entry(&mut self, command: &str) {
        self.persistent.append(command, self.persistent.current_term);
    }

    /// Apply committed entries.
    pub fn apply_committed(&mut self) {
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            if let Some(entry) = self
                .persistent
                .log
                .iter()
                .find(|e| e.index == self.last_applied)
            {
                let parts: Vec<&str> = entry.command.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "SET" {
                    self.state_machine
                        .insert(parts[1].to_string(), parts[2].to_string());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_survives_restart() {
        let mut node = PersistentRaftNode::new(0);
        node.persistent.current_term = 5;
        node.persistent.voted_for = Some(2);
        node.persistent.append("SET x 1", 5);

        // Save to disk
        let disk = node.persist();

        // Crash and restart
        let restarted = PersistentRaftNode::crash_and_restart(&disk, 0);
        assert_eq!(restarted.persistent.current_term, 5);
        assert_eq!(restarted.persistent.voted_for, Some(2));
        assert_eq!(restarted.persistent.log.len(), 1);
    }

    #[test]
    fn leader_re_elected_after_crash() {
        // Create node with some state
        let mut node = PersistentRaftNode::new(0);
        node.persistent.current_term = 3;
        let disk = node.persist();

        // Crash and restart
        let mut restarted = PersistentRaftNode::crash_and_restart(&disk, 0);

        // Can start election
        restarted.start_election();
        assert_eq!(restarted.persistent.current_term, 4);
        assert_eq!(restarted.persistent.voted_for, Some(0));
        assert_eq!(restarted.state, NodeState::Candidate);
    }

    #[test]
    fn voted_for_prevents_double_voting() {
        let mut node = PersistentRaftNode::new(0);

        // Vote for candidate 1
        let granted = node.handle_vote_request(1, 1, 0);
        assert!(granted);

        // Try to vote for candidate 2 in same term
        let granted2 = node.handle_vote_request(1, 2, 0);
        assert!(!granted2);
    }

    #[test]
    fn log_entries_survive_restart() {
        let mut node = PersistentRaftNode::new(0);
        node.persistent.current_term = 1;
        node.append_entry("SET a 1");
        node.append_entry("SET b 2");
        node.append_entry("SET c 3");

        let disk = node.persist();
        let restarted = PersistentRaftNode::crash_and_restart(&disk, 0);

        assert_eq!(restarted.persistent.log.len(), 3);
        assert_eq!(restarted.persistent.log[0].command, "SET a 1");
        assert_eq!(restarted.persistent.log[2].command, "SET c 3");
    }

    #[test]
    fn higher_term_causes_step_down() {
        let mut node = PersistentRaftNode::new(0);
        node.state = NodeState::Leader;
        node.persistent.current_term = 2;

        let _granted = node.handle_vote_request(5, 1, 0);
        assert_eq!(node.state, NodeState::Follower);
        assert_eq!(node.persistent.current_term, 5);
    }
}
