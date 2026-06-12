//! # Exercise 01: Token Bucket Rate Limiter
//!
//! ## Learning Objective
//! Implement a token bucket rate limiter -- the most common algorithm for
//! controlling request rates while allowing short bursts. Understand how
//! tokens accumulate over time and how burst capacity relates to steady-state
//! throughput.
//!
//! ## Flash Sale Context
//! When a flash sale opens, the first few seconds see a massive burst of
//! traffic. A token bucket lets you allow a controlled burst (the bucket
//! capacity) while enforcing a steady-state rate (the refill rate). This
//! prevents your API from being overwhelmed while still giving users a fair
//! chance to participate.
//!
//! ## Instructions
//! 1. Implement `TokenBucket::new(capacity, refill_rate)` -- create a bucket
//!    with the given capacity and refill rate (tokens per second)
//! 2. Implement `try_acquire(tokens)` -- attempt to take `tokens` from the
//!    bucket, returning `true` if successful, `false` if insufficient tokens
//! 3. Implement `refill()` -- add tokens based on elapsed time since last
//!    refill, up to the bucket capacity
//!
//! ## Hints
//! - Use `std::time::Instant` for measuring elapsed time
//! - Refill should happen lazily (on each `try_acquire` call) rather than
//!   on a timer
//! - Tokens should be `f64` to allow fractional accumulation
//! - Cap tokens at `capacity` to prevent unbounded accumulation

use std::time::Instant;

/// A token bucket rate limiter.
///
/// The bucket holds up to `capacity` tokens. Tokens are added at `refill_rate`
/// tokens per second. Each request consumes one or more tokens. If insufficient
/// tokens are available, the request is rejected.
pub struct TokenBucket {
    // TODO: Add fields for capacity, current tokens, refill_rate, and last_refill time
}

impl TokenBucket {
    /// Create a new token bucket with the given capacity and refill rate.
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of tokens the bucket can hold
    /// * `refill_rate` - Tokens added per second
    pub fn new(capacity: u64, refill_rate: f64) -> Self {
        todo!("Implement token bucket creation")
    }

    /// Attempt to acquire the specified number of tokens.
    ///
    /// First refills the bucket based on elapsed time, then checks if enough
    /// tokens are available. If so, deducts them and returns `true`. Otherwise
    /// returns `false` without modifying the bucket.
    ///
    /// # Arguments
    /// * `tokens` - Number of tokens to acquire
    pub fn try_acquire(&mut self, tokens: u64) -> bool {
        todo!("Implement token acquisition with lazy refill")
    }

    /// Refill the bucket based on elapsed time since last refill.
    ///
    /// Adds `refill_rate * elapsed_seconds` tokens, capped at `capacity`.
    fn refill(&mut self) {
        todo!("Implement token refill logic")
    }

    /// Get the current number of tokens available (after refill).
    pub fn available_tokens(&mut self) -> f64 {
        todo!("Return current token count after refill")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_acquisition() {
        let mut bucket = TokenBucket::new(10, 1.0);
        // Should be able to acquire tokens up to capacity
        assert!(bucket.try_acquire(1), "Should acquire 1 token from full bucket");
        assert!(bucket.try_acquire(5), "Should acquire 5 tokens");
        assert!(bucket.try_acquire(4), "Should acquire remaining 4 tokens");
        assert!(!bucket.try_acquire(1), "Bucket should be empty");
    }

    #[test]
    fn test_bucket_exhaustion() {
        let mut bucket = TokenBucket::new(5, 1.0);
        for _ in 0..5 {
            assert!(bucket.try_acquire(1));
        }
        assert!(!bucket.try_acquire(1), "Should fail after exhaustion");
    }

    #[test]
    fn test_refill_over_time() {
        let mut bucket = TokenBucket::new(10, 100.0); // 100 tokens/sec
        // Drain the bucket
        assert!(bucket.try_acquire(10));
        assert!(!bucket.try_acquire(1));

        // Wait for refill
        thread::sleep(Duration::from_millis(100)); // ~10 tokens
        assert!(bucket.try_acquire(1), "Should succeed after refill");
    }

    #[test]
    fn test_burst_handling() {
        let mut bucket = TokenBucket::new(100, 1.0);
        // Should handle a burst of 100
        assert!(bucket.try_acquire(100), "Should handle full burst");
        assert!(!bucket.try_acquire(1), "Burst should exhaust bucket");

        // After 50ms, should have ~0 tokens (rate is 1/sec)
        thread::sleep(Duration::from_millis(50));
        assert!(!bucket.try_acquire(1), "Not enough time for refill at 1/sec");
    }

    #[test]
    fn test_capacity_not_exceeded() {
        let mut bucket = TokenBucket::new(5, 1000.0); // High refill rate
        thread::sleep(Duration::from_millis(100)); // Would add 100 tokens
        let available = bucket.available_tokens();
        assert!(available <= 5.0, "Tokens should not exceed capacity, got {available}");
    }
}
