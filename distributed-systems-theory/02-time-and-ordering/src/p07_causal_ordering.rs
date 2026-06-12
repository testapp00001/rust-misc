//! # Exercise: Causal Ordering of Events
//!
//! ## Theory
//!
//! **Causal ordering** ensures that events are processed in an order that respects
//! the happened-before relation. If event a causally precedes event b (a -> b), then
//! all processes must observe a before b.
//!
//! In a distributed system, events from different processes may arrive in arbitrary
//! order due to network delays. Causal ordering requires buffering events until
//! all their causal predecessors have been processed, then performing a topological
//! sort.
//!
//! ## Proof / Intuition
//!
//! Given a set of events with vector clocks, we can determine the causal partial
//! order. A topological sort of this partial order gives a valid causal ordering.
//!
//! Key properties:
//! - If a -> b, then a appears before b in the causal order
//! - If a || b (concurrent), either order is valid
//! - Multiple valid orderings may exist (any topological sort is acceptable)
//!
//! ## Implementation Task
//!
//! Implement:
//! - `CausalOrder` that collects events from multiple processes
//! - `add_event(event, vc)` to add events with their vector clocks
//! - `get_causal_order()` to return events in a causally-respecting order
//!
//! ## Verification
//!
//! - Verify order respects causality
//! - Verify concurrent events can appear in any order

use std::collections::{HashMap, HashSet, VecDeque};

use super::p05_vector_clocks::VectorClock;

/// An event with its vector clock.
#[derive(Debug, Clone)]
pub struct TimestampedEvent {
    /// The process that generated this event.
    pub process_id: usize,
    /// The event index within the process.
    pub event_index: usize,
    /// The vector clock at the time of this event.
    pub vector_clock: VectorClock,
    /// The payload of the event.
    pub payload: String,
}

/// Collects events and produces a causal ordering.
pub struct CausalOrder {
    /// All events indexed by (process_id, event_index).
    events: HashMap<(usize, usize), TimestampedEvent>,
    /// Number of processes.
    _num_processes: usize,
}

impl CausalOrder {
    /// Create a new causal order tracker.
    pub fn new(num_processes: usize) -> Self {
        Self {
            events: HashMap::new(),
            _num_processes: num_processes,
        }
    }

    /// Add an event with its vector clock.
    pub fn add_event(&mut self, event: TimestampedEvent) {
        let key = (event.process_id, event.event_index);
        self.events.insert(key, event);
    }

    /// Check if event a causally precedes event b.
    fn happens_before(&self, a: &TimestampedEvent, b: &TimestampedEvent) -> bool {
        a.vector_clock.precedes(&b.vector_clock.vector)
    }

    /// Get events in a causally-respecting order using topological sort.
    ///
    /// Returns a Vec of events ordered such that if a -> b, then a appears before b.
    /// If multiple valid orderings exist, returns one of them.
    pub fn get_causal_order(&self) -> Vec<TimestampedEvent> {
        let event_list: Vec<&TimestampedEvent> = self.events.values().collect();
        let n = event_list.len();

        if n == 0 {
            return Vec::new();
        }

        // Build adjacency list: edge from a to b if a happens before b
        let mut in_degree: Vec<usize> = vec![0; n];
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

        for i in 0..n {
            for j in 0..n {
                if i != j && self.happens_before(event_list[i], event_list[j]) {
                    adj[i].push(j);
                    in_degree[j] += 1;
                }
            }
        }

        // Kahn's algorithm for topological sort
        let mut queue: VecDeque<usize> = VecDeque::new();
        for i in 0..n {
            if in_degree[i] == 0 {
                queue.push_back(i);
            }
        }

        let mut sorted = Vec::new();
        while let Some(node) = queue.pop_front() {
            sorted.push(event_list[node].clone());
            for &neighbor in &adj[node] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push_back(neighbor);
                }
            }
        }

        sorted
    }

    /// Verify that a given ordering respects causality.
    pub fn verify_ordering(ordering: &[TimestampedEvent]) -> bool {
        let mut seen: HashSet<(usize, usize)> = HashSet::new();

        for event in ordering {
            // All events that this event causally depends on should have been seen
            for (&(pid, eid), _dep_event) in Self::events_map(ordering).iter() {
                if pid == event.process_id && eid < event.event_index {
                    // Same-process dependency
                    if !seen.contains(&(pid, eid)) {
                        return false;
                    }
                }
            }
            seen.insert((event.process_id, event.event_index));
        }

        true
    }

    /// Helper: build a map from (process_id, event_index) to event.
    fn events_map(events: &[TimestampedEvent]) -> HashMap<(usize, usize), &TimestampedEvent> {
        events
            .iter()
            .map(|e| ((e.process_id, e.event_index), e))
            .collect()
    }

    /// Get the number of events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(pid: usize, eid: usize, vc: Vec<u64>) -> TimestampedEvent {
        TimestampedEvent {
            process_id: pid,
            event_index: eid,
            vector_clock: VectorClock::from_vector(vc),
            payload: format!("P{pid}:E{eid}"),
        }
    }

    #[test]
    fn causal_order_respects_happened_before() {
        let mut order = CausalOrder::new(2);

        // P0: E0 -> E1 -> E2
        let e0 = make_event(0, 0, vec![1, 0]);
        let e1 = make_event(0, 1, vec![2, 0]);
        let e2 = make_event(0, 2, vec![3, 0]);

        // P1: E0 (receives from P0)
        let e3 = make_event(1, 0, vec![3, 1]);

        order.add_event(e0);
        order.add_event(e1);
        order.add_event(e2);
        order.add_event(e3);

        let sorted = order.get_causal_order();

        // P0: E0 before E1 before E2 must hold
        let positions: HashMap<&str, usize> = sorted
            .iter()
            .enumerate()
            .map(|(i, e)| (e.payload.as_str(), i))
            .collect();

        assert!(positions["P0:E0"] < positions["P0:E1"]);
        assert!(positions["P0:E1"] < positions["P0:E2"]);
        // P1:E0 depends on P0:E2 (vc=[3,1] > [3,0])
        assert!(positions["P0:E2"] < positions["P1:E0"]);
    }

    #[test]
    fn concurrent_events_can_appear_in_any_order() {
        let mut order = CausalOrder::new(2);

        // Two concurrent events
        let e0 = make_event(0, 0, vec![1, 0]);
        let e1 = make_event(1, 0, vec![0, 1]);

        order.add_event(e0);
        order.add_event(e1);

        let sorted = order.get_causal_order();
        assert_eq!(sorted.len(), 2);

        // Both orderings are valid for concurrent events
        // Just verify both events appear
        let ids: Vec<_> = sorted.iter().map(|e| (e.process_id, e.event_index)).collect();
        assert!(ids.contains(&(0, 0)));
        assert!(ids.contains(&(1, 0)));
    }

    #[test]
    fn empty_order() {
        let order = CausalOrder::new(2);
        let sorted = order.get_causal_order();
        assert!(sorted.is_empty());
    }

    #[test]
    fn single_process_chain() {
        let mut order = CausalOrder::new(1);

        for i in 0..5 {
            order.add_event(make_event(0, i, vec![(i + 1) as u64]));
        }

        let sorted = order.get_causal_order();
        assert_eq!(sorted.len(), 5);

        // Events should be in order
        for i in 0..5 {
            assert_eq!(sorted[i].event_index, i);
        }
    }

    #[test]
    fn diamond_dependency() {
        // Diamond: A -> B, A -> C, B -> D, C -> D
        // A = P0:E0, B = P0:E1, C = P1:E0, D = P1:E1
        let mut order = CausalOrder::new(2);

        let a = make_event(0, 0, vec![1, 0]); // A
        let b = make_event(0, 1, vec![2, 0]); // B (after A)
        let c = make_event(1, 0, vec![1, 1]); // C (after A, received msg)
        let d = make_event(1, 1, vec![2, 2]); // D (after B and C)

        order.add_event(a);
        order.add_event(b);
        order.add_event(c);
        order.add_event(d);

        let sorted = order.get_causal_order();
        let positions: HashMap<&str, usize> = sorted
            .iter()
            .enumerate()
            .map(|(i, e)| (e.payload.as_str(), i))
            .collect();

        // A before B and C
        assert!(positions["P0:E0"] < positions["P0:E1"]);
        assert!(positions["P0:E0"] < positions["P1:E0"]);
        // B and C before D
        assert!(positions["P0:E1"] < positions["P1:E1"]);
        assert!(positions["P1:E0"] < positions["P1:E1"]);
    }
}
