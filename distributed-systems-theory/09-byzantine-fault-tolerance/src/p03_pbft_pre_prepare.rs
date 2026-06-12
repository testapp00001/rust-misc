//! # Exercise: PBFT Pre-prepare Phase
//!
//! ## Theory
//!
//! The **Pre-prepare phase** is the first of three phases in the PBFT consensus
//! protocol. It is initiated by the **leader** (primary) and serves to propose
//! an ordering for client requests.
//!
//! The protocol works as follows:
//!
//! 1. A client sends a request to the leader.
//! 2. The leader assigns a **sequence number** to the request and broadcasts a
//!    **PRE-PREPARE** message to all replicas. The message contains:
//!    - `view_number`: The current view (leader epoch).
//!    - `sequence_number`: The order assigned by the leader.
//!    - `request_digest`: A hash of the request content for integrity.
//!    - `leader_id`: The ID of the node sending the pre-prepare.
//!
//! 3. Each replica validates the Pre-prepare message:
//!    - The view number must match the replica's current view.
//!    - The message must come from the expected leader for this view.
//!    - The sequence number must be within the acceptable range (no gaps
//!      or duplicates for the current batch).
//!    - The request digest must be valid.
//!
//! 4. If valid, the replica accepts the Pre-prepare and moves to the Prepare
//!    phase. If invalid, the replica rejects it and may initiate a view change.
//!
//! ## Proof / Intuition
//!
//! The Pre-prepare phase establishes the **leader's proposal** for request ordering.
//! Since the leader is the only node that can assign sequence numbers, this phase
//! ensures that there is a single proposed order for each request.
//!
//! The validation steps are critical:
//! - **View check**: Ensures the message is from the current epoch, not a stale
//!   leader's message.
//! - **Leader check**: Ensures only the designated leader can propose ordering.
//!   A Byzantine node that is not the leader cannot forge a valid pre-prepare.
//! - **Sequence check**: Prevents the leader from assigning conflicting sequence
//!   numbers to different requests.
//!
//! ## Implementation Task
//!
//! Implement:
//!
//! - `PrePrepareMessage` struct with: view_number, sequence_number,
//!   request_digest, leader_id
//! - `PBFTNodePrePrepare` struct with: node_id, total_nodes, view_number,
//!   next_sequence, accepted_pre_prepares (Vec), my_requests (Vec)
//! - Methods:
//!   - `new(node_id, total_nodes)` - create a node
//!   - `is_leader()` - check if this node is the leader
//!   - `create_pre_prepare(request_digest) -> PrePrepareMessage` - leader creates
//!     a Pre-prepare (only valid for the leader)
//!   - `receive_pre_prepare(msg) -> bool` - validate and accept/reject a
//!     Pre-prepare message
//!   - `get_accepted_count()` - return the number of accepted Pre-prepare messages
//!
//! ## Verification
//!
//! - Verify leader creates Pre-prepare with correct fields
//! - Verify non-leader accepts a valid Pre-prepare
//! - Verify non-leader rejects a Pre-prepare from a non-leader
//! - Verify reject Pre-prepare with wrong view number

/// A Pre-prepare message sent by the leader to propose a request ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrePrepareMessage {
    /// The view number this proposal belongs to.
    pub view_number: u64,
    /// The sequence number assigned by the leader.
    pub sequence_number: u64,
    /// A digest (hash) of the request content.
    pub request_digest: String,
    /// The ID of the leader sending this message.
    pub leader_id: usize,
}

/// A PBFT node participating in the Pre-prepare phase.
///
/// The leader creates Pre-prepare messages and broadcasts them. Non-leader
/// nodes validate incoming Pre-prepare messages and accept or reject them.
#[derive(Debug, Clone)]
pub struct PBFTNodePrePrepare {
    /// This node's identifier.
    pub node_id: usize,
    /// Total number of nodes in the system.
    pub total_nodes: usize,
    /// Current view number.
    pub view_number: u64,
    /// Next sequence number to assign (used by the leader).
    pub next_sequence: u64,
    /// Pre-prepare messages this node has accepted.
    pub accepted_pre_prepares: Vec<PrePrepareMessage>,
    /// Requests submitted to this node by clients.
    pub my_requests: Vec<String>,
}

impl PBFTNodePrePrepare {
    /// Create a new PBFT node for the Pre-prepare phase.
    ///
    /// Node 0 is the leader in view 0.
    pub fn new(node_id: usize, total_nodes: usize) -> Self {
        assert!(
            node_id < total_nodes,
            "node_id must be less than total_nodes"
        );
        Self {
            node_id,
            total_nodes,
            view_number: 0,
            next_sequence: 0,
            accepted_pre_prepares: Vec::new(),
            my_requests: Vec::new(),
        }
    }

    /// Check if this node is the current leader.
    ///
    /// The leader for view `v` is node `v mod total_nodes`.
    pub fn is_leader(&self) -> bool {
        (self.view_number as usize) % self.total_nodes == self.node_id
    }

    /// Create a Pre-prepare message as the leader.
    ///
    /// Assigns the next available sequence number to the given request digest.
    /// Returns `None` if this node is not the leader.
    pub fn create_pre_prepare(&mut self, request_digest: String) -> Option<PrePrepareMessage> {
        if !self.is_leader() {
            return None;
        }

        let msg = PrePrepareMessage {
            view_number: self.view_number,
            sequence_number: self.next_sequence,
            request_digest,
            leader_id: self.node_id,
        };

        self.next_sequence += 1;
        Some(msg)
    }

    /// Validate and accept or reject a Pre-prepare message.
    ///
    /// A Pre-prepare is accepted if:
    /// 1. The view number matches this node's current view.
    /// 2. The message comes from the expected leader for this view.
    /// 3. The sequence number is the next expected sequence (no gaps).
    ///
    /// Returns `true` if the message was accepted, `false` if rejected.
    pub fn receive_pre_prepare(&mut self, msg: &PrePrepareMessage) -> bool {
        // Check view number
        if msg.view_number != self.view_number {
            return false;
        }

        // Check that the sender is the expected leader
        let expected_leader = (self.view_number as usize) % self.total_nodes;
        if msg.leader_id != expected_leader {
            return false;
        }

        // Check sequence number is the next expected one
        if msg.sequence_number != self.next_sequence {
            return false;
        }

        // Accept the pre-prepare
        self.next_sequence += 1;
        self.accepted_pre_prepares.push(msg.clone());
        true
    }

    /// Return the number of Pre-prepare messages this node has accepted.
    pub fn get_accepted_count(&self) -> usize {
        self.accepted_pre_prepares.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leader_creates_valid_pre_prepare() {
        let mut leader = PBFTNodePrePrepare::new(0, 4);
        assert!(leader.is_leader(), "node 0 should be the leader");

        let msg = leader
            .create_pre_prepare("request_hash_1".to_string())
            .expect("leader should create a Pre-prepare");

        assert_eq!(msg.view_number, 0);
        assert_eq!(msg.sequence_number, 0);
        assert_eq!(msg.request_digest, "request_hash_1");
        assert_eq!(msg.leader_id, 0);

        // Next call should increment sequence number
        let msg2 = leader
            .create_pre_prepare("request_hash_2".to_string())
            .expect("leader should create a second Pre-prepare");
        assert_eq!(msg2.sequence_number, 1);
    }

    #[test]
    fn non_leader_accepts_valid_pre_prepare() {
        let mut leader = PBFTNodePrePrepare::new(0, 4);
        let mut replica = PBFTNodePrePrepare::new(1, 4);

        assert!(!replica.is_leader(), "node 1 should not be the leader");

        let msg = leader
            .create_pre_prepare("digest".to_string())
            .unwrap();

        let accepted = replica.receive_pre_prepare(&msg);
        assert!(accepted, "replica should accept a valid Pre-prepare");
        assert_eq!(replica.get_accepted_count(), 1);
    }

    #[test]
    fn reject_pre_prepare_from_non_leader() {
        let mut replica = PBFTNodePrePrepare::new(1, 4);

        // A non-leader node (node 1) tries to create a Pre-prepare
        // This should fail at creation
        let result = replica.create_pre_prepare("digest".to_string());
        assert!(
            result.is_none(),
            "non-leader should not be able to create a Pre-prepare"
        );

        // A forged Pre-prepare from node 1 should be rejected by other nodes
        let forged_msg = PrePrepareMessage {
            view_number: 0,
            sequence_number: 0,
            request_digest: "forged_digest".to_string(),
            leader_id: 1, // node 1 is not the leader for view 0
        };

        let mut other_replica = PBFTNodePrePrepare::new(2, 4);
        let accepted = other_replica.receive_pre_prepare(&forged_msg);
        assert!(
            !accepted,
            "should reject Pre-prepare from non-leader node"
        );
    }

    #[test]
    fn reject_wrong_view_number() {
        let mut replica = PBFTNodePrePrepare::new(1, 4);

        let wrong_view_msg = PrePrepareMessage {
            view_number: 5, // wrong view number
            sequence_number: 0,
            request_digest: "digest".to_string(),
            leader_id: 0,
        };

        let accepted = replica.receive_pre_prepare(&wrong_view_msg);
        assert!(
            !accepted,
            "should reject Pre-prepare with wrong view number"
        );
    }

    #[test]
    fn reject_wrong_sequence_number() {
        let mut leader = PBFTNodePrePrepare::new(0, 4);
        let mut replica = PBFTNodePrePrepare::new(1, 4);

        // Leader creates a message with sequence 0
        let msg = leader
            .create_pre_prepare("digest".to_string())
            .unwrap();

        // Replicate receives it successfully
        assert!(replica.receive_pre_prepare(&msg));

        // Now leader creates the next message (sequence 1)
        let msg2 = leader
            .create_pre_prepare("digest2".to_string())
            .unwrap();
        assert_eq!(msg2.sequence_number, 1);

        // A message with a gap (sequence 5) should be rejected
        let gap_msg = PrePrepareMessage {
            view_number: 0,
            sequence_number: 5,
            request_digest: "digest_gap".to_string(),
            leader_id: 0,
        };
        assert!(
            !replica.receive_pre_prepare(&gap_msg),
            "should reject Pre-prepare with wrong sequence number"
        );
    }
}
