//! # Exercise 04: Leaky Bucket Rate Limiter
//!
//! ## Learning Objective
//! Implement a leaky bucket rate limiter that enforces a constant output rate.
//! Unlike the token bucket (which allows bursts), the leaky bucket smooths
//! traffic into a steady stream, which is useful for protecting downstream
//! services that need predictable load.
//!
//! ## Flash Sale Context
//! After the admission control layer decides to process a request, the leaky
//! bucket ensures the order processing service receives requests at a constant
//! rate. This prevents database connection pool exhaustion and keeps latency
//! predictable for the orders that do get processed.
//!
//! ## Instructions
//! 1. Implement `LeakyBucket::new(capacity, leak_rate)` -- create a bucket
//!    that can hold up to `capacity` requests and drains at `leak_rate`
//!    requests per second
//! 2. Implement `try_add()` -- add a request to the bucket if there's room,
//!    returning `true` on success, `false` if the bucket would overflow
//! 3. Implement `leak()` -- drain the bucket based on elapsed time
//!
//! ## Hints
//! - The bucket tracks the number of pending requests (water level)
//! - `leak()` removes requests at a constant rate based on elapsed time
//! - Use `f64` for the water level to allow fractional draining
//! - The water level should never go below 0

use std::time::Instant;

/// A leaky bucket rate limiter.
///
/// Requests enter the bucket (increasing the water level) and are drained
/// at a constant rate. If the bucket is full, new requests are rejected.
/// This produces a smooth, constant-rate output regardless of input burstiness.
pub struct LeakyBucket {
    // TODO: Add fields for capacity, leak_rate, water_level, and last_leak time
}

impl LeakyBucket {
    /// Create a new leaky bucket.
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of requests the bucket can hold
    /// * `leak_rate` - Requests drained per second
    pub fn new(capacity: u64, leak_rate: f64) -> Self {
        todo!("Implement leaky bucket creation")
    }

    /// Attempt to add a request to the bucket.
    ///
    /// First leaks based on elapsed time, then checks if there's room.
    /// If the bucket has capacity, adds the request and returns `true`.
    /// Otherwise returns `false`.
    pub fn try_add(&mut self) -> bool {
        todo!("Implement request addition with leak")
    }

    /// Drain the bucket based on elapsed time since last leak.
    ///
    /// Removes `leak_rate * elapsed_seconds` requests from the bucket,
    /// but never below zero.
    fn leak(&mut self) {
        todo!("Implement constant-rate draining")
    }

    /// Get the current water level (pending requests in the bucket).
    pub fn water_level(&mut self) -> f64 {
        todo!("Return current water level after leaking")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_burst_absorption() {
        let mut bucket = LeakyBucket::new(5, 0.001);
        // Should absorb a burst of 5
        for i in 0..5 {
            assert!(bucket.try_add(), "Request {i} should fit in burst");
        }
        // 6th should overflow
        assert!(!bucket.try_add(), "Should reject when bucket is full");
    }

    #[test]
    fn test_constant_drain_rate() {
        let mut bucket = LeakyBucket::new(10, 100.0); // 100/sec
        // Fill the bucket
        for _ in 0..10 {
            bucket.try_add();
        }

        // After 50ms, ~5 requests should have leaked
        thread::sleep(Duration::from_millis(50));
        let level = bucket.water_level();
        assert!(level < 10.0, "Water level should decrease, got {level}");
        assert!(level >= 3.0, "Should still have some requests, got {level}");
    }

    #[test]
    fn test_overflow() {
        let mut bucket = LeakyBucket::new(3, 0.001); // Small bucket, negligible leak
        assert!(bucket.try_add());
        assert!(bucket.try_add());
        assert!(bucket.try_add());
        assert!(!bucket.try_add(), "Should overflow");
    }

    #[test]
    fn test_drain_to_zero() {
        let mut bucket = LeakyBucket::new(5, 1000.0); // Fast drain
        for _ in 0..5 {
            bucket.try_add();
        }
        thread::sleep(Duration::from_millis(100)); // Should drain completely
        let level = bucket.water_level();
        assert!(level <= 0.01, "Should drain to near zero, got {level}");
    }
}
