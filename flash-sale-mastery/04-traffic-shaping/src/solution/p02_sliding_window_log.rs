//! # Solution 02: Sliding Window Log Rate Limiter
//!
//! Complete implementation of a sliding window log rate limiter.

use std::collections::VecDeque;
use std::time::Instant;

/// A sliding window log rate limiter.
pub struct SlidingWindowLog {
    window_ms: u64,
    max_requests: u64,
    timestamps: VecDeque<Instant>,
    start_time: Instant,
}

impl SlidingWindowLog {
    /// Create a new sliding window log rate limiter.
    pub fn new(window_ms: u64, max_requests: u64) -> Self {
        Self {
            window_ms,
            max_requests,
            timestamps: VecDeque::new(),
            start_time: Instant::now(),
        }
    }

    /// Attempt to acquire a request slot.
    pub fn try_acquire(&mut self) -> bool {
        self.cleanup();
        if self.timestamps.len() < self.max_requests as usize {
            self.timestamps.push_back(Instant::now());
            true
        } else {
            false
        }
    }

    /// Remove timestamps that have fallen outside the current window.
    fn cleanup(&mut self) {
        let cutoff = Instant::now() - std::time::Duration::from_millis(self.window_ms);
        while let Some(&front) = self.timestamps.front() {
            if front < cutoff {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get the current count of requests in the window.
    pub fn current_count(&mut self) -> usize {
        self.cleanup();
        self.timestamps.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_within_limit() {
        let mut limiter = SlidingWindowLog::new(1000, 5);
        for i in 0..5 {
            assert!(limiter.try_acquire(), "Request {i} should succeed");
        }
        assert_eq!(limiter.current_count(), 5);
    }

    #[test]
    fn test_exceeded_limit() {
        let mut limiter = SlidingWindowLog::new(1000, 3);
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire(), "4th request should be rejected");
    }

    #[test]
    fn test_window_slides() {
        let mut limiter = SlidingWindowLog::new(100, 2);
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire(), "Should be rate limited");

        thread::sleep(Duration::from_millis(150));
        assert!(limiter.try_acquire(), "Should succeed after window slides");
    }

    #[test]
    fn test_cleanup_removes_old_entries() {
        let mut limiter = SlidingWindowLog::new(50, 10);
        for _ in 0..10 {
            limiter.try_acquire();
        }
        assert_eq!(limiter.current_count(), 10);

        thread::sleep(Duration::from_millis(100));
        limiter.cleanup();
        assert_eq!(limiter.current_count(), 0, "All entries should have expired");
    }
}
