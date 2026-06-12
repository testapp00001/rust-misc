//! # Solution 04: Leaky Bucket Rate Limiter
//!
//! Complete implementation of a leaky bucket rate limiter with constant drain rate.

use std::time::Instant;

/// A leaky bucket rate limiter.
pub struct LeakyBucket {
    capacity: f64,
    leak_rate: f64,
    water_level: f64,
    last_leak: Instant,
}

impl LeakyBucket {
    /// Create a new leaky bucket.
    pub fn new(capacity: u64, leak_rate: f64) -> Self {
        Self {
            capacity: capacity as f64,
            leak_rate,
            water_level: 0.0,
            last_leak: Instant::now(),
        }
    }

    /// Attempt to add a request to the bucket.
    pub fn try_add(&mut self) -> bool {
        self.leak();
        // Use a small epsilon to handle floating-point drift from leak calculations
        if self.water_level + 1.0 <= self.capacity + 1e-9 {
            self.water_level += 1.0;
            true
        } else {
            false
        }
    }

    /// Drain the bucket based on elapsed time since last leak.
    fn leak(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_leak).as_secs_f64();
        self.water_level = (self.water_level - elapsed * self.leak_rate).max(0.0);
        self.last_leak = now;
    }

    /// Get the current water level (pending requests in the bucket).
    pub fn water_level(&mut self) -> f64 {
        self.leak();
        self.water_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_burst_absorption() {
        // Use a very low leak rate so drain during rapid adds is negligible
        let mut bucket = LeakyBucket::new(5, 0.001);
        for i in 0..5 {
            assert!(bucket.try_add(), "Request {i} should fit in burst");
        }
        assert!(!bucket.try_add(), "Should reject when bucket is full");
    }

    #[test]
    fn test_constant_drain_rate() {
        let mut bucket = LeakyBucket::new(10, 100.0);
        for _ in 0..10 {
            bucket.try_add();
        }

        thread::sleep(Duration::from_millis(50));
        let level = bucket.water_level();
        assert!(level < 10.0, "Water level should decrease, got {level}");
        assert!(level >= 3.0, "Should still have some requests, got {level}");
    }

    #[test]
    fn test_overflow() {
        // Use a very low leak rate so drain during rapid adds is negligible
        let mut bucket = LeakyBucket::new(3, 0.001);
        assert!(bucket.try_add());
        assert!(bucket.try_add());
        assert!(bucket.try_add());
        assert!(!bucket.try_add(), "Should overflow");
    }

    #[test]
    fn test_drain_to_zero() {
        let mut bucket = LeakyBucket::new(5, 1000.0);
        for _ in 0..5 {
            bucket.try_add();
        }
        thread::sleep(Duration::from_millis(100));
        let level = bucket.water_level();
        assert!(level <= 0.01, "Should drain to near zero, got {level}");
    }
}
