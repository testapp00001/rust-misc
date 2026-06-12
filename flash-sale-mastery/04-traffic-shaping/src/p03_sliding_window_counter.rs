//! # Exercise 03: Sliding Window Counter Rate Limiter
//!
//! ## Learning Objective
//! Implement a sliding window counter rate limiter that uses two fixed windows
//! to approximate a true sliding window. This trades some accuracy for
//! significant memory savings compared to the log approach.
//!
//! ## Flash Sale Context
//! At 100K requests per second, storing individual timestamps (as in the log
//! approach) would consume ~800KB per second per rate limit key. The counter
//! approach uses only two counters regardless of request volume, making it
//! suitable for high-throughput rate limiting at the API gateway level.
//!
//! ## Instructions
//! 1. Implement `SlidingWindowCounter::new(window_ms, max_requests)` -- create
//!    a limiter using two fixed sub-windows
//! 2. Implement `try_acquire()` -- calculate the weighted count from the
//!    previous and current windows, then decide to allow or reject
//!
//! ## Hints
//! - Split the window into two halves: "previous" and "current"
//! - The weighted count is: `prev_count * overlap_fraction + curr_count`
//! - `overlap_fraction` is how much of the previous window overlaps with
//!   the sliding window
//! - Rotate windows when time crosses a boundary

/// A sliding window counter rate limiter.
///
/// Uses two fixed sub-windows to approximate a sliding window count.
/// O(1) memory regardless of request volume.
pub struct SlidingWindowCounter {
    // TODO: Add fields for window size, max requests, previous/Current counts,
    // and the current window start time
}

impl SlidingWindowCounter {
    /// Create a new sliding window counter rate limiter.
    ///
    /// # Arguments
    /// * `window_ms` - Total window duration in milliseconds
    /// * `max_requests` - Maximum requests allowed within the sliding window
    pub fn new(window_ms: u64, max_requests: u64) -> Self {
        todo!("Implement sliding window counter creation")
    }

    /// Attempt to acquire a request slot.
    ///
    /// Calculates the weighted count across the previous and current
    /// sub-windows. If the weighted count is below the limit, increments
    /// the current window counter and returns `true`.
    pub fn try_acquire(&mut self) -> bool {
        todo!("Implement request acquisition with weighted count")
    }

    /// Get the approximate count of requests in the current sliding window.
    pub fn current_count(&self) -> f64 {
        todo!("Calculate weighted count across sub-windows")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_rate_limiting() {
        let mut limiter = SlidingWindowCounter::new(1000, 5);
        for _ in 0..5 {
            assert!(limiter.try_acquire());
        }
        assert!(!limiter.try_acquire(), "Should reject after limit reached");
    }

    #[test]
    fn test_accuracy_vs_log_approach() {
        // The counter approach is approximate but should be close
        let mut limiter = SlidingWindowCounter::new(1000, 100);
        for _ in 0..100 {
            assert!(limiter.try_acquire());
        }
        // The 101st should be rejected (or very close to the boundary)
        assert!(!limiter.try_acquire());
    }

    #[test]
    fn test_window_rotation() {
        let mut limiter = SlidingWindowCounter::new(100, 5);
        for _ in 0..5 {
            limiter.try_acquire();
        }
        assert!(!limiter.try_acquire());

        // Wait for a full window rotation
        thread::sleep(Duration::from_millis(150));
        assert!(limiter.try_acquire(), "Should succeed after window rotation");
    }

    #[test]
    fn test_memory_efficiency() {
        // The counter approach uses O(1) memory -- just two counters
        let mut limiter = SlidingWindowCounter::new(1000, 1000);
        // Fill up to the limit
        for _ in 0..1000 {
            limiter.try_acquire();
        }
        // Memory usage is constant regardless of how many requests we tracked
        let count = limiter.current_count();
        assert!(count <= 1000.0);
    }
}
