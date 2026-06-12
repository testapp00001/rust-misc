//! Leader election orchestration.
//!
//! [`LeaderElection`] wraps a shared [`RaftNode`] and drives the election
//! protocol: it increments the term, sends `RequestVote` RPCs to every peer,
//! tallies votes, and transitions the node to leader when a quorum is reached.
//!
//! The module also provides an election-loop driver that sleeps for a
//! randomised timeout and triggers a new election when the timeout fires.

use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::Rng;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::error::Result;
use super::raft_node::{RaftNode, RaftState, RequestVoteRequest};

/// Drives the leader-election protocol on top of a [`RaftNode`].
pub struct LeaderElection {
    /// Shared Raft state.
    node: Arc<Mutex<RaftNode>>,
    /// Wall-clock time of the last heartbeat received from the current leader.
    last_heartbeat: std::sync::Mutex<Instant>,
}

impl LeaderElection {
    /// Wrap an existing Raft node for election orchestration.
    pub fn new(node: Arc<Mutex<RaftNode>>) -> Self {
        Self {
            node,
            last_heartbeat: std::sync::Mutex::new(Instant::now()),
        }
    }

    /// Run a single election round.
    ///
    /// This method:
    /// 1. Transitions the node to candidate state.
    /// 2. Sends `RequestVote` RPCs to every peer concurrently.
    /// 3. Tallies votes and either becomes leader or stays candidate.
    pub async fn run_election(&self) -> Result<()> {
        // ── 1. Become candidate ────────────────────────────────────────────
        {
            let mut node = self.node.lock().await;
            node.become_candidate();
        }

        // Snapshot the values we need for the RPCs (cheap clones).
        let (term, candidate_id, last_log_index, last_log_term, peers) = {
            let node = self.node.lock().await;
            (
                node.current_term,
                node.id,
                node.last_log_index(),
                node.last_log_term(),
                node.peers.clone(),
            )
        };

        info!(
            candidate = candidate_id,
            term,
            peer_count = peers.len(),
            "Starting election"
        );

        let request = RequestVoteRequest {
            term,
            candidate_id,
            last_log_index,
            last_log_term,
        };

        let majority = ((peers.len() + 1) / 2 + 1) as u64;

        // ── 2. Send RequestVote RPCs concurrently ──────────────────────────
        let mut handles = Vec::with_capacity(peers.len());
        for &peer_id in &peers {
            let req = request.clone();
            let node_ref = Arc::clone(&self.node);
            handles.push(tokio::spawn(async move {
                // Obtain a reference to the transport without holding the full
                // node lock for the duration of the RPC.
                let transport = {
                    let n = node_ref.lock().await;
                    Arc::clone(&n.transport)
                };
                transport.send_request_vote(peer_id, req).await
            }));
        }

        // ── 3. Tally votes ────────────────────────────────────────────────
        let mut votes: u64 = 1; // Vote for self.

        for handle in handles {
            match handle.await {
                Ok(Ok(resp)) => {
                    let mut node = self.node.lock().await;

                    if resp.term > node.current_term {
                        node.become_follower(resp.term, None);
                        info!(
                            node_id = node.id,
                            new_term = resp.term,
                            "Stepping down: higher term discovered during election"
                        );
                        return Ok(());
                    }

                    if resp.vote_granted {
                        votes += 1;
                    }
                }
                Ok(Err(e)) => {
                    warn!(peer = ?e, "RequestVote RPC failed");
                }
                Err(e) => {
                    warn!(error = %e, "RequestVote task panicked");
                }
            }
        }

        // ── 4. Decide outcome ──────────────────────────────────────────────
        let mut node = self.node.lock().await;

        if votes >= majority && node.state == RaftState::Candidate {
            node.become_leader();
            // Emit a NoOp so the new leader can commit without waiting for a
            // client request.
            let noop = super::raft_node::LogEntry {
                term: node.current_term,
                index: node.last_log_index() + 1,
                command: super::raft_node::Command::NoOp,
            };
            node.log.push(noop);
            info!(
                node_id = node.id,
                term = node.current_term,
                votes,
                majority,
                "Won election, became leader"
            );
        } else if node.state == RaftState::Candidate {
            debug!(
                node_id = node.id,
                votes,
                majority,
                "Lost election"
            );
        }

        Ok(())
    }

    // ── Heartbeat / timeout helpers ────────────────────────────────────────

    /// Record that a valid AppendEntries was received (resets the election
    /// timer).
    pub fn reset_heartbeat(&self) {
        if let Ok(mut last) = self.last_heartbeat.lock() {
            *last = Instant::now();
        }
    }

    /// Returns `true` if no heartbeat has been received within the
    /// randomised election timeout.
    pub fn check_election_timeout(&self) -> bool {
        let timeout_ms = match self.node.try_lock() {
            Ok(node) => node.election_timeout_ms,
            Err(_) => return false,
        };

        let elapsed = match self.last_heartbeat.lock() {
            Ok(last) => last.elapsed(),
            Err(_) => return false,
        };

        // Randomise: timeout is base + [0, 50%).
        let jitter_range = timeout_ms / 2;
        let jitter: u64 = rand::thread_rng().gen_range(0..=jitter_range);
        let total_timeout = Duration::from_millis(timeout_ms + jitter);

        elapsed >= total_timeout
    }

    /// Long-running election loop.
    ///
    /// Sleeps for a randomised election timeout, then triggers a new election
    /// if the node is not already the leader.  Stops when `shutdown` is set.
    pub async fn run_election_loop(
        &self,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) {
        loop {
            let timeout_ms = {
                let node = self.node.lock().await;
                node.election_timeout_ms
            };

            // Randomise the timeout to avoid persistent split votes.
            let jitter: u64 = rand::thread_rng().gen_range(0..=timeout_ms / 3);
            let timeout = Duration::from_millis(timeout_ms + jitter);

            tokio::select! {
                _ = tokio::time::sleep(timeout) => {}
                _ = shutdown.changed() => {
                    debug!("Election loop shutting down");
                    return;
                }
            }

            let should_elect = {
                let node = self.node.lock().await;
                !node.is_leader()
            };

            if should_elect {
                if let Err(e) = self.run_election().await {
                    warn!(error = %e, "Election round failed");
                }
            }
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::raft_node::*;
    use crate::error::MqError;

    // ── Mock transport ─────────────────────────────────────────────────────

    /// A configurable mock transport for unit tests.
    struct MockTransport {
        vote_responses: std::sync::Mutex<Vec<Result<RequestVoteResponse>>>,
    }

    impl MockTransport {
        fn with_responses(responses: Vec<Result<RequestVoteResponse>>) -> Self {
            Self {
                vote_responses: std::sync::Mutex::new(responses),
            }
        }
    }

    #[async_trait::async_trait]
    impl RpcTransport for MockTransport {
        async fn send_append_entries(
            &self,
            _peer_id: u64,
            _req: AppendEntriesRequest,
        ) -> Result<AppendEntriesResponse> {
            Ok(AppendEntriesResponse {
                term: 0,
                success: true,
                match_index: 0,
            })
        }

        async fn send_request_vote(
            &self,
            _peer_id: u64,
            _req: RequestVoteRequest,
        ) -> Result<RequestVoteResponse> {
            let mut queue = self.vote_responses.lock().unwrap();
            if queue.is_empty() {
                Ok(RequestVoteResponse {
                    term: 0,
                    vote_granted: false,
                })
            } else {
                queue.remove(0)
            }
        }
    }

    // ── Helpers ────────────────────────────────────────────────────────────

    fn make_node_with_transport(
        id: u64,
        peers: Vec<u64>,
        transport: Arc<dyn RpcTransport>,
    ) -> Arc<Mutex<RaftNode>> {
        Arc::new(Mutex::new(
            RaftNode::new(id, peers, 1500, 300).with_transport(transport),
        ))
    }

    // ── Tests ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn election_wins_with_majority() {
        let node = make_node_with_transport(
            1,
            vec![2, 3],
            Arc::new(MockTransport::with_responses(vec![
                Ok(RequestVoteResponse {
                    term: 1,
                    vote_granted: true,
                }),
                Ok(RequestVoteResponse {
                    term: 1,
                    vote_granted: true,
                }),
            ])),
        );

        let election = LeaderElection::new(node.clone());
        election.run_election().await.unwrap();

        let node = node.lock().await;
        assert_eq!(node.state, RaftState::Leader);
        assert_eq!(node.current_term, 1);
        assert_eq!(node.leader_id, Some(1));
    }

    #[tokio::test]
    async fn election_loses_without_majority() {
        let node = make_node_with_transport(
            1,
            vec![2, 3, 4, 5],
            Arc::new(MockTransport::with_responses(vec![
                Ok(RequestVoteResponse {
                    term: 1,
                    vote_granted: true,
                }),
                Ok(RequestVoteResponse {
                    term: 1,
                    vote_granted: false,
                }),
                Ok(RequestVoteResponse {
                    term: 1,
                    vote_granted: false,
                }),
                Ok(RequestVoteResponse {
                    term: 1,
                    vote_granted: false,
                }),
            ])),
        );

        let election = LeaderElection::new(node.clone());
        election.run_election().await.unwrap();

        let node = node.lock().await;
        assert_eq!(node.state, RaftState::Candidate);
    }

    #[tokio::test]
    async fn election_steps_down_on_higher_term() {
        let node = make_node_with_transport(
            1,
            vec![2],
            Arc::new(MockTransport::with_responses(vec![
                Ok(RequestVoteResponse {
                    term: 5,
                    vote_granted: false,
                }),
            ])),
        );

        let election = LeaderElection::new(node.clone());
        election.run_election().await.unwrap();

        let node = node.lock().await;
        assert_eq!(node.state, RaftState::Follower);
        assert_eq!(node.current_term, 5);
    }

    #[tokio::test]
    async fn election_tolerates_rpc_failure() {
        let node = make_node_with_transport(
            1,
            vec![2, 3],
            Arc::new(MockTransport::with_responses(vec![
                Err(MqError::Network("timeout".into())),
                Ok(RequestVoteResponse {
                    term: 1,
                    vote_granted: true,
                }),
            ])),
        );

        let election = LeaderElection::new(node.clone());
        election.run_election().await.unwrap();

        // 1 self-vote + 1 from peer 3 = 2 votes; cluster of 3 needs 2.
        let node = node.lock().await;
        assert_eq!(node.state, RaftState::Leader);
    }

    #[tokio::test]
    async fn leader_emits_noop_after_election() {
        let node = make_node_with_transport(
            1,
            vec![2],
            Arc::new(MockTransport::with_responses(vec![
                Ok(RequestVoteResponse {
                    term: 1,
                    vote_granted: true,
                }),
            ])),
        );

        let election = LeaderElection::new(node.clone());
        election.run_election().await.unwrap();

        let node = node.lock().await;
        assert_eq!(node.log.len(), 1);
        assert_eq!(node.log[0].command, Command::NoOp);
        assert_eq!(node.log[0].term, 1);
    }

    #[test]
    fn heartbeat_reset() {
        let node = make_node_with_transport(
            1,
            vec![],
            Arc::new(MockTransport::with_responses(vec![])),
        );
        let election = LeaderElection::new(node);

        // Initially the timeout should not have fired.
        assert!(!election.check_election_timeout());

        // After reset, it should definitely not have fired.
        election.reset_heartbeat();
        assert!(!election.check_election_timeout());
    }

    #[test]
    fn election_timeout_returns_false_when_lock_unavailable() {
        let node = make_node_with_transport(
            1,
            vec![],
            Arc::new(MockTransport::with_responses(vec![])),
        );
        let election = LeaderElection::new(node);

        // Hold the node lock from another thread to simulate contention.
        let _guard = election.node.blocking_lock();

        // check_election_timeout uses try_lock -- should return false, not block.
        // Note: blocking_lock is only safe outside a tokio runtime.
        // We call this outside async context.
        assert!(!election.check_election_timeout());
    }
}
