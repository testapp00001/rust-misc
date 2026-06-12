//! Core Raft state machine.
//!
//! [`RaftNode`] holds all persistent state (current term, voted-for, log) and
//! volatile state (commit index, last applied, leader bookkeeping).  Every
//! incoming RPC is handled by a dedicated method that performs the Raft paper's
//! state-transition rules deterministically.
//!
//! Network I/O is injected through the [`RpcTransport`] trait so the node
//! stays testable without real sockets.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::{MqError, Result};

// ── Raft Role ───────────────────────────────────────────────────────────────

/// The three roles a Raft node can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

// ── Log Entry ───────────────────────────────────────────────────────────────

/// A single entry in the replicated log.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    /// Term when the entry was received by the leader.
    pub term: u64,
    /// Position in the log (1-based).
    pub index: u64,
    /// The command to apply to the state machine.
    pub command: Command,
}

// ── Command ─────────────────────────────────────────────────────────────────

/// A command that can be proposed to the replicated state machine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Command {
    /// Append data to a topic/partition.
    Produce {
        topic: String,
        partition: u32,
        data: Vec<u8>,
    },
    /// Create a new topic with the given partition count.
    CreateTopic {
        name: String,
        partitions: u32,
    },
    /// No-operation entry emitted by a newly elected leader to establish its
    /// commit index without changing application state.
    NoOp,
}

// ── RPC Messages ────────────────────────────────────────────────────────────

/// AppendEntries RPC request (leader -> follower).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesRequest {
    /// Leader's current term.
    pub term: u64,
    /// Leader's node ID so followers can redirect clients.
    pub leader_id: u64,
    /// Index of log entry immediately preceding the new ones.
    pub prev_log_index: u64,
    /// Term of the log entry at `prev_log_index`.
    pub prev_log_term: u64,
    /// Log entries to store (empty for heartbeat).
    pub entries: Vec<LogEntry>,
    /// Leader's commit index.
    pub leader_commit: u64,
}

/// AppendEntries RPC response (follower -> leader).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesResponse {
    /// Follower's current term (for leader to update itself).
    pub term: u64,
    /// True if the follower contained the matching entry at `prev_log_index`.
    pub success: bool,
    /// Highest log index the follower now has (optimisation for leader).
    pub match_index: u64,
}

/// RequestVote RPC request (candidate -> voter).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVoteRequest {
    /// Candidate's current term.
    pub term: u64,
    /// Candidate's node ID.
    pub candidate_id: u64,
    /// Index of candidate's last log entry.
    pub last_log_index: u64,
    /// Term of candidate's last log entry.
    pub last_log_term: u64,
}

/// RequestVote RPC response (voter -> candidate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVoteResponse {
    /// Responder's current term (for candidate to update itself).
    pub term: u64,
    /// True means the candidate received the vote.
    pub vote_granted: bool,
}

// ── Transport Trait ─────────────────────────────────────────────────────────

/// Abstraction over the network layer used by the Raft protocol.
///
/// Implement this trait with real TCP/gRPC connections in production and with
/// in-memory channels in tests.
#[async_trait]
pub trait RpcTransport: Send + Sync {
    /// Send an AppendEntries RPC to `peer_id`.
    async fn send_append_entries(
        &self,
        peer_id: u64,
        req: AppendEntriesRequest,
    ) -> Result<AppendEntriesResponse>;

    /// Send a RequestVote RPC to `peer_id`.
    async fn send_request_vote(
        &self,
        peer_id: u64,
        req: RequestVoteRequest,
    ) -> Result<RequestVoteResponse>;
}

/// A transport that always returns an error.  Useful as a default when the
/// network layer has not been wired up yet.
pub struct NullTransport;

#[async_trait]
impl RpcTransport for NullTransport {
    async fn send_append_entries(
        &self,
        _peer_id: u64,
        _req: AppendEntriesRequest,
    ) -> Result<AppendEntriesResponse> {
        Err(MqError::Network(
            "NullTransport: no transport configured".into(),
        ))
    }

    async fn send_request_vote(
        &self,
        _peer_id: u64,
        _req: RequestVoteRequest,
    ) -> Result<RequestVoteResponse> {
        Err(MqError::Network(
            "NullTransport: no transport configured".into(),
        ))
    }
}

// ── Raft Node ───────────────────────────────────────────────────────────────

/// The core Raft node holding all protocol state.
///
/// Fields are `pub` so that orchestration layers ([`super::LeaderElection`],
/// [`super::LogReplication`]) can inspect and drive the state machine.
/// External callers should go through the async handler methods which enforce
/// the invariants described in the Raft paper (Sections 5.1-5.4).
pub struct RaftNode {
    // ── Persistent state (survive crashes in a real implementation) ────────
    /// Unique identifier for this node.
    pub id: u64,
    /// Latest term this node has seen (monotonically increasing).
    pub current_term: u64,
    /// ID of the node this follower voted for in the current term, if any.
    pub voted_for: Option<u64>,
    /// Append-only log of commands.
    pub log: Vec<LogEntry>,

    // ── Volatile state ─────────────────────────────────────────────────────
    /// Current role of this node.
    pub state: RaftState,
    /// Highest log index known to be committed (applied to all nodes).
    pub commit_index: u64,
    /// Highest log index applied to the local state machine.
    pub last_applied: u64,
    /// ID of the current leader, if known.
    pub leader_id: Option<u64>,

    // ── Leader-only volatile state ─────────────────────────────────────────
    /// For each peer, the next log index to send.
    pub next_index: HashMap<u64, u64>,
    /// For each peer, the highest log index known to be replicated.
    pub match_index: HashMap<u64, u64>,

    // ── Configuration ──────────────────────────────────────────────────────
    /// Node IDs of all peers (excluding self).
    pub peers: Vec<u64>,
    /// Election timeout in milliseconds (randomised at runtime).
    pub election_timeout_ms: u64,
    /// Heartbeat interval in milliseconds.
    pub heartbeat_interval_ms: u64,

    // ── I/O ────────────────────────────────────────────────────────────────
    /// Network transport used for RPCs.
    pub transport: Arc<dyn RpcTransport>,

    /// Optional callback invoked for every committed log entry.  The callback
    /// receives a reference to the entry and is expected to be fast (non-blocking).
    pub apply_fn: Option<Box<dyn Fn(&LogEntry) + Send + Sync>>,
}

impl RaftNode {
    /// Create a new Raft node in the `Follower` state with term 0.
    pub fn new(
        id: u64,
        peers: Vec<u64>,
        election_timeout_ms: u64,
        heartbeat_interval_ms: u64,
    ) -> Self {
        Self {
            id,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            state: RaftState::Follower,
            commit_index: 0,
            last_applied: 0,
            leader_id: None,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            peers,
            election_timeout_ms,
            heartbeat_interval_ms,
            transport: Arc::new(NullTransport),
            apply_fn: None,
        }
    }

    /// Builder-style setter for the network transport.
    pub fn with_transport(mut self, transport: Arc<dyn RpcTransport>) -> Self {
        self.transport = transport;
        self
    }

    /// Builder-style setter for the state-machine apply callback.
    pub fn with_apply_fn(
        mut self,
        f: impl Fn(&LogEntry) + Send + Sync + 'static,
    ) -> Self {
        self.apply_fn = Some(Box::new(f));
        self
    }

    // ── Convenience queries ────────────────────────────────────────────────

    /// Returns `true` when this node is the cluster leader.
    pub fn is_leader(&self) -> bool {
        self.state == RaftState::Leader
    }

    /// Index of the last log entry, or 0 when the log is empty.
    pub fn last_log_index(&self) -> u64 {
        self.log.last().map_or(0, |e| e.index)
    }

    /// Term of the last log entry, or 0 when the log is empty.
    pub fn last_log_term(&self) -> u64 {
        self.log.last().map_or(0, |e| e.term)
    }

    /// Return the term of the entry at `index`, or 0 if no such entry exists.
    /// Index 0 (before the start of the log) always returns term 0.
    pub fn term_at(&self, index: u64) -> u64 {
        if index == 0 {
            return 0;
        }
        self.log
            .iter()
            .find(|e| e.index == index)
            .map_or(0, |e| e.term)
    }

    // ── State transitions (internal) ───────────────────────────────────────

    /// Transition to candidate state, incrementing the term and voting for self.
    pub fn become_candidate(&mut self) {
        self.current_term += 1;
        self.state = RaftState::Candidate;
        self.voted_for = Some(self.id);
        self.leader_id = None;
    }

    /// Transition to leader state, initialising next_index / match_index.
    pub fn become_leader(&mut self) {
        self.state = RaftState::Leader;
        self.leader_id = Some(self.id);
        let last_idx = self.last_log_index();
        for &peer in &self.peers {
            self.next_index.insert(peer, last_idx + 1);
            self.match_index.insert(peer, 0);
        }
    }

    /// Step down to follower, resetting voted_for and updating the term.
    pub fn become_follower(&mut self, term: u64, leader_id: Option<u64>) {
        self.state = RaftState::Follower;
        self.current_term = term;
        self.voted_for = None;
        self.leader_id = leader_id;
    }

    // ── RPC Handlers ───────────────────────────────────────────────────────

    /// Handle an incoming **AppendEntries** RPC from the leader.
    ///
    /// Implements Raft paper Section 5.3 (log consistency check) and
    /// Section 5.4.2 (commit index advancement).
    pub async fn handle_append_entries(
        &mut self,
        req: AppendEntriesRequest,
    ) -> AppendEntriesResponse {
        // 1. Reply false if term < currentTerm (§5.1).
        if req.term < self.current_term {
            debug!(
                node_id = self.id,
                req_term = req.term,
                current_term = self.current_term,
                "Rejecting AppendEntries: stale term"
            );
            return AppendEntriesResponse {
                term: self.current_term,
                success: false,
                match_index: 0,
            };
        }

        // Accept the leader's term.
        if req.term > self.current_term {
            self.become_follower(req.term, Some(req.leader_id));
        } else {
            // Same term -- just acknowledge the leader.
            self.leader_id = Some(req.leader_id);
            // If we are a candidate in this term and see a valid AppendEntries,
            // the leader already won the election; step down.
            if self.state == RaftState::Candidate {
                self.state = RaftState::Follower;
            }
        }

        // 2. Reply false if log does not contain an entry at prevLogIndex
        //    whose term matches prevLogTerm (§5.3).
        if req.prev_log_index > 0 {
            match self.log.iter().find(|e| e.index == req.prev_log_index) {
                Some(entry) if entry.term != req.prev_log_term => {
                    // Delete the conflicting entry and all that follow.
                    self.log.retain(|e| e.index < req.prev_log_index);
                    debug!(
                        node_id = self.id,
                        prev_log_index = req.prev_log_index,
                        "Rejecting AppendEntries: term mismatch at prevLogIndex"
                    );
                    return AppendEntriesResponse {
                        term: self.current_term,
                        success: false,
                        match_index: 0,
                    };
                }
                None => {
                    debug!(
                        node_id = self.id,
                        prev_log_index = req.prev_log_index,
                        "Rejecting AppendEntries: no entry at prevLogIndex"
                    );
                    return AppendEntriesResponse {
                        term: self.current_term,
                        success: false,
                        match_index: 0,
                    };
                }
                _ => { /* Entry exists with matching term -- proceed. */ }
            }
        }

        // 3. Append any new entries not already in the log, deleting
        //    conflicting entries first (§5.3).
        for entry in &req.entries {
            if let Some(pos) = self.log.iter().position(|e| e.index == entry.index) {
                if self.log[pos].term != entry.term {
                    // Truncate from the conflicting position onward and append.
                    self.log.truncate(pos);
                    self.log.push(entry.clone());
                }
                // Term matches -- entry is already present, skip.
            } else {
                self.log.push(entry.clone());
            }
        }

        // 4. If leaderCommit > commitIndex, set commitIndex to the minimum
        //    of leaderCommit and the index of the last new entry (§5.4.2).
        if req.leader_commit > self.commit_index {
            let new_commit = req.leader_commit.min(self.last_log_index());
            self.commit_index = new_commit;
            self.apply_committed();
        }

        let match_index = self.last_log_index();
        debug!(
            node_id = self.id,
            term = self.current_term,
            match_index,
            "AppendEntries accepted"
        );

        AppendEntriesResponse {
            term: self.current_term,
            success: true,
            match_index,
        }
    }

    /// Handle an incoming **RequestVote** RPC from a candidate.
    ///
    /// Implements Raft paper Section 5.2 (vote granting rules) and
    /// Section 5.4.1 (up-to-date log check).
    pub async fn handle_request_vote(
        &mut self,
        req: RequestVoteRequest,
    ) -> RequestVoteResponse {
        // 1. Reply false if term < currentTerm (§5.1).
        if req.term < self.current_term {
            debug!(
                node_id = self.id,
                req_term = req.term,
                current_term = self.current_term,
                "Rejecting vote: stale term"
            );
            return RequestVoteResponse {
                term: self.current_term,
                vote_granted: false,
            };
        }

        // If the candidate's term is higher, update our own term and step down.
        if req.term > self.current_term {
            self.become_follower(req.term, None);
        }

        // 2. Check log up-to-dateness (§5.4.1).
        //    The candidate's log is "at least as up-to-date" if its last entry
        //    has a strictly higher term, or the same term but an equal or
        //    greater index.
        let candidate_log_is_up_to_date = req.last_log_term > self.last_log_term()
            || (req.last_log_term == self.last_log_term()
                && req.last_log_index >= self.last_log_index());

        // 3. Grant vote if the log is up-to-date AND we haven't already voted
        //    for a different candidate in this term (§5.2).
        let vote_granted = candidate_log_is_up_to_date
            && (self.voted_for.is_none() || self.voted_for == Some(req.candidate_id));

        if vote_granted {
            self.voted_for = Some(req.candidate_id);
            info!(
                node_id = self.id,
                candidate = req.candidate_id,
                term = req.term,
                "Granted vote"
            );
        } else {
            debug!(
                node_id = self.id,
                candidate = req.candidate_id,
                term = req.term,
                voted_for = ?self.voted_for,
                log_up_to_date = candidate_log_is_up_to_date,
                "Rejected vote"
            );
        }

        RequestVoteResponse {
            term: self.current_term,
            vote_granted,
        }
    }

    // ── Election ───────────────────────────────────────────────────────────

    /// Start an election: become candidate, request votes from all peers, and
    /// either become leader or stay candidate.
    ///
    /// RPCs are fired concurrently through [`tokio::spawn`] so the `&mut self`
    /// borrow is not held across `.await` points.
    pub async fn start_election(&mut self) -> Result<()> {
        self.become_candidate();

        let term = self.current_term;
        let candidate_id = self.id;
        let last_log_index = self.last_log_index();
        let last_log_term = self.last_log_term();
        let peers = self.peers.clone();

        info!(node_id = self.id, term, "Starting election");

        let request = RequestVoteRequest {
            term,
            candidate_id,
            last_log_index,
            last_log_term,
        };

        // Fire RPCs concurrently (transport is behind Arc, no &mut self needed).
        let mut handles = Vec::with_capacity(peers.len());
        for &peer_id in &peers {
            let req = request.clone();
            let transport = Arc::clone(&self.transport);
            handles.push(tokio::spawn(async move {
                transport.send_request_vote(peer_id, req).await
            }));
        }

        let mut votes: u64 = 1; // Vote for self.
        let majority = (peers.len() as u64) / 2 + 1;

        for handle in handles {
            match handle.await {
                Ok(Ok(resp)) => {
                    if resp.term > self.current_term {
                        // Another node has a higher term -- step down.
                        self.become_follower(resp.term, None);
                        return Ok(());
                    }
                    if resp.vote_granted {
                        votes += 1;
                    }
                }
                Ok(Err(e)) => {
                    warn!(node_id = self.id, error = %e, "RequestVote RPC failed");
                }
                Err(e) => {
                    warn!(node_id = self.id, error = %e, "RequestVote task panicked");
                }
            }
        }

        if votes >= majority && self.state == RaftState::Candidate {
            self.become_leader();
            // Emit a NoOp entry so the new leader can establish its commit index
            // without serving a client request (Raft论文 Section 8).
            let noop = LogEntry {
                term: self.current_term,
                index: self.last_log_index() + 1,
                command: Command::NoOp,
            };
            self.log.push(noop);
            info!(
                node_id = self.id,
                term = self.current_term,
                votes,
                "Won election, became leader"
            );
        } else if self.state == RaftState::Candidate {
            debug!(
                node_id = self.id,
                votes,
                majority,
                "Lost election"
            );
        }

        Ok(())
    }

    // ── Proposal ───────────────────────────────────────────────────────────

    /// Propose a new command.  Only the leader may propose.
    ///
    /// The entry is appended to the local log and replicated to all peers
    /// concurrently.  Returns the log index assigned to the entry.
    pub async fn propose(&mut self, command: Command) -> Result<u64> {
        if self.state != RaftState::Leader {
            return Err(MqError::NotLeader {
                leader_id: self.leader_id,
            });
        }

        let index = self.last_log_index() + 1;
        let entry = LogEntry {
            term: self.current_term,
            index,
            command,
        };
        self.log.push(entry);

        // Replicate to all peers.
        self.replicate_to_all_peers().await?;

        // Attempt to advance the commit index.
        self.advance_commit_index();
        self.apply_committed();

        Ok(index)
    }

    /// Build and send AppendEntries RPCs to every peer concurrently.
    async fn replicate_to_all_peers(&mut self) -> Result<()> {
        if self.state != RaftState::Leader {
            return Ok(());
        }

        let peers = self.peers.clone();
        let mut handles = Vec::with_capacity(peers.len());

        for &peer_id in &peers {
            let next_idx = self.next_index.get(&peer_id).copied().unwrap_or(1);
            let prev_log_index = next_idx.saturating_sub(1);
            let prev_log_term = self.term_at(prev_log_index);
            let entries: Vec<LogEntry> = self
                .log
                .iter()
                .filter(|e| e.index >= next_idx)
                .cloned()
                .collect();
            let leader_commit = self.commit_index;
            let transport = Arc::clone(&self.transport);

            let req = AppendEntriesRequest {
                term: self.current_term,
                leader_id: self.id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit,
            };

            handles.push((
                peer_id,
                tokio::spawn(async move {
                    transport.send_append_entries(peer_id, req).await
                }),
            ));
        }

        for (peer_id, handle) in handles {
            match handle.await {
                Ok(Ok(resp)) => {
                    if resp.term > self.current_term {
                        self.become_follower(resp.term, None);
                        return Ok(());
                    }
                    if resp.success {
                        self.next_index
                            .insert(peer_id, resp.match_index + 1);
                        self.match_index.insert(peer_id, resp.match_index);
                    } else {
                        // Decrement nextIndex for this peer; the next call will
                        // retry with fewer entries.
                        let next = self.next_index.get(&peer_id).copied().unwrap_or(1);
                        self.next_index
                            .insert(peer_id, next.saturating_sub(1));
                    }
                }
                Ok(Err(e)) => {
                    warn!(
                        node_id = self.id,
                        peer = peer_id,
                        error = %e,
                        "AppendEntries RPC failed"
                    );
                }
                Err(e) => {
                    warn!(
                        node_id = self.id,
                        peer = peer_id,
                        error = %e,
                        "AppendEntries task panicked"
                    );
                }
            }
        }

        Ok(())
    }

    // ── Commit Index ───────────────────────────────────────────────────────

    /// Advance `commit_index` to the highest index that is replicated on a
    /// majority of nodes **and** belongs to the current term (§5.4.2).
    pub fn advance_commit_index(&mut self) {
        if self.state != RaftState::Leader {
            return;
        }

        // Collect every matchIndex plus the leader's own last_log_index.
        let mut indices: Vec<u64> = self.match_index.values().copied().collect();
        indices.push(self.last_log_index());
        // Sort descending so we check the highest possible commit index first.
        indices.sort_unstable_by(|a, b| b.cmp(a));

        let majority = (self.peers.len() as u64) / 2 + 1;

        for &n in &indices {
            if n <= self.commit_index {
                break;
            }
            let replicated_count = indices.iter().filter(|&&idx| idx >= n).count() as u64;
            if replicated_count >= majority {
                // Only commit entries from the current term (§5.4.2).
                if let Some(entry) = self.log.iter().find(|e| e.index == n) {
                    if entry.term == self.current_term {
                        self.commit_index = n;
                        debug!(
                            node_id = self.id,
                            commit_index = n,
                            "Advanced commit index"
                        );
                        break;
                    }
                }
            }
        }
    }

    // ── Apply Pipeline ─────────────────────────────────────────────────────

    /// Apply all committed-but-not-yet-applied entries to the state machine.
    pub fn apply_committed(&mut self) {
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            if let Some(entry) = self.log.iter().find(|e| e.index == self.last_applied) {
                if let Some(ref apply_fn) = self.apply_fn {
                    apply_fn(entry);
                }
            }
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: u64, peers: Vec<u64>) -> RaftNode {
        RaftNode::new(id, peers, 1500, 300)
    }

    fn push_entries(node: &mut RaftNode, entries: &[(u64, u64)]) {
        // (term, index) pairs.
        for &(term, index) in entries {
            node.log.push(LogEntry {
                term,
                index,
                command: Command::NoOp,
            });
        }
    }

    // ── Constructor & basic queries ────────────────────────────────────────

    #[test]
    fn new_node_defaults() {
        let node = make_node(1, vec![2, 3]);
        assert_eq!(node.id, 1);
        assert_eq!(node.current_term, 0);
        assert!(node.voted_for.is_none());
        assert!(node.log.is_empty());
        assert_eq!(node.state, RaftState::Follower);
        assert_eq!(node.commit_index, 0);
        assert_eq!(node.last_applied, 0);
        assert!(!node.is_leader());
        assert_eq!(node.peers, vec![2, 3]);
    }

    #[test]
    fn last_log_on_empty_log() {
        let node = make_node(1, vec![]);
        assert_eq!(node.last_log_index(), 0);
        assert_eq!(node.last_log_term(), 0);
    }

    #[test]
    fn last_log_with_entries() {
        let mut node = make_node(1, vec![]);
        push_entries(&mut node, &[(1, 1), (1, 2), (2, 3)]);
        assert_eq!(node.last_log_index(), 3);
        assert_eq!(node.last_log_term(), 2);
    }

    #[test]
    fn term_at_existing_indices() {
        let mut node = make_node(1, vec![]);
        push_entries(&mut node, &[(1, 1), (2, 2)]);
        assert_eq!(node.term_at(0), 0);
        assert_eq!(node.term_at(1), 1);
        assert_eq!(node.term_at(2), 2);
        assert_eq!(node.term_at(3), 0);
    }

    // ── State transitions ──────────────────────────────────────────────────

    #[test]
    fn become_candidate_increments_term() {
        let mut node = make_node(1, vec![2, 3]);
        node.become_candidate();
        assert_eq!(node.current_term, 1);
        assert_eq!(node.state, RaftState::Candidate);
        assert_eq!(node.voted_for, Some(1));
        assert!(node.leader_id.is_none());
    }

    #[test]
    fn become_leader_initializes_indices() {
        let mut node = make_node(1, vec![2, 3]);
        push_entries(&mut node, &[(1, 1)]);
        node.become_candidate();
        node.become_leader();
        assert_eq!(node.state, RaftState::Leader);
        assert_eq!(node.leader_id, Some(1));
        assert_eq!(node.next_index.get(&2), Some(&2));
        assert_eq!(node.next_index.get(&3), Some(&2));
        assert_eq!(node.match_index.get(&2), Some(&0));
        assert_eq!(node.match_index.get(&3), Some(&0));
    }

    #[test]
    fn become_follower_resets_state() {
        let mut node = make_node(1, vec![2, 3]);
        node.become_candidate();
        node.become_leader();
        node.become_follower(5, Some(2));
        assert_eq!(node.state, RaftState::Follower);
        assert_eq!(node.current_term, 5);
        assert!(node.voted_for.is_none());
        assert_eq!(node.leader_id, Some(2));
    }

    // ── handle_append_entries ──────────────────────────────────────────────

    #[tokio::test]
    async fn append_entries_rejects_stale_term() {
        let mut node = make_node(1, vec![]);
        node.current_term = 5;
        let resp = node
            .handle_append_entries(AppendEntriesRequest {
                term: 3,
                leader_id: 2,
                prev_log_index: 0,
                prev_log_term: 0,
                entries: vec![],
                leader_commit: 0,
            })
            .await;
        assert!(!resp.success);
        assert_eq!(resp.term, 5);
    }

    #[tokio::test]
    async fn append_entries_rejects_missing_prev_log() {
        let mut node = make_node(1, vec![]);
        node.current_term = 1;
        let resp = node
            .handle_append_entries(AppendEntriesRequest {
                term: 1,
                leader_id: 2,
                prev_log_index: 5,
                prev_log_term: 1,
                entries: vec![],
                leader_commit: 0,
            })
            .await;
        assert!(!resp.success);
    }

    #[tokio::test]
    async fn append_entries_rejects_term_mismatch_at_prev_log() {
        let mut node = make_node(1, vec![]);
        push_entries(&mut node, &[(1, 1), (1, 2)]);
        let resp = node
            .handle_append_entries(AppendEntriesRequest {
                term: 1,
                leader_id: 2,
                prev_log_index: 2,
                prev_log_term: 2, // Mismatch: entry at idx 2 has term 1.
                entries: vec![],
                leader_commit: 0,
            })
            .await;
        assert!(!resp.success);
        // The conflicting entry (index 2) and anything after should be deleted.
        // Entry at index 1 remains.
        assert_eq!(node.log.len(), 1);
        assert_eq!(node.log[0].index, 1);
    }

    #[tokio::test]
    async fn append_entries_appends_new_entries() {
        let mut node = make_node(1, vec![]);
        let entry = LogEntry {
            term: 1,
            index: 1,
            command: Command::NoOp,
        };
        let resp = node
            .handle_append_entries(AppendEntriesRequest {
                term: 1,
                leader_id: 2,
                prev_log_index: 0,
                prev_log_term: 0,
                entries: vec![entry],
                leader_commit: 0,
            })
            .await;
        assert!(resp.success);
        assert_eq!(node.log.len(), 1);
        assert_eq!(resp.match_index, 1);
    }

    #[tokio::test]
    async fn append_entries_skips_existing_matching_entries() {
        let mut node = make_node(1, vec![]);
        push_entries(&mut node, &[(1, 1)]);

        let resp = node
            .handle_append_entries(AppendEntriesRequest {
                term: 1,
                leader_id: 2,
                prev_log_index: 1,
                prev_log_term: 1,
                entries: vec![LogEntry {
                    term: 1,
                    index: 1,
                    command: Command::NoOp,
                }],
                leader_commit: 0,
            })
            .await;
        assert!(resp.success);
        assert_eq!(node.log.len(), 1); // Not duplicated.
    }

    #[tokio::test]
    async fn append_entries_truncates_conflict_and_appends() {
        let mut node = make_node(1, vec![]);
        push_entries(&mut node, &[(1, 1), (1, 2), (2, 3)]);

        // Leader sends a different entry at index 2 (term 3 instead of 1).
        let new_entry = LogEntry {
            term: 3,
            index: 2,
            command: Command::NoOp,
        };
        let resp = node
            .handle_append_entries(AppendEntriesRequest {
                term: 3,
                leader_id: 2,
                prev_log_index: 1,
                prev_log_term: 1,
                entries: vec![new_entry],
                leader_commit: 0,
            })
            .await;
        assert!(resp.success);
        // After truncation: [term=1,idx=1] then [term=3,idx=2]
        assert_eq!(node.log.len(), 2);
        assert_eq!(node.log[0].term, 1);
        assert_eq!(node.log[0].index, 1);
        assert_eq!(node.log[1].term, 3);
        assert_eq!(node.log[1].index, 2);
    }

    #[tokio::test]
    async fn append_entries_updates_commit_index() {
        let mut node = make_node(1, vec![]);
        let entry = LogEntry {
            term: 1,
            index: 1,
            command: Command::NoOp,
        };
        let resp = node
            .handle_append_entries(AppendEntriesRequest {
                term: 1,
                leader_id: 2,
                prev_log_index: 0,
                prev_log_term: 0,
                entries: vec![entry],
                leader_commit: 1,
            })
            .await;
        assert!(resp.success);
        assert_eq!(node.commit_index, 1);
        assert_eq!(node.last_applied, 1);
    }

    #[tokio::test]
    async fn append_entries_commit_clamped_to_log_length() {
        let mut node = make_node(1, vec![]);
        let entry = LogEntry {
            term: 1,
            index: 1,
            command: Command::NoOp,
        };
        node.handle_append_entries(AppendEntriesRequest {
            term: 1,
            leader_id: 2,
            prev_log_index: 0,
            prev_log_term: 0,
            entries: vec![entry],
            leader_commit: 100, // Way beyond our log.
        })
        .await;
        assert_eq!(node.commit_index, 1); // Clamped to last_log_index.
    }

    #[tokio::test]
    async fn append_entries_candidate_steps_down_on_valid_leader() {
        let mut node = make_node(1, vec![]);
        node.current_term = 1;
        node.state = RaftState::Candidate;

        node.handle_append_entries(AppendEntriesRequest {
            term: 1,
            leader_id: 2,
            prev_log_index: 0,
            prev_log_term: 0,
            entries: vec![],
            leader_commit: 0,
        })
        .await;

        assert_eq!(node.state, RaftState::Follower);
        assert_eq!(node.leader_id, Some(2));
    }

    // ── handle_request_vote ────────────────────────────────────────────────

    #[tokio::test]
    async fn request_vote_rejects_stale_term() {
        let mut node = make_node(1, vec![]);
        node.current_term = 5;
        let resp = node
            .handle_request_vote(RequestVoteRequest {
                term: 3,
                candidate_id: 2,
                last_log_index: 10,
                last_log_term: 5,
            })
            .await;
        assert!(!resp.vote_granted);
        assert_eq!(resp.term, 5);
    }

    #[tokio::test]
    async fn request_vote_grants_when_log_up_to_date() {
        let mut node = make_node(1, vec![]);
        let resp = node
            .handle_request_vote(RequestVoteRequest {
                term: 1,
                candidate_id: 2,
                last_log_index: 0,
                last_log_term: 0,
            })
            .await;
        assert!(resp.vote_granted);
        assert_eq!(node.voted_for, Some(2));
    }

    #[tokio::test]
    async fn request_vote_rejects_when_log_is_behind() {
        let mut node = make_node(1, vec![]);
        push_entries(&mut node, &[(1, 1), (2, 2)]);

        let resp = node
            .handle_request_vote(RequestVoteRequest {
                term: 1,
                candidate_id: 2,
                last_log_index: 1,
                last_log_term: 1,
            })
            .await;
        assert!(!resp.vote_granted);
    }

    #[tokio::test]
    async fn request_vote_rejects_when_voted_for_another() {
        let mut node = make_node(1, vec![]);
        node.handle_request_vote(RequestVoteRequest {
            term: 1,
            candidate_id: 2,
            last_log_index: 0,
            last_log_term: 0,
        })
        .await;

        let resp = node
            .handle_request_vote(RequestVoteRequest {
                term: 1,
                candidate_id: 3,
                last_log_index: 0,
                last_log_term: 0,
            })
            .await;
        assert!(!resp.vote_granted);
    }

    #[tokio::test]
    async fn request_vote_grants_when_same_candidate_retries() {
        let mut node = make_node(1, vec![]);
        node.handle_request_vote(RequestVoteRequest {
            term: 1,
            candidate_id: 2,
            last_log_index: 0,
            last_log_term: 0,
        })
        .await;

        let resp = node
            .handle_request_vote(RequestVoteRequest {
                term: 1,
                candidate_id: 2,
                last_log_index: 0,
                last_log_term: 0,
            })
            .await;
        assert!(resp.vote_granted);
    }

    #[tokio::test]
    async fn request_vote_steps_down_on_higher_term() {
        let mut node = make_node(1, vec![]);
        node.become_candidate(); // term = 1

        let resp = node
            .handle_request_vote(RequestVoteRequest {
                term: 5,
                candidate_id: 2,
                last_log_index: 0,
                last_log_term: 0,
            })
            .await;
        assert!(resp.vote_granted);
        assert_eq!(node.state, RaftState::Follower);
        assert_eq!(node.current_term, 5);
    }

    #[tokio::test]
    async fn request_vote_equal_log_term_higher_index_wins() {
        let mut node = make_node(1, vec![]);
        push_entries(&mut node, &[(1, 1)]);

        // Candidate has same term but higher index -- should be granted.
        let resp = node
            .handle_request_vote(RequestVoteRequest {
                term: 1,
                candidate_id: 2,
                last_log_index: 2,
                last_log_term: 1,
            })
            .await;
        assert!(resp.vote_granted);
    }

    // ── Propose ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn propose_rejects_non_leader() {
        let mut node = make_node(1, vec![]);
        let result = node.propose(Command::NoOp).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn propose_appends_to_log_as_leader() {
        let mut node = make_node(1, vec![]);
        node.become_candidate();
        node.become_leader();

        let idx = node
            .propose(Command::CreateTopic {
                name: "orders".into(),
                partitions: 3,
            })
            .await
            .unwrap();
        assert_eq!(idx, 1);
        assert_eq!(node.log.len(), 1);
        assert_eq!(
            node.log[0].command,
            Command::CreateTopic {
                name: "orders".into(),
                partitions: 3,
            }
        );
    }

    #[tokio::test]
    async fn propose_assigns_sequential_indices() {
        let mut node = make_node(1, vec![]);
        node.become_candidate();
        node.become_leader();

        let idx1 = node.propose(Command::NoOp).await.unwrap();
        let idx2 = node.propose(Command::NoOp).await.unwrap();
        let idx3 = node.propose(Command::NoOp).await.unwrap();
        assert_eq!(idx1, 1);
        assert_eq!(idx2, 2);
        assert_eq!(idx3, 3);
    }

    // ── advance_commit_index ───────────────────────────────────────────────

    #[test]
    fn advance_commit_majority_replicated() {
        let mut node = make_node(1, vec![2, 3]);
        node.state = RaftState::Leader;
        node.current_term = 1;
        push_entries(&mut node, &[(1, 1), (1, 2), (1, 3)]);

        // Peers 2 and 3 have replicated up to index 2.
        node.match_index.insert(2, 2);
        node.match_index.insert(3, 2);

        node.advance_commit_index();
        // Indices: [2, 2, 3] sorted desc = [3, 2, 2].
        // index 3: only 1 node (leader) -- not majority.
        // index 2: all 3 nodes -- majority, term matches.
        assert_eq!(node.commit_index, 2);
    }

    #[test]
    fn advance_commit_only_current_term_entries() {
        let mut node = make_node(1, vec![2, 3]);
        node.state = RaftState::Leader;
        node.current_term = 2;
        node.log.push(LogEntry {
            term: 1,
            index: 1,
            command: Command::NoOp,
        });
        node.log.push(LogEntry {
            term: 2,
            index: 2,
            command: Command::NoOp,
        });

        node.match_index.insert(2, 1);
        node.match_index.insert(3, 1);

        node.advance_commit_index();
        // All nodes have index 1, but it is term 1 (not current term 2).
        // commit_index stays 0.
        assert_eq!(node.commit_index, 0);
    }

    #[test]
    fn advance_commit_ignores_when_not_leader() {
        let mut node = make_node(1, vec![2]);
        node.state = RaftState::Follower;
        node.commit_index = 0;
        node.advance_commit_index();
        assert_eq!(node.commit_index, 0);
    }

    #[test]
    fn advance_commit_no_progress_without_majority() {
        let mut node = make_node(1, vec![2, 3]);
        node.state = RaftState::Leader;
        node.current_term = 1;
        push_entries(&mut node, &[(1, 1), (1, 2), (1, 3)]);

        // Only one peer replicated up to index 3.
        node.match_index.insert(2, 3);
        node.match_index.insert(3, 0);

        node.advance_commit_index();
        // Indices: [0, 3, 3] sorted desc = [3, 3, 0].
        // index 3: 2 nodes >= 3, but majority needs 2 (cluster of 3). So 2 >= 2 = majority.
        // Wait -- cluster_size = 3 nodes (self + 2 peers), majority = 3/2+1 = 2.
        // So index 3 IS majority (leader has 3, peer 2 has 3). Should advance.
        assert_eq!(node.commit_index, 3);
    }

    // ── apply_committed ────────────────────────────────────────────────────

    #[test]
    fn apply_committed_invokes_callback() {
        use std::sync::Arc;

        let mut node = make_node(1, vec![]);
        push_entries(&mut node, &[(1, 1), (1, 2)]);
        node.commit_index = 2;

        let applied = Arc::new(std::sync::Mutex::new(Vec::new()));
        let applied_clone = Arc::clone(&applied);
        node.apply_fn = Some(Box::new(move |entry: &LogEntry| {
            applied_clone.lock().unwrap().push(entry.index);
        }));

        node.apply_committed();
        assert_eq!(node.last_applied, 2);
        assert_eq!(*applied.lock().unwrap(), vec![1, 2]);
    }

    #[test]
    fn apply_committed_is_idempotent() {
        let mut node = make_node(1, vec![]);
        push_entries(&mut node, &[(1, 1)]);
        node.commit_index = 1;
        node.apply_fn = Some(Box::new(|_| {}));

        node.apply_committed();
        assert_eq!(node.last_applied, 1);

        // Second call should be a no-op.
        node.apply_committed();
        assert_eq!(node.last_applied, 1);
    }

    #[test]
    fn apply_committed_without_callback_is_safe() {
        let mut node = make_node(1, vec![]);
        push_entries(&mut node, &[(1, 1), (1, 2), (1, 3)]);
        node.commit_index = 3;
        // No apply_fn set.
        node.apply_committed();
        assert_eq!(node.last_applied, 3);
    }
}
