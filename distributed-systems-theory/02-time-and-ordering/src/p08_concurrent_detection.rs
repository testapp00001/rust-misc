//! # Exercise: Concurrent Event Detection
//!
//! ## Theory
//!
//! In a distributed system, events can be either **causally related** (one happened
//! before the other) or **concurrent** (neither happened before the other). Detecting
//! concurrency is crucial for:
//!
//! - Conflict detection in replicated databases
//! - Determining which operations can be reordered
//! - Identifying potential inconsistencies
//!
//! Two events are concurrent if and only if neither vector clock precedes the other.
//!
//! ## Proof / Intuition
//!
//! Given events a and b with vector clocks VC(a) and VC(b):
//! - a -> b iff VC(a) < VC(b) (component-wise)
//! - b -> a iff VC(b) < VC(a) (component-wise)
//! - a || b iff neither VC(a) < VC(b) nor VC(b) < VC(a)
//!
//! The third case (concurrency) occurs when:
//! - VC(a)[i] < VC(b)[i] for some i, AND VC(a)[j] > VC(b)[j] for some j
//!
//! This means each process has "seen" different things, and neither has a complete
//! causal history of the other.
//!
//! ## Implementation Task
//!
//! - Simulate 3 processes exchanging messages
//! - Track events with vector clocks
//! - Classify event pairs as concurrent or causally related
//!
//! ## Verification
//!
//! - Verify correct classification of concurrent events
//! - Verify correct classification of causal events

use super::p05_vector_clocks::VectorClock;

/// An event in a simulated distributed system.
#[derive(Debug, Clone)]
pub struct SimEvent {
    /// Process ID.
    pub process_id: usize,
    /// Event index within the process.
    pub event_index: usize,
    /// Vector clock at this event.
    pub clock: VectorClock,
    /// Description of the event.
    pub description: String,
}

/// Relationship between two events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventRelation {
    /// a happened before b.
    HappensBefore,
    /// b happened before a.
    HappenedAfter,
    /// a and b are concurrent.
    Concurrent,
    /// Same event.
    Same,
}

/// Simulate a distributed system with 3 processes exchanging messages.
pub struct DistributedSimulation {
    /// Vector clocks for each process.
    clocks: Vec<VectorClock>,
    /// All events generated during simulation.
    events: Vec<SimEvent>,
    /// Message log: (sender, receiver, vector_clock_at_send).
    messages: Vec<(usize, usize, Vec<u64>)>,
}

impl DistributedSimulation {
    /// Create a new simulation with the given number of processes.
    pub fn new(num_processes: usize) -> Self {
        let clocks = (0..num_processes)
            .map(|_| VectorClock::new(num_processes))
            .collect();
        Self {
            clocks,
            events: Vec::new(),
            messages: Vec::new(),
        }
    }

    /// Execute a local event on a process.
    pub fn local_event(&mut self, process_id: usize, description: &str) {
        self.clocks[process_id].local_event(process_id);
        self.events.push(SimEvent {
            process_id,
            event_index: self.events
                .iter()
                .filter(|e| e.process_id == process_id)
                .count(),
            clock: self.clocks[process_id].clone(),
            description: description.to_string(),
        });
    }

    /// Send a message from one process to another.
    pub fn send_message(&mut self, from: usize, to: usize, description: &str) {
        // Send event
        self.clocks[from].local_event(from);
        let send_clock = self.clocks[from].send_clock();
        self.events.push(SimEvent {
            process_id: from,
            event_index: self.events
                .iter()
                .filter(|e| e.process_id == from)
                .count(),
            clock: self.clocks[from].clone(),
            description: format!("{description} (send)"),
        });

        // Record the message
        self.messages.push((from, to, send_clock));
    }

    /// Receive a message (process the oldest undelivered message for this process).
    pub fn receive_message(&mut self, process_id: usize, description: &str) {
        // Find the first message destined for this process
        if let Some(idx) = self.messages
            .iter()
            .position(|&(_, to, _)| to == process_id)
        {
            let (_, _, send_clock) = self.messages.remove(idx);
            self.clocks[process_id].receive_clock(process_id, &send_clock);
            self.events.push(SimEvent {
                process_id,
                event_index: self.events
                    .iter()
                    .filter(|e| e.process_id == process_id)
                    .count(),
                clock: self.clocks[process_id].clone(),
                description: format!("{description} (receive)"),
            });
        }
    }

    /// Classify the relationship between two events.
    pub fn classify_relation(a: &SimEvent, b: &SimEvent) -> EventRelation {
        if a.process_id == b.process_id && a.event_index == b.event_index {
            return EventRelation::Same;
        }

        let a_precedes_b = a.clock.precedes(&b.clock.vector);
        let b_precedes_a = b.clock.precedes(&a.clock.vector);

        if a_precedes_b && !b_precedes_a {
            EventRelation::HappensBefore
        } else if b_precedes_a && !a_precedes_b {
            EventRelation::HappenedAfter
        } else {
            EventRelation::Concurrent
        }
    }

    /// Get all events.
    pub fn events(&self) -> &[SimEvent] {
        &self.events
    }

    /// Get all event pairs with their relationships.
    pub fn all_relationships(&self) -> Vec<(SimEvent, SimEvent, EventRelation)> {
        let mut results = Vec::new();
        for i in 0..self.events.len() {
            for j in (i + 1)..self.events.len() {
                let relation = Self::classify_relation(&self.events[i], &self.events[j]);
                results.push((
                    self.events[i].clone(),
                    self.events[j].clone(),
                    relation,
                ));
            }
        }
        results
    }

    /// Count concurrent pairs.
    pub fn count_concurrent_pairs(&self) -> usize {
        self.all_relationships()
            .iter()
            .filter(|(_, _, r)| *r == EventRelation::Concurrent)
            .count()
    }

    /// Count causal pairs.
    pub fn count_causal_pairs(&self) -> usize {
        self.all_relationships()
            .iter()
            .filter(|(_, _, r)| {
                *r == EventRelation::HappensBefore || *r == EventRelation::HappenedAfter
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_events_on_same_process_are_causal() {
        let mut sim = DistributedSimulation::new(2);

        sim.local_event(0, "A");
        sim.local_event(0, "B");
        sim.local_event(0, "C");

        let events = sim.events();
        let relation = DistributedSimulation::classify_relation(&events[0], &events[2]);
        assert_eq!(relation, EventRelation::HappensBefore);

        let relation_rev = DistributedSimulation::classify_relation(&events[2], &events[0]);
        assert_eq!(relation_rev, EventRelation::HappenedAfter);
    }

    #[test]
    fn events_before_message_are_causal() {
        let mut sim = DistributedSimulation::new(2);

        sim.local_event(0, "A");
        sim.send_message(0, 1, "msg1");
        sim.receive_message(1, "B");

        let events = sim.events();
        // P0:A -> P0:send -> P1:receive
        let relation = DistributedSimulation::classify_relation(&events[0], &events[2]);
        assert_eq!(relation, EventRelation::HappensBefore);
    }

    #[test]
    fn independent_events_are_concurrent() {
        let mut sim = DistributedSimulation::new(2);

        sim.local_event(0, "A");
        sim.local_event(1, "B");

        let events = sim.events();
        let relation = DistributedSimulation::classify_relation(&events[0], &events[1]);
        assert_eq!(relation, EventRelation::Concurrent);
    }

    #[test]
    fn three_process_simulation() {
        let mut sim = DistributedSimulation::new(3);

        // P0 sends to P1
        sim.send_message(0, 1, "msg01");

        // P1 sends to P2
        sim.local_event(1, "P1 local");
        sim.send_message(1, 2, "msg12");

        // P0 sends to P2
        sim.send_message(0, 2, "msg02");

        // Receive messages
        sim.receive_message(1, "P1 recv");
        sim.receive_message(2, "P2 recv1");
        sim.receive_message(2, "P2 recv2");

        // Verify we have events
        assert!(sim.events().len() > 0);

        // Verify some concurrency exists
        let _concurrent = sim.count_concurrent_pairs();
        let causal = sim.count_causal_pairs();

        // There should be both concurrent and causal relationships
        assert!(
            causal > 0,
            "there should be some causal relationships"
        );
    }

    #[test]
    fn same_event_is_same() {
        let mut sim = DistributedSimulation::new(2);
        sim.local_event(0, "X");

        let events = sim.events();
        let relation = DistributedSimulation::classify_relation(&events[0], &events[0]);
        assert_eq!(relation, EventRelation::Same);
    }

    #[test]
    fn classification_is_symmetric() {
        let mut sim = DistributedSimulation::new(2);

        sim.local_event(0, "A");
        sim.local_event(1, "B");

        let events = sim.events();
        let r1 = DistributedSimulation::classify_relation(&events[0], &events[1]);
        let r2 = DistributedSimulation::classify_relation(&events[1], &events[0]);

        match r1 {
            EventRelation::HappensBefore => assert_eq!(r2, EventRelation::HappenedAfter),
            EventRelation::HappenedAfter => assert_eq!(r2, EventRelation::HappensBefore),
            EventRelation::Concurrent => assert_eq!(r2, EventRelation::Concurrent),
            EventRelation::Same => assert_eq!(r2, EventRelation::Same),
        }
    }
}
