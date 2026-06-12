//! # Solution 01: Token Bucket Rate Limiter
//!
//! Complete implementation of a token bucket rate limiter with lazy refill.

use std::time::Instant;

/// A token bucket rate limiter.
///
/// The bucket holds up to `capacity` tokens. Tokens are added at `refill_rate`
/// tokens per second. Each request consumes one or more tokens. If insufficient
/// tokens are available, the request is rejected.
pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket with the given capacity and refill rate.
    pub fn new(capacity: u64, refill_rate: f64) -> Self {
        Self {
            capacity: capacity as f64,
            tokens: capacity as f64,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Attempt to acquire the specified number of tokens.
    ///
    /// First refills the bucket based on elapsed time, then checks if enough
    /// tokens are available. If so, deducts them and returns `true`.
    pub fn try_acquire(&mut self, tokens: u64) -> bool {
        self.refill();
        let requested = tokens as f64;
        if self.tokens >= requested {
            self.tokens -= requested;
            true
        } else {
            false
        }
    }

    /// Refill the bucket based on elapsed time since last refill.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
    }

    /// Get the current number of tokens available (after refill).
    pub fn available_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
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
        let mut bucket = TokenBucket::new(10, 100.0);
        assert!(bucket.try_acquire(10));
        assert!(!bucket.try_acquire(1));

        thread::sleep(Duration::from_millis(100));
        assert!(bucket.try_acquire(1), "Should succeed after refill");
    }

    #[test]
    fn test_burst_handling() {
        let mut bucket = TokenBucket::new(100, 1.0);
        assert!(bucket.try_acquire(100), "Should handle full burst");
        assert!(!bucket.try_acquire(1), "Burst should exhaust bucket");

        thread::sleep(Duration::from_millis(50));
        assert!(!bucket.try_acquire(1), "Not enough time for refill at 1/sec");
    }

    #[test]
    fn test_capacity_not_exceeded() {
        let mut bucket = TokenBucket::new(5, 1000.0);
        thread::sleep(Duration::from_millis(100));
        let available = bucket.available_tokens();
        assert!(available <= 5.0, "Tokens should not exceed capacity, got {available}");
    }
}
