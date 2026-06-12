//! # Exercise 02: Sliding Window Log Rate Limiter
//!
//! ## Learning Objective
//! Implement a sliding window log rate limiter that tracks exact timestamps
//! of each request. This provides the most accurate rate limiting but at the
//! cost of higher memory usage (one timestamp per request).
//!
//! ## Flash Sale Context
//! When you need precise rate limiting -- for example, ensuring an account
//! cannot make more than 10 purchase attempts per 60-second window -- the
//! sliding window log approach gives exact counts without boundary artifacts.
//! It's ideal for low-volume, high-value operations like purchase attempts.
//!
//! ## Instructions
//! 1. Implement `SlidingWindowLog::new(window_ms, max_requests)` -- create a
//!    limiter with a time window and max allowed requests per window
//! 2. Implement `try_acquire()` -- record the current timestamp and check if
//!    the window limit has been exceeded
//! 3. Implement `cleanup()` -- remove timestamps older than the window
//!
//! ## Hints
//! - Use `VecDeque` or `Vec` to store timestamps in order
//! - Timestamps can be milliseconds since some reference point
//! - `cleanup` should be called before each check to keep memory bounded
//! - Use `std::time::Instant` for monotonic timestamps

use std::collections::VecDeque;
use std::time::Instant;

/// A sliding window log rate limiter.
///
/// Stores the timestamp of every request within the current window.
/// Provides exact request counts at the cost of O(n) memory per window.
pub struct SlidingWindowLog {
    // TODO: Add fields for window duration, max requests, and timestamp log
}

impl SlidingWindowLog {
    /// Create a new sliding window log rate limiter.
    ///
    /// # Arguments
    /// * `window_ms` - Window duration in milliseconds
    /// * `max_requests` - Maximum requests allowed within the window
    pub fn new(window_ms: u64, max_requests: u64) -> Self {
        todo!("Implement sliding window log creation")
    }

    /// Attempt to acquire a request slot.
    ///
    /// Cleans up expired timestamps, then checks if the number of requests
    /// in the current window is below the limit. If so, records the new
    /// timestamp and returns `true`. Otherwise returns `false`.
    pub fn try_acquire(&mut self) -> bool {
        todo!("Implement request acquisition with window cleanup")
    }

    /// Remove timestamps that have fallen outside the current window.
    fn cleanup(&mut self) {
        todo!("Remove expired timestamps")
    }

    /// Get the current count of requests in the window.
    pub fn current_count(&mut self) -> usize {
        todo!("Return count of requests in current window")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_within_limit() {
        let mut limiter = SlidingWindowLog::new(1000, 5); // 5 per second
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
        let mut limiter = SlidingWindowLog::new(100, 2); // 2 per 100ms
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire(), "Should be rate limited");

        // Wait for window to slide
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
