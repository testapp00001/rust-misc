//! # Exercise: Eventual Convergence Simulator
//!
//! ## Theory
//!
//! Eventual consistency (or eventual convergence) guarantees that if no new
//! updates are made to the system, all replicas will eventually converge to the
//! same state. This is the weakest consistency model commonly offered by
//! distributed datastores.
//!
//! Convergence is achieved through anti-entropy protocols such as:
//! - Gossip protocols: each replica periodically shares its state with random
//!   peers.
//! - Read repair: on reads, discrepancies are detected and repaired.
//! - Merkle trees: efficient comparison of replica states.
//!
//! ## Proof / Intuition
//!
//! Consider three replicas R1, R2, R3. If we write "x=1" to R1, "y=2" to R2,
//! and then allow all messages to be delivered, every replica should eventually
//! hold both {"x": "1", "y": "2"}.
//!
//! The convergence time depends on the message delivery delay. With synchronous
//! delivery, convergence is immediate. With asynchronous delivery, it takes
//! ceil(log2(n)) rounds of gossip in the best case.
//!
//! ## Implementation Task
//!
//! Implement a `ConvergenceTest` that:
//! 1. Creates replicas.
//! 2. Writes values to individual replicas.
//! 3. Delivers messages with optional delays.
//! 4. Checks that all replicas converge to the same state.
//! 5. Simulates convergence over multiple rounds.
//!
//! ## Verification
//!
//! Run `cargo test eventual_convergence` to verify your implementation.

use std::collections::HashMap;

/// A replica node that stores key-value data.
#[derive(Debug, Clone)]
pub struct Replica {
    /// Unique identifier for this replica.
    pub id: usize,
    /// The key-value data store.
    pub data: HashMap<String, String>,
    /// Messages waiting to be delivered to this replica.
    pub pending_messages: Vec<Message>,
}

impl Replica {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            data: HashMap::new(),
            pending_messages: Vec::new(),
        }
    }

    /// Apply a message's data to this replica (merge).
    pub fn apply_message(&mut self, msg: &Message) {
        for (k, v) in &msg.data {
            self.data.insert(k.clone(), v.clone());
        }
    }
}

/// A message carrying state between replicas.
#[derive(Debug, Clone)]
pub struct Message {
    /// The source replica.
    pub from: usize,
    /// The destination replica.
    pub to: usize,
    /// The data payload (key-value pairs).
    pub data: HashMap<String, String>,
    /// Simulated delay in rounds before this message can be delivered.
    pub delay: u64,
}

/// A test harness for simulating eventual convergence.
#[derive(Debug)]
pub struct ConvergenceTest {
    /// All replicas in the system.
    pub replicas: HashMap<usize, Replica>,
    /// Messages waiting to be delivered, with remaining delay.
    pub message_queue: Vec<(Message, u64)>,
}

impl ConvergenceTest {
    /// Create a new convergence test.
    pub fn new() -> Self {
        Self {
            replicas: HashMap::new(),
            message_queue: Vec::new(),
        }
    }

    /// Add a replica to the system.
    pub fn add_replica(&mut self, id: usize) {
        self.replicas.insert(id, Replica::new(id));
    }

    /// Write a value to a specific replica.
    /// Returns the list of replica IDs that should receive the update
    /// (all other replicas).
    pub fn write_to(&mut self, replica_id: usize, key: &str, value: &str) -> Vec<usize> {
        if let Some(replica) = self.replicas.get_mut(&replica_id) {
            replica
                .data
                .insert(key.to_string(), value.to_string());
        }

        // Return the IDs of all other replicas (they need to receive the update).
        self.replicas
            .keys()
            .copied()
            .filter(|&id| id != replica_id)
            .collect()
    }

    /// Enqueue a message for delivery.
    pub fn send_message(&mut self, msg: Message) {
        self.message_queue.push((msg.clone(), msg.delay));
    }

    /// Deliver all messages that have zero remaining delay.
    /// Messages with non-zero delay have their delay decremented.
    /// Returns the number of messages delivered.
    pub fn deliver_messages(&mut self) -> usize {
        let mut delivered = 0;
        let mut remaining = Vec::new();

        for (msg, delay) in self.message_queue.drain(..) {
            if delay == 0 {
                if let Some(replica) = self.replicas.get_mut(&msg.to) {
                    replica.apply_message(&msg);
                    delivered += 1;
                }
            } else {
                remaining.push((msg, delay - 1));
            }
        }

        self.message_queue = remaining;
        delivered
    }

    /// Check if all replicas have converged to the same state.
    pub fn check_convergence(&self) -> bool {
        let mut states: Vec<HashMap<String, String>> = self
            .replicas
            .values()
            .map(|r| r.data.clone())
            .collect();

        if states.is_empty() {
            return true;
        }

        states.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        let first = &states[0];
        states.iter().all(|s| s == first)
    }

    /// Run the convergence simulation for a maximum number of rounds.
    /// Returns (converged, rounds_taken).
    pub fn simulate(&mut self, max_rounds: u64) -> (bool, u64) {
        for round in 0..max_rounds {
            // Deliver messages.
            self.deliver_messages();

            // Check convergence.
            if self.check_convergence() {
                return (true, round + 1);
            }
        }

        (false, max_rounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_message(from: usize, to: usize, data: HashMap<String, String>) -> Message {
        Message {
            from,
            to,
            data,
            delay: 0,
        }
    }

    #[test]
    fn test_all_replicas_converge_after_delivery() {
        let mut test = ConvergenceTest::new();
        test.add_replica(0);
        test.add_replica(1);
        test.add_replica(2);

        // Write to replica 0 and send to others.
        let affected = test.write_to(0, "x", "1");
        let mut data = HashMap::new();
        data.insert("x".to_string(), "1".to_string());

        for &target in &affected {
            let msg = make_message(0, target, data.clone());
            test.send_message(msg);
        }

        // Deliver all messages.
        let delivered = test.deliver_messages();
        assert_eq!(delivered, 2);

        // All replicas should now have x=1.
        assert!(test.check_convergence());
        for replica in test.replicas.values() {
            assert_eq!(replica.data.get("x").unwrap(), "1");
        }
    }

    #[test]
    fn test_convergence_time_depends_on_delay() {
        let mut test = ConvergenceTest::new();
        test.add_replica(0);
        test.add_replica(1);

        // Write to replica 0, send message with delay=2.
        let affected = test.write_to(0, "x", "1");
        let mut data = HashMap::new();
        data.insert("x".to_string(), "1".to_string());

        for &target in &affected {
            let msg = Message {
                from: 0,
                to: target,
                data: data.clone(),
                delay: 2,
            };
            test.send_message(msg);
        }

        // Round 1: delay decrements to 1, no delivery.
        let delivered = test.deliver_messages();
        assert_eq!(delivered, 0);
        assert!(!test.check_convergence());

        // Round 2: delay decrements to 0, still not delivered.
        let delivered = test.deliver_messages();
        assert_eq!(delivered, 0);
        assert!(!test.check_convergence());

        // Round 3: delay is 0, message delivered.
        let delivered = test.deliver_messages();
        assert_eq!(delivered, 1);
        assert!(test.check_convergence());
    }

    #[test]
    fn test_writes_on_different_replicas_converge() {
        let mut test = ConvergenceTest::new();
        test.add_replica(0);
        test.add_replica(1);
        test.add_replica(2);

        // Write different values to different replicas.
        test.write_to(0, "x", "1");
        test.write_to(1, "y", "2");
        test.write_to(2, "z", "3");

        // Send all-state to all replicas (gossip).
        let replica_ids: Vec<usize> = test.replicas.keys().copied().collect();
        for &source in &replica_ids {
            let source_data = test.replicas[&source].data.clone();
            for &target in &replica_ids {
                if source != target {
                    let msg = Message {
                        from: source,
                        to: target,
                        data: source_data.clone(),
                        delay: 0,
                    };
                    test.send_message(msg);
                }
            }
        }

        // Deliver all messages.
        test.deliver_messages();

        // After one round of gossip, all replicas should have all three keys.
        assert!(test.check_convergence());
        for replica in test.replicas.values() {
            assert_eq!(replica.data.len(), 3);
            assert_eq!(replica.data.get("x").unwrap(), "1");
            assert_eq!(replica.data.get("y").unwrap(), "2");
            assert_eq!(replica.data.get("z").unwrap(), "3");
        }
    }

    #[test]
    fn test_simulate_converges() {
        let mut test = ConvergenceTest::new();
        test.add_replica(0);
        test.add_replica(1);

        // Write to replica 0 with a delay=1 message.
        let affected = test.write_to(0, "x", "42");
        let mut data = HashMap::new();
        data.insert("x".to_string(), "42".to_string());
        for &target in &affected {
            let msg = Message {
                from: 0,
                to: target,
                data: data.clone(),
                delay: 1,
            };
            test.send_message(msg);
        }

        let (converged, rounds) = test.simulate(10);
        assert!(converged);
        assert!(rounds >= 1);
    }
}
