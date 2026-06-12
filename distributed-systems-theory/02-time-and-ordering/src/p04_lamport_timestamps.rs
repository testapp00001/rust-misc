//! # Exercise: Lamport Timestamps
//!
//! ## Theory
//!
//! **Lamport Timestamps** (Lamport, 1978) are a logical clock scheme that assigns
//! a monotonically increasing integer to each event in a distributed system. They
//! capture causality in the sense that if a -> b, then L(a) < L(b).
//!
//! The rules are simple:
//! 1. Before executing a local event, increment the counter
//! 2. When sending a message, increment the counter and attach it to the message
//! 3. When receiving a message, set counter = max(local, received) + 1
//!
//! ## Proof / Intuition
//!
//! **Theorem**: If a -> b, then L(a) < L(b).
//!
//! **Proof sketch**: By induction on the length of the causal chain from a to b.
//! Each step (local event, send, receive) increments the counter, so the timestamp
//! strictly increases along any causal path.
//!
//! **Important limitation**: L(a) < L(b) does NOT imply a -> b. Two concurrent
//! events can have any relationship between their Lamport timestamps. This is
//! because Lamport timestamps only track causality, not concurrency.
//!
//! ## Implementation Task
//!
//! Implement `LamportClock` with:
//! - `local_event()` - increment and return
//! - `send_timestamp()` - increment and return (attach to message)
//! - `receive_timestamp(received)` - max(local, received) + 1
//!
//! ## Verification
//!
//! - Verify monotonicity within a process
//! - Verify causality preservation (a -> b implies L(a) < L(b))
//! - Show that L(a) < L(b) does NOT imply a -> b

/// A Lamport logical clock.
#[derive(Debug, Clone)]
pub struct LamportClock {
    /// The current counter value.
    counter: u64,
}

impl LamportClock {
    /// Create a new Lamport clock starting at 0.
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    /// Create a new Lamport clock with a specific starting value.
    pub fn with_value(value: u64) -> Self {
        Self { counter: value }
    }

    /// Execute a local event: increment counter and return the timestamp.
    pub fn local_event(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    /// Prepare to send a message: increment counter and return the timestamp
    /// to attach to the message.
    pub fn send_timestamp(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    /// Process a received message: update counter based on received timestamp.
    /// Returns the new timestamp (when the receive event occurred).
    pub fn receive_timestamp(&mut self, received: u64) -> u64 {
        self.counter = std::cmp::max(self.counter, received) + 1;
        self.counter
    }

    /// Get the current counter value without incrementing.
    pub fn now(&self) -> u64 {
        self.counter
    }
}

impl Default for LamportClock {
    fn default() -> Self {
        Self::new()
    }
}

/// A message with a Lamport timestamp.
#[derive(Debug, Clone)]
pub struct TimestampedMessage {
    /// The Lamport timestamp when the message was sent.
    pub timestamp: u64,
    /// The payload.
    pub payload: String,
}

/// Simulate two processes exchanging messages with Lamport timestamps.
///
/// Returns a log of events with their Lamport timestamps.
pub fn simulate_two_process_exchange(
    p0_events: usize,
    p1_events: usize,
    messages_from_p0: usize,
    messages_from_p1: usize,
) -> Vec<(usize, String, u64)> {
    let mut clock0 = LamportClock::new();
    let mut clock1 = LamportClock::new();
    let mut log = Vec::new();

    let mut msg_queue_0_to_1: Vec<u64> = Vec::new();
    let mut msg_queue_1_to_0: Vec<u64> = Vec::new();

    let mut p0_sent = 0;
    let mut p1_sent = 0;

    for i in 0..p0_events.max(p1_events) {
        // Process 0 events
        if i < p0_events {
            if p0_sent < messages_from_p0 {
                let ts = clock0.send_timestamp();
                msg_queue_0_to_1.push(ts);
                log.push((0, format!("P0 send (msg {p0_sent})"), ts));
                p0_sent += 1;
            } else {
                let ts = clock0.local_event();
                log.push((0, format!("P0 local event {i}"), ts));
            }

            // Check if P0 has messages from P1
            if let Some(ts) = msg_queue_1_to_0.pop() {
                let recv_ts = clock0.receive_timestamp(ts);
                log.push((0, format!("P0 receive (from P1, msg_ts={ts})"), recv_ts));
            }
        }

        // Process 1 events
        if i < p1_events {
            // Check if P1 has messages from P0 first
            if let Some(ts) = msg_queue_0_to_1.pop() {
                let recv_ts = clock1.receive_timestamp(ts);
                log.push((1, format!("P1 receive (from P0, msg_ts={ts})"), recv_ts));
            }

            if p1_sent < messages_from_p1 {
                let ts = clock1.send_timestamp();
                msg_queue_1_to_0.push(ts);
                log.push((1, format!("P1 send (msg {p1_sent})"), ts));
                p1_sent += 1;
            } else {
                let ts = clock1.local_event();
                log.push((1, format!("P1 local event {i}"), ts));
            }
        }
    }

    log
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monotonicity_within_process() {
        let mut clock = LamportClock::new();

        let mut timestamps = Vec::new();
        for _ in 0..10 {
            timestamps.push(clock.local_event());
        }

        // Timestamps should be strictly increasing
        for i in 1..timestamps.len() {
            assert!(
                timestamps[i] > timestamps[i - 1],
                "Lamport timestamps should be monotonically increasing"
            );
        }
    }

    #[test]
    fn causality_preservation() {
        // Simulate: P0 sends message to P1
        let mut clock0 = LamportClock::new();
        let mut clock1 = LamportClock::new();

        // P0 local event
        let ts_send = clock0.send_timestamp();

        // P0 sends message to P1
        let msg = TimestampedMessage {
            timestamp: ts_send,
            payload: "hello".to_string(),
        };

        // P1 receives message
        let ts_recv = clock1.receive_timestamp(msg.timestamp);

        // Causality: send happened before receive
        assert!(
            ts_send < ts_recv,
            "send timestamp {ts_send} should be < receive timestamp {ts_recv}"
        );
    }

    #[test]
    fn receive_takes_max() {
        let mut clock0 = LamportClock::new();
        let mut clock1 = LamportClock::new();

        // P0 does some events (clock goes to 5)
        for _ in 0..5 {
            clock0.send_timestamp();
        }

        // P1 does some events (clock goes to 3)
        for _ in 0..3 {
            clock1.local_event();
        }

        // P1 receives message from P0 with timestamp 5
        let ts = clock1.receive_timestamp(5);

        // P1's clock should jump to max(3, 5) + 1 = 6
        assert_eq!(ts, 6);
    }

    #[test]
    fn lamport_lt_does_not_imply_happened_before() {
        // Counterexample: L(a) < L(b) but a does NOT happen before b
        //
        // P0: event at L=10
        // P1: event at L=5
        //
        // P0's event has L=10 > L=5, but P0's event does NOT happen before P1's event
        // (they are in different processes with no message chain)

        let mut clock0 = LamportClock::with_value(9);
        let mut clock1 = LamportClock::with_value(4);

        let ts_a = clock0.local_event(); // L=10
        let ts_b = clock1.local_event(); // L=5

        // L(b) < L(a), but b does NOT happen before a
        assert!(ts_b < ts_a);
        // They are concurrent (different processes, no message)
        // This demonstrates that L(a) < L(b) does not imply a -> b
    }

    #[test]
    fn send_and_receive_increment() {
        let mut clock = LamportClock::new();

        let ts1 = clock.send_timestamp(); // 1
        assert_eq!(ts1, 1);

        let ts2 = clock.receive_timestamp(ts1); // max(1, 1) + 1 = 2
        assert_eq!(ts2, 2);

        let ts3 = clock.local_event(); // 3
        assert_eq!(ts3, 3);
    }

    #[test]
    fn concurrent_messages_independent() {
        // Two processes with no message exchange should have independent clocks
        let mut clock0 = LamportClock::new();
        let mut clock1 = LamportClock::new();

        let ts0 = clock0.local_event(); // 1
        let ts1 = clock1.local_event(); // 1

        // Both have timestamp 1 but are concurrent
        assert_eq!(ts0, ts1);
    }
}
