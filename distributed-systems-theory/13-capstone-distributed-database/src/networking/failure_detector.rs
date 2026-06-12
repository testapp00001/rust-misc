//! Heartbeat-based failure detector.
//!
//! Each node periodically sends heartbeats to its peers. The failure detector tracks
//! the last heartbeat time from each known node and transitions nodes through
//! `Alive` -> `Suspected` -> `Dead` when heartbeats are not received within the
//! configured timeout.

use std::collections::HashMap;

/// The liveness status of a node as observed by the failure detector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStatus {
    /// The node is communicating normally.
    Alive,
    /// The node has missed heartbeats but has not exceeded the second threshold.
    Suspected,
    /// The node is considered permanently unreachable.
    Dead,
}

/// A heartbeat-based failure detector that tracks the liveness of peer nodes.
#[derive(Debug)]
pub struct FailureDetector {
    /// The id of the node running this detector.
    node_id: u64,
    /// The current status of each monitored node.
    node_statuses: HashMap<u64, NodeStatus>,
    /// The last heartbeat timestamp received from each node.
    heartbeats: HashMap<u64, u64>,
    /// The time threshold (in arbitrary units) after which a node is suspected/dead.
    timeout: u64,
    /// The current simulated time.
    current_time: u64,
}

impl FailureDetector {
    /// Create a new failure detector for the given node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node running this detector.
    /// * `timeout` - The heartbeat timeout threshold.
    pub fn new(node_id: u64, timeout: u64) -> Self {
        Self {
            node_id,
            node_statuses: HashMap::new(),
            heartbeats: HashMap::new(),
            timeout,
            current_time: 0,
        }
    }

    /// Register a node to be monitored.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to monitor.
    pub fn register_node(&mut self, node_id: u64) {
        self.node_statuses.insert(node_id, NodeStatus::Alive);
        self.heartbeats.insert(node_id, self.current_time);
    }

    /// Record a heartbeat received from a node.
    ///
    /// Resets the node's status to `Alive` and updates its last heartbeat time.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node that sent the heartbeat.
    /// * `time` - The current time when the heartbeat was received.
    pub fn receive_heartbeat(&mut self, node_id: u64, time: u64) {
        self.heartbeats.insert(node_id, time);
        self.node_statuses.insert(node_id, NodeStatus::Alive);
    }

    /// Check all monitored nodes for failures based on the current time.
    ///
    /// If a node has not sent a heartbeat within `timeout` units of `current_time`,
    /// it is marked as `Suspected`. If it exceeds twice the timeout, it is marked
    /// as `Dead`.
    ///
    /// # Arguments
    ///
    /// * `current_time` - The current simulated time.
    pub fn check_failures(&mut self, current_time: u64) {
        self.current_time = current_time;
        for (&node_id, &last_hb) in &self.heartbeats {
            let elapsed = current_time.saturating_sub(last_hb);
            if elapsed > self.timeout * 2 {
                self.node_statuses.insert(node_id, NodeStatus::Dead);
            } else if elapsed > self.timeout {
                self.node_statuses.insert(node_id, NodeStatus::Suspected);
            }
        }
    }

    /// Return the current status of a monitored node.
    ///
    /// Defaults to `Alive` if the node has not been registered.
    pub fn get_status(&self, node_id: u64) -> NodeStatus {
        self.node_statuses
            .get(&node_id)
            .cloned()
            .unwrap_or(NodeStatus::Alive)
    }

    /// Return `true` if the given node is currently `Alive`.
    pub fn is_alive(&self, node_id: u64) -> bool {
        self.get_status(node_id) == NodeStatus::Alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_status() {
        let mut fd = FailureDetector::new(1, 10);
        fd.register_node(2);
        assert!(fd.is_alive(2));
    }

    #[test]
    fn test_heartbeat_resets_status() {
        let mut fd = FailureDetector::new(1, 5);
        fd.register_node(2);

        // Simulate time passing to make the node suspected.
        fd.check_failures(10);
        assert_eq!(fd.get_status(2), NodeStatus::Suspected);

        // Heartbeat resets to alive.
        fd.receive_heartbeat(2, 11);
        assert!(fd.is_alive(2));
    }

    #[test]
    fn test_suspected_after_timeout() {
        let mut fd = FailureDetector::new(1, 5);
        fd.register_node(2);
        fd.receive_heartbeat(2, 0);

        fd.check_failures(6);
        assert_eq!(fd.get_status(2), NodeStatus::Suspected);
    }

    #[test]
    fn test_dead_after_double_timeout() {
        let mut fd = FailureDetector::new(1, 5);
        fd.register_node(2);
        fd.receive_heartbeat(2, 0);

        fd.check_failures(20);
        assert_eq!(fd.get_status(2), NodeStatus::Dead);
    }

    #[test]
    fn test_alive_within_timeout() {
        let mut fd = FailureDetector::new(1, 10);
        fd.register_node(2);
        fd.receive_heartbeat(2, 0);

        fd.check_failures(5);
        assert!(fd.is_alive(2));
    }

    #[test]
    fn test_unregistered_node_defaults_alive() {
        let fd = FailureDetector::new(1, 10);
        assert!(fd.is_alive(999));
    }
}
