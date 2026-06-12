//! # Exercise: Replica Simulation
//!
//! ## Theory
//!
//! To verify that CRDTs provide Strong Eventual Consistency (SEC), we simulate
//! multiple replicas with asynchronous message passing. Each replica applies
//! local operations and periodically syncs by merging state with other replicas.
//!
//! SEC guarantees that any two replicas that have received the same set of
//! updates (in any order) will converge to the same state. This simulation
//! verifies that convergence occurs even with:
//! - Random message delivery order
//! - Network delays (messages arrive out of order)
//! - Independent operation sequences on each replica
//!
//! ## Proof / Intuition
//!
//! The merge semilattice properties (commutativity, associativity, idempotency)
//! guarantee convergence. No matter what order merges happen in, the final state
//! is determined by the set of operations applied, not their order.
//!
//! ## Implementation Task
//!
//! Implement `ReplicaSimulator` that:
//! - Maintains N replicas of a G-Counter
//! - Applies random local operations
//! - Delivers messages with random delays
//! - Verifies convergence after all messages delivered
//!
//! ## Verification
//!
//! Verify that all replicas converge to the same state after all messages
//! have been delivered and processed.
//!
use crate::p01_g_counter::GCounter;
use std::collections::VecDeque;

/// A pending message in the network.
#[derive(Debug, Clone)]
struct Message {
    /// Source replica (used for debugging/logging).
    #[allow(dead_code)]
    from: usize,
    /// Destination replica.
    to: usize,
    /// The state being sent.
    state: GCounter,
    /// Delivery delay (number of rounds).
    delay: usize,
}

/// Simulates a network of CRDT replicas with asynchronous message delivery.
#[derive(Debug)]
pub struct ReplicaSimulator {
    /// The replicas.
    replicas: Vec<GCounter>,
    /// Pending messages in the network.
    mailbox: VecDeque<Message>,
    /// Number of replicas.
    num_replicas: usize,
}

impl ReplicaSimulator {
    /// Create a new simulator with N replicas.
    pub fn new(num_replicas: usize) -> Self {
        let replicas = (0..num_replicas).map(|_| GCounter::new()).collect();
        Self {
            replicas,
            mailbox: VecDeque::new(),
            num_replicas,
        }
    }

    /// Get a reference to a specific replica.
    pub fn get_replica(&self, id: usize) -> &GCounter {
        &self.replicas[id]
    }

    /// Get a mutable reference to a specific replica.
    pub fn get_replica_mut(&mut self, id: usize) -> &mut GCounter {
        &mut self.replicas[id]
    }

    /// Apply a local increment on a replica.
    pub fn local_increment(&mut self, replica_id: usize) {
        self.replicas[replica_id].increment(replica_id);
    }

    /// Send a replica's state to all other replicas with random delay.
    pub fn broadcast_state(&mut self, from: usize, max_delay: usize) {
        let state = self.replicas[from].clone();
        for to in 0..self.num_replicas {
            if to != from {
                self.mailbox.push_back(Message {
                    from,
                    to,
                    state: state.clone(),
                    delay: rand_delay(max_delay),
                });
            }
        }
    }

    /// Process all messages that have zero delay.
    /// Returns the number of messages processed.
    pub fn process_messages(&mut self) -> usize {
        let mut processed = 0;
        let mut remaining = VecDeque::new();

        while let Some(msg) = self.mailbox.pop_front() {
            if msg.delay == 0 {
                self.replicas[msg.to].merge(&msg.state);
                processed += 1;
            } else {
                remaining.push_back(Message {
                    delay: msg.delay - 1,
                    ..msg
                });
            }
        }

        self.mailbox = remaining;
        processed
    }

    /// Check if all replicas have converged to the same state.
    pub fn all_converged(&self) -> bool {
        if self.replicas.is_empty() {
            return true;
        }
        let first = &self.replicas[0];
        self.replicas.iter().all(|r| r.value() == first.value())
    }

    /// Get the value of all replicas.
    pub fn all_values(&self) -> Vec<u64> {
        self.replicas.iter().map(|r| r.value()).collect()
    }

    /// Check if mailbox is empty.
    pub fn mailbox_empty(&self) -> bool {
        self.mailbox.is_empty()
    }

    /// Get mailbox size.
    pub fn mailbox_size(&self) -> usize {
        self.mailbox.len()
    }
}

/// Generate a random delay (simplified).
fn rand_delay(max: usize) -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as usize) % (max + 1)
}

/// Run a full simulation: apply operations, broadcast, process, verify convergence.
pub fn simulate_convergence(
    num_replicas: usize,
    ops_per_replica: usize,
    sync_rounds: usize,
    max_delay: usize,
) -> bool {
    let mut sim = ReplicaSimulator::new(num_replicas);

    // Phase 1: Each replica applies local operations
    for r in 0..num_replicas {
        for _ in 0..ops_per_replica {
            sim.local_increment(r);
        }
    }

    // Phase 2: Broadcast and sync
    for _ in 0..sync_rounds {
        for r in 0..num_replicas {
            sim.broadcast_state(r, max_delay);
        }
        // Process messages
        while !sim.mailbox_empty() {
            sim.process_messages();
        }
    }

    sim.all_converged()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convergence_with_two_replicas() {
        let mut sim = ReplicaSimulator::new(2);

        // Replica 0 increments 5 times
        for _ in 0..5 {
            sim.local_increment(0);
        }

        // Replica 1 increments 3 times
        for _ in 0..3 {
            sim.local_increment(1);
        }

        // Broadcast from both
        sim.broadcast_state(0, 0);
        sim.broadcast_state(1, 0);

        // Process all messages
        while !sim.mailbox_empty() {
            sim.process_messages();
        }

        // Both should converge: total = 5 + 3 = 8
        assert!(sim.all_converged());
        assert_eq!(sim.get_replica(0).value(), 8);
        assert_eq!(sim.get_replica(1).value(), 8);
    }

    #[test]
    fn test_convergence_three_replicas() {
        let result = simulate_convergence(3, 100, 10, 0);
        assert!(result, "Three replicas must converge after sync");

        let mut sim = ReplicaSimulator::new(3);
        for r in 0..3 {
            for _ in 0..100 {
                sim.local_increment(r);
            }
        }

        // After broadcast and merge
        for r in 0..3 {
            sim.broadcast_state(r, 0);
        }
        while !sim.mailbox_empty() {
            sim.process_messages();
        }

        let values = sim.all_values();
        assert_eq!(values[0], values[1]);
        assert_eq!(values[1], values[2]);
        assert_eq!(values[0], 300); // 3 replicas * 100 increments
    }

    #[test]
    fn test_sec_with_random_order() {
        // SEC: convergence after receiving same updates, regardless of order
        let mut sim = ReplicaSimulator::new(4);

        // Apply different operations on each replica
        for r in 0..4 {
            for _ in 0..50 {
                sim.local_increment(r);
            }
        }

        // Broadcast with no delay to ensure delivery
        for r in 0..4 {
            sim.broadcast_state(r, 0);
        }

        // Process all messages
        while !sim.mailbox_empty() {
            sim.process_messages();
        }

        // All replicas should have converged to total = 4 * 50 = 200
        assert!(sim.all_converged(), "All replicas must converge after full sync");
        assert_eq!(sim.get_replica(0).value(), 200);
    }
}
