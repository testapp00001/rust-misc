//! Hybrid Logical Clock (HLC).
//!
//! An HLC combines a physical time component with a logical counter to produce
//! timestamps that are both close to physical time and causally ordered. When two
//! events are causally related, their HLC timestamps reflect that ordering. When
//! they are concurrent, the physical component breaks ties.
//!
//! The combined timestamp is encoded as `(physical_time << 32) | logical_time`,
//! assuming both fit in 32 bits.

/// A Hybrid Logical Clock that produces causally ordered timestamps.
#[derive(Debug)]
pub struct HLC {
    /// The most recently observed physical time.
    physical_time: u64,
    /// The logical counter for events within the same physical tick.
    logical_time: u64,
    /// The id of the node owning this clock.
    node_id: u64,
}

impl HLC {
    /// Create a new HLC for the given node.
    ///
    /// Both physical and logical times start at zero.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The unique identifier of the node this clock belongs to.
    pub fn new(node_id: u64) -> Self {
        Self {
            physical_time: 0,
            logical_time: 0,
            node_id,
        }
    }

    /// Generate a new timestamp for a local event.
    ///
    /// Increments the logical time component and returns the combined timestamp.
    pub fn now(&mut self) -> u64 {
        self.logical_time += 1;
        self.timestamp()
    }

    /// Merge a received timestamp into this clock.
    ///
    /// Called when a message arrives with an HLC timestamp. The physical component
    /// takes the maximum of local and received, and the logical component is
    /// adjusted to maintain causal ordering.
    ///
    /// # Arguments
    ///
    /// * `received_time` - The HLC timestamp from the received message.
    pub fn update(&mut self, received_time: u64) {
        let recv_physical = received_time >> 32;
        let recv_logical = received_time & 0xFFFF_FFFF;

        if recv_physical > self.physical_time {
            self.physical_time = recv_physical;
            self.logical_time = recv_logical + 1;
        } else if recv_physical == self.physical_time {
            if recv_logical >= self.logical_time {
                self.logical_time = recv_logical + 1;
            } else {
                self.logical_time += 1;
            }
        } else {
            self.logical_time += 1;
        }
    }

    /// Return the physical time component of the clock.
    pub fn get_physical(&self) -> u64 {
        self.physical_time
    }

    /// Return the logical time component of the clock.
    pub fn get_logical(&self) -> u64 {
        self.logical_time
    }

    /// Return the combined HLC timestamp.
    ///
    /// The timestamp encodes the physical time in the upper 32 bits and the
    /// logical time in the lower 32 bits.
    pub fn timestamp(&self) -> u64 {
        (self.physical_time << 32) | (self.logical_time & 0xFFFF_FFFF)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clock() {
        let hlc = HLC::new(1);
        assert_eq!(hlc.get_physical(), 0);
        assert_eq!(hlc.get_logical(), 0);
    }

    #[test]
    fn test_now_increments() {
        let mut hlc = HLC::new(1);
        let t1 = hlc.now();
        let t2 = hlc.now();
        assert!(t2 > t1);
        assert_eq!(hlc.get_logical(), 2);
    }

    #[test]
    fn test_update_with_larger_physical() {
        let mut hlc = HLC::new(1);
        hlc.now(); // logical = 1

        // Receive a timestamp with a larger physical time.
        let received = (10u64 << 32) | 5;
        hlc.update(received);

        assert_eq!(hlc.get_physical(), 10);
        assert_eq!(hlc.get_logical(), 6); // recv_logical + 1
    }

    #[test]
    fn test_update_with_same_physical() {
        let mut hlc = HLC::new(1);
        hlc.now(); // physical=0, logical=1

        // Receive a timestamp with same physical but higher logical.
        let received = (0u64 << 32) | 3;
        hlc.update(received);
        assert_eq!(hlc.get_logical(), 4); // recv_logical + 1

        // Receive a timestamp with same physical but lower logical.
        let hlc2_before = hlc.logical_time;
        let received = (0u64 << 32) | 1;
        hlc.update(received);
        assert_eq!(hlc.get_logical(), hlc2_before + 1);
    }

    #[test]
    fn test_update_with_smaller_physical() {
        let mut hlc = HLC::new(1);
        hlc.update(10 << 32); // physical=10, logical=1

        // Receive an older physical time.
        hlc.update(5 << 32);
        assert_eq!(hlc.get_physical(), 10); // unchanged
        assert_eq!(hlc.get_logical(), 2);   // incremented
    }

    #[test]
    fn test_timestamp_encoding() {
        let mut hlc = HLC::new(1);
        hlc.update((5u64 << 32) | 3);
        let ts = hlc.timestamp();
        assert_eq!(ts >> 32, 5);
        assert_eq!(ts & 0xFFFF_FFFF, 4); // logical became 3+1=4
    }
}
