//! # Exercise: Failure Detectors
//!
//! ## Theory
//!
//! Failure detectors are a mechanism to circumvent FLP impossibility. A failure
//! detector assigns each process a suspicion status: either "alive" or "suspected."
//!
//! The key classes of failure detectors:
//!
//! - **Perfect Failure Detector (P):** Never makes mistakes. A process is suspected
//!   if and only if it has crashed. Requires synchronous system.
//!
//! - **Eventually Perfect Failure Detector (diamond-P):** Eventually stops making
//!   mistakes. After some unknown time, it correctly suspects only crashed processes.
//!
//! - **Omega Failure Detector (omega):** Eventually assigns the same "leader" to all
//!   correct processes. The weakest detector sufficient for consensus.
//!
//! ## Proof / Intuition
//!
//! The omega failure detector is the weakest failure detector that can solve consensus:
//! - It eventually stabilizes to a single leader.
//! - Once stable, all correct processes follow the leader's decision.
//! - This sidesteps FLP because the system eventually behaves synchronously with
//!   respect to the leader.
//!
//! For our implementation, we use heartbeat-based detection:
//! - Each process periodically sends heartbeats.
//!   - A detector tracks the last heartbeat time.
//!   - If no heartbeat arrives within `suspect_timeout`, the process is suspected.
//! - An unreliable detector may make false positives (suspect alive processes) but
//!   eventually becomes accurate.
//!
//! ## Implementation Task
//!
//! Implement failure detectors:
//!
//! - `FailureDetector` with `suspect_timeout` and `heartbeat_interval`.
//! - `receive_heartbeat(&mut self, node_id: NodeId)`.
//! - `is_suspected(&self, node_id: NodeId) -> bool`.
//! - `tick(&mut self)` to advance time and update suspicions.
//! - `PerfectDetector`: never false positives.
//! - `UnreliableDetector`: may false positive but eventually becomes accurate.
//!
//! ## Verification
//!
//! - Verify correct detection of crashed node.
//! - Verify false positives occur with unreliable detector and slow node.
//! - Verify eventual accuracy of the unreliable detector.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Node identifier.
pub type NodeId = u32;

/// Status of a node as tracked by the failure detector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStatus {
    /// Node is considered alive.
    Alive,
    /// Node is suspected of having crashed.
    Suspected,
    /// Node status is unknown (never received heartbeat).
    Unknown,
}

/// A heartbeat record for a node.
#[derive(Debug, Clone)]
struct HeartbeatRecord {
    /// Timestamp of last received heartbeat.
    last_heartbeat: Instant,
    /// Number of heartbeats received.
    count: u64,
}

/// A failure detector that uses heartbeat-based monitoring.
pub struct FailureDetector {
    /// How long to wait without a heartbeat before suspecting a node.
    pub suspect_timeout: Duration,
    /// Expected interval between heartbeats.
    pub heartbeat_interval: Duration,
    /// Heartbeat records for each node.
    records: HashMap<NodeId, HeartbeatRecord>,
    /// Current suspicion status for each node.
    suspicions: HashMap<NodeId, NodeStatus>,
    /// Total false positives observed (nodes suspected but actually alive).
    pub false_positives: u64,
    /// Total correct detections (crashed nodes correctly suspected).
    pub correct_detections: u64,
    /// Total heartbeat rounds elapsed.
    pub rounds_elapsed: u64,
    /// If true, the detector becomes accurate after stabilization.
    pub eventually_accurate: bool,
    /// Round at which the detector becomes accurate.
    pub stabilization_round: Option<u64>,
}

impl FailureDetector {
    /// Creates a new failure detector.
    ///
    /// # Arguments
    /// * `suspect_timeout` - Duration without heartbeat before suspecting a node.
    /// * `heartbeat_interval` - Expected interval between heartbeats.
    /// * `eventually_accurate` - If true, the detector stabilizes after some rounds.
    pub fn new(
        suspect_timeout: Duration,
        heartbeat_interval: Duration,
        eventually_accurate: bool,
    ) -> Self {
        Self {
            suspect_timeout,
            heartbeat_interval,
            records: HashMap::new(),
            suspicions: HashMap::new(),
            false_positives: 0,
            correct_detections: 0,
            rounds_elapsed: 0,
            eventually_accurate,
            stabilization_round: None,
        }
    }

    /// Record a heartbeat from a node.
    pub fn receive_heartbeat(&mut self, node_id: NodeId) {
        let now = Instant::now();
        self.records.insert(
            node_id,
            HeartbeatRecord {
                last_heartbeat: now,
                count: self
                    .records
                    .get(&node_id)
                    .map_or(1, |r| r.count + 1),
            },
        );
        self.suspicions.insert(node_id, NodeStatus::Alive);
    }

    /// Check if a node is currently suspected.
    pub fn is_suspected(&self, node_id: NodeId) -> bool {
        matches!(self.suspicions.get(&node_id), Some(NodeStatus::Suspected))
    }

    /// Get the status of a node.
    pub fn get_status(&self, node_id: NodeId) -> NodeStatus {
        self.suspicions
            .get(&node_id)
            .cloned()
            .unwrap_or(NodeStatus::Unknown)
    }

    /// Advance the detector by one heartbeat interval.
    /// Checks for missed heartbeats and updates suspicions.
    pub fn tick(&mut self) {
        self.rounds_elapsed += 1;

        // If eventually accurate mode, set stabilization round.
        if self.eventually_accurate && self.stabilization_round.is_none() {
            // Stabilize after 10 rounds.
            if self.rounds_elapsed >= 10 {
                self.stabilization_round = Some(self.rounds_elapsed);
            }
        }

        let now = Instant::now();
        for (&node_id, record) in &self.records {
            let elapsed = now.duration_since(record.last_heartbeat);
            if elapsed >= self.suspect_timeout {
                let is_actually_alive = self
                    .suspicions
                    .get(&node_id)
                    .map(|s| *s == NodeStatus::Alive)
                    .unwrap_or(false);

                if is_actually_alive {
                    // If we are stabilized and eventually accurate, don't false positive.
                    if self.eventually_accurate && self.stabilization_round.is_some() {
                        continue;
                    }
                    self.false_positives += 1;
                } else {
                    self.correct_detections += 1;
                }

                self.suspicions.insert(node_id, NodeStatus::Suspected);
            }
        }
    }

    /// Check if a node is truly crashed (for testing ground truth).
    /// In this simulation, a node is crashed if it never sent any heartbeat.
    pub fn is_truly_crashed(&self, node_id: NodeId, known_crashed: &[NodeId]) -> bool {
        known_crashed.contains(&node_id)
    }
}

/// A perfect failure detector: no false positives, no false negatives.
/// Only suspects a node if it has truly crashed.
pub struct PerfectDetector {
    /// Nodes that have crashed (ground truth).
    crashed_nodes: Vec<NodeId>,
    /// Nodes that have been heard from.
    alive_nodes: HashMap<NodeId, bool>,
}

impl PerfectDetector {
    /// Creates a new perfect detector with known crashed nodes.
    pub fn new(crashed_nodes: Vec<NodeId>) -> Self {
        Self {
            crashed_nodes,
            alive_nodes: HashMap::new(),
        }
    }

    /// Record a heartbeat from a node. Removes it from crashed list
    /// (perfect detector never falsely suspects).
    pub fn receive_heartbeat(&mut self, node_id: NodeId) {
        self.alive_nodes.insert(node_id, true);
    }

    /// Check if a node is suspected. For a perfect detector, this is accurate.
    pub fn is_suspected(&self, node_id: NodeId) -> bool {
        self.crashed_nodes.contains(&node_id)
            && !self.alive_nodes.get(&node_id).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_detector_detects_crashed_node() {
        let detector = PerfectDetector::new(vec![2, 5]);
        assert!(
            detector.is_suspected(2),
            "Perfect detector should suspect crashed node 2"
        );
        assert!(
            detector.is_suspected(5),
            "Perfect detector should suspect crashed node 5"
        );
        assert!(
            !detector.is_suspected(0),
            "Perfect detector should not suspect alive node 0"
        );
    }

    #[test]
    fn perfect_detector_no_false_positive() {
        let mut detector = PerfectDetector::new(vec![2]);
        // Node 1 is alive and sends heartbeats.
        detector.receive_heartbeat(1);
        assert!(
            !detector.is_suspected(1),
            "Perfect detector should never falsely suspect an alive node"
        );
    }

    #[test]
    fn unreliable_detector_false_positives_with_slow_node() {
        let mut detector = FailureDetector::new(
            Duration::from_millis(50),
            Duration::from_millis(10),
            false,
        );

        // Node 1 is alive but slow -- it sent one heartbeat earlier, then went quiet.
        detector.receive_heartbeat(0); // Node 0 is alive.
        detector.receive_heartbeat(1); // Node 1 is alive but will be slow from now on.

        // Simulate time passing without node 1 sending heartbeats.
        std::thread::sleep(Duration::from_millis(60));
        detector.tick();

        // Node 1 is alive but should be suspected (false positive).
        assert!(
            detector.is_suspected(1),
            "Unreliable detector should suspect a slow node"
        );
    }

    #[test]
    fn failure_detector_eventual_accuracy() {
        let mut detector = FailureDetector::new(
            Duration::from_millis(5),
            Duration::from_millis(1),
            true, // eventually accurate
        );

        // Simulate some rounds with false positives.
        for _ in 0..15 {
            detector.tick();
        }

        assert!(
            detector.stabilization_round.is_some(),
            "Detector should stabilize after enough rounds"
        );
        assert!(
            detector.stabilization_round.unwrap() <= 10,
            "Detector should stabilize by round 10"
        );
    }

    #[test]
    fn detector_tracks_heartbeat_count() {
        let mut detector = FailureDetector::new(
            Duration::from_secs(10),
            Duration::from_millis(10),
            false,
        );

        for _ in 0..5 {
            detector.receive_heartbeat(0);
        }

        // Check internal record.
        let record = detector.records.get(&0).unwrap();
        assert_eq!(record.count, 5, "Should track total heartbeat count");
    }

    #[test]
    fn detector_returns_unknown_for_unseen_node() {
        let detector = FailureDetector::new(
            Duration::from_secs(1),
            Duration::from_millis(10),
            false,
        );
        assert_eq!(
            detector.get_status(99),
            NodeStatus::Unknown,
            "Unknown node should have Unknown status"
        );
    }
}
