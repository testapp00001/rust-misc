//! # Exercise: PBFT Commit Phase
//!
//! ## Theory
//!
//! The **Commit phase** is the third and final phase of the PBFT consensus
//! protocol. It follows the Prepare phase and ensures that a request is
//! durably agreed upon before any node executes it.
//!
//! The full three-phase PBFT flow is:
//!
//! 1. **Pre-prepare** (Leader -> All): Leader proposes an ordering.
//! 2. **Prepare** (All -> All): Replicas confirm they accepted the Pre-prepare.
//!    A request becomes "prepared" when 2f + 1 Prepare messages are collected.
//! 3. **Commit** (All -> All): Replicas confirm they are prepared. A request
//!    becomes "committed" when 2f + 1 Commit messages are collected.
//! 4. **Execute**: After committing, the replica executes the request and
//!    returns the result to the client.
//!
//! The Commit phase is necessary because a node might be "prepared" but have
//! not yet told all other nodes. Without the Commit phase, a node could execute
//! a request that other nodes have not yet agreed to, leading to inconsistency.
//!
//! ## Proof / Intuition
//!
//! The Commit phase provides the final safety guarantee. Once a node has received
//! 2f + 1 Commit messages, it knows that at least f + 1 honest nodes are also
//! prepared (since at most f nodes are Byzantine). This means:
//!
//! 1. At least one honest node in any quorum overlap has prepared the same
//!    request for the same sequence number.
//! 2. No honest node can be prepared for a different request at the same
//!    sequence number (because Prepare quorums overlap).
//! 3. Therefore, all honest nodes will eventually commit and execute the same
//!    request in the same order.
//!
//! The two quorum thresholds (2f + 1 for Prepare and 2f + 1 for Commit) work
//! together: the Prepare quorum ensures agreement on the request, and the
//! Commit quorum ensures that enough nodes have confirmed before execution.
//!
//! ## Implementation Task
//!
//! Implement:
//!
//! - `CommitMessage` struct with: view_number, sequence_number,
//!   request_digest, node_id
//! - `PBFTNodeCommit` struct with: node_id, total_nodes, view_number,
//!   sequence_number, prepares_received, commits_received, is_prepared,
//!   is_committed, executed, executed_request
//! - Methods:
//!   - `new(node_id, total_nodes)` - create a node
//!   - `create_prepare() -> PrepareMessage` - create a Prepare message
//!   - `receive_prepare(msg) -> bool` - process a Prepare message
//!   - `create_commit() -> Option<CommitMessage>` - create a Commit message
//!     (only if prepared)
//!   - `receive_commit(msg) -> bool` - process a Commit message
//!   - `is_committed()` - check if committed
//!   - `execute() -> Option<String>` - execute after 2f+1 commits
//!
//! ## Verification
//!
//! - Verify Commit message is only created after prepared state
//! - Verify execution only happens after 2f+1 commits
//! - Verify full 3-phase flow produces the correct result
//! - Verify duplicate messages are rejected

/// A Commit message broadcast by a replica after reaching the prepared state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitMessage {
    /// The view number this Commit belongs to.
    pub view_number: u64,
    /// The sequence number being committed.
    pub sequence_number: u64,
    /// A digest of the request content.
    pub request_digest: String,
    /// The ID of the node sending this Commit.
    pub node_id: usize,
}

/// A Prepare message broadcast during the Prepare phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareMessage {
    /// The view number this Prepare belongs to.
    pub view_number: u64,
    /// The sequence number being prepared.
    pub sequence_number: u64,
    /// A digest of the request content.
    pub request_digest: String,
    /// The ID of the node sending this Prepare.
    pub node_id: usize,
}

/// A PBFT node implementing the full three-phase commit protocol.
///
/// The node tracks Prepare and Commit messages, determines when a request
/// reaches the prepared and committed states, and executes the request
/// after sufficient Commit messages are received.
#[derive(Debug, Clone)]
pub struct PBFTNodeCommit {
    /// This node's identifier.
    pub node_id: usize,
    /// Total number of nodes in the system.
    pub total_nodes: usize,
    /// Current view number.
    pub view_number: u64,
    /// The sequence number being processed.
    pub sequence_number: u64,
    /// Prepare messages received from other nodes.
    pub prepares_received: Vec<PrepareMessage>,
    /// Commit messages received from other nodes.
    pub commits_received: Vec<CommitMessage>,
    /// Whether the request has reached the prepared state.
    pub is_prepared: bool,
    /// Whether the request has reached the committed state.
    pub is_committed: bool,
    /// Whether the request has been executed.
    pub executed: bool,
    /// The result of executing the request, if any.
    pub executed_request: Option<String>,
}

impl PBFTNodeCommit {
    /// Create a new PBFT node for the commit phase.
    pub fn new(node_id: usize, total_nodes: usize) -> Self {
        assert!(
            node_id < total_nodes,
            "node_id must be less than total_nodes"
        );
        Self {
            node_id,
            total_nodes,
            view_number: 0,
            sequence_number: 0,
            prepares_received: Vec::new(),
            commits_received: Vec::new(),
            is_prepared: false,
            is_committed: false,
            executed: false,
            executed_request: None,
        }
    }

    /// Create a Prepare message for the current view and sequence number.
    pub fn create_prepare(&self) -> PrepareMessage {
        PrepareMessage {
            view_number: self.view_number,
            sequence_number: self.sequence_number,
            request_digest: String::new(),
            node_id: self.node_id,
        }
    }

    /// Create a Prepare message with a specific request digest.
    pub fn create_prepare_with_digest(&self, request_digest: &str) -> PrepareMessage {
        PrepareMessage {
            view_number: self.view_number,
            sequence_number: self.sequence_number,
            request_digest: request_digest.to_string(),
            node_id: self.node_id,
        }
    }

    /// Receive and validate a Prepare message.
    ///
    /// Returns `true` if the message was accepted (valid and not a duplicate).
    /// Also updates the prepared state if enough Prepare messages have been collected.
    pub fn receive_prepare(&mut self, msg: PrepareMessage) -> bool {
        if msg.view_number != self.view_number {
            return false;
        }
        if msg.sequence_number != self.sequence_number {
            return false;
        }
        if msg.node_id == self.node_id {
            return false;
        }
        if self
            .prepares_received
            .iter()
            .any(|m| m.node_id == msg.node_id)
        {
            return false;
        }

        self.prepares_received.push(msg);
        self.check_prepared();
        true
    }

    /// Create a Commit message. Returns `None` if the node is not yet prepared.
    ///
    /// A node only sends a Commit message after reaching the prepared state
    /// (2f + 1 Prepare messages including its own).
    pub fn create_commit(&self) -> Option<CommitMessage> {
        if !self.is_prepared {
            return None;
        }

        Some(CommitMessage {
            view_number: self.view_number,
            sequence_number: self.sequence_number,
            request_digest: String::new(),
            node_id: self.node_id,
        })
    }

    /// Create a Commit message with a specific request digest.
    pub fn create_commit_with_digest(&self, request_digest: &str) -> Option<CommitMessage> {
        if !self.is_prepared {
            return None;
        }

        Some(CommitMessage {
            view_number: self.view_number,
            sequence_number: self.sequence_number,
            request_digest: request_digest.to_string(),
            node_id: self.node_id,
        })
    }

    /// Receive and validate a Commit message.
    ///
    /// Returns `true` if the message was accepted (valid and not a duplicate).
    /// Also updates the committed state if enough Commit messages have been collected.
    pub fn receive_commit(&mut self, msg: CommitMessage) -> bool {
        if msg.view_number != self.view_number {
            return false;
        }
        if msg.sequence_number != self.sequence_number {
            return false;
        }
        if msg.node_id == self.node_id {
            return false;
        }
        if self
            .commits_received
            .iter()
            .any(|m| m.node_id == msg.node_id)
        {
            return false;
        }

        self.commits_received.push(msg);
        self.check_committed();
        true
    }

    /// Check if the node has reached the prepared state.
    fn check_prepared(&mut self) {
        let f = (self.total_nodes - 1) / 3;
        let quorum = 2 * f + 1;
        let total = 1 + self.prepares_received.len(); // +1 for own Pre-prepare
        self.is_prepared = total >= quorum;
    }

    /// Check if the node has reached the committed state.
    fn check_committed(&mut self) {
        let f = (self.total_nodes - 1) / 3;
        let quorum = 2 * f + 1;
        let total = 1 + self.commits_received.len(); // +1 for own Commit
        self.is_committed = total >= quorum;
    }

    /// Execute the request if committed. Returns the request digest if
    /// execution occurred, or `None` if not yet committed or already executed.
    ///
    /// Execution is the final step: the node applies the request to its state
    /// machine and returns the result to the client.
    pub fn execute(&mut self) -> Option<String> {
        if !self.is_committed || self.executed {
            return None;
        }

        self.executed = true;
        self.executed_request = Some("executed".to_string());
        self.executed_request.clone()
    }

    /// Calculate the quorum size (2f + 1) needed for prepared/committed state.
    pub fn quorum_size(&self) -> usize {
        let f = (self.total_nodes - 1) / 3;
        2 * f + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_only_after_prepared() {
        let mut node = PBFTNodeCommit::new(0, 4);
        // Node is not prepared, so create_commit should return None
        assert!(
            node.create_commit().is_none(),
            "should not create Commit before prepared"
        );

        // Make node prepared by receiving 2f+1 Prepares (quorum=3, need 2 external)
        for i in 1..=2 {
            let msg = PrepareMessage {
                view_number: 0,
                sequence_number: 0,
                request_digest: "digest".to_string(),
                node_id: i,
            };
            node.receive_prepare(msg);
        }

        assert!(node.is_prepared, "node should be prepared after 2 Prepares");

        // Now create_commit should return Some
        let commit = node.create_commit();
        assert!(
            commit.is_some(),
            "should create Commit after prepared"
        );
    }

    #[test]
    fn execution_after_two_f_plus_one_commits() {
        let mut node = PBFTNodeCommit::new(0, 4);
        // quorum = 2*(4-1)/3 + 1 = 2*1 + 1 = 3

        // First, make the node prepared
        for i in 1..=2 {
            let msg = PrepareMessage {
                view_number: 0,
                sequence_number: 0,
                request_digest: "digest".to_string(),
                node_id: i,
            };
            node.receive_prepare(msg);
        }
        assert!(node.is_prepared);

        // Create own commit
        let _ = node.create_commit();

        // Execute before having enough commits should fail
        assert!(
            node.execute().is_none(),
            "should not execute before committed"
        );

        // Receive 2 external Commit messages -> total = 3 = quorum
        for i in 1..=2 {
            let msg = CommitMessage {
                view_number: 0,
                sequence_number: 0,
                request_digest: "digest".to_string(),
                node_id: i,
            };
            node.receive_commit(msg);
        }

        assert!(node.is_committed, "node should be committed after 2 Commits");

        // Now execution should succeed
        let result = node.execute();
        assert!(result.is_some(), "should execute after committed");
        assert!(node.executed, "executed flag should be set");
    }

    #[test]
    fn full_three_phase_flow() {
        let mut node = PBFTNodeCommit::new(0, 4);
        let request_digest = "client_request_abc123";

        // Phase 1: Pre-prepare (node accepts its own pre-prepare)
        // In a real system, the leader sends this. Here we simulate acceptance.

        // Phase 2: Prepare
        // Node creates its own Prepare
        let own_prepare = node.create_prepare_with_digest(request_digest);
        assert_eq!(own_prepare.node_id, 0);

        // Receive Prepares from nodes 1, 2 (quorum = 3, own + 2 = 3)
        for i in 1..=2 {
            let msg = PrepareMessage {
                view_number: 0,
                sequence_number: 0,
                request_digest: request_digest.to_string(),
                node_id: i,
            };
            node.receive_prepare(msg);
        }
        assert!(node.is_prepared, "should be prepared after 3 Prepares");

        // Phase 3: Commit
        // Node creates its own Commit
        let own_commit = node.create_commit_with_digest(request_digest);
        assert!(own_commit.is_some(), "should create Commit after prepared");

        // Receive Commits from nodes 1, 2 (quorum = 3, own + 2 = 3)
        for i in 1..=2 {
            let msg = CommitMessage {
                view_number: 0,
                sequence_number: 0,
                request_digest: request_digest.to_string(),
                node_id: i,
            };
            node.receive_commit(msg);
        }
        assert!(node.is_committed, "should be committed after 3 Commits");

        // Execute
        let result = node.execute();
        assert!(result.is_some(), "should produce a result after execution");
        assert!(node.executed, "should be marked as executed");
    }

    #[test]
    fn duplicate_messages_rejected() {
        let mut node = PBFTNodeCommit::new(0, 4);

        let msg = PrepareMessage {
            view_number: 0,
            sequence_number: 0,
            request_digest: "digest".to_string(),
            node_id: 1,
        };

        assert!(node.receive_prepare(msg.clone()));
        assert!(
            !node.receive_prepare(msg),
            "duplicate Prepare should be rejected"
        );

        // Make node prepared and try duplicate commit
        let msg2 = PrepareMessage {
            view_number: 0,
            sequence_number: 0,
            request_digest: "digest".to_string(),
            node_id: 2,
        };
        node.receive_prepare(msg2);

        let commit = CommitMessage {
            view_number: 0,
            sequence_number: 0,
            request_digest: "digest".to_string(),
            node_id: 1,
        };
        assert!(node.receive_commit(commit.clone()));
        assert!(
            !node.receive_commit(commit),
            "duplicate Commit should be rejected"
        );
    }

    #[test]
    fn execution_is_idempotent() {
        let mut node = PBFTNodeCommit::new(0, 4);

        // Prepare and commit
        for i in 1..=2 {
            node.receive_prepare(PrepareMessage {
                view_number: 0,
                sequence_number: 0,
                request_digest: "digest".to_string(),
                node_id: i,
            });
        }
        let _ = node.create_commit();
        for i in 1..=2 {
            node.receive_commit(CommitMessage {
                view_number: 0,
                sequence_number: 0,
                request_digest: "digest".to_string(),
                node_id: i,
            });
        }

        // First execution succeeds
        let first = node.execute();
        assert!(first.is_some());

        // Second execution returns None (already executed)
        let second = node.execute();
        assert!(
            second.is_none(),
            "should not execute again after first execution"
        );
    }
}
