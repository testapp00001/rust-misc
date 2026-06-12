//! # Exercise: PBFT View Change
//!
//! ## Theory
//!
//! **Practical Byzantine Fault Tolerance (PBFT)**, introduced by Castro and Liskov
//! (1999), achieves consensus even in the presence of Byzantine (arbitrarily faulty)
//! nodes, provided that fewer than one-third of the total nodes are faulty.
//!
//! The **view change** mechanism is PBFT's fault-recovery protocol. When the
//! current leader (called the "primary") is suspected of being faulty -- either
//! because it is not issuing proposals, or because it is issuing incorrect ones --
//! any replica can initiate a view change. Each view is numbered, and the new
//! primary for view `v+1` is node `v+1 mod n`.
//!
//! The protocol works as follows:
//! 1. A replica suspects the leader is faulty (e.g., timeout).
//! 2. It sends a `VIEW-CHANGE` message to all other replicas with the new view number.
//! 3. Once a replica collects `2f + 1` VIEW-CHANGE messages for the next view,
//!    it knows a quorum agrees the leader is faulty.
//! 4. The new leader for the next view collects `2f + 1` view changes and
//!    broadcasts a NEW-VIEW message, completing the transition.
//!
//! ## Proof / Intuition
//!
//! **Safety**: The view change protocol ensures that no committed decisions are
//! lost. The new leader must include in its NEW-VIEW message the set of requests
//! it received, and replicas can verify these against their own logs.
//!
//! **Liveness**: In a synchronous system with fewer than `f < n/3` faulty nodes,
//! at least one correct node will eventually suspect the faulty leader and
//! initiate a view change. Since at most `f` nodes are faulty, at least `n - f`
//! nodes are correct, which gives us at least `2f + 1` correct nodes (when
//! `n >= 3f + 1`), ensuring the quorum is reached.
//!
//! ## Implementation Task
//!
//! Implement `PBFTViewChange` with:
//! - `new(node_id, total_nodes)` - initialize a node
//! - `detect_leader_timeout()` - track timeouts and detect leader failure
//! - `initiate_view_change()` - create a view change message
//! - `receive_view_change(msg)` - process incoming view change messages
//! - `becomes_new_leader()` - check if this node should become the new leader
//! - `get_current_view()` - return the current view number
//!
//! ## Verification
//!
//! - Verify view change triggers after sufficient timeout calls
//! - Verify new leader takes over after collecting 2f+1 view changes
//! - Verify system resumes with incremented view number

/// A message sent when a node suspects the leader is faulty and wants to
/// transition to a new view.
#[derive(Debug, Clone)]
pub struct ViewChangeMessage {
    /// The view number the sender is proposing.
    pub new_view_number: u64,
    /// The ID of the node sending this view change.
    pub node_id: usize,
    /// The last sequence number this node had committed (used by new leader
    /// to reconstruct state).
    pub last_committed_sequence: usize,
}

/// A PBFT replica node participating in the view change protocol.
#[derive(Debug, Clone)]
pub struct PBFTViewChange {
    /// This node's identifier.
    pub node_id: usize,
    /// Total number of nodes in the system.
    pub total_nodes: usize,
    /// The current consensus view number.
    pub current_view: u64,
    /// Whether this node is the current leader for this view.
    pub is_leader: bool,
    /// The node suspected of being faulty, if any.
    pub suspected_leader: Option<usize>,
    /// Collection of view change messages received for the next view.
    pub view_change_received: Vec<ViewChangeMessage>,
    /// Consecutive timeout count toward leader suspicion threshold.
    pub timeout_counter: u32,
}

/// Number of consecutive timeouts before a node suspects the leader.
const TIMEOUT_THRESHOLD: u32 = 3;

impl PBFTViewChange {
    /// Create a new PBFT view change participant.
    ///
    /// The leader for view `v` is node `v % total_nodes`.
    pub fn new(node_id: usize, total_nodes: usize) -> Self {
        let current_view = 0;
        let is_leader = (current_view as usize) % total_nodes == node_id;
        Self {
            node_id,
            total_nodes,
            current_view,
            is_leader,
            suspected_leader: None,
            view_change_received: Vec::new(),
            timeout_counter: 0,
        }
    }

    /// Record a leader timeout. Returns `true` when the timeout threshold
    /// has been exceeded and the leader should be suspected.
    pub fn detect_leader_timeout(&mut self) -> bool {
        self.timeout_counter += 1;
        if self.timeout_counter >= TIMEOUT_THRESHOLD {
            self.suspected_leader = Some(self.current_leader_id());
            true
        } else {
            false
        }
    }

    /// Initiate a view change by creating a `ViewChangeMessage` for the
    /// next view number.
    pub fn initiate_view_change(&mut self) -> ViewChangeMessage {
        let new_view = self.current_view + 1;
        ViewChangeMessage {
            new_view_number: new_view,
            node_id: self.node_id,
            last_committed_sequence: 0,
        }
    }

    /// Receive a view change message from another node. Returns `true` if
    /// collecting this message brings the node to quorum (2f+1) for the next view.
    pub fn receive_view_change(&mut self, msg: ViewChangeMessage) -> bool {
        if msg.new_view_number == self.current_view + 1 {
            self.view_change_received.push(msg);
        }
        self.has_quorum()
    }

    /// Check whether this node should become the new leader.
    ///
    /// A node becomes the new leader if:
    /// 1. It is the designated leader for the next view (`(current_view+1) % n`).
    /// 2. It has collected at least `2f + 1` view change messages.
    pub fn becomes_new_leader(&self) -> bool {
        let next_view_leader = ((self.current_view + 1) as usize) % self.total_nodes;
        self.node_id == next_view_leader && self.has_quorum()
    }

    /// Get the current view number.
    pub fn get_current_view(&self) -> u64 {
        self.current_view
    }

    /// Transition to the next view. Called after the new leader broadcasts
    /// a NEW-VIEW message.
    pub fn advance_view(&mut self) {
        self.current_view += 1;
        self.is_leader =
            (self.current_view as usize) % self.total_nodes == self.node_id;
        self.view_change_received.clear();
        self.timeout_counter = 0;
        self.suspected_leader = None;
    }

    /// Calculate the node ID of the leader for a given view.
    fn current_leader_id(&self) -> usize {
        (self.current_view as usize) % self.total_nodes
    }

    /// Check whether we have collected a quorum of `2f + 1` view changes.
    /// With `n` total nodes and at most `f` Byzantine faults, `f = (n - 1) / 3`.
    fn has_quorum(&self) -> bool {
        let f = (self.total_nodes - 1) / 3;
        let quorum = 2 * f + 1;
        self.view_change_received.len() >= quorum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_change_triggers_after_timeout() {
        let mut node = PBFTViewChange::new(0, 4);

        // Initially no suspicion
        assert!(!node.detect_leader_timeout());
        assert!(!node.detect_leader_timeout());
        assert!(node.suspected_leader.is_none());

        // Third timeout triggers suspicion
        assert!(node.detect_leader_timeout());
        assert!(node.suspected_leader.is_some());
        // Leader of view 0 with 4 nodes is node 0
        assert_eq!(node.suspected_leader.unwrap(), 0);
    }

    #[test]
    fn new_leader_takes_over_after_quorum() {
        // 4 nodes: f = (4-1)/3 = 1, quorum = 2*1+1 = 3
        let mut leader_node = PBFTViewChange::new(1, 4);

        // Node 1 is not leader for view 0 (leader is node 0)
        assert!(!leader_node.is_leader);

        // Simulate receiving view change messages from nodes 0, 1, 2
        let msg0 = ViewChangeMessage {
            new_view_number: 1,
            node_id: 0,
            last_committed_sequence: 0,
        };
        let msg1 = ViewChangeMessage {
            new_view_number: 1,
            node_id: 1,
            last_committed_sequence: 0,
        };
        let msg2 = ViewChangeMessage {
            new_view_number: 1,
            node_id: 2,
            last_committed_sequence: 0,
        };

        assert!(!leader_node.receive_view_change(msg0));
        assert!(!leader_node.receive_view_change(msg1));

        // After 3 view changes (2f+1 = 3), quorum is reached
        assert!(leader_node.receive_view_change(msg2));

        // Node 1 is the designated leader for view 1 (1 % 4 = 1)
        assert!(leader_node.becomes_new_leader());
    }

    #[test]
    fn system_resumes_with_new_view_number() {
        let mut node = PBFTViewChange::new(2, 4);
        assert_eq!(node.get_current_view(), 0);

        // Trigger timeout and view change
        node.detect_leader_timeout();
        node.detect_leader_timeout();
        node.detect_leader_timeout();

        let msg = node.initiate_view_change();
        assert_eq!(msg.new_view_number, 1);

        node.advance_view();
        assert_eq!(node.get_current_view(), 1);
        // After advancing, timeouts are reset
        assert_eq!(node.timeout_counter, 0);
        assert!(node.suspected_leader.is_none());
    }

    #[test]
    fn quorum_requires_two_f_plus_one() {
        // With 7 nodes: f = (7-1)/3 = 2, quorum = 2*2+1 = 5
        let mut node = PBFTViewChange::new(0, 7);

        for i in 1..=4 {
            let msg = ViewChangeMessage {
                new_view_number: 1,
                node_id: i,
                last_committed_sequence: 0,
            };
            assert!(!node.receive_view_change(msg), "should not have quorum at {i}");
        }

        let msg = ViewChangeMessage {
            new_view_number: 1,
            node_id: 5,
            last_committed_sequence: 0,
        };
        assert!(node.receive_view_change(msg), "should reach quorum at 5 messages");
    }

    #[test]
    fn wrong_view_number_ignored() {
        let mut node = PBFTViewChange::new(0, 4);

        let msg = ViewChangeMessage {
            new_view_number: 5, // wrong view
            node_id: 1,
            last_committed_sequence: 0,
        };

        assert!(!node.receive_view_change(msg));
        assert!(node.view_change_received.is_empty());
    }
}
