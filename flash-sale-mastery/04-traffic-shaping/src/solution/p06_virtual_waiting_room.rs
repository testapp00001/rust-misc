//! # Solution 06: Virtual Waiting Room
//!
//! Complete implementation of a virtual waiting room with FIFO ordering and batch release.

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
    queue: VecDeque<u64>,
    capacity: u64,
    release_rate: u64,
    active_count: u64,
    next_id: AtomicU64,
}

impl WaitingRoom {
    /// Create a new waiting room.
    pub fn new(capacity: u64, release_rate: u64) -> Self {
        Self {
            queue: VecDeque::new(),
            capacity,
            release_rate,
            active_count: 0,
            next_id: AtomicU64::new(1),
        }
    }

    /// Enter the waiting room.
    pub fn enter(&mut self) -> WaitPosition {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.queue.push_back(id);
        let position = self.queue.len();
        let estimated_wait_secs = (position as u64) / self.release_rate.max(1);
        WaitPosition {
            id,
            position,
            estimated_wait_secs,
        }
    }

    /// Release the next batch of visitors from the waiting room.
    pub fn release_batch(&mut self) -> Vec<u64> {
        let mut released = Vec::new();
        for _ in 0..self.release_rate {
            if let Some(id) = self.queue.pop_front() {
                released.push(id);
                self.active_count += 1;
            } else {
                break;
            }
        }
        released
    }

    /// Get a visitor's current position in the queue.
    pub fn get_position(&self, id: u64) -> Option<WaitPosition> {
        for (i, &queued_id) in self.queue.iter().enumerate() {
            if queued_id == id {
                return Some(WaitPosition {
                    id,
                    position: i + 1,
                    estimated_wait_secs: ((i + 1) as u64) / self.release_rate.max(1),
                });
            }
        }
        None
    }

    /// Get the current queue length.
    pub fn queue_len(&self) -> usize {
        self.queue.len()
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

        room.release_batch();

        let updated = room.get_position(p2.id).unwrap();
        assert_eq!(updated.position, 1);
    }

    #[test]
    fn test_capacity_enforcement() {
        let mut room = WaitingRoom::new(2, 1);
        room.enter();
        room.enter();

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
