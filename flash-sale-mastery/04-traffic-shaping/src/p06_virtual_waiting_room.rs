//! # Exercise 06: Virtual Waiting Room
//!
//! ## Learning Objective
//! Implement a virtual waiting room that queues excess visitors and releases
//! them in controlled batches. This is the pattern used by major ticketing
//! and flash sale platforms to handle demand spikes gracefully.
//!
//! ## Flash Sale Context
//! When a flash sale opens, 100K users might hit the site simultaneously.
//! Instead of letting them all through (crashing the system) or showing an
//! error page (losing sales), a virtual waiting room holds them in a queue
//! with a position indicator and estimated wait time, releasing them in
//! manageable batches.
//!
//! ## Instructions
//! 1. Implement `WaitingRoom::new(capacity, release_rate)` -- create a waiting
//!    room that allows `capacity` concurrent users and releases `release_rate`
//!    users per batch
//! 2. Implement `enter()` -- assign a visitor a position in the queue
//! 3. Implement `release_batch()` -- release the next batch of visitors
//! 4. Implement `get_position()` -- check a visitor's current position
//!
//! ## Hints
//! - Use a `VecDeque` for FIFO ordering
//! - Each visitor gets a unique ID (use an atomic counter)
//! - Position should update as visitors ahead are released
//! - `release_batch` returns the released visitors' IDs

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

/// A visitor's position in the waiting room.
#[derive(Debug, Clone, PartialEq)]
pub struct WaitPosition {
    pub id: u64,
    pub position: usize,
    pub estimated_wait_secs: u64,
}

/// Virtual waiting room with FIFO ordering and batch release.
pub struct WaitingRoom {
    // TODO: Add fields for queue, capacity, release_rate, and active count
}

impl WaitingRoom {
    /// Create a new waiting room.
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of users allowed in concurrently
    /// * `release_rate` - Number of users to release per batch
    pub fn new(capacity: u64, release_rate: u64) -> Self {
        todo!("Implement waiting room creation")
    }

    /// Enter the waiting room.
    ///
    /// Assigns the visitor a unique ID and position in the queue.
    /// Returns a `WaitPosition` with the visitor's queue position and
    /// estimated wait time.
    pub fn enter(&mut self) -> WaitPosition {
        todo!("Implement visitor entry")
    }

    /// Release the next batch of visitors from the waiting room.
    ///
    /// Releases up to `release_rate` visitors. Returns the IDs of
    /// released visitors. Returns an empty vec if the queue is empty.
    pub fn release_batch(&mut self) -> Vec<u64> {
        todo!("Implement batch release")
    }

    /// Get a visitor's current position in the queue.
    ///
    /// Returns `None` if the visitor ID is not found (already released
    /// or never entered).
    pub fn get_position(&self, id: u64) -> Option<WaitPosition> {
        todo!("Implement position lookup")
    }

    /// Get the current queue length.
    pub fn queue_len(&self) -> usize {
        todo!("Return queue length")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fifo_ordering() {
        let mut room = WaitingRoom::new(10, 5);
        let pos1 = room.enter();
        let pos2 = room.enter();
        let pos3 = room.enter();

        assert_eq!(pos1.position, 1, "First visitor should be position 1");
        assert_eq!(pos2.position, 2);
        assert_eq!(pos3.position, 3);
    }

    #[test]
    fn test_batch_release() {
        let mut room = WaitingRoom::new(10, 2);
        let _p1 = room.enter();
        let _p2 = room.enter();
        let _p3 = room.enter();
        let _p4 = room.enter();

        let released = room.release_batch();
        assert_eq!(released.len(), 2, "Should release 2 visitors");

        let released2 = room.release_batch();
        assert_eq!(released2.len(), 2, "Should release remaining 2");
    }

    #[test]
    fn test_position_updates_after_release() {
        let mut room = WaitingRoom::new(10, 1);
        let p1 = room.enter();
        let p2 = room.enter();
        let p3 = room.enter();

        assert_eq!(p1.position, 1);
        assert_eq!(p3.position, 3);

        // Release the first visitor
        room.release_batch();

        // p2 should now be position 1
        let updated = room.get_position(p2.id).unwrap();
        assert_eq!(updated.position, 1);
    }

    #[test]
    fn test_capacity_enforcement() {
        let mut room = WaitingRoom::new(2, 1);
        room.enter();
        room.enter();

        // The waiting room should still accept visitors into the queue
        // (capacity refers to concurrent users, not queue size)
        let pos = room.enter();
        assert!(pos.position > 0, "Visitor should get a queue position");
    }

    #[test]
    fn test_released_visitor_not_found() {
        let mut room = WaitingRoom::new(10, 1);
        let p1 = room.enter();
        room.release_batch();

        assert!(
            room.get_position(p1.id).is_none(),
            "Released visitor should not be found"
        );
    }
}
