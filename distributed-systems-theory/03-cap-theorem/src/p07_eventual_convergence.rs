//! # Exercise: Eventual Convergence
//!
//! ## Theory
//!
//! Eventual consistency is the weakest consistency guarantee: if no new writes
//! are performed, all replicas will eventually converge to the same state.
//! The key question is: how long does convergence take?
//!
//! Convergence time depends on:
//! - **Message delay:** Higher latency means slower propagation.
//! - **Number of replicas:** More replicas means more messages to deliver.
//! - **Conflict resolution strategy:** LWW converges immediately once all
//!   messages are delivered; more complex strategies may require additional
//!   rounds.
//!
//! In an eventually consistent system, replicas independently apply updates
//! and periodically exchange their state. The system is correct as long as
//! convergence is guaranteed given enough time.
//!
//! ## Proof / Intuition
//!
//! Convergence can be proven by induction on the number of message delivery
//! rounds. In each round, every replica shares its state with at least one
//! other replica. After enough rounds, all replicas have seen all updates.
//! With deterministic conflict resolution (like LWW), this guarantees
//! convergence.
//!
//! The maximum convergence time is bounded by `D * (N - 1)` where D is the
//! maximum message delay and N is the number of replicas, assuming each
//! round delivers messages between one pair of replicas.
//!
//! ## Implementation Task
//!
//! Implement a simulation of eventual convergence:
//! - 3 replicas with independent local state
//! - Delayed message delivery (messages arrive after a configurable delay)
//! - `tick()` method to advance the simulation clock
//! - `measure_convergence_time() -> Option<u64>` to find when all replicas agree
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Replicas eventually converge
//! - Convergence time depends on message delay
//! - All replicas have the same final state

use std::collections::HashMap;

/// A value with a logical timestamp for conflict resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedValue {
    pub value: String,
    pub timestamp: u64,
}

/// A replica with local state and a message inbox.
#[derive(Debug, Clone)]
pub struct Replica {
    pub id: usize,
    pub store: HashMap<String, VersionedValue>,
    /// Pending messages: (key, value, timestamp).
    inbox: Vec<(String, VersionedValue)>,
}

impl Replica {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            store: HashMap::new(),
            inbox: Vec::new(),
        }
    }

    /// Apply a local write.
    pub fn put(&mut self, key: &str, value: &str, timestamp: u64) {
        self.store.insert(
            key.to_string(),
            VersionedValue {
                value: value.to_string(),
                timestamp,
            },
        );
    }

    /// Queue an incoming message for later processing.
    pub fn receive(&mut self, key: String, value: VersionedValue) {
        self.inbox.push((key, value));
    }

    /// Process all pending messages using last-write-wins.
    pub fn process_inbox(&mut self) {
        let messages: Vec<_> = self.inbox.drain(..).collect();
        for (key, remote) in messages {
            match self.store.get(&key) {
                Some(local) if local.timestamp >= remote.timestamp => {
                    // Keep local; no-op
                }
                _ => {
                    self.store.insert(key, remote);
                }
            }
        }
    }

    /// Generate a sync message for a given key.
    pub fn get_sync_data(&self, key: &str) -> Option<(String, VersionedValue)> {
        self.store.get(key).map(|v| (key.to_string(), v.clone()))
    }

    /// Check if this replica's store matches another's.
    pub fn store_matches(&self, other: &Replica) -> bool {
        self.store == other.store
    }
}

/// Simulation of eventual convergence with delayed message delivery.
pub struct ConvergenceSimulation {
    replicas: Vec<Replica>,
    /// Pending deliveries: (delay_remaining, from_id, key, value).
    pending: Vec<(u64, usize, String, VersionedValue)>,
    /// Current simulation clock.
    clock: u64,
    /// Message delivery delay in ticks.
    delivery_delay: u64,
    /// Snapshot of clock when convergence was first detected.
    convergence_time: Option<u64>,
}

impl ConvergenceSimulation {
    /// Create a new simulation with the given number of replicas and delay.
    pub fn new(num_replicas: usize, delivery_delay: u64) -> Self {
        let replicas = (0..num_replicas).map(Replica::new).collect();
        Self {
            replicas,
            pending: Vec::new(),
            clock: 0,
            delivery_delay,
            convergence_time: None,
        }
    }

    /// Create a 3-node simulation (the standard case).
    pub fn three_node(delivery_delay: u64) -> Self {
        Self::new(3, delivery_delay)
    }

    /// Write to a specific replica.
    pub fn put(&mut self, replica_id: usize, key: &str, value: &str) {
        let ts = self.clock;
        self.replicas[replica_id].put(key, value, ts);
    }

    /// Broadcast a value from one replica to all others (with delay).
    pub fn broadcast(&mut self, from_id: usize, key: &str) {
        let data = self.replicas[from_id].get_sync_data(key);
        if let Some((k, v)) = data {
            for replica in &self.replicas {
                if replica.id != from_id {
                    self.pending.push((
                        self.delivery_delay,
                        from_id,
                        k.clone(),
                        v.clone(),
                    ));
                }
            }
        }
    }

    /// Advance the simulation clock by one tick.
    /// Delivers any messages whose delay has expired.
    pub fn tick(&mut self) {
        self.clock += 1;

        let mut delivered = Vec::new();
        let mut remaining = Vec::new();

        for (delay, from_id, key, value) in self.pending.drain(..) {
            if delay <= 1 {
                delivered.push((from_id, key, value));
            } else {
                remaining.push((delay - 1, from_id, key, value));
            }
        }
        self.pending = remaining;

        for (_from_id, key, value) in delivered {
            for replica in &mut self.replicas {
                replica.receive(key.clone(), value.clone());
            }
        }

        // Process inboxes
        for replica in &mut self.replicas {
            replica.process_inbox();
        }

        // Check convergence
        if self.convergence_time.is_none() && self.is_converged() {
            self.convergence_time = Some(self.clock);
        }
    }

    /// Check if all replicas have the same store state.
    pub fn is_converged(&self) -> bool {
        if self.replicas.is_empty() {
            return true;
        }
        let first = &self.replicas[0];
        self.replicas[1..].iter().all(|r| r.store_matches(first))
    }

    /// Return the clock tick when convergence was first detected.
    pub fn convergence_time(&self) -> Option<u64> {
        self.convergence_time
    }

    /// Return the current clock tick.
    pub fn current_time(&self) -> u64 {
        self.clock
    }

    /// Return a reference to all replicas.
    pub fn replicas(&self) -> &[Replica] {
        &self.replicas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converged_when_no_writes() {
        let sim = ConvergenceSimulation::three_node(1);
        assert!(sim.is_converged());
    }

    #[test]
    fn converges_after_delivery() {
        let mut sim = ConvergenceSimulation::three_node(1);

        // Write to replica 0 and broadcast
        sim.put(0, "x", "hello");
        sim.broadcast(0, "x");
        // Not yet converged (message pending)
        assert!(!sim.is_converged());

        // Tick enough times for delivery
        for _ in 0..3 {
            sim.tick();
        }
        assert!(sim.is_converged());
    }

    #[test]
    fn convergence_time_depends_on_delay() {
        let mut sim_fast = ConvergenceSimulation::three_node(1);
        let mut sim_slow = ConvergenceSimulation::three_node(5);

        // Write and broadcast on both
        sim_fast.put(0, "x", "v");
        sim_fast.broadcast(0, "x");
        sim_slow.put(0, "x", "v");
        sim_slow.broadcast(0, "x");

        // Run until converged
        for _ in 0..20 {
            sim_fast.tick();
        }
        for _ in 0..20 {
            sim_slow.tick();
        }

        let fast_time = sim_fast.convergence_time().unwrap();
        let slow_time = sim_slow.convergence_time().unwrap();

        assert!(
            fast_time < slow_time,
            "faster delay should converge sooner: fast={fast_time}, slow={slow_time}"
        );
    }

    #[test]
    fn final_state_is_consistent() {
        let mut sim = ConvergenceSimulation::three_node(2);

        sim.put(0, "a", "1");
        sim.put(1, "b", "2");
        sim.broadcast(0, "a");
        sim.broadcast(1, "b");

        for _ in 0..20 {
            sim.tick();
        }

        assert!(sim.is_converged());
        // All replicas should have both keys
        for replica in sim.replicas() {
            assert_eq!(replica.store.get("a").map(|v| &v.value), Some(&"1".to_string()));
            assert_eq!(replica.store.get("b").map(|v| &v.value), Some(&"2".to_string()));
        }
    }

    #[test]
    fn conflicting_writes_resolve_via_lww() {
        let mut sim = ConvergenceSimulation::three_node(1);

        // Write different values to different replicas at different times
        sim.put(0, "x", "from_0");
        sim.broadcast(0, "x");

        sim.tick(); // clock advances

        sim.put(1, "x", "from_1");
        sim.broadcast(1, "x");

        // Run until converged
        for _ in 0..20 {
            sim.tick();
        }

        assert!(sim.is_converged());
        // The later write (from_1) should win via LWW
        for replica in sim.replicas() {
            assert_eq!(
                replica.store.get("x").map(|v| &v.value),
                Some(&"from_1".to_string())
            );
        }
    }
}
