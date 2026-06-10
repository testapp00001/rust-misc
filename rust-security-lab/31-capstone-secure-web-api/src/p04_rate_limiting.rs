//! # Lesson 04: Rate Limiting Middleware
//!
//! ## Why Rate Limiting?
//!
//! Without rate limiting, a single client can overwhelm your API with thousands of
//! requests per second, causing:
//! - Denial of service for legitimate users
//! - Brute-force attacks on authentication endpoints
//! - Resource exhaustion (CPU, memory, database connections)
//! - Increased costs in pay-per-request cloud environments
//!
//! ## Rate Limiting Algorithms
//!
//! ### Token Bucket
//! A bucket holds tokens. Each request consumes one token. Tokens refill at a
//! constant rate. Allows bursts up to the bucket size while enforcing an average rate.
//!
//! ```text
//! Bucket capacity: 10 tokens
//! Refill rate: 1 token/second
//!
//! t=0:  10 tokens available
//! t=0:  8 requests arrive -> 2 tokens remaining
//! t=5:  5 tokens refilled -> 7 tokens available
//! ```
//!
//! ### Sliding Window Log
//! Records the timestamp of each request. Count requests within the window.
//! Most accurate but requires more memory (stores every timestamp).
//!
//! ### Sliding Window Counter
//! Weighted average of current and previous window counts. Smooth and memory-efficient.
//!
//! ```text
//! Window: 60 seconds, Limit: 100 requests
//! Previous window: 80 requests
//! Current window (30/60 through): 40 requests
//! Weighted count: 80 * (30/60) + 40 = 80
//! 80 < 100 -> ALLOWED
//! ```
//!
//! ## Attack Context
//!
//! - **Brute force**: Attacker tries millions of passwords. Defense: 5 attempts/min on login.
//! - **Credential stuffing**: Attacker tests leaked credentials. Defense: rate limit by IP.
//! - **API abuse**: Bot scrapes all data. Defense: per-user and per-IP limits.
//! - **DDoS**: Distributed flood of requests. Defense: global rate limit + CDN/WAF.

use std::collections::HashMap;

/// Rate limiter decision.
#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitDecision {
    /// Request is allowed
    Allow {
        /// Remaining requests in the current window
        remaining: u32,
    },
    /// Request is rejected (rate limit exceeded)
    Reject {
        /// Seconds until the rate limit resets
        retry_after_secs: u64,
    },
}

/// Token bucket rate limiter state for a single key.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    /// Maximum number of tokens the bucket can hold
    pub capacity: u32,
    /// Current number of tokens available
    pub tokens: f64,
    /// Tokens added per second
    pub refill_rate: f64,
    /// Last time tokens were refilled (Unix timestamp in seconds)
    pub last_refill: u64,
}

/// Sliding window rate limiter entry.
#[derive(Debug, Clone)]
pub struct SlidingWindowEntry {
    /// Request timestamps in the current window
    pub timestamps: Vec<u64>,
}

/// A rate limiter that supports multiple strategies.
#[derive(Debug)]
pub struct RateLimiter {
    /// Per-key token buckets
    pub buckets: HashMap<String, TokenBucket>,
    /// Bucket configuration: (capacity, refill_rate)
    pub bucket_config: (u32, f64),
    /// Per-key sliding window logs
    pub window_logs: HashMap<String, SlidingWindowEntry>,
    /// Window configuration: (window_seconds, max_requests)
    pub window_config: (u64, u32),
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(bucket_capacity: u32, bucket_refill_rate: f64, window_seconds: u64, window_max: u32) -> Self {
        Self {
            buckets: HashMap::new(),
            bucket_config: (bucket_capacity, bucket_refill_rate),
            window_logs: HashMap::new(),
            window_config: (window_seconds, window_max),
        }
    }
}

/// Exercise 1: Implement token bucket rate limiting.
///
/// Steps:
/// 1. Get or create a bucket for the given key with the configured capacity and refill rate
/// 2. Calculate tokens to add based on elapsed time since last refill
/// 3. Cap tokens at the bucket capacity
/// 4. If at least 1 token is available, consume it and return Allow
/// 5. Otherwise return Reject with the time until at least 1 token is available
pub fn check_token_bucket(
    limiter: &mut RateLimiter,
    key: &str,
    current_time: u64,
) -> RateLimitDecision {
    todo!("Implement token bucket rate limiting")
}

/// Exercise 2: Implement sliding window rate limiting.
///
/// Steps:
/// 1. Get or create a window log for the given key
/// 2. Remove timestamps older than the window (current_time - window_seconds)
/// 3. If the count of remaining timestamps is below the max, allow and record the timestamp
/// 4. Otherwise reject with retry_after = oldest_timestamp + window_seconds - current_time
pub fn check_sliding_window(
    limiter: &mut RateLimiter,
    key: &str,
    current_time: u64,
) -> RateLimitDecision {
    todo!("Implement sliding window rate limiting")
}

/// Exercise 3: Implement a composite rate limiter that checks both strategies.
///
/// A request must pass BOTH the token bucket AND the sliding window to be allowed.
/// If either rejects, the request is rejected.
///
/// Check the token bucket first. If it rejects, return that rejection.
/// Then check the sliding window. If it rejects, return that rejection.
/// Otherwise return Allow with the remaining count from the sliding window.
pub fn check_rate_limit(
    limiter: &mut RateLimiter,
    key: &str,
    current_time: u64,
) -> RateLimitDecision {
    todo!("Implement composite rate limiting")
}

/// Exercise 4: Implement per-user rate limiting with different tiers.
///
/// Different user tiers get different rate limits:
/// - "free": 10 requests per minute, bucket capacity 5
/// - "pro": 100 requests per minute, bucket capacity 20
/// - "enterprise": 1000 requests per minute, bucket capacity 100
///
/// Create a RateLimiter configured for the given tier and check the rate limit.
/// If the tier is unknown, use "free" tier limits.
pub fn check_tiered_rate_limit(
    user_id: &str,
    tier: &str,
    current_time: u64,
) -> RateLimitDecision {
    todo!("Implement tiered rate limiting")
}

/// Exercise 5: Implement a rate limit key builder.
///
/// Build a rate limit key that combines multiple dimensions:
/// - IP-based: "ip:<ip_address>"
/// - User-based: "user:<user_id>"
/// - Endpoint-based: "endpoint:<method>:<path>"
/// - Combined: "user:<user_id>:<method>:<path>"
pub fn build_rate_limit_key(user_id: &str, method: &str, path: &str) -> String {
    todo!("Build a composite rate limit key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_allows_within_capacity() {
        let mut limiter = RateLimiter::new(5, 1.0, 60, 100);
        for _ in 0..5 {
            let decision = check_token_bucket(&mut limiter, "user1", 1000);
            assert_eq!(decision, RateLimitDecision::Allow { remaining: _ });
        }
    }

    #[test]
    fn test_token_bucket_rejects_over_capacity() {
        let mut limiter = RateLimiter::new(3, 1.0, 60, 100);
        for _ in 0..3 {
            check_token_bucket(&mut limiter, "user1", 1000);
        }
        let decision = check_token_bucket(&mut limiter, "user1", 1000);
        assert!(matches!(decision, RateLimitDecision::Reject { .. }));
    }

    #[test]
    fn test_token_bucket_refills_over_time() {
        let mut limiter = RateLimiter::new(3, 1.0, 60, 100);
        // Exhaust tokens
        for _ in 0..3 {
            check_token_bucket(&mut limiter, "user1", 1000);
        }
        // Wait 2 seconds (2 tokens refilled)
        let decision = check_token_bucket(&mut limiter, "user1", 1002);
        assert_eq!(decision, RateLimitDecision::Allow { remaining: _ });
    }

    #[test]
    fn test_sliding_window_allows_within_limit() {
        let mut limiter = RateLimiter::new(100, 1.0, 60, 5);
        for i in 0..5 {
            let decision = check_sliding_window(&mut limiter, "user1", 1000 + i);
            assert!(matches!(decision, RateLimitDecision::Allow { .. }));
        }
    }

    #[test]
    fn test_sliding_window_rejects_over_limit() {
        let mut limiter = RateLimiter::new(100, 1.0, 60, 3);
        for i in 0..3 {
            check_sliding_window(&mut limiter, "user1", 1000 + i);
        }
        let decision = check_sliding_window(&mut limiter, "user1", 1000 + 3);
        assert!(matches!(decision, RateLimitDecision::Reject { .. }));
    }

    #[test]
    fn test_sliding_window_expires_old_entries() {
        let mut limiter = RateLimiter::new(100, 1.0, 60, 3);
        for i in 0..3 {
            check_sliding_window(&mut limiter, "user1", 1000 + i);
        }
        // Wait for window to expire
        let decision = check_sliding_window(&mut limiter, "user1", 1100);
        assert!(matches!(decision, RateLimitDecision::Allow { .. }));
    }

    #[test]
    fn test_composite_rate_limit() {
        let mut limiter = RateLimiter::new(2, 1.0, 60, 5);
        // First 2 should pass both checks
        check_rate_limit(&mut limiter, "user1", 1000);
        let decision = check_rate_limit(&mut limiter, "user1", 1000);
        assert!(matches!(decision, RateLimitDecision::Allow { .. }));
        // Third should fail token bucket
        let decision = check_rate_limit(&mut limiter, "user1", 1000);
        assert!(matches!(decision, RateLimitDecision::Reject { .. }));
    }

    #[test]
    fn test_tiered_rate_limit_free() {
        let decision = check_tiered_rate_limit("user1", "free", 1000);
        assert!(matches!(decision, RateLimitDecision::Allow { .. }));
    }

    #[test]
    fn test_build_rate_limit_key() {
        let key = build_rate_limit_key("user123", "GET", "/api/users");
        assert_eq!(key, "user:user123:GET:/api/users");
    }

    #[test]
    fn test_different_keys_independent() {
        let mut limiter = RateLimiter::new(1, 1.0, 60, 100);
        // Exhaust user1's bucket
        check_token_bucket(&mut limiter, "user1", 1000);
        let decision = check_token_bucket(&mut limiter, "user1", 1000);
        assert!(matches!(decision, RateLimitDecision::Reject { .. }));

        // user2 should still be allowed
        let decision = check_token_bucket(&mut limiter, "user2", 1000);
        assert!(matches!(decision, RateLimitDecision::Allow { .. }));
    }
}
