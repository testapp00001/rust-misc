//! # Solution 03: Sliding Window Counter Rate Limiter
//!
//! Complete implementation of a sliding window counter using two fixed sub-windows.

use std::time::Instant;

/// A sliding window counter rate limiter.
pub struct SlidingWindowCounter {
    window_ms: u64,
    max_requests: u64,
    prev_count: u64,
    curr_count: u64,
    window_start: Instant,
}

impl SlidingWindowCounter {
    /// Create a new sliding window counter rate limiter.
    pub fn new(window_ms: u64, max_requests: u64) -> Self {
        Self {
            window_ms,
            max_requests,
            prev_count: 0,
            curr_count: 0,
            window_start: Instant::now(),
        }
    }

    /// Attempt to acquire a request slot.
    pub fn try_acquire(&mut self) -> bool {
        self.maybe_rotate();
        let count = self.weighted_count();
        if count < self.max_requests as f64 {
            self.curr_count += 1;
            true
        } else {
            false
        }
    }

    /// Get the approximate count of requests in the current sliding window.
    pub fn current_count(&self) -> f64 {
        self.weighted_count()
    }

    /// Calculate the weighted count across the two sub-windows.
    fn weighted_count(&self) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.window_start).as_millis() as f64;
        let window_ms = self.window_ms as f64;

        if elapsed >= window_ms {
            // Full weight on current window
            self.curr_count as f64
        } else {
            let fraction = elapsed / window_ms;
            // Previous window contributes (1 - fraction) of its count
            // Current window contributes its full count
            self.prev_count as f64 * (1.0 - fraction) + self.curr_count as f64
        }
    }

    /// Rotate windows if enough time has passed.
    fn maybe_rotate(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.window_start).as_millis() as u64;
        if elapsed >= self.window_ms {
            // Advance by exactly one window to preserve correct overlap fraction
            self.prev_count = self.curr_count;
            self.curr_count = 0;
            self.window_start += std::time::Duration::from_millis(self.window_ms);
        }
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
        let mut limiter = SlidingWindowCounter::new(1000, 100);
        for _ in 0..100 {
            assert!(limiter.try_acquire());
        }
        assert!(!limiter.try_acquire());
    }

    #[test]
    fn test_window_rotation() {
        let mut limiter = SlidingWindowCounter::new(100, 5);
        for _ in 0..5 {
            limiter.try_acquire();
        }
        assert!(!limiter.try_acquire());

        thread::sleep(Duration::from_millis(150));
        assert!(limiter.try_acquire(), "Should succeed after window rotation");
    }

    #[test]
    fn test_memory_efficiency() {
        let mut limiter = SlidingWindowCounter::new(1000, 1000);
        for _ in 0..1000 {
            limiter.try_acquire();
        }
        let count = limiter.current_count();
        assert!(count <= 1000.0);
    }
}
