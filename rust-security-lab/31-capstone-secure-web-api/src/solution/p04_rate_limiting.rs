//! # Lesson 04: Rate Limiting — Solution
//!
//! Token bucket, sliding window, per-user limits.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitDecision {
    Allow { remaining: u32 },
    Reject { retry_after_secs: u64 },
}

#[derive(Debug, Clone)]
pub struct TokenBucket {
    pub capacity: u32,
    pub tokens: f64,
    pub refill_rate: f64,
    pub last_refill: u64,
}

#[derive(Debug, Clone)]
pub struct SlidingWindowEntry {
    pub timestamps: Vec<u64>,
}

#[derive(Debug)]
pub struct RateLimiter {
    pub buckets: HashMap<String, TokenBucket>,
    pub bucket_config: (u32, f64),
    pub window_logs: HashMap<String, SlidingWindowEntry>,
    pub window_config: (u64, u32),
}

impl RateLimiter {
    pub fn new(bucket_capacity: u32, bucket_refill_rate: f64, window_seconds: u64, window_max: u32) -> Self {
        Self {
            buckets: HashMap::new(),
            bucket_config: (bucket_capacity, bucket_refill_rate),
            window_logs: HashMap::new(),
            window_config: (window_seconds, window_max),
        }
    }
}

pub fn check_token_bucket(
    limiter: &mut RateLimiter,
    key: &str,
    current_time: u64,
) -> RateLimitDecision {
    let (capacity, refill_rate) = limiter.bucket_config;

    let bucket = limiter.buckets.entry(key.to_string()).or_insert(TokenBucket {
        capacity,
        tokens: capacity as f64,
        refill_rate,
        last_refill: current_time,
    });

    // Calculate tokens to add based on elapsed time
    if current_time > bucket.last_refill {
        let elapsed = (current_time - bucket.last_refill) as f64;
        bucket.tokens = (bucket.tokens + elapsed * bucket.refill_rate).min(bucket.capacity as f64);
        bucket.last_refill = current_time;
    }

    if bucket.tokens >= 1.0 {
        bucket.tokens -= 1.0;
        RateLimitDecision::Allow {
            remaining: bucket.tokens as u32,
        }
    } else {
        // Calculate time until at least 1 token is available
        let deficit = 1.0 - bucket.tokens;
        let wait_secs = (deficit / bucket.refill_rate).ceil() as u64;
        RateLimitDecision::Reject {
            retry_after_secs: wait_secs,
        }
    }
}

pub fn check_sliding_window(
    limiter: &mut RateLimiter,
    key: &str,
    current_time: u64,
) -> RateLimitDecision {
    let (window_seconds, max_requests) = limiter.window_config;
    let window_start = current_time.saturating_sub(window_seconds);

    let entry = limiter
        .window_logs
        .entry(key.to_string())
        .or_insert(SlidingWindowEntry {
            timestamps: Vec::new(),
        });

    // Remove timestamps outside the window
    entry.timestamps.retain(|&ts| ts > window_start);

    if (entry.timestamps.len() as u32) < max_requests {
        entry.timestamps.push(current_time);
        RateLimitDecision::Allow {
            remaining: max_requests - entry.timestamps.len() as u32,
        }
    } else {
        // Calculate retry_after based on the oldest entry
        let oldest = entry.timestamps.first().copied().unwrap_or(current_time);
        let retry_after = oldest + window_seconds - current_time;
        RateLimitDecision::Reject {
            retry_after_secs: retry_after.max(1),
        }
    }
}

pub fn check_rate_limit(
    limiter: &mut RateLimiter,
    key: &str,
    current_time: u64,
) -> RateLimitDecision {
    // Check token bucket first
    match check_token_bucket(limiter, key, current_time) {
        RateLimitDecision::Reject { retry_after_secs } => {
            return RateLimitDecision::Reject { retry_after_secs };
        }
        RateLimitDecision::Allow { .. } => {}
    }

    // Then check sliding window
    check_sliding_window(limiter, key, current_time)
}

pub fn check_tiered_rate_limit(
    user_id: &str,
    tier: &str,
    current_time: u64,
) -> RateLimitDecision {
    let (bucket_capacity, refill_rate, window_max) = match tier {
        "pro" => (20, 100.0 / 60.0, 100),
        "enterprise" => (100, 1000.0 / 60.0, 1000),
        _ => (5, 10.0 / 60.0, 10), // "free" or unknown
    };

    let mut limiter = RateLimiter::new(bucket_capacity, refill_rate, 60, window_max);
    check_rate_limit(&mut limiter, user_id, current_time)
}

pub fn build_rate_limit_key(user_id: &str, method: &str, path: &str) -> String {
    format!("user:{}:{}:{}", user_id, method, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_allows_within_capacity() {
        let mut limiter = RateLimiter::new(5, 1.0, 60, 100);
        for _ in 0..5 {
            let decision = check_token_bucket(&mut limiter, "user1", 1000);
            assert!(matches!(decision, RateLimitDecision::Allow { .. }));
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
        for _ in 0..3 {
            check_token_bucket(&mut limiter, "user1", 1000);
        }
        let decision = check_token_bucket(&mut limiter, "user1", 1002);
        assert!(matches!(decision, RateLimitDecision::Allow { .. }));
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
        let decision = check_sliding_window(&mut limiter, "user1", 1100);
        assert!(matches!(decision, RateLimitDecision::Allow { .. }));
    }

    #[test]
    fn test_composite_rate_limit() {
        let mut limiter = RateLimiter::new(2, 1.0, 60, 5);
        check_rate_limit(&mut limiter, "user1", 1000);
        let decision = check_rate_limit(&mut limiter, "user1", 1000);
        assert!(matches!(decision, RateLimitDecision::Allow { .. }));
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
        check_token_bucket(&mut limiter, "user1", 1000);
        let decision = check_token_bucket(&mut limiter, "user1", 1000);
        assert!(matches!(decision, RateLimitDecision::Reject { .. }));

        let decision = check_token_bucket(&mut limiter, "user2", 1000);
        assert!(matches!(decision, RateLimitDecision::Allow { .. }));
    }
}
