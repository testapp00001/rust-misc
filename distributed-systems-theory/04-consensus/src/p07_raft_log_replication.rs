//! # Exercise: Raft Log Replication
//!
//! ## Theory
//!
//! Once a leader is elected, it handles client requests by appending them to
//! its log and replicating entries to followers via AppendEntries RPCs. A log
//! entry is committed when a majority of nodes have replicated it. The leader
//! then applies committed entries to its state machine and responds to the
//! client.
//!
//! AppendEntries also serves as a heartbeat. Followers that don't hear from
//! a leader eventually time out and start a new election.
//!
//! ## Proof / Intuition
//!
//! Log safety is maintained by the AppendEntries consistency check: the leader
//! includes the index and term of the previous log entry. If the follower
//! doesn't have an entry at that index with that term, it rejects the request.
//! This ensures that the leader's log is always at least as up-to-date as
//! any follower's log.
//!
//! ## Implementation Task
//!
//! Implement Raft log replication:
//! - `LogEntry` with term, index, command
//! - `append_entries()` on followers
//! - Leader sends entries, collects acknowledgments
//! - Commit on majority acknowledgment
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Log consistency is maintained
//! - Entries commit on majority ack
//! - Follower catches up with leader

/// A log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: String,
}

/// A Raft node for log replication.
#[derive(Debug)]
pub struct LogNode {
    pub id: usize,
    pub current_term: u64,
    pub log: Vec<LogEntry>,
    pub commit_index: u64,
    pub last_applied: u64,
    /// State machine output (applied commands).
    pub state_machine: Vec<String>,
    /// For leader: next index to send to each peer.
    pub next_index: Vec<u64>,
    /// For leader: highest replicated index on each peer.
    pub match_index: Vec<u64>,
    pub is_leader: bool,
}

/// Result of an AppendEntries RPC.
#[derive(Debug, Clone)]
pub struct AppendEntriesResponse {
    pub term: u64,
    pub success: bool,
    /// The last index the follower has (for backtracking).
    pub match_index: u64,
}

impl LogNode {
    pub fn new(id: usize, num_peers: usize) -> Self {
        Self {
            id,
            current_term: 0,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            state_machine: Vec::new(),
            next_index: vec![1; num_peers],
            match_index: vec![0; num_peers],
            is_leader: false,
        }
    }

    /// Leader: append a new entry to the log.
    pub fn leader_append(&mut self, command: &str) -> LogEntry {
        let index = self.log.len() as u64 + 1;
        let entry = LogEntry {
            term: self.current_term,
            index,
            command: command.to_string(),
        };
        self.log.push(entry.clone());
        entry
    }

    /// Follower: handle an AppendEntries RPC.
    pub fn append_entries(
        &mut self,
        term: u64,
        _leader_id: usize,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: u64,
    ) -> AppendEntriesResponse {
        // Reject if term is stale
        if term < self.current_term {
            return AppendEntriesResponse {
                term: self.current_term,
                success: false,
                match_index: self.log.len() as u64,
            };
        }

        self.current_term = term;

        // Consistency check: verify previous entry matches
        if prev_log_index > 0 {
            if let Some(entry) = self.log.iter().find(|e| e.index == prev_log_index) {
                if entry.term != prev_log_term {
                    return AppendEntriesResponse {
                        term: self.current_term,
                        success: false,
                        match_index: self.log.len() as u64,
                    };
                }
            } else {
                return AppendEntriesResponse {
                    term: self.current_term,
                    success: false,
                    match_index: self.log.len() as u64,
                };
            }
        }

        // Append new entries (and truncate conflicting ones)
        for entry in entries {
            if let Some(existing) = self.log.iter().find(|e| e.index == entry.index) {
                if existing.term != entry.term {
                    // Truncate from this index and insert
                    self.log.retain(|e| e.index < entry.index);
                    self.log.push(entry);
                }
                // If term matches, skip (already have it)
            } else {
                self.log.push(entry);
            }
        }

        // Sort by index
        self.log.sort_by_key(|e| e.index);

        // Update commit index
        if leader_commit > self.commit_index {
            self.commit_index = leader_commit.min(self.log.len() as u64);
            self.apply_committed();
        }

        AppendEntriesResponse {
            term: self.current_term,
            success: true,
            match_index: self.log.len() as u64,
        }
    }

    /// Apply committed entries to the state machine.
    fn apply_committed(&mut self) {
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            if let Some(entry) = self.log.iter().find(|e| e.index == self.last_applied) {
                self.state_machine.push(entry.command.clone());
            }
        }
    }

    /// Leader: process an AppendEntries response and potentially commit.
    pub fn handle_append_response(
        &mut self,
        peer_id: usize,
        response: AppendEntriesResponse,
    ) -> bool {
        if response.success {
            self.match_index[peer_id] = response.match_index;
            self.next_index[peer_id] = response.match_index + 1;

            // Check if we can advance commit index
            self.try_advance_commit()
        } else {
            // Decrement next_index and retry
            if self.next_index[peer_id] > 1 {
                self.next_index[peer_id] -= 1;
            }
            false
        }
    }

    /// Try to advance the commit index based on majority replication.
    fn try_advance_commit(&mut self) -> bool {
        let old_commit = self.commit_index;

        for n in (self.commit_index + 1)..=(self.log.len() as u64) {
            // Check if entry at index n has majority
            let entry_term = self
                .log
                .iter()
                .find(|e| e.index == n)
                .map(|e| e.term);

            if let Some(term) = entry_term {
                if term == self.current_term {
                    let replicated = self.match_index.iter().filter(|&&m| m >= n).count() + 1; // +1 for leader
                    let total = self.match_index.len() + 1;
                    if replicated > total / 2 {
                        self.commit_index = n;
                    }
                }
            }
        }

        if self.commit_index > old_commit {
            self.apply_committed();
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
    fn log_consistency_maintained() {
        let mut leader = LogNode::new(0, 2);
        leader.is_leader = true;
        leader.current_term = 1;

        leader.leader_append("cmd1");
        leader.leader_append("cmd2");

        assert_eq!(leader.log.len(), 2);
        assert_eq!(leader.log[0].command, "cmd1");
        assert_eq!(leader.log[1].command, "cmd2");
    }

    #[test]
    fn commit_on_majority_ack() {
        let mut leader = LogNode::new(0, 2);
        leader.is_leader = true;
        leader.current_term = 1;

        leader.leader_append("write_x");

        // Simulate responses from 2 followers
        let resp1 = AppendEntriesResponse {
            term: 1,
            success: true,
            match_index: 1,
        };
        let resp2 = AppendEntriesResponse {
            term: 1,
            success: true,
            match_index: 1,
        };

        leader.handle_append_response(0, resp1);
        leader.handle_append_response(1, resp2);

        // 3 out of 3 nodes have it (leader + 2 followers) = majority
        assert_eq!(leader.commit_index, 1);
        assert_eq!(leader.state_machine, vec!["write_x"]);
    }

    #[test]
    fn follower_catches_up() {
        let mut leader = LogNode::new(0, 1);
        leader.is_leader = true;
        leader.current_term = 1;
        leader.leader_append("a");
        leader.leader_append("b");
        leader.leader_append("c");

        let mut follower = LogNode::new(1, 1);

        // Send all entries
        let response = follower.append_entries(
            1,
            0,
            0, 0, // No previous entry
            leader.log.clone(),
            0,
        );
        assert!(response.success);
        assert_eq!(follower.log.len(), 3);
        assert_eq!(follower.log[2].command, "c");
    }

    #[test]
    fn follower_rejects_stale_term() {
        let mut follower = LogNode::new(1, 1);
        follower.current_term = 5;

        let response = follower.append_entries(
            3, // Stale term
            0,
            0,
            0,
            vec![],
            0,
        );
        assert!(!response.success);
    }

    #[test]
    fn consistency_check_rejects_mismatch() {
        let mut follower = LogNode::new(1, 1);
        follower.current_term = 1;
        follower.log.push(LogEntry {
            term: 1,
            index: 1,
            command: "old".to_string(),
        });

        // Try to append with wrong prev_log_term
        let response = follower.append_entries(
            1,
            0,
            1,  // prev_log_index = 1
            2,  // wrong term
            vec![LogEntry {
                term: 1,
                index: 2,
                command: "new".to_string(),
            }],
            0,
        );
        assert!(!response.success);
    }
}
