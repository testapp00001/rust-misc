//! # Exercise: Raft State Machine
//!
//! ## Theory
//!
//! A Raft node operates in one of three states:
//!
//! - **Follower:** The passive state. Followers respond to RPCs from leaders
//!   and candidates. If they hear nothing, they become candidates.
//! - **Candidate:** An intermediate state during leader election. The candidate
//!   requests votes from other nodes. If it wins a majority, it becomes leader.
//! - **Leader:** The active state. The leader handles client requests and
//!   replicates log entries to followers.
//!
//! Each state transition is driven by messages and timeouts:
//! - Follower -> Candidate: election timeout fires (no heartbeat received)
//! - Candidate -> Leader: receives majority of votes
//! - Candidate -> Follower: discovers a leader with higher term
//! - Leader -> Follower: discovers a higher term
//!
//! ## Proof / Intuition
//!
//! The term number acts as a logical clock. Each election increments the term.
//! Nodes always defer to higher terms, ensuring that stale leaders are
//! quickly replaced. This is the key safety mechanism: a leader is only
//! legitimate if its term is the current term.
//!
//! ## Implementation Task
//!
//! Implement the Raft state machine:
//! - `RaftState` enum: Follower, Candidate, Leader
//! - `RaftNode` struct with state, term, voted_for, log, commit_index
//! - `RaftMessage` enum for RPCs
//! - State transitions with proper term handling
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Initial state is Follower
//! - Term increments on election
//! - Leader steps down on higher term
//! - State transitions are correct

/// The three possible states of a Raft node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

/// A log entry in the Raft log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: String,
}

/// Messages that can be exchanged between Raft nodes.
#[derive(Debug, Clone)]
pub enum RaftMessage {
    RequestVote {
        term: u64,
        candidate_id: usize,
        last_log_index: u64,
        last_log_term: u64,
    },
    RequestVoteResponse {
        term: u64,
        vote_granted: bool,
    },
    AppendEntries {
        term: u64,
        leader_id: usize,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: u64,
    },
    AppendEntriesResponse {
        term: u64,
        success: bool,
    },
}

/// A Raft node with state machine logic.
#[derive(Debug)]
pub struct RaftNode {
    /// Unique identifier.
    pub id: usize,
    /// Current state.
    pub state: RaftState,
    /// Current term (persists across restarts in real Raft).
    pub current_term: u64,
    /// The candidate this node voted for in the current term (None if not voted).
    pub voted_for: Option<usize>,
    /// The log of entries.
    pub log: Vec<LogEntry>,
    /// Index of the highest log entry known to be committed.
    pub commit_index: u64,
    /// Index of the highest log entry applied to the state machine.
    pub last_applied: u64,
    /// For leader: index of next log entry to send to each follower.
    pub next_index: Vec<u64>,
    /// For leader: index of highest log entry known to be replicated on each follower.
    pub match_index: Vec<u64>,
    /// Number of votes received (during candidacy).
    pub votes_received: usize,
}

impl RaftNode {
    /// Create a new Raft node in the Follower state.
    pub fn new(id: usize, num_peers: usize) -> Self {
        Self {
            id,
            state: RaftState::Follower,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            next_index: vec![1; num_peers],
            match_index: vec![0; num_peers],
            votes_received: 0,
        }
    }

    /// Transition to Candidate state and start an election.
    pub fn start_election(&mut self) -> RaftMessage {
        self.state = RaftState::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self.id);
        self.votes_received = 1; // Vote for self

        let (last_log_index, last_log_term) = self.last_log_info();

        RaftMessage::RequestVote {
            term: self.current_term,
            candidate_id: self.id,
            last_log_index,
            last_log_term,
        }
    }

    /// Handle a RequestVote message.
    ///
    /// Grant vote if:
    /// - The candidate's term is >= our current term
    /// - We haven't voted for someone else in this term
    /// - The candidate's log is at least as up-to-date as ours
    pub fn handle_request_vote(&mut self, msg: RaftMessage) -> RaftMessage {
        if let RaftMessage::RequestVote {
            term,
            candidate_id,
            last_log_index,
            last_log_term,
        } = msg
        {
            // If term is higher, update and become follower
            if term > self.current_term {
                self.current_term = term;
                self.state = RaftState::Follower;
                self.voted_for = None;
            }

            let mut vote_granted = false;

            if term >= self.current_term {
                let can_vote = self.voted_for.is_none()
                    || self.voted_for == Some(candidate_id);

                let (my_last_index, my_last_term) = self.last_log_info();
                let log_ok = last_log_term > my_last_term
                    || (last_log_term == my_last_term
                        && last_log_index >= my_last_index);

                if can_vote && log_ok {
                    self.voted_for = Some(candidate_id);
                    vote_granted = true;
                }
            }

            RaftMessage::RequestVoteResponse {
                term: self.current_term,
                vote_granted,
            }
        } else {
            panic!("expected RequestVote message");
        }
    }

    /// Handle winning an election (received majority of votes).
    pub fn become_leader(&mut self, num_peers: usize) {
        self.state = RaftState::Leader;
        self.next_index = vec![self.log.len() as u64 + 1; num_peers];
        self.match_index = vec![0; num_peers];
    }

    /// Handle discovering a higher term (step down to follower).
    pub fn step_down(&mut self, new_term: u64) {
        self.current_term = new_term;
        self.state = RaftState::Follower;
        self.voted_for = None;
    }

    /// Append a log entry (leader only).
    pub fn append_entry(&mut self, command: &str) -> LogEntry {
        let index = self.log.len() as u64 + 1;
        let entry = LogEntry {
            term: self.current_term,
            index,
            command: command.to_string(),
        };
        self.log.push(entry.clone());
        entry
    }

    /// Get the last log index and term.
    fn last_log_info(&self) -> (u64, u64) {
        match self.log.last() {
            Some(entry) => (entry.index, entry.term),
            None => (0, 0),
        }
    }

    /// Check if this node has majority votes.
    pub fn has_majority(&self, total_nodes: usize) -> bool {
        self.votes_received > total_nodes / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_is_follower() {
        let node = RaftNode::new(0, 4);
        assert_eq!(node.state, RaftState::Follower);
        assert_eq!(node.current_term, 0);
        assert!(node.voted_for.is_none());
    }

    #[test]
    fn term_increments_on_election() {
        let mut node = RaftNode::new(0, 4);
        let msg = node.start_election();
        assert_eq!(node.current_term, 1);
        assert_eq!(node.state, RaftState::Candidate);

        if let RaftMessage::RequestVote { term, .. } = msg {
            assert_eq!(term, 1);
        } else {
            panic!("expected RequestVote");
        }
    }

    #[test]
    fn leader_steps_down_on_higher_term() {
        let mut node = RaftNode::new(0, 4);
        node.state = RaftState::Leader;
        node.current_term = 5;

        node.step_down(10);
        assert_eq!(node.state, RaftState::Follower);
        assert_eq!(node.current_term, 10);
    }

    #[test]
    fn vote_granted_for_valid_candidate() {
        let mut node = RaftNode::new(0, 4);
        let msg = RaftMessage::RequestVote {
            term: 1,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        };
        let response = node.handle_request_vote(msg);
        if let RaftMessage::RequestVoteResponse {
            vote_granted, ..
        } = response
        {
            assert!(vote_granted);
        }
    }

    #[test]
    fn vote_denied_for_lower_term() {
        let mut node = RaftNode::new(0, 4);
        node.current_term = 5;
        let msg = RaftMessage::RequestVote {
            term: 3,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        };
        let response = node.handle_request_vote(msg);
        if let RaftMessage::RequestVoteResponse {
            vote_granted, ..
        } = response
        {
            assert!(!vote_granted);
        }
    }

    #[test]
    fn majority_detection() {
        let mut node = RaftNode::new(0, 4);
        node.votes_received = 3;
        assert!(node.has_majority(5)); // 3 > 2

        node.votes_received = 2;
        assert!(!node.has_majority(5)); // 2 is not > 2
    }
}
