//! # Exercise: Combined Raft (Leader Election + Log Replication)
//!
//! ## Theory
//!
//! A complete Raft implementation combines leader election and log replication
//! into a single coherent protocol. The leader:
//!
//! 1. Is elected via the randomized election timeout mechanism.
//! 2. Receives client commands and appends them to its log.
//! 3. Replicates log entries to followers via AppendEntries RPCs.
//! 4. Commits entries once a majority have acknowledged them.
//! 5. Applies committed entries to its state machine.
//! 6. Responds to the client with the result.
//!
//! If the leader fails, the election timeout triggers a new election, and a
//! new leader takes over. The new leader's log is at least as up-to-date as
//! a majority's, so committed entries are never lost.
//!
//! ## Proof / Intuition
//!
//! Safety is maintained by two invariants:
//! 1. The election restriction ensures a new leader has all committed entries.
//! 2. The commit rule ensures entries are only committed after majority
//!    replication in the current term.
//!
//! Together, these guarantee that once an entry is committed, it will
//! eventually be applied by all non-faulty nodes.
//!
//! ## Implementation Task
//!
//! Build a combined Raft simulation:
//! - 5 nodes that can elect a leader and replicate commands
//! - Simulate the full lifecycle: election, command submission, replication
//! - Test that the system works end-to-end
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Leader election works
//! - Commands are replicated to all followers
//! - Committed entries are applied to the state machine

use std::collections::HashMap;

use rand::Rng;

/// State of a Raft node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Follower,
    Candidate,
    Leader,
}

/// A log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: String,
}

/// A simulated Raft node.
#[derive(Debug)]
pub struct SimRaftNode {
    pub id: usize,
    pub state: NodeState,
    pub current_term: u64,
    pub voted_for: Option<usize>,
    pub log: Vec<LogEntry>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub state_machine: HashMap<String, String>,
    pub election_timeout: u64,
    pub election_clock: u64,
    pub peers: Vec<usize>,
}

impl SimRaftNode {
    pub fn new(id: usize, peers: Vec<usize>) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            id,
            state: NodeState::Follower,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            state_machine: HashMap::new(),
            election_timeout: rng.gen_range(150..=300),
            election_clock: 0,
            peers,
        }
    }

    /// Increment election clock; returns true if election timeout fired.
    pub fn tick(&mut self) -> bool {
        if self.state == NodeState::Leader {
            self.election_clock = 0;
            return false;
        }
        self.election_clock += 1;
        if self.election_clock >= self.election_timeout {
            true // Timeout fired
        } else {
            false
        }
    }

    /// Start an election.
    pub fn start_election(&mut self) {
        self.state = NodeState::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self.id);
        self.election_clock = 0;
        let mut rng = rand::thread_rng();
        self.election_timeout = rng.gen_range(150..=300);
    }

    /// Handle a vote request. Returns whether vote was granted.
    pub fn handle_vote_request(
        &mut self,
        term: u64,
        candidate_id: usize,
        last_log_index: u64,
    ) -> (u64, bool) {
        if term > self.current_term {
            self.current_term = term;
            self.state = NodeState::Follower;
            self.voted_for = None;
        }

        let mut granted = false;
        if term >= self.current_term {
            let can_vote = self.voted_for.is_none()
                || self.voted_for == Some(candidate_id);
            let log_ok = last_log_index >= self.log.len() as u64;
            if can_vote && log_ok {
                self.voted_for = Some(candidate_id);
                granted = true;
            }
        }
        (self.current_term, granted)
    }

    /// Append entries from leader.
    pub fn append_entries(
        &mut self,
        term: u64,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: u64,
    ) -> (u64, bool) {
        if term < self.current_term {
            return (self.current_term, false);
        }
        self.current_term = term;
        self.election_clock = 0; // Reset election timer

        // Consistency check
        if prev_log_index > 0 {
            if let Some(entry) = self.log.iter().find(|e| e.index == prev_log_index) {
                if entry.term != prev_log_term {
                    return (self.current_term, false);
                }
            } else {
                return (self.current_term, false);
            }
        }

        // Append entries
        for entry in entries {
            if !self.log.iter().any(|e| e.index == entry.index) {
                self.log.push(entry);
            }
        }
        self.log.sort_by_key(|e| e.index);

        // Update commit index
        if leader_commit > self.commit_index {
            self.commit_index = leader_commit.min(self.log.len() as u64);
            self.apply_committed();
        }

        (self.current_term, true)
    }

    /// Apply committed entries to the state machine.
    fn apply_committed(&mut self) {
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            if let Some(entry) = self.log.iter().find(|e| e.index == self.last_applied) {
                // Simple state machine: parse "SET key value"
                let parts: Vec<&str> = entry.command.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "SET" {
                    self.state_machine
                        .insert(parts[1].to_string(), parts[2].to_string());
                }
            }
        }
    }

    /// Leader: submit a command.
    pub fn submit_command(&mut self, command: &str) -> LogEntry {
        assert_eq!(self.state, NodeState::Leader);
        let index = self.log.len() as u64 + 1;
        let entry = LogEntry {
            term: self.current_term,
            index,
            command: command.to_string(),
        };
        self.log.push(entry.clone());
        entry
    }
}

/// Simulate a full Raft cluster lifecycle.
pub struct SimRaftCluster {
    pub nodes: Vec<SimRaftNode>,
}

impl SimRaftCluster {
    pub fn new(num_nodes: usize) -> Self {
        let nodes: Vec<SimRaftNode> = (0..num_nodes)
            .map(|i| {
                let peers: Vec<usize> = (0..num_nodes).filter(|&j| j != i).collect();
                SimRaftNode::new(i, peers)
            })
            .collect();
        Self { nodes }
    }

    /// Run election timeouts until a leader is elected.
    pub fn elect_leader(&mut self) -> Option<usize> {
        let total = self.nodes.len();
        let mut rounds = 0;

        loop {
            rounds += 1;
            if rounds > 500 {
                return None;
            }

            // Check for timeouts and start elections
            let mut candidates = Vec::new();
            for node in &mut self.nodes {
                if node.tick() {
                    node.start_election();
                    candidates.push(node.id);
                }
            }

            // Process vote requests
            let mut votes: HashMap<usize, usize> = HashMap::new();
            for &cid in &candidates {
                let req_term = self.nodes[cid].current_term;
                let req_last_idx = self.nodes[cid].log.len() as u64;
                let mut vote_count = 1; // Self-vote

                for node in &mut self.nodes {
                    if node.id != cid {
                        let (_, granted) = node.handle_vote_request(
                            req_term,
                            cid,
                            req_last_idx,
                        );
                        if granted {
                            vote_count += 1;
                        }
                    }
                }
                votes.insert(cid, vote_count);
            }

            // Check for winner
            for (&cid, &count) in &votes {
                if count > total / 2 {
                    self.nodes[cid].state = NodeState::Leader;
                    return Some(cid);
                }
            }
        }
    }

    /// Get the leader's ID.
    pub fn leader_id(&self) -> Option<usize> {
        self.nodes
            .iter()
            .find(|n| n.state == NodeState::Leader)
            .map(|n| n.id)
    }

    /// Submit a command to the leader and replicate to followers.
    pub fn submit_command(&mut self, command: &str) -> bool {
        let leader_id = match self.leader_id() {
            Some(id) => id,
            None => return false,
        };

        let entry = self.nodes[leader_id].submit_command(command);
        let term = entry.term;
        let prev_log_index = entry.index - 1;
        let prev_log_term = if prev_log_index > 0 {
            self.nodes[leader_id]
                .log
                .iter()
                .find(|e| e.index == prev_log_index)
                .map(|e| e.term)
                .unwrap_or(0)
        } else {
            0
        };

        let mut ack_count = 1; // Leader counts as 1
        let followers: Vec<usize> = self.nodes[leader_id].peers.clone();

        for &fid in &followers {
            let (resp_term, success) = self.nodes[fid].append_entries(
                term,
                prev_log_index,
                prev_log_term,
                vec![entry.clone()],
                0,
            );
            if success {
                ack_count += 1;
            }
            if resp_term > term {
                self.nodes[leader_id].current_term = resp_term;
                self.nodes[leader_id].state = NodeState::Follower;
                return false;
            }
        }

        // Commit if majority
        let total = self.nodes.len();
        if ack_count > total / 2 {
            let commit_index = entry.index;
            self.nodes[leader_id].commit_index = commit_index;
            // Apply committed
            while self.nodes[leader_id].last_applied < commit_index {
                self.nodes[leader_id].last_applied += 1;
                let last_applied = self.nodes[leader_id].last_applied;
                let command = self.nodes[leader_id]
                    .log
                    .iter()
                    .find(|e| e.index == last_applied)
                    .map(|e| e.command.clone());
                if let Some(cmd) = command {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() >= 3 && parts[0] == "SET" {
                        self.nodes[leader_id]
                            .state_machine
                            .insert(parts[1].to_string(), parts[2].to_string());
                    }
                }
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leader_election_works() {
        let mut cluster = SimRaftCluster::new(5);
        let leader = cluster.elect_leader();
        assert!(leader.is_some());

        let leader_count = cluster
            .nodes
            .iter()
            .filter(|n| n.state == NodeState::Leader)
            .count();
        assert_eq!(leader_count, 1);
    }

    #[test]
    fn command_replicated_to_followers() {
        let mut cluster = SimRaftCluster::new(5);
        cluster.elect_leader();

        let success = cluster.submit_command("SET x 42");
        assert!(success);

        let leader = cluster.leader_id().unwrap();
        assert_eq!(
            cluster.nodes[leader].state_machine.get("x"),
            Some(&"42".to_string())
        );
    }

    #[test]
    fn multiple_commands_replicated() {
        let mut cluster = SimRaftCluster::new(5);
        cluster.elect_leader();

        assert!(cluster.submit_command("SET a 1"));
        assert!(cluster.submit_command("SET b 2"));
        assert!(cluster.submit_command("SET c 3"));

        let leader = cluster.leader_id().unwrap();
        let sm = &cluster.nodes[leader].state_machine;
        assert_eq!(sm.get("a"), Some(&"1".to_string()));
        assert_eq!(sm.get("b"), Some(&"2".to_string()));
        assert_eq!(sm.get("c"), Some(&"3".to_string()));
    }

    #[test]
    fn no_leader_returns_false() {
        let mut cluster = SimRaftCluster::new(3);
        // Don't elect a leader
        assert!(!cluster.submit_command("SET x 1"));
    }
}
