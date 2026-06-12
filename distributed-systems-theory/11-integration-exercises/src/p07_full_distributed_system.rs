//! # Exercise: Full Distributed System
//!
//! ## Theory
//!
//! This exercise combines all the concepts from the previous modules into a
//! single unified distributed system. The system supports two consistency
//! modes:
//!
//! - **Strong consistency**: Uses Raft consensus (Module 01) for operations
//!   that require linearizability. All writes go through the Raft leader
//!   and are committed to a majority before acknowledged.
//!
//! - **Eventual consistency**: Uses CRDTs (Module 03) for operations that
//!   can tolerate temporary inconsistency. Writes are local and propagate
//!   asynchronously, converging via merge operations.
//!
//! The system also uses:
//! - Hybrid Logical Clocks (HLC) for ordering events across nodes
//! - Consistent hashing (Module 05) for shard routing
//! - PBFT (Module 06) as an alternative consensus for Byzantine-tolerant
//!   scenarios
//!
//! ## Proof / Intuition
//!
//! The key design principle is choosing the right consistency model for each
//! operation. Strong consistency is expensive (requires consensus round-trip)
//! but guarantees linearizability. Eventual consistency is cheap (local write)
//! but requires conflict resolution.
//!
//! The HLC combines physical time with a logical counter, ensuring that:
//! - If event A causally precedes B, then HLC(A) < HLC(B)
//! - If A and B are concurrent, their HLCs are ordered by physical time
//!
//! This allows us to detect concurrent operations and apply appropriate
//! merge strategies.
//!
//! ## Implementation Task
//!
//! 1. Implement a Hybrid Logical Clock (HLC) for event ordering
//! 2. Build a FullDistributedSystem that integrates Raft, CRDT, and
//!    consistent hashing
//! 3. Support both strong and eventual consistency modes
//! 4. Route operations based on the current consistency mode
//! 5. Verify correct behavior in both modes and mode switching
//!
//! ## Verification
//!
//! - Test strong mode: put/get operations through Raft consensus
//! - Test eventual mode: put/get operations through CRDT
//! - Test mode switching: operations route correctly after switching

use std::collections::HashMap;

// =============================================================================
// Hybrid Logical Clock (HLC)
// =============================================================================

/// A Hybrid Logical Clock that combines physical time with a logical counter.
/// Used to order events across distributed nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HLC {
    /// Physical time component (milliseconds since epoch).
    physical_time: u64,
    /// Logical counter to distinguish events at the same physical time.
    logical_time: u64,
    /// ID of the node this HLC belongs to.
    node_id: u64,
}

impl HLC {
    /// Create a new HLC for a given node.
    pub fn new(node_id: u64) -> Self {
        Self {
            physical_time: 0,
            logical_time: 0,
            node_id,
        }
    }

    /// Create an HLC with initial physical time.
    pub fn with_time(node_id: u64, physical_time: u64) -> Self {
        Self {
            physical_time,
            logical_time: 0,
            node_id,
        }
    }

    /// Increment the clock for a local event.
    /// Returns the current timestamp (physical_time, logical_time).
    pub fn increment(&mut self, current_physical: u64) -> (u64, u64) {
        if current_physical > self.physical_time {
            self.physical_time = current_physical;
            self.logical_time = 0;
        } else {
            self.logical_time += 1;
        }
        (self.physical_time, self.logical_time)
    }

    /// Update the clock when receiving a message from another node.
    /// Ensures causality: the local clock is at least as large as the
    /// received timestamp.
    pub fn update(&mut self, received: &HLC, current_physical: u64) -> (u64, u64) {
        if current_physical > self.physical_time && current_physical > received.physical_time {
            self.physical_time = current_physical;
            self.logical_time = 0;
        } else if received.physical_time > self.physical_time {
            self.physical_time = received.physical_time;
            self.logical_time = received.logical_time + 1;
        } else if received.physical_time == self.physical_time {
            self.logical_time = self.logical_time.max(received.logical_time) + 1;
        } else {
            self.logical_time += 1;
        }
        (self.physical_time, self.logical_time)
    }

    /// Get the current timestamp.
    pub fn timestamp(&self) -> (u64, u64) {
        (self.physical_time, self.logical_time)
    }

    /// Get the node ID.
    pub fn node_id(&self) -> u64 {
        self.node_id
    }

    /// Check if this HLC happened before another HLC.
    pub fn happened_before(&self, other: &HLC) -> bool {
        self.physical_time < other.physical_time
            || (self.physical_time == other.physical_time
                && self.logical_time < other.logical_time)
    }

    /// Check if this HLC is concurrent with another HLC.
    pub fn is_concurrent_with(&self, other: &HLC) -> bool {
        !self.happened_before(other) && !other.happened_before(self)
    }
}

// =============================================================================
// Simplified Raft for Strong Consistency
// =============================================================================

/// Simplified Raft state for the full system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaftMode {
    Follower,
    Candidate,
    Leader,
}

/// A simplified Raft node for strong consistency operations.
#[derive(Debug, Clone)]
pub struct SimpleRaftNode {
    pub id: u64,
    pub state: RaftMode,
    pub term: u64,
    pub log: Vec<(u64, String)>, // (term, command)
    pub commit_index: usize,
    pub state_machine: HashMap<String, String>,
    pub leader_id: Option<u64>,
    pub peers: Vec<u64>,
    pub votes_received: Vec<u64>,
}

impl SimpleRaftNode {
    pub fn new(id: u64, peers: Vec<u64>) -> Self {
        Self {
            id,
            state: RaftMode::Follower,
            term: 0,
            log: Vec::new(),
            commit_index: 0,
            state_machine: HashMap::new(),
            leader_id: None,
            peers,
            votes_received: Vec::new(),
        }
    }

    pub fn become_leader(&mut self) {
        self.state = RaftMode::Leader;
        self.leader_id = Some(self.id);
    }

    pub fn append_entry(&mut self, command: &str) -> usize {
        self.log.push((self.term, command.to_string()));
        self.log.len()
    }

    pub fn commit_entries(&mut self) {
        while self.commit_index < self.log.len() {
            let cmd = self.log[self.commit_index].1.clone();
            self.apply_command(&cmd);
            self.commit_index += 1;
        }
    }

    fn apply_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.splitn(3, ' ').collect();
        if parts.len() >= 2 {
            match parts[0] {
                "PUT" => {
                    if parts.len() >= 3 {
                        self.state_machine
                            .insert(parts[1].to_string(), parts[2].to_string());
                    }
                }
                "DELETE" => {
                    self.state_machine.remove(parts[1]);
                }
                _ => {}
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.state_machine.get(key).map(|s| s.as_str())
    }

    pub fn put(&mut self, key: &str, value: &str) {
        let cmd = format!("PUT {} {}", key, value);
        self.append_entry(&cmd);
    }

    pub fn delete(&mut self, key: &str) {
        let cmd = format!("DELETE {}", key);
        self.append_entry(&cmd);
    }

    pub fn is_leader(&self) -> bool {
        self.state == RaftMode::Leader
    }
}

// =============================================================================
// Simplified CRDT for Eventual Consistency
// =============================================================================

/// A simplified CRDT replica for eventual consistency operations.
#[derive(Debug, Clone)]
pub struct SimpleCRDTReplica {
    pub id: u64,
    pub data: HashMap<String, String>,
    /// Vector clock per key for conflict detection.
    pub clocks: HashMap<String, HashMap<u64, u64>>,
}

impl SimpleCRDTReplica {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            data: HashMap::new(),
            clocks: HashMap::new(),
        }
    }

    pub fn write(&mut self, key: &str, value: &str, hlc: &HLC) {
        let (phys, log) = hlc.timestamp();
        let clock = self
            .clocks
            .entry(key.to_string())
            .or_insert_with(HashMap::new);
        let current = clock.get(&self.id).copied().unwrap_or(0);
        if phys > current {
            clock.insert(self.id, phys);
            self.data.insert(key.to_string(), value.to_string());
        } else {
            // Use logical time as tiebreaker
            let logical_counter = clock.get(&self.id).copied().unwrap_or(0);
            if phys >= logical_counter {
                clock.insert(self.id, phys + log);
                self.data.insert(key.to_string(), value.to_string());
            }
        }
    }

    pub fn read(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub fn merge_replica(&mut self, other: &SimpleCRDTReplica) {
        for (key, value) in &other.data {
            let other_clock = other.clocks.get(key);
            let local_clock = self.clocks.get(key);

            let should_update = match (local_clock, other_clock) {
                (None, _) => true,
                (Some(local), Some(other_c)) => {
                    // LWW: compare clocks
                    let local_max: u64 = local.values().max().copied().unwrap_or(0);
                    let other_max: u64 = other_c.values().max().copied().unwrap_or(0);
                    other_max >= local_max
                }
                _ => false,
            };

            if should_update {
                self.data.insert(key.clone(), value.clone());
                if let Some(other_c) = other_clock {
                    let local_c = self
                        .clocks
                        .entry(key.clone())
                        .or_insert_with(HashMap::new);
                    for (node_id, time) in other_c {
                        let current = local_c.get(node_id).copied().unwrap_or(0);
                        if *time > current {
                            local_c.insert(*node_id, *time);
                        }
                    }
                }
            }
        }
    }
}

// =============================================================================
// Full Distributed System
// =============================================================================

/// Consistency mode for the distributed system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsistencyMode {
    /// Strong consistency via Raft consensus.
    Strong,
    /// Eventual consistency via CRDTs.
    Eventual,
}

/// A full distributed system combining Raft, CRDT, and consistent hashing.
pub struct FullDistributedSystem {
    /// Raft nodes for strong consistency.
    pub raft_nodes: Vec<SimpleRaftNode>,
    /// CRDT replicas for eventual consistency.
    pub crdt_replicas: Vec<SimpleCRDTReplica>,
    /// Hybrid Logical Clock for event ordering.
    pub hlc: HLC,
    /// Current consistency mode.
    pub mode: ConsistencyMode,
    /// Shard routing map (key prefix -> shard index).
    shard_map: HashMap<String, usize>,
    /// Number of shards.
    num_shards: usize,
}

impl FullDistributedSystem {
    /// Create a new full distributed system.
    pub fn new(node_id: u64, num_raft_nodes: usize, num_crdt_replicas: usize) -> Self {
        let mut raft_nodes = Vec::new();
        for i in 0..num_raft_nodes {
            let peers: Vec<u64> = (0..num_raft_nodes)
                .filter(|&j| j != i)
                .map(|j| j as u64)
                .collect();
            raft_nodes.push(SimpleRaftNode::new(i as u64, peers));
        }

        let mut crdt_replicas = Vec::new();
        for i in 0..num_crdt_replicas {
            crdt_replicas.push(SimpleCRDTReplica::new((num_raft_nodes + i) as u64));
        }

        // Set the first Raft node as leader
        if !raft_nodes.is_empty() {
            raft_nodes[0].become_leader();
        }

        Self {
            raft_nodes,
            crdt_replicas,
            hlc: HLC::new(node_id),
            mode: ConsistencyMode::Strong,
            shard_map: HashMap::new(),
            num_shards: num_raft_nodes.max(num_crdt_replicas).max(1),
        }
    }

    /// Put a value using strong consistency (Raft).
    pub fn put_strong(&mut self, key: &str, value: &str) -> bool {
        // Find the leader
        let leader_idx = self.raft_nodes.iter().position(|n| n.is_leader());

        if let Some(idx) = leader_idx {
            // Append to leader's log
            let cmd = format!("PUT {} {}", key, value);
            self.raft_nodes[idx].append_entry(&cmd);

            // Simulate replication: copy log to all followers
            let log = self.raft_nodes[idx].log.clone();
            for (i, node) in self.raft_nodes.iter_mut().enumerate() {
                if i != idx {
                    node.log = log.clone();
                }
            }

            // Commit on all nodes
            for node in &mut self.raft_nodes {
                node.commit_entries();
            }
            true
        } else {
            false
        }
    }

    /// Get a value using strong consistency (any node has the latest).
    pub fn get_strong(&self, key: &str) -> Option<&str> {
        // Any node should have the committed state
        self.raft_nodes
            .first()
            .and_then(|n| n.get(key))
    }

    /// Put a value using eventual consistency (CRDT).
    pub fn put_eventual(&mut self, key: &str, value: &str) {
        let hlc = self.hlc.clone();

        // Write to all CRDT replicas (local write)
        for replica in &mut self.crdt_replicas {
            replica.write(key, value, &hlc);
        }

        // Increment HLC
        self.hlc.increment(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );
    }

    /// Get a value using eventual consistency.
    pub fn get_eventual(&self, key: &str) -> Option<&str> {
        // Read from the first CRDT replica
        self.crdt_replicas
            .first()
            .and_then(|r| r.read(key))
    }

    /// Switch the consistency mode.
    pub fn switch_mode(&mut self, new_mode: ConsistencyMode) {
        self.mode = new_mode;
    }

    /// Put a value using the current consistency mode.
    pub fn put(&mut self, key: &str, value: &str) -> bool {
        match self.mode {
            ConsistencyMode::Strong => self.put_strong(key, value),
            ConsistencyMode::Eventual => {
                self.put_eventual(key, value);
                true
            }
        }
    }

    /// Get a value using the current consistency mode.
    pub fn get(&self, key: &str) -> Option<&str> {
        match self.mode {
            ConsistencyMode::Strong => self.get_strong(key),
            ConsistencyMode::Eventual => self.get_eventual(key),
        }
    }

    /// Get the current consistency mode.
    pub fn current_mode(&self) -> ConsistencyMode {
        self.mode
    }

    /// Sync CRDT replicas (simulate anti-entropy).
    pub fn sync_crdt_replicas(&mut self) {
        if self.crdt_replicas.len() < 2 {
            return;
        }

        // Collect clones first to avoid borrow issues
        let clones: Vec<SimpleCRDTReplica> = self.crdt_replicas.clone();

        // Merge each replica with the first one
        for replica in &mut self.crdt_replicas {
            for source in &clones {
                if source.id != replica.id {
                    replica.merge_replica(source);
                }
            }
        }
    }

    /// Get the number of raft nodes.
    pub fn raft_node_count(&self) -> usize {
        self.raft_nodes.len()
    }

    /// Get the number of CRDT replicas.
    pub fn crdt_replica_count(&self) -> usize {
        self.crdt_replicas.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hlc_basic_operations() {
        let mut hlc1 = HLC::new(1);
        let mut hlc2 = HLC::new(2);

        // Increment hlc1
        let ts1 = hlc1.increment(100);
        assert_eq!(ts1, (100, 0));

        // Increment hlc1 again at same physical time
        let ts2 = hlc1.increment(100);
        assert_eq!(ts2, (100, 1));

        // hlc2 receives hlc1's timestamp
        let ts3 = hlc2.update(&hlc1, 100);
        assert!(ts3.0 >= 100);
        assert!(ts3.1 > 0 || ts3.0 > 100);
    }

    #[test]
    fn test_hlc_happens_before() {
        let mut hlc1 = HLC::new(1);
        let mut hlc2 = HLC::new(2);

        hlc1.increment(100);
        hlc2.increment(200);

        assert!(hlc1.happened_before(&hlc2));
        assert!(!hlc2.happened_before(&hlc1));
    }

    #[test]
    fn test_hlc_concurrent_detection() {
        let mut hlc1 = HLC::new(1);
        let mut hlc2 = HLC::new(2);

        // Both at same physical time
        hlc1.increment(100);
        hlc2.increment(100);

        // They should be concurrent
        assert!(hlc1.is_concurrent_with(&hlc2));
    }

    #[test]
    fn test_strong_mode_put_get() {
        let mut system = FullDistributedSystem::new(0, 3, 0);
        system.switch_mode(ConsistencyMode::Strong);

        let result = system.put("name", "Alice");
        assert!(result, "Put should succeed with leader");

        let value = system.get("name");
        assert_eq!(value, Some("Alice"));
    }

    #[test]
    fn test_eventual_mode_put_get() {
        let mut system = FullDistributedSystem::new(0, 0, 3);
        system.switch_mode(ConsistencyMode::Eventual);

        system.put("city", "Portland");
        let value = system.get("city");
        assert_eq!(value, Some("Portland"));
    }

    #[test]
    fn test_mode_switching() {
        let mut system = FullDistributedSystem::new(0, 3, 3);

        // Start in strong mode
        system.switch_mode(ConsistencyMode::Strong);
        system.put("key1", "strong_value");
        assert_eq!(system.get("key1"), Some("strong_value"));
        assert_eq!(system.current_mode(), ConsistencyMode::Strong);

        // Switch to eventual mode
        system.switch_mode(ConsistencyMode::Eventual);
        system.put("key2", "eventual_value");
        assert_eq!(system.get("key2"), Some("eventual_value"));
        assert_eq!(system.current_mode(), ConsistencyMode::Eventual);

        // Switch back to strong mode
        system.switch_mode(ConsistencyMode::Strong);
        assert_eq!(system.get("key1"), Some("strong_value"));
    }

    #[test]
    fn test_crdt_merge_across_replicas() {
        let mut system = FullDistributedSystem::new(0, 0, 3);

        // Write to all replicas
        system.put_eventual("fruit", "apple");

        // Manually modify one replica to simulate divergence
        system.crdt_replicas[1]
            .data
            .insert("veggie".to_string(), "carrot".to_string());

        // Sync
        system.sync_crdt_replicas();

        // All replicas should have both values
        for replica in &system.crdt_replicas {
            assert_eq!(replica.read("fruit"), Some("apple"));
            assert_eq!(replica.read("veggie"), Some("carrot"));
        }
    }

    #[test]
    fn test_raft_multiple_operations() {
        let mut system = FullDistributedSystem::new(0, 3, 0);
        system.switch_mode(ConsistencyMode::Strong);

        system.put("x", "10");
        system.put("y", "20");
        system.put("z", "30");

        assert_eq!(system.get("x"), Some("10"));
        assert_eq!(system.get("y"), Some("20"));
        assert_eq!(system.get("z"), Some("30"));

        // Verify all raft nodes have the same state
        for node in &system.raft_nodes {
            assert_eq!(node.get("x"), Some("10"));
            assert_eq!(node.get("y"), Some("20"));
            assert_eq!(node.get("z"), Some("30"));
        }
    }
}
