//! Log replication orchestration.
//!
//! [`LogReplication`] wraps a shared [`RaftNode`] and handles the leader's
//! replication duties:
//!
//! - Sending `AppendEntries` RPCs to individual peers
//! - Retrying with decremented `nextIndex` on log inconsistencies
//! - Advancing the commit index once a quorum acknowledges an entry
//! - Driving the heartbeat loop that maintains leadership

use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::error::Result;
use super::raft_node::{
    AppendEntriesRequest, LogEntry, RaftNode, RaftState,
};

/// Drives log replication from the leader to its followers.
pub struct LogReplication {
    /// Shared Raft state.
    node: Arc<Mutex<RaftNode>>,
}

impl LogReplication {
    /// Wrap an existing Raft node for log-replication orchestration.
    pub fn new(node: Arc<Mutex<RaftNode>>) -> Self {
        Self { node }
    }

    /// Replicate log entries to a single peer.
    ///
    /// Builds an `AppendEntries` request from the peer's `nextIndex`, sends
    /// the RPC, and updates `nextIndex` / `matchIndex` on success.  On
    /// failure (log inconsistency) the `nextIndex` is decremented so the next
    /// call retries with more preceding entries.
    ///
    /// Returns `Ok(true)` on success, `Ok(false)` on log-inconsistency
    /// (caller should retry), and `Err` on transport failures.
    pub async fn replicate_to_peer(&self, peer_id: u64) -> Result<bool> {
        // ── Build the request while holding the lock ───────────────────────
        let request = {
            let node = self.node.lock().await;

            if node.state != RaftState::Leader {
                return Ok(false);
            }

            let next_idx = node.next_index.get(&peer_id).copied().unwrap_or(1);
            let prev_log_index = next_idx.saturating_sub(1);
            let prev_log_term = node.term_at(prev_log_index);
            let entries: Vec<LogEntry> = node
                .log
                .iter()
                .filter(|e| e.index >= next_idx)
                .cloned()
                .collect();
            let leader_commit = node.commit_index;

            AppendEntriesRequest {
                term: node.current_term,
                leader_id: node.id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit,
            }
        };

        // ── Send the RPC (lock NOT held) ──────────────────────────────────
        let response = {
            let node = self.node.lock().await;
            let transport = Arc::clone(&node.transport);
            drop(node);
            transport.send_append_entries(peer_id, request).await?
        };

        // ── Process the response ──────────────────────────────────────────
        let mut node = self.node.lock().await;

        // If the responder has a higher term, step down.
        if response.term > node.current_term {
            node.become_follower(response.term, None);
            info!(
                node_id = node.id,
                new_term = response.term,
                "Stepping down after AppendEntries response"
            );
            return Ok(false);
        }

        if response.success {
            node.next_index
                .insert(peer_id, response.match_index + 1);
            node.match_index.insert(peer_id, response.match_index);
            self.try_advance_commit(&mut node);
            Ok(true)
        } else {
            // Decrement nextIndex for this peer and let the caller retry.
            let next = node.next_index.get(&peer_id).copied().unwrap_or(1);
            let new_next = next.saturating_sub(1);
            node.next_index.insert(peer_id, new_next);
            debug!(
                node_id = node.id,
                peer = peer_id,
                old_next = next,
                new_next,
                "Decremented nextIndex after log inconsistency"
            );
            Ok(false)
        }
    }

    /// Advance `commit_index` after a peer successfully replicated.
    ///
    /// Finds the highest index `N` where a majority of `matchIndex` values
    /// are >= `N` **and** the entry at `N` belongs to the current term
    /// (Raft paper Section 5.4.2).
    fn try_advance_commit(&self, node: &mut RaftNode) {
        if node.state != RaftState::Leader {
            return;
        }

        // Collect every matchIndex plus the leader's own last log index.
        let mut indices: Vec<u64> = node.match_index.values().copied().collect();
        indices.push(node.last_log_index());
        // Sort descending to check the highest potential commit index first.
        indices.sort_unstable_by(|a, b| b.cmp(a));

        let majority = (node.peers.len() as u64) / 2 + 1;

        for &n in &indices {
            if n <= node.commit_index {
                break;
            }
            let replicated_count =
                indices.iter().filter(|&&idx| idx >= n).count() as u64;
            if replicated_count >= majority {
                if let Some(entry) = node.log.iter().find(|e| e.index == n) {
                    if entry.term == node.current_term {
                        node.commit_index = n;
                        node.apply_committed();
                        debug!(
                            node_id = node.id,
                            commit_index = n,
                            "Advanced commit index via replication"
                        );
                        return;
                    }
                }
            }
        }
    }

    /// Send a heartbeat (empty `AppendEntries`) to every peer.
    ///
    /// Heartbeats reset followers' election timers and carry the leader's
    /// current `commit_index`.
    pub async fn send_heartbeat(&self) -> Result<()> {
        let peers = {
            let node = self.node.lock().await;
            if !node.is_leader() {
                return Ok(());
            }
            node.peers.clone()
        };

        for peer_id in peers {
            if let Err(e) = self.replicate_to_peer(peer_id).await {
                warn!(
                    peer = peer_id,
                    error = %e,
                    "Heartbeat AppendEntries failed"
                );
            }
        }
        Ok(())
    }

    /// Long-running heartbeat loop.
    ///
    /// Sends heartbeats at the configured interval while the node is leader.
    /// Stops when `shutdown` is set.
    pub async fn run_heartbeat_loop(
        &self,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) {
        loop {
            let interval = {
                let node = self.node.lock().await;
                std::time::Duration::from_millis(node.heartbeat_interval_ms)
            };

            tokio::select! {
                _ = tokio::time::sleep(interval) => {
                    if let Err(e) = self.send_heartbeat().await {
                        warn!(error = %e, "Heartbeat round failed");
                    }
                }
                _ = shutdown.changed() => {
                    debug!("Heartbeat loop shutting down");
                    return;
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

    // ── Mock transport ─────────────────────────────────────────────────────

    struct MockTransport {
        responses: std::sync::Mutex<Vec<Result<AppendEntriesResponse>>>,
    }

    impl MockTransport {
        /// Default: every peer responds with success and match = prev + entries.
        fn all_success() -> Self {
            Self {
                responses: std::sync::Mutex::new(vec![]),
            }
        }

        fn with_responses(responses: Vec<Result<AppendEntriesResponse>>) -> Self {
            Self {
                responses: std::sync::Mutex::new(responses),
            }
        }
    }

    #[async_trait::async_trait]
    impl RpcTransport for MockTransport {
        async fn send_append_entries(
            &self,
            _peer_id: u64,
            req: AppendEntriesRequest,
        ) -> Result<AppendEntriesResponse> {
            let mut queue = self.responses.lock().unwrap();
            if !queue.is_empty() {
                return queue.remove(0);
            }
            Ok(AppendEntriesResponse {
                term: req.term,
                success: true,
                match_index: req.prev_log_index + req.entries.len() as u64,
            })
        }

        async fn send_request_vote(
            &self,
            _peer_id: u64,
            _req: RequestVoteRequest,
        ) -> Result<RequestVoteResponse> {
            Ok(RequestVoteResponse {
                term: 0,
                vote_granted: false,
            })
        }
    }

    // ── Helpers ────────────────────────────────────────────────────────────

    fn leader_node(
        id: u64,
        peers: Vec<u64>,
        transport: Arc<dyn RpcTransport>,
    ) -> Arc<Mutex<RaftNode>> {
        let mut node =
            RaftNode::new(id, peers.clone(), 1500, 300).with_transport(transport);
        node.state = RaftState::Leader;
        node.current_term = 1;
        node.log.push(LogEntry {
            term: 1,
            index: 1,
            command: Command::NoOp,
        });
        for &p in &peers {
            node.next_index.insert(p, 1);
            node.match_index.insert(p, 0);
        }
        Arc::new(Mutex::new(node))
    }

    // ── Tests ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn replicate_to_peer_success() {
        let node = leader_node(1, vec![2, 3], Arc::new(MockTransport::all_success()));
        let replication = LogReplication::new(node.clone());

        let ok = replication.replicate_to_peer(2).await.unwrap();
        assert!(ok);

        let node = node.lock().await;
        assert_eq!(node.next_index.get(&2), Some(&2));
        assert_eq!(node.match_index.get(&2), Some(&1));
    }

    #[tokio::test]
    async fn replicate_to_peer_failure_decrements_next_index() {
        let node = leader_node(
            1,
            vec![2],
            Arc::new(MockTransport::with_responses(vec![
                Ok(AppendEntriesResponse {
                    term: 1,
                    success: false,
                    match_index: 0,
                }),
            ])),
        );

        let replication = LogReplication::new(node.clone());
        let ok = replication.replicate_to_peer(2).await.unwrap();
        assert!(!ok);

        let node = node.lock().await;
        assert_eq!(node.next_index.get(&2), Some(&0));
    }

    #[tokio::test]
    async fn replicate_to_peer_step_down_on_higher_term() {
        let node = leader_node(
            1,
            vec![2],
            Arc::new(MockTransport::with_responses(vec![
                Ok(AppendEntriesResponse {
                    term: 5,
                    success: true,
                    match_index: 1,
                }),
            ])),
        );

        let replication = LogReplication::new(node.clone());
        let ok = replication.replicate_to_peer(2).await.unwrap();
        assert!(!ok); // Not success because we stepped down.

        let node = node.lock().await;
        assert_eq!(node.state, RaftState::Follower);
        assert_eq!(node.current_term, 5);
    }

    #[tokio::test]
    async fn replicate_to_peer_not_leader_returns_false() {
        let node = Arc::new(Mutex::new({
            let mut n =
                RaftNode::new(1, vec![2], 1500, 300).with_transport(Arc::new(MockTransport::all_success()));
            n.state = RaftState::Follower;
            n
        }));

        let replication = LogReplication::new(node);
        let ok = replication.replicate_to_peer(2).await.unwrap();
        assert!(!ok);
    }

    #[tokio::test]
    async fn commit_advances_after_quorum_replication() {
        // 3-node cluster (self + 2 peers).  Need 2 nodes (majority) to commit.
        let node = leader_node(
            1,
            vec![2, 3],
            Arc::new(MockTransport::all_success()),
        );

        let replication = LogReplication::new(node.clone());

        // Replicate to both peers.
        replication.replicate_to_peer(2).await.unwrap();
        replication.replicate_to_peer(3).await.unwrap();

        let node = node.lock().await;
        // Leader has index 1, both peers now have index 1 -> majority.
        assert_eq!(node.commit_index, 1);
        assert_eq!(node.last_applied, 1);
    }

    #[tokio::test]
    async fn commit_does_not_advance_without_majority() {
        // 5-node cluster (self + 4 peers).  Need 3 nodes for majority.
        let mut node_state =
            RaftNode::new(1, vec![2, 3, 4, 5], 1500, 300)
                .with_transport(Arc::new(MockTransport::all_success()));
        node_state.state = RaftState::Leader;
        node_state.current_term = 1;
        node_state.log.push(LogEntry {
            term: 1,
            index: 1,
            command: Command::NoOp,
        });
        for &p in &[2, 3, 4, 5] {
            node_state.next_index.insert(p, 1);
            node_state.match_index.insert(p, 0);
        }
        let node = Arc::new(Mutex::new(node_state));

        let replication = LogReplication::new(node.clone());

        // Only replicate to one peer.
        replication.replicate_to_peer(2).await.unwrap();

        let node = node.lock().await;
        // Leader(1) + peer 2(1) = 2 nodes.  Need 3 for 5-node cluster.
        assert_eq!(node.commit_index, 0);
    }

    #[tokio::test]
    async fn send_heartbeat_skips_when_not_leader() {
        let node = Arc::new(Mutex::new({
            let mut n =
                RaftNode::new(1, vec![2], 1500, 300).with_transport(Arc::new(MockTransport::all_success()));
            n.state = RaftState::Follower;
            n
        }));

        let replication = LogReplication::new(node);
        // Should be a no-op, no error.
        replication.send_heartbeat().await.unwrap();
    }

    #[tokio::test]
    async fn send_heartbeat_sends_to_all_peers() {
        let node = leader_node(
            1,
            vec![2, 3],
            Arc::new(MockTransport::all_success()),
        );

        let replication = LogReplication::new(node.clone());
        replication.send_heartbeat().await.unwrap();

        // Both peers should now have match_index = 1.
        let node = node.lock().await;
        assert_eq!(node.match_index.get(&2), Some(&1));
        assert_eq!(node.match_index.get(&3), Some(&1));
    }

    #[tokio::test]
    async fn replication_multiple_entries() {
        let node = leader_node(
            1,
            vec![2],
            Arc::new(MockTransport::all_success()),
        );

        // Add more entries to the leader's log.
        {
            let mut n = node.lock().await;
            n.log.push(LogEntry {
                term: 1,
                index: 2,
                command: Command::CreateTopic {
                    name: "events".into(),
                    partitions: 4,
                },
            });
            n.log.push(LogEntry {
                term: 1,
                index: 3,
                command: Command::Produce {
                    topic: "events".into(),
                    partition: 0,
                    data: vec![0xDE, 0xAD],
                },
            });
        }

        let replication = LogReplication::new(node.clone());
        replication.replicate_to_peer(2).await.unwrap();

        let node = node.lock().await;
        assert_eq!(node.match_index.get(&2), Some(&3));
        assert_eq!(node.next_index.get(&2), Some(&4));
        // Commit should advance: leader(3) + peer(3) = majority.
        assert_eq!(node.commit_index, 3);
        assert_eq!(node.last_applied, 3);
    }

    #[tokio::test]
    async fn retry_after_log_inconsistency_converges() {
        // Peer returns failure first (log inconsistency), then success.
        let node = leader_node(
            1,
            vec![2],
            Arc::new(MockTransport::with_responses(vec![
                // First call: fail
                Ok(AppendEntriesResponse {
                    term: 1,
                    success: false,
                    match_index: 0,
                }),
                // Second call: success
                Ok(AppendEntriesResponse {
                    term: 1,
                    success: true,
                    match_index: 1,
                }),
            ])),
        );

        let replication = LogReplication::new(node.clone());

        // First attempt fails.
        let ok = replication.replicate_to_peer(2).await.unwrap();
        assert!(!ok);

        {
            let n = node.lock().await;
            assert_eq!(n.next_index.get(&2), Some(&0));
        }

        // Retry succeeds.
        let ok = replication.replicate_to_peer(2).await.unwrap();
        assert!(ok);

        let node = node.lock().await;
        assert_eq!(node.match_index.get(&2), Some(&1));
    }
}
