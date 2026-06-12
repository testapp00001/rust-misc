//! # Exercise: Raft Leader Election
//!
//! ## Theory
//!
//! Raft leader election uses randomized timeouts to avoid split votes. Each
//! follower has an election timeout (150-300ms in the specification). If a
//! follower receives no heartbeat from a leader before its timeout fires, it
//! becomes a candidate, increments its term, and requests votes.
//!
//! The randomized timeout ensures that in most cases, one candidate will time
//! out before others, giving it a head start in collecting votes. This
//! dramatically reduces the probability of split votes.
//!
//! ## Proof / Intuition
//!
//! With N nodes and random timeouts uniformly distributed in [T, 2T], the
//! probability that two nodes time out within the same delta approaches zero
//! as T increases. Even if a split vote occurs, the next election round uses
//! new random timeouts, making another split vote unlikely.
//!
//! A candidate wins the election if it receives votes from a majority of nodes
//! (including itself). The election restriction ensures that only candidates
//! with up-to-date logs can win.
//!
//! ## Implementation Task
//!
//! Implement Raft leader election:
//! - `ElectionTimeout` with randomized duration
//! - `start_election()` -- become Candidate, request votes
//! - `handle_request_vote()` -- grant or deny votes
//! - `handle_vote_response()` -- count votes, become Leader on majority
//! - Simulate 5 nodes with one election
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - A single leader is elected
//! - Split votes are handled (eventual leader emergence)
//! - Term increments correctly

use std::collections::HashMap;

use rand::Rng;

/// Election timeout with randomized duration.
#[derive(Debug, Clone)]
pub struct ElectionTimeout {
    /// When this timeout was set (tick number).
    pub set_at: u64,
    /// Duration in ticks before the timeout fires.
    pub duration: u64,
}

impl ElectionTimeout {
    /// Create a new random timeout between min and max ticks.
    pub fn random(min: u64, max: u64) -> Self {
        let mut rng = rand::thread_rng();
        let duration = rng.gen_range(min..=max);
        Self {
            set_at: 0,
            duration,
        }
    }

    /// Check if the timeout has fired at the given tick.
    pub fn has_fired(&self, current_tick: u64) -> bool {
        current_tick >= self.set_at + self.duration
    }

    /// Reset the timeout at the given tick.
    pub fn reset(&mut self, current_tick: u64) {
        self.set_at = current_tick;
    }
}

/// A Raft node for leader election simulation.
#[derive(Debug)]
pub struct ElectionNode {
    pub id: usize,
    pub state: NodeState,
    pub current_term: u64,
    pub voted_for: Option<usize>,
    pub election_timeout: ElectionTimeout,
    pub log_length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Follower,
    Candidate,
    Leader,
}

/// A vote request.
#[derive(Debug, Clone)]
pub struct VoteRequest {
    pub term: u64,
    pub candidate_id: usize,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

/// A vote response.
#[derive(Debug, Clone)]
pub struct VoteResponse {
    pub term: u64,
    pub vote_granted: bool,
    pub voter_id: usize,
}

impl ElectionNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            state: NodeState::Follower,
            current_term: 0,
            voted_for: None,
            election_timeout: ElectionTimeout::random(150, 300),
            log_length: 0,
        }
    }

    /// Start an election by becoming a candidate.
    pub fn start_election(&mut self) -> VoteRequest {
        self.state = NodeState::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self.id);
        self.election_timeout.reset(0); // Reset timeout for next round

        VoteRequest {
            term: self.current_term,
            candidate_id: self.id,
            last_log_index: self.log_length as u64,
            last_log_term: 0,
        }
    }

    /// Handle a vote request from a candidate.
    pub fn handle_vote_request(&mut self, req: VoteRequest) -> VoteResponse {
        // Update term if higher
        if req.term > self.current_term {
            self.current_term = req.term;
            self.state = NodeState::Follower;
            self.voted_for = None;
        }

        let mut vote_granted = false;

        if req.term >= self.current_term {
            let can_vote = self.voted_for.is_none()
                || self.voted_for == Some(req.candidate_id);

            let log_ok = req.last_log_index >= self.log_length as u64;

            if can_vote && log_ok {
                self.voted_for = Some(req.candidate_id);
                vote_granted = true;
            }
        }

        VoteResponse {
            term: self.current_term,
            vote_granted,
            voter_id: self.id,
        }
    }

    /// Process a vote response (called by the candidate).
    pub fn handle_vote_response(&mut self, resp: VoteResponse) {
        if resp.term > self.current_term {
            self.current_term = resp.term;
            self.state = NodeState::Follower;
            self.voted_for = None;
        }
    }

    /// Become leader.
    pub fn become_leader(&mut self) {
        self.state = NodeState::Leader;
    }

    /// Step down to follower.
    pub fn step_down(&mut self) {
        self.state = NodeState::Follower;
        self.voted_for = None;
    }
}

/// Simulate a full election round among the given nodes.
///
/// Returns the ID of the elected leader, or None if no leader was elected.
pub fn simulate_election(nodes: &mut [ElectionNode]) -> Option<usize> {
    let total = nodes.len();
    let mut round = 0;

    loop {
        round += 1;
        if round > 100 {
            return None; // Safety: prevent infinite loop
        }

        // Find candidates (nodes that would start election)
        let candidates: Vec<usize> = nodes
            .iter()
            .filter(|n| n.state == NodeState::Follower || n.state == NodeState::Candidate)
            .map(|n| n.id)
            .collect();

        if candidates.is_empty() {
            // Already have a leader
            return nodes.iter().find(|n| n.state == NodeState::Leader).map(|n| n.id);
        }

        // Each candidate sends vote requests
        let mut votes: HashMap<usize, usize> = HashMap::new(); // candidate -> vote count

        for &cid in &candidates {
            let req = {
                let candidate = &mut nodes[cid];
                candidate.start_election()
            };

            // Collect votes
            let mut vote_count = 1; // Vote for self
            for node in nodes.iter_mut() {
                if node.id != cid {
                    let resp = node.handle_vote_request(VoteRequest {
                        term: req.term,
                        candidate_id: req.candidate_id,
                        last_log_index: req.last_log_index,
                        last_log_term: req.last_log_term,
                    });
                    if resp.vote_granted {
                        vote_count += 1;
                    }
                }
            }
            votes.insert(cid, vote_count);
        }

        // Check if any candidate won
        for (&cid, &count) in &votes {
            if count > total / 2 {
                nodes[cid].become_leader();
                return Some(cid);
            }
        }

        // No winner; advance time to trigger new elections
        for node in nodes.iter_mut() {
            if node.state != NodeState::Leader {
                node.election_timeout.set_at += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_leader_elected() {
        let mut nodes: Vec<ElectionNode> = (0..5).map(ElectionNode::new).collect();
        let leader = simulate_election(&mut nodes);
        assert!(leader.is_some());

        let leader_count = nodes
            .iter()
            .filter(|n| n.state == NodeState::Leader)
            .count();
        assert_eq!(leader_count, 1, "exactly one leader should be elected");
    }

    #[test]
    fn term_increments() {
        let mut node = ElectionNode::new(0);
        assert_eq!(node.current_term, 0);

        let _req = node.start_election();
        assert_eq!(node.current_term, 1);
        assert_eq!(node.state, NodeState::Candidate);
    }

    #[test]
    fn split_vote_handled() {
        // Run election multiple times to ensure it eventually succeeds
        let mut successes = 0;
        for _ in 0..20 {
            let mut nodes: Vec<ElectionNode> = (0..5).map(ElectionNode::new).collect();
            if simulate_election(&mut nodes).is_some() {
                successes += 1;
            }
        }
        assert!(
            successes > 15,
            "election should succeed most of the time: {successes}/20"
        );
    }

    #[test]
    fn vote_granted_only_once_per_term() {
        let mut voter = ElectionNode::new(0);
        voter.current_term = 1;

        let req1 = VoteRequest {
            term: 1,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        };
        let resp1 = voter.handle_vote_request(req1);
        assert!(resp1.vote_granted);

        // Same term, different candidate -- should deny
        let req2 = VoteRequest {
            term: 1,
            candidate_id: 2,
            last_log_index: 0,
            last_log_term: 0,
        };
        let resp2 = voter.handle_vote_request(req2);
        assert!(!resp2.vote_granted);
    }

    #[test]
    fn higher_term_causes_step_down() {
        let mut node = ElectionNode::new(0);
        node.current_term = 3;
        node.state = NodeState::Leader;

        let req = VoteRequest {
            term: 5,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        };
        node.handle_vote_request(req);
        assert_eq!(node.state, NodeState::Follower);
        assert_eq!(node.current_term, 5);
    }
}
