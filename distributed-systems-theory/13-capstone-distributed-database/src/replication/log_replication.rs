//! Log replication between a leader node and its followers.
//!
//! This module simulates the core of Raft-style log replication: a leader appends
//! entries to its local log and then replicates them to followers. Each follower
//! responds with success if the terms match, or with a failure/stale-term rejection
//! otherwise.

/// A single entry in the replication log.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The 1-based index of this entry.
    pub index: u64,
    /// The term during which this entry was created.
    pub term: u64,
    /// The data payload of the entry.
    pub data: String,
}

/// The result of attempting to replicate an entry to a single follower.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplicationResult {
    /// The follower accepted the entry.
    Success,
    /// The follower rejected the entry for the given reason.
    Failed(String),
    /// The follower has a higher term, indicating a new leader exists.
    StaleTerm(u64),
}

/// Manages log replication from a leader to its set of followers.
#[derive(Debug)]
pub struct LogReplicator {
    /// The id of the leader node.
    pub node_id: u64,
    /// The ids of follower nodes.
    followers: Vec<u64>,
    /// The leader's local log.
    log: Vec<LogEntry>,
}

impl LogReplicator {
    /// Create a new `LogReplicator` for the given leader and followers.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the leader node.
    /// * `followers` - The ids of the follower nodes.
    pub fn new(node_id: u64, followers: Vec<u64>) -> Self {
        Self {
            node_id,
            followers,
            log: Vec::new(),
        }
    }

    /// Append a new entry to the local log.
    ///
    /// # Arguments
    ///
    /// * `data` - The data payload for the entry.
    /// * `term` - The current Raft term.
    ///
    /// # Returns
    ///
    /// The 1-based index of the newly appended entry.
    pub fn append_entry(&mut self, data: String, term: u64) -> u64 {
        let index = self.log.len() as u64 + 1;
        self.log.push(LogEntry { index, term, data });
        index
    }

    /// Replicate a batch of entries to all followers.
    ///
    /// In this simulation, a follower succeeds if the entry's term matches the
    /// leader's current term (the term of the last local entry). If the follower
    /// has a higher term, a `StaleTerm` result is returned. Other failures are
    /// reported as `Failed`.
    ///
    /// # Arguments
    ///
    /// * `entries` - The entries to replicate.
    ///
    /// # Returns
    ///
    /// A vector of `(follower_id, ReplicationResult)` pairs, one per follower.
    pub fn replicate(&self, entries: Vec<LogEntry>) -> Vec<(u64, ReplicationResult)> {
        let leader_term = self.log.last().map_or(0, |e| e.term);
        self.followers
            .iter()
            .map(|&follower_id| {
                // Simulate: followers accept if the entry term <= leader term.
                // If a follower were ahead, it would reject with StaleTerm.
                let result = if entries.iter().all(|e| e.term <= leader_term) {
                    ReplicationResult::Success
                } else {
                    ReplicationResult::StaleTerm(leader_term + 1)
                };
                (follower_id, result)
            })
            .collect()
    }

    /// Return a reference to the leader's local log.
    pub fn get_log(&self) -> &[LogEntry] {
        &self.log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_entry() {
        let mut repl = LogReplicator::new(1, vec![2, 3]);
        let idx = repl.append_entry("data1".into(), 1);
        assert_eq!(idx, 1);
        let idx = repl.append_entry("data2".into(), 1);
        assert_eq!(idx, 2);
        assert_eq!(repl.get_log().len(), 2);
    }

    #[test]
    fn test_replicate_success() {
        let mut repl = LogReplicator::new(1, vec![2, 3]);
        repl.append_entry("data".into(), 1);
        let entries = vec![LogEntry {
            index: 1,
            term: 1,
            data: "data".into(),
        }];
        let results = repl.replicate(entries);
        assert_eq!(results.len(), 2);
        for (id, result) in &results {
            assert!(*result == ReplicationResult::Success, "node {} failed", id);
        }
    }

    #[test]
    fn test_replicate_stale_term() {
        let mut repl = LogReplicator::new(1, vec![2]);
        repl.append_entry("data".into(), 2);
        let entries = vec![LogEntry {
            index: 1,
            term: 3,
            data: "data".into(),
        }];
        let results = repl.replicate(entries);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].1, ReplicationResult::StaleTerm(_)));
    }
}
