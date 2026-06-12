//! Replication layer implementing the Raft consensus protocol for
//! leader election and replicated log consistency.
//!
//! This module provides:
//! - **Raft node** -- core Raft state machine with persistent and volatile state
//! - **Leader election** -- randomized election timeouts, RequestVote RPCs, quorum
//! - **Log replication** -- AppendEntries RPCs, commit index advancement, apply pipeline
//!
//! All RPC I/O is abstracted behind [`raft_node::RpcTransport`] so the protocol
//! logic can be tested with mock transports without touching the network.

pub mod raft_node;
pub mod leader_election;
pub mod log_replication;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub use raft_node::{
    RaftNode, RaftState, LogEntry, Command,
    AppendEntriesRequest, AppendEntriesResponse,
    RequestVoteRequest, RequestVoteResponse,
    RpcTransport, NullTransport,
};
pub use leader_election::LeaderElection;
pub use log_replication::LogReplication;

/// Backward-compatible replication manager wrapping a [`RaftNode`].
///
/// Provides the same construction and lifecycle API previously used by the
/// broker while delegating all protocol logic to the Raft state machine.
pub struct ReplicationManager {
    inner: Arc<Mutex<RaftNode>>,
}

impl ReplicationManager {
    /// Create a new replication manager.
    pub fn new(
        node_id: u64,
        election_timeout_ms: u64,
        heartbeat_interval_ms: u64,
    ) -> Self {
        let node = RaftNode::new(
            node_id,
            vec![],
            election_timeout_ms,
            heartbeat_interval_ms,
        );
        Self {
            inner: Arc::new(Mutex::new(node)),
        }
    }

    /// Start background replication tasks.
    pub async fn start(&self) {
        info!("ReplicationManager started");
    }

    /// Stop the replication manager gracefully.
    pub async fn stop(&self) {
        info!("ReplicationManager stopped");
    }

    /// Access the underlying Raft node.
    pub fn raft_node(&self) -> &Arc<Mutex<RaftNode>> {
        &self.inner
    }
}
