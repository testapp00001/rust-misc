//! # Exercise: Vector Clocks
//!
//! ## Theory
//!
//! **Vector Clocks** (Fidge 1988, Mattern 1989) extend Lamport Timestamps to
//! provide **complete causality tracking**. Each process maintains a vector of
//! counters, one per process. The vector tracks the causal history of each event.
//!
//! Rules:
//! 1. Before local event: increment own component
//! 2. On send: increment own component, send entire vector with message
//! 3. On receive: element-wise max with received vector, then increment own component
//!
//! Properties:
//! - VC(a) < VC(b) (component-wise) implies a -> b
//! - a -> b implies VC(a) < VC(b)
//! - Two events are concurrent iff neither VC(a) <= VC(b) nor VC(b) <= VC(a)
//!
//! ## Proof / Intuition
//!
//! The vector clock captures "what each process knows about" at the time of the
//! event. When process P sends a message with its vector, the receiver learns about
//! all events P knew about. The element-wise max operation merges causal histories.
//!
//! The key insight: VC(a) < VC(b) is both necessary AND sufficient for a -> b,
//! unlike Lamport timestamps where L(a) < L(b) is only necessary.
//!
//! ## Implementation Task
//!
//! Implement `VectorClock` with:
//! - `local_event(process_id)` - increment own component
//! - `send_clock()` - return copy of vector
//! - `receive_clock(process_id, received)` - element-wise max, then increment self
//! - `precedes(other)` - component-wise <
//! - `concurrent_with(other)` - neither precedes the other
//!
//! ## Verification
//!
//! - Verify causality: if a -> b, then VC(a) < VC(b)
//! - Verify concurrency detection
//! - Verify that VC(a) < VC(b) implies a -> b (completeness)

/// A vector clock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorClock {
    /// The vector of counters, one per process.
    /// Index is the process ID.
    pub vector: Vec<u64>,
}

impl VectorClock {
    /// Create a new vector clock for a system with `n` processes.
    pub fn new(num_processes: usize) -> Self {
        Self {
            vector: vec![0; num_processes],
        }
    }

    /// Create a vector clock with specific values.
    pub fn from_vector(vector: Vec<u64>) -> Self {
        Self { vector }
    }

    /// Execute a local event: increment the component for `process_id`.
    pub fn local_event(&mut self, process_id: usize) {
        assert!(
            process_id < self.vector.len(),
            "process_id {process_id} out of range for vector of length {}",
            self.vector.len()
        );
        self.vector[process_id] += 1;
    }

    /// Get a copy of the current vector (for sending with a message).
    pub fn send_clock(&self) -> Vec<u64> {
        self.vector.clone()
    }

    /// Process a received message: element-wise max with received vector,
    /// then increment own component.
    pub fn receive_clock(&mut self, process_id: usize, received: &[u64]) {
        assert!(
            process_id < self.vector.len(),
            "process_id {process_id} out of range"
        );
        assert_eq!(
            self.vector.len(),
            received.len(),
            "received vector must have same length"
        );

        // Element-wise max
        for i in 0..self.vector.len() {
            self.vector[i] = self.vector[i].max(received[i]);
        }

        // Increment own component
        self.vector[process_id] += 1;
    }

    /// Check if this vector clock strictly precedes another (component-wise <).
    pub fn precedes(&self, other: &[u64]) -> bool {
        assert_eq!(self.vector.len(), other.len(), "vectors must have same length");

        let mut strictly_less = false;
        for i in 0..self.vector.len() {
            if self.vector[i] > other[i] {
                return false;
            }
            if self.vector[i] < other[i] {
                strictly_less = true;
            }
        }
        strictly_less
    }

    /// Check if this vector clock is equal to another.
    pub fn equals(&self, other: &[u64]) -> bool {
        self.vector == other
    }

    /// Check if this vector clock is concurrent with another (neither precedes).
    pub fn concurrent_with(&self, other: &[u64]) -> bool {
        !self.precedes(other) && !self.equals(other) && !Self::vector_precedes(other, &self.vector)
    }

    /// Helper: check if vector `a` strictly precedes vector `b` (component-wise <).
    fn vector_precedes(a: &[u64], b: &[u64]) -> bool {
        assert_eq!(a.len(), b.len(), "vectors must have same length");

        let mut strictly_less = false;
        for i in 0..a.len() {
            if a[i] > b[i] {
                return false;
            }
            if a[i] < b[i] {
                strictly_less = true;
            }
        }
        strictly_less
    }

    /// Get the number of processes.
    pub fn num_processes(&self) -> usize {
        self.vector.len()
    }
}

impl std::fmt::Display for VectorClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.vector.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_event_increments_own_component() {
        let mut vc = VectorClock::new(3);
        vc.local_event(0);
        assert_eq!(vc.vector, vec![1, 0, 0]);

        vc.local_event(1);
        assert_eq!(vc.vector, vec![1, 1, 0]);

        vc.local_event(0);
        assert_eq!(vc.vector, vec![2, 1, 0]);
    }

    #[test]
    fn receive_clock_merges_and_increments() {
        let mut vc = VectorClock::new(3);
        vc.local_event(0); // [1, 0, 0]
        vc.local_event(0); // [2, 0, 0]

        // Receive message from process 1 with vector [0, 3, 1]
        vc.receive_clock(1, &[0, 3, 1]);

        // Element-wise max: [2, 3, 1], then increment P1: [2, 4, 1]
        assert_eq!(vc.vector, vec![2, 4, 1]);
    }

    #[test]
    fn causality_implied_by_precedes() {
        // a -> b means VC(a) < VC(b)
        let vc_a = VectorClock::from_vector(vec![1, 0, 0]);
        let vc_b = VectorClock::from_vector(vec![1, 1, 0]);

        assert!(vc_a.precedes(&vc_b.vector));
    }

    #[test]
    fn precedes_requires_all_components_less_or_equal() {
        // [1, 3] does NOT precede [2, 2] because 3 > 2
        let vc_a = VectorClock::from_vector(vec![1, 3]);
        assert!(!vc_a.precedes(&vec![2, 2]));
    }

    #[test]
    fn concurrent_events_detected() {
        // [1, 0] and [0, 1] are concurrent
        let vc_a = VectorClock::from_vector(vec![1, 0]);
        let vc_b = VectorClock::from_vector(vec![0, 1]);

        assert!(vc_a.concurrent_with(&vc_b.vector));
        assert!(vc_b.concurrent_with(&vc_a.vector));
    }

    #[test]
    fn identical_vectors_are_not_concurrent() {
        let vc_a = VectorClock::from_vector(vec![1, 1]);
        assert!(!vc_a.concurrent_with(&vec![1, 1]));
    }

    #[test]
    fn send_clock_returns_copy() {
        let mut vc = VectorClock::new(2);
        vc.local_event(0);
        vc.local_event(1);

        let sent = vc.send_clock();
        assert_eq!(sent, vec![1, 1]);

        // Modifying the copy shouldn't affect original
        let mut modified = sent;
        modified[0] = 99;
        assert_eq!(vc.vector, vec![1, 1]);
    }

    #[test]
    fn display_format() {
        let vc = VectorClock::from_vector(vec![1, 2, 3]);
        assert_eq!(format!("{vc}"), "[1, 2, 3]");
    }
}
