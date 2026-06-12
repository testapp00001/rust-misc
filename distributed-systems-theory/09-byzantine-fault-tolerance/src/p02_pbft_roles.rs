//! # Exercise: PBFT Roles
//!
//! ## Theory
//!
//! **Practical Byzantine Fault Tolerance (PBFT)**, introduced by Castro and Liskov
//! (1999), assigns distinct roles to nodes in the system:
//!
//! - **Leader (Primary)**: Responsible for ordering client requests. The leader
//!   assigns sequence numbers to incoming requests and broadcasts Pre-prepare
//!   messages. There is exactly one leader per view. The leader for view `v` is
//!   node `v mod n`.
//!
//! - **Replica**: Every node in the system is a replica. Replicas receive
//!   requests from clients, validate them, and participate in the three-phase
//!   consensus protocol (Pre-prepare, Prepare, Commit).
//!
//! The protocol uses three message types:
//!
//! 1. **Pre-prepare**: Sent by the leader to all replicas. Contains the view
//!    number, sequence number, and a digest of the request. This message
//!    proposes an ordering for the request.
//!
//! 2. **Prepare**: Sent by replicas after accepting a Pre-prepare. A request
//!    is "prepared" when a node receives 2f + 1 matching Prepare messages
//!    (including its own).
//!
//! 3. **Commit**: Sent by replicas after a request is prepared. A request
//!    is "committed" when a node receives 2f + 1 Commit messages. After
//!    committing, the request is executed.
//!
//! 4. **View-Change**: Sent when the leader is suspected of being faulty.
//!    A new leader takes over by collecting 2f + 1 view-change messages.
//!
//! ## Proof / Intuition
//!
//! The role assignment ensures a clear separation of responsibilities:
//!
//! - The leader proposes an order; replicas validate and agree on it.
//! - The three-phase protocol ensures both **safety** (all honest replicas
//!   process requests in the same order) and **liveness** (the system makes
//!   progress as long as fewer than n/3 nodes are faulty).
//!
//! The message log is the node's record of all protocol messages it has received.
//! By counting messages of each type, a node can determine whether quorum
//! thresholds have been met.
//!
//! ## Implementation Task
//!
//! Implement:
//!
//! - `PBFTRole` enum: `Leader`, `Replica`
//! - `PBFTMessage` enum with variants:
//!   - `PrePrepare { view_number, sequence_number, request_digest, node_id }`
//!   - `Prepare { view_number, sequence_number, request_digest, node_id }`
//!   - `Commit { view_number, sequence_number, request_digest, node_id }`
//!   - `ViewChange { new_view_number, node_id }`
//! - `PBFTNode` struct with fields:
//!   - `node_id: usize`
//!   - `role: PBFTRole`
//!   - `view_number: u64`
//!   - `sequence_number: u64`
//!   - `message_log: HashMap<String, Vec<PBFTMessage>>`
//!   - `prepared: bool`
//!   - `committed: bool`
//! - Methods:
//!   - `new(node_id, total_nodes)` - create a node; node 0 is leader in view 0
//!   - `is_leader()` - check if this node is the current leader
//!   - `log_message(msg)` - add a message to the log
//!   - `count_messages(msg_type, view, seq)` - count messages matching the criteria
//!
//! ## Verification
//!
//! - Verify node initialization (node 0 is leader)
//! - Verify leader election (deterministic: view % n)
//! - Verify message logging and counting

use std::collections::HashMap;

/// The role of a PBFT node in the consensus protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PBFTRole {
    /// The leader (primary) proposes an ordering for requests.
    Leader,
    /// A replica validates and agrees on the ordering proposed by the leader.
    Replica,
}

/// A message in the PBFT consensus protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PBFTMessage {
    /// Sent by the leader to propose a request with a sequence number.
    PrePrepare {
        /// The view number this message belongs to.
        view_number: u64,
        /// The sequence number assigned by the leader.
        sequence_number: u64,
        /// A digest (hash) of the request content.
        request_digest: String,
        /// The ID of the node sending this message.
        node_id: usize,
    },
    /// Sent by replicas after accepting a valid Pre-prepare.
    Prepare {
        /// The view number this message belongs to.
        view_number: u64,
        /// The sequence number being prepared.
        sequence_number: u64,
        /// A digest of the request content.
        request_digest: String,
        /// The ID of the node sending this message.
        node_id: usize,
    },
    /// Sent by replicas after a request reaches the prepared state.
    Commit {
        /// The view number this message belongs to.
        view_number: u64,
        /// The sequence number being committed.
        sequence_number: u64,
        /// A digest of the request content.
        request_digest: String,
        /// The ID of the node sending this message.
        node_id: usize,
    },
    /// Sent when the current leader is suspected of being faulty.
    ViewChange {
        /// The new view number being proposed.
        new_view_number: u64,
        /// The ID of the node sending this view change.
        node_id: usize,
    },
}

impl PBFTMessage {
    /// Return a string key identifying the message type.
    pub fn type_name(&self) -> String {
        match self {
            PBFTMessage::PrePrepare { .. } => "PrePrepare".to_string(),
            PBFTMessage::Prepare { .. } => "Prepare".to_string(),
            PBFTMessage::Commit { .. } => "Commit".to_string(),
            PBFTMessage::ViewChange { .. } => "ViewChange".to_string(),
        }
    }

    /// Return the view number of this message.
    pub fn view_number(&self) -> u64 {
        match self {
            PBFTMessage::PrePrepare { view_number, .. }
            | PBFTMessage::Prepare { view_number, .. }
            | PBFTMessage::Commit { view_number, .. } => *view_number,
            PBFTMessage::ViewChange {
                new_view_number, ..
            } => *new_view_number,
        }
    }

    /// Return the sequence number, if applicable.
    pub fn sequence_number(&self) -> Option<u64> {
        match self {
            PBFTMessage::PrePrepare { sequence_number, .. }
            | PBFTMessage::Prepare { sequence_number, .. }
            | PBFTMessage::Commit { sequence_number, .. } => Some(*sequence_number),
            PBFTMessage::ViewChange { .. } => None,
        }
    }

    /// Return the node ID of the sender.
    pub fn node_id(&self) -> usize {
        match self {
            PBFTMessage::PrePrepare { node_id, .. }
            | PBFTMessage::Prepare { node_id, .. }
            | PBFTMessage::Commit { node_id, .. }
            | PBFTMessage::ViewChange { node_id, .. } => *node_id,
        }
    }
}

/// A node in the PBFT consensus protocol.
///
/// Each node maintains a message log organized by message type. The node
/// tracks whether the current request has reached the prepared and committed
/// states.
#[derive(Debug, Clone)]
pub struct PBFTNode {
    /// Unique identifier for this node.
    pub node_id: usize,
    /// The role of this node (Leader or Replica).
    pub role: PBFTRole,
    /// Current view number.
    pub view_number: u64,
    /// Next sequence number to assign (used by the leader).
    pub sequence_number: u64,
    /// Message log organized by message type name.
    pub message_log: HashMap<String, Vec<PBFTMessage>>,
    /// Whether the current request has reached the prepared state.
    pub prepared: bool,
    /// Whether the current request has reached the committed state.
    pub committed: bool,
}

impl PBFTNode {
    /// Create a new PBFT node.
    ///
    /// Node 0 is always the leader in view 0. All other nodes start as replicas.
    pub fn new(node_id: usize, total_nodes: usize) -> Self {
        assert!(
            node_id < total_nodes,
            "node_id must be less than total_nodes"
        );
        let role = if node_id == 0 {
            PBFTRole::Leader
        } else {
            PBFTRole::Replica
        };
        Self {
            node_id,
            role,
            view_number: 0,
            sequence_number: 0,
            message_log: HashMap::new(),
            prepared: false,
            committed: false,
        }
    }

    /// Check if this node is the current leader.
    ///
    /// The leader for view `v` is node `v mod total_nodes`.
    pub fn is_leader(&self) -> bool {
        self.role == PBFTRole::Leader
    }

    /// Add a message to the node's message log.
    ///
    /// Messages are organized by their type name (e.g., "PrePrepare", "Prepare").
    pub fn log_message(&mut self, msg: PBFTMessage) {
        let type_name = msg.type_name();
        self.message_log
            .entry(type_name)
            .or_insert_with(Vec::new)
            .push(msg);
    }

    /// Count messages of a given type in the log that match the specified
    /// view number and sequence number.
    ///
    /// For `ViewChange` messages, the sequence number filter is ignored.
    pub fn count_messages(&self, msg_type: &str, view: u64, seq: u64) -> usize {
        self.message_log
            .get(msg_type)
            .map(|msgs| {
                msgs.iter()
                    .filter(|m| m.view_number() == view)
                    .filter(|m| m.sequence_number().map_or(true, |s| s == seq))
                    .count()
            })
            .unwrap_or(0)
    }

    /// Return the total number of messages in the log across all types.
    pub fn total_messages(&self) -> usize {
        self.message_log.values().map(|msgs| msgs.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_initialization() {
        let leader = PBFTNode::new(0, 4);
        assert_eq!(leader.node_id, 0);
        assert_eq!(leader.role, PBFTRole::Leader);
        assert!(leader.is_leader());
        assert_eq!(leader.view_number, 0);
        assert_eq!(leader.sequence_number, 0);
        assert!(!leader.prepared);
        assert!(!leader.committed);

        let replica = PBFTNode::new(1, 4);
        assert_eq!(replica.node_id, 1);
        assert_eq!(replica.role, PBFTRole::Replica);
        assert!(!replica.is_leader());
    }

    #[test]
    fn leader_election_is_deterministic() {
        let total_nodes = 4;
        // In view 0, node 0 is leader
        let node = PBFTNode::new(0, total_nodes);
        assert!(node.is_leader(), "node 0 should be leader in view 0");

        let node = PBFTNode::new(1, total_nodes);
        assert!(!node.is_leader(), "node 1 should not be leader in view 0");
    }

    #[test]
    fn message_logging_and_counting() {
        let mut node = PBFTNode::new(0, 4);

        // Log a Pre-prepare message
        let msg = PBFTMessage::PrePrepare {
            view_number: 0,
            sequence_number: 0,
            request_digest: "abc123".to_string(),
            node_id: 0,
        };
        node.log_message(msg);

        assert_eq!(
            node.count_messages("PrePrepare", 0, 0),
            1,
            "should count the logged Pre-prepare"
        );
        assert_eq!(
            node.count_messages("PrePrepare", 1, 0),
            0,
            "should not match wrong view"
        );
        assert_eq!(
            node.count_messages("Prepare", 0, 0),
            0,
            "should not count wrong type"
        );
    }

    #[test]
    fn multiple_messages_of_same_type() {
        let mut node = PBFTNode::new(0, 4);

        for i in 0..5 {
            node.log_message(PBFTMessage::Prepare {
                view_number: 0,
                sequence_number: 0,
                request_digest: "digest".to_string(),
                node_id: i,
            });
        }

        assert_eq!(
            node.count_messages("Prepare", 0, 0),
            5,
            "should count all 5 Prepare messages"
        );
        assert_eq!(
            node.total_messages(),
            5,
            "total messages should be 5"
        );
    }
}
