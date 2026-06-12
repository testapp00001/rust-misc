//! # Exercise: PBFT Prepare Phase
//!
//! ## Theory
//!
//! The **Prepare phase** is the second phase of the PBFT consensus protocol.
//! After a replica accepts a valid Pre-prepare message from the leader, it
//! broadcasts a **PREPARE** message to all other nodes.
//!
//! The Prepare phase ensures that enough nodes have accepted the same request
//! and sequence number before any node executes it. The key threshold is:
//!
//! > A request is **prepared** when a node has received **2f + 1** matching
//! > Prepare messages (including its own Pre-prepare acceptance), where
//! > **f = (n - 1) / 3** is the maximum number of Byzantine faults the system
//! > can tolerate.
//!
//! For example:
//! - With 4 nodes (f=1): 2*1 + 1 = 3 Prepare messages needed
//! - With 7 nodes (f=2): 2*2 + 1 = 5 Prepare messages needed
//! - With 10 nodes (f=3): 2*3 + 1 = 7 Prepare messages needed
//!
//! ## Proof / Intuition
//!
//! Why 2f + 1? The quorum of 2f + 1 ensures that any two quorums overlap in
//! at least f + 1 nodes. Since at most f nodes are Byzantine, at least one
//! node in the overlap is honest. This means:
//!
//! 1. If two honest nodes are both "prepared" for the same sequence number,
//!    they must have seen the same request (because honest nodes only prepare
//!    for the request in the Pre-prepare they accepted).
//!
//! 2. The 2f + 1 threshold guarantees that at least f + 1 honest nodes have
//!    accepted the Pre-prepare, making it impossible for a Byzantine leader
//!    to have sent conflicting Pre-prepares to different honest nodes without
//!    being detected.
//!
//! The Prepare phase thus provides the safety guarantee: all honest replicas
//! will process requests in the same order.
//!
//! ## Implementation Task
//!
//! Implement:
//!
//! - `PrepareMessage` struct with: view_number, sequence_number,
//!   request_digest, node_id
//! - `PBFTNodePrepare` struct with: node_id, total_nodes, view_number,
//!   sequence_number, prepares_received (Vec of PrepareMessage),
//!   is_prepared (bool)
//! - Methods:
//!   - `new(node_id, total_nodes)` - create a node
//!   - `create_prepare() -> PrepareMessage` - create a Prepare message
//!   - `receive_prepare(msg) -> bool` - validate and record a Prepare message
//!   - `is_prepared()` - returns true when 2f+1 Prepare messages are received
//!
//! ## Verification
//!
//! - Verify Prepare message creation with correct fields
//! - Verify 2f+1 threshold triggers the prepared state
//! - Verify Prepare messages from different nodes are all counted
//! - Verify duplicate Prepare messages from the same node are not double-counted

/// A Prepare message broadcast by a replica after accepting a Pre-prepare.
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

/// A PBFT node participating in the Prepare phase.
///
/// After accepting a valid Pre-prepare, the node broadcasts a Prepare message.
/// The node tracks incoming Prepare messages and determines when the request
/// has reached the "prepared" state (2f + 1 matching Prepare messages).
#[derive(Debug, Clone)]
pub struct PBFTNodePrepare {
    /// This node's identifier.
    pub node_id: usize,
    /// Total number of nodes in the system.
    pub total_nodes: usize,
    /// Current view number.
    pub view_number: u64,
    /// The sequence number being prepared.
    pub sequence_number: u64,
    /// Prepare messages received from other nodes (excluding this node's own).
    pub prepares_received: Vec<PrepareMessage>,
    /// Whether the request has reached the prepared state.
    is_prepared_flag: bool,
}

impl PBFTNodePrepare {
    /// Create a new PBFT node for the Prepare phase.
    ///
    /// The node starts in the unprepared state.
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
            is_prepared_flag: false,
        }
    }

    /// Create a Prepare message for the current view and sequence number.
    ///
    /// This message is broadcast to all other nodes after the node accepts
    /// a valid Pre-prepare.
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

    /// Receive and validate a Prepare message from another node.
    ///
    /// A Prepare message is accepted if:
    /// 1. The view number matches this node's current view.
    /// 2. The sequence number matches.
    /// 3. The message is from a different node (not from self).
    /// 4. No duplicate from the same node.
    ///
    /// Returns `true` if the message was accepted, `false` if rejected.
    pub fn receive_prepare(&mut self, msg: PrepareMessage) -> bool {
        // Validate view number
        if msg.view_number != self.view_number {
            return false;
        }

        // Validate sequence number
        if msg.sequence_number != self.sequence_number {
            return false;
        }

        // Reject messages from self
        if msg.node_id == self.node_id {
            return false;
        }

        // Reject duplicates from the same node
        if self
            .prepares_received
            .iter()
            .any(|m| m.node_id == msg.node_id)
        {
            return false;
        }

        self.prepares_received.push(msg);

        // Check if we've reached the prepared state
        self.check_prepared();

        true
    }

    /// Check if the request has reached the prepared state.
    ///
    /// A request is prepared when the node has received 2f + 1 Prepare messages
    /// (including its own implicit Prepare from accepting the Pre-prepare).
    fn check_prepared(&mut self) {
        let f = (self.total_nodes - 1) / 3;
        let quorum = 2 * f + 1;

        // Count: 1 (own Pre-prepare acceptance) + received Prepares
        let total_prepares = 1 + self.prepares_received.len();
        self.is_prepared_flag = total_prepares >= quorum;
    }

    /// Returns true when 2f + 1 Prepare messages have been received
    /// (counting the node's own Pre-prepare acceptance as one).
    pub fn is_prepared(&self) -> bool {
        self.is_prepared_flag
    }

    /// Calculate the Byzantine fault tolerance parameter f.
    pub fn max_byzantine_faults(&self) -> usize {
        (self.total_nodes - 1) / 3
    }

    /// Calculate the quorum size (2f + 1) needed for the prepared state.
    pub fn quorum_size(&self) -> usize {
        let f = self.max_byzantine_faults();
        2 * f + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_creation() {
        let node = PBFTNodePrepare::new(0, 4);
        let msg = node.create_prepare();

        assert_eq!(msg.view_number, 0);
        assert_eq!(msg.sequence_number, 0);
        assert_eq!(msg.node_id, 0);
    }

    #[test]
    fn two_f_plus_one_threshold() {
        // With 4 nodes: f = (4-1)/3 = 1, quorum = 2*1+1 = 3
        // Need 2 external Prepares + 1 own = 3 total
        let mut node = PBFTNodePrepare::new(0, 4);
        assert_eq!(node.quorum_size(), 3);
        assert!(!node.is_prepared(), "should not be prepared initially");

        // Receive first Prepare
        let msg1 = PrepareMessage {
            view_number: 0,
            sequence_number: 0,
            request_digest: "digest".to_string(),
            node_id: 1,
        };
        assert!(node.receive_prepare(msg1));
        assert!(
            !node.is_prepared(),
            "should not be prepared with 1 external Prepare (total=2)"
        );

        // Receive second Prepare -> total = 3 = quorum
        let msg2 = PrepareMessage {
            view_number: 0,
            sequence_number: 0,
            request_digest: "digest".to_string(),
            node_id: 2,
        };
        assert!(node.receive_prepare(msg2));
        assert!(
            node.is_prepared(),
            "should be prepared with 2 external Prepares (total=3=quorum)"
        );
    }

    #[test]
    fn prepares_from_different_nodes_counted() {
        let mut node = PBFTNodePrepare::new(0, 7);
        // With 7 nodes: f = (7-1)/3 = 2, quorum = 2*2+1 = 5
        assert_eq!(node.quorum_size(), 5);

        for i in 1..=4 {
            let msg = PrepareMessage {
                view_number: 0,
                sequence_number: 0,
                request_digest: "digest".to_string(),
                node_id: i,
            };
            let accepted = node.receive_prepare(msg);
            assert!(accepted, "Prepare from node {i} should be accepted");
        }

        // Total = 1 (own) + 4 (external) = 5 = quorum
        assert!(
            node.is_prepared(),
            "should be prepared with 4 external Prepares (total=5=quorum)"
        );
    }

    #[test]
    fn duplicate_prepare_rejected() {
        let mut node = PBFTNodePrepare::new(0, 4);

        let msg = PrepareMessage {
            view_number: 0,
            sequence_number: 0,
            request_digest: "digest".to_string(),
            node_id: 1,
        };

        assert!(node.receive_prepare(msg.clone()));
        assert!(
            !node.receive_prepare(msg),
            "duplicate Prepare from same node should be rejected"
        );
        assert_eq!(
            node.prepares_received.len(),
            1,
            "should only have 1 Prepare after duplicate"
        );
    }

    #[test]
    fn wrong_view_or_sequence_rejected() {
        let mut node = PBFTNodePrepare::new(0, 4);

        let wrong_view = PrepareMessage {
            view_number: 5,
            sequence_number: 0,
            request_digest: "digest".to_string(),
            node_id: 1,
        };
        assert!(!node.receive_prepare(wrong_view), "wrong view should be rejected");

        let wrong_seq = PrepareMessage {
            view_number: 0,
            sequence_number: 99,
            request_digest: "digest".to_string(),
            node_id: 1,
        };
        assert!(!node.receive_prepare(wrong_seq), "wrong sequence should be rejected");
    }
}
