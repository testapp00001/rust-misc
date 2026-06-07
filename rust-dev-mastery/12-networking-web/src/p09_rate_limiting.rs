//! # Rate Limiting
//!
//! Rate limiting protects services from abuse and ensures fair resource allocation.
//! This lesson covers multiple rate limiting algorithms, per-user limits, and
//! building rate limiters as composable middleware.
//!
//! ## Key Concepts
//! - Token bucket algorithm
//! - Sliding window log
//! - Sliding window counter
//! - Per-user and per-IP limits
//! - Rate limit response headers
//! - Distributed rate limiting concepts

use std::collections::{HashMap, VecDeque};
use std::time::Duration;

// ---------------------------------------------------------------------------
// 1. Token Bucket Algorithm
// ---------------------------------------------------------------------------

/// The token bucket algorithm allows bursts up to the bucket size while
/// maintaining an average rate defined by the refill rate.
///
/// Use case: API endpoints where occasional bursts are acceptable.
#[derive(Debug)]
pub struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: f64, // timestamp in seconds
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: capacity as f64,
            refill_rate,
            last_refill: 0.0,
        }
    }

    /// Try to consume `n` tokens. Returns true if allowed.
    pub fn try_consume(&mut self, n: u32, now: f64) -> bool {
        self.refill(now);
        let requested = n as f64;
        if self.tokens >= requested {
            self.tokens -= requested;
            true
        } else {
            false
        }
    }

    /// Try to consume exactly 1 token.
    pub fn try_acquire(&mut self, now: f64) -> bool {
        self.try_consume(1, now)
    }

    fn refill(&mut self, now: f64) {
        let elapsed = now - self.last_refill;
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
            self.last_refill = now;
        }
    }

    pub fn available_tokens(&self) -> u32 {
        self.tokens.floor() as u32
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }
}

// ---------------------------------------------------------------------------
// 2. Sliding Window Log
// ---------------------------------------------------------------------------

/// The sliding window log stores timestamps of each request, providing
/// exact counts but using more memory.
///
/// Use case: When exact precision is required and request volume is moderate.
#[derive(Debug)]
pub struct SlidingWindowLog {
    max_requests: usize,
    window_secs: f64,
    timestamps: VecDeque<f64>,
}

impl SlidingWindowLog {
    pub fn new(max_requests: usize, window_secs: f64) -> Self {
        Self {
            max_requests,
            window_secs,
            timestamps: VecDeque::new(),
        }
    }

    /// Try to allow a request at the given timestamp.
    pub fn try_acquire(&mut self, now: f64) -> bool {
        self.evict(now);
        if self.timestamps.len() < self.max_requests {
            self.timestamps.push_back(now);
            true
        } else {
            false
        }
    }

    /// How many requests are currently in the window.
    pub fn current_count(&self, now: f64) -> usize {
        let window_start = now - self.window_secs;
        self.timestamps.iter().filter(|t| **t >= window_start).count()
    }

    fn evict(&mut self, now: f64) {
        let window_start = now - self.window_secs;
        while self.timestamps.front().map_or(false, |t| *t <= window_start) {
            self.timestamps.pop_front();
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Sliding Window Counter
// ---------------------------------------------------------------------------

/// The sliding window counter is a memory-efficient approximation that
/// combines the current and previous window counts.
///
/// Use case: High-throughput systems where memory efficiency matters.
#[derive(Debug)]
pub struct SlidingWindowCounter {
    max_requests: u32,
    window_secs: f64,
    prev_count: u32,
    curr_count: u32,
    curr_window_start: f64,
}

impl SlidingWindowCounter {
    pub fn new(max_requests: u32, window_secs: f64) -> Self {
        Self {
            max_requests,
            window_secs,
            prev_count: 0,
            curr_count: 0,
            curr_window_start: 0.0,
        }
    }

    /// Try to allow a request.
    pub fn try_acquire(&mut self, now: f64) -> bool {
        self.advance_window(now);

        let estimated = self.estimated_count(now);
        if estimated < self.max_requests as f64 {
            self.curr_count += 1;
            true
        } else {
            false
        }
    }

    /// Get the estimated current count using weighted average.
    fn estimated_count(&self, now: f64) -> f64 {
        let elapsed = now - self.curr_window_start;
        let weight = 1.0 - (elapsed / self.window_secs);
        (self.prev_count as f64) * weight + (self.curr_count as f64)
    }

    fn advance_window(&mut self, now: f64) {
        let windows_elapsed =
            ((now - self.curr_window_start) / self.window_secs).floor() as u32;

        if windows_elapsed >= 2 {
            self.prev_count = 0;
            self.curr_count = 0;
            self.curr_window_start = now;
        } else if windows_elapsed == 1 {
            self.prev_count = self.curr_count;
            self.curr_count = 0;
            self.curr_window_start += self.window_secs;
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Fixed Window Counter (simplest approach)
// ---------------------------------------------------------------------------

/// Fixed window counter divides time into fixed intervals and counts
/// requests within each interval.
#[derive(Debug)]
pub struct FixedWindowCounter {
    max_requests: u32,
    window_secs: f64,
    count: u32,
    window_start: f64,
}

impl FixedWindowCounter {
    pub fn new(max_requests: u32, window_secs: f64) -> Self {
        Self {
            max_requests,
            window_secs,
            count: 0,
            window_start: 0.0,
        }
    }

    pub fn try_acquire(&mut self, now: f64) -> bool {
        if now - self.window_start >= self.window_secs {
            self.count = 0;
            self.window_start = now;
        }

        if self.count < self.max_requests {
            self.count += 1;
            true
        } else {
            false
        }
    }

    pub fn current_count(&self) -> u32 {
        self.count
    }

    pub fn remaining(&self) -> u32 {
        self.max_requests.saturating_sub(self.count)
    }
}

// ---------------------------------------------------------------------------
// 5. Per-Key Rate Limiter
// ---------------------------------------------------------------------------

/// Manages rate limits for multiple keys (users, IPs, API keys, etc.)
#[derive(Debug)]
pub struct KeyedRateLimiter<L> {
    limits: HashMap<String, L>,
    default_config: LimiterConfig,
}

#[derive(Debug, Clone)]
pub struct LimiterConfig {
    pub max_requests: u32,
    pub window_secs: f64,
}

impl KeyedRateLimiter<FixedWindowCounter> {
    pub fn new_fixed_window(config: LimiterConfig) -> Self {
        Self {
            limits: HashMap::new(),
            default_config: config,
        }
    }

    pub fn try_acquire(&mut self, key: &str, now: f64) -> bool {
        let config = &self.default_config;
        let limiter = self
            .limits
            .entry(key.to_string())
            .or_insert_with(|| FixedWindowCounter::new(config.max_requests, config.window_secs));
        limiter.try_acquire(now)
    }

    pub fn remaining(&self, key: &str) -> u32 {
        self.limits
            .get(key)
            .map(|l| l.remaining())
            .unwrap_or(self.default_config.max_requests)
    }

    pub fn active_keys(&self) -> usize {
        self.limits.len()
    }
}

// ---------------------------------------------------------------------------
// 6. Rate Limit Headers
// ---------------------------------------------------------------------------

/// Standard rate limit response headers (draft-ietf-httpapi-ratelimit-headers).
#[derive(Debug, Clone)]
pub struct RateLimitHeaders {
    pub limit: u32,
    pub remaining: u32,
    pub reset: u64, // seconds until reset
}

impl RateLimitHeaders {
    pub fn new(limit: u32, remaining: u32, reset: u64) -> Self {
        Self {
            limit,
            remaining,
            reset,
        }
    }

    /// Convert to HTTP headers.
    pub fn to_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("ratelimit-limit".into(), self.limit.to_string());
        headers.insert("ratelimit-remaining".into(), self.remaining.to_string());
        headers.insert("ratelimit-reset".into(), self.reset.to_string());
        headers
    }
}

// ---------------------------------------------------------------------------
// 7. Rate Limit Result
// ---------------------------------------------------------------------------

/// The result of a rate limit check.
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed {
        remaining: u32,
        limit: u32,
        reset_at: u64,
    },
    Denied {
        retry_after: u64,
        limit: u32,
    },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }

    pub fn headers(&self) -> HashMap<String, String> {
        match self {
            RateLimitResult::Allowed {
                remaining,
                limit,
                reset_at,
            } => {
                let mut h = HashMap::new();
                h.insert("ratelimit-limit".into(), limit.to_string());
                h.insert("ratelimit-remaining".into(), remaining.to_string());
                h.insert("ratelimit-reset".into(), reset_at.to_string());
                h
            }
            RateLimitResult::Denied { retry_after, limit } => {
                let mut h = HashMap::new();
                h.insert("ratelimit-limit".into(), limit.to_string());
                h.insert("ratelimit-remaining".into(), "0".into());
                h.insert("retry-after".into(), retry_after.to_string());
                h
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Composite Rate Limiter (multiple tiers)
// ---------------------------------------------------------------------------

/// Applies multiple rate limit tiers (e.g., per-second AND per-minute).
#[derive(Debug)]
pub struct TieredRateLimiter {
    tiers: Vec<(String, FixedWindowCounter)>,
}

impl TieredRateLimiter {
    pub fn new() -> Self {
        Self { tiers: Vec::new() }
    }

    pub fn add_tier(
        mut self,
        name: &str,
        max_requests: u32,
        window_secs: f64,
    ) -> Self {
        self.tiers.push((
            name.into(),
            FixedWindowCounter::new(max_requests, window_secs),
        ));
        self
    }

    /// Try to acquire a slot. All tiers must allow it.
    pub fn try_acquire(&mut self, now: f64) -> Result<(), String> {
        for (name, tier) in &mut self.tiers {
            if !tier.try_acquire(now) {
                return Err(format!("rate limit exceeded for tier: {name}"));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(5, 1.0); // 5 capacity, 1/sec refill
        assert_eq!(bucket.capacity(), 5);

        // Can consume up to capacity
        for i in 0..5 {
            assert!(bucket.try_acquire(0.0), "token {i} should be available");
        }
        // 6th should fail
        assert!(!bucket.try_acquire(0.0));
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(3, 2.0); // 2 tokens/sec

        // Drain
        assert!(bucket.try_acquire(0.0));
        assert!(bucket.try_acquire(0.0));
        assert!(bucket.try_acquire(0.0));
        assert!(!bucket.try_acquire(0.0));

        // Wait 1 second -> refill 2 tokens
        assert!(bucket.try_acquire(1.0));
        assert!(bucket.try_acquire(1.0));
        assert!(!bucket.try_acquire(1.0));
    }

    #[test]
    fn test_token_bucket_partial_refill() {
        let mut bucket = TokenBucket::new(10, 5.0);

        // Drain all
        for _ in 0..10 {
            bucket.try_acquire(0.0);
        }

        // 0.5 seconds -> 2.5 tokens refilled -> 2 available
        assert!(bucket.try_acquire(0.5));
        assert!(bucket.try_acquire(0.5));
        assert!(!bucket.try_acquire(0.5));
    }

    #[test]
    fn test_token_bucket_consume_multiple() {
        let mut bucket = TokenBucket::new(10, 1.0);
        assert!(bucket.try_consume(5, 0.0));
        assert!(bucket.try_consume(5, 0.0));
        assert!(!bucket.try_consume(1, 0.0));
    }

    #[test]
    fn test_sliding_window_log_basic() {
        let mut log = SlidingWindowLog::new(3, 10.0);

        assert!(log.try_acquire(1.0));
        assert!(log.try_acquire(2.0));
        assert!(log.try_acquire(3.0));
        assert!(!log.try_acquire(4.0)); // at limit
    }

    #[test]
    fn test_sliding_window_log_eviction() {
        let mut log = SlidingWindowLog::new(2, 5.0);

        assert!(log.try_acquire(1.0));
        assert!(log.try_acquire(2.0));
        assert!(!log.try_acquire(3.0)); // at limit

        // After window passes
        assert!(log.try_acquire(7.0)); // first two are evicted
    }

    #[test]
    fn test_sliding_window_log_count() {
        let mut log = SlidingWindowLog::new(10, 5.0);
        log.try_acquire(1.0);
        log.try_acquire(2.0);
        log.try_acquire(3.0);

        assert_eq!(log.current_count(4.0), 3);
        assert_eq!(log.current_count(7.0), 2); // first one evicted
        assert_eq!(log.current_count(10.0), 0); // all evicted
    }

    #[test]
    fn test_sliding_window_counter_basic() {
        let mut counter = SlidingWindowCounter::new(5, 10.0);

        for _ in 0..5 {
            assert!(counter.try_acquire(0.0));
        }
        // Approximate: might allow or deny depending on weight calculation
    }

    #[test]
    fn test_fixed_window_counter() {
        let mut counter = FixedWindowCounter::new(3, 10.0);

        assert!(counter.try_acquire(0.0));
        assert!(counter.try_acquire(0.0));
        assert!(counter.try_acquire(0.0));
        assert!(!counter.try_acquire(0.0));
        assert_eq!(counter.remaining(), 0);
    }

    #[test]
    fn test_fixed_window_counter_reset() {
        let mut counter = FixedWindowCounter::new(2, 5.0);

        assert!(counter.try_acquire(0.0));
        assert!(counter.try_acquire(0.0));
        assert!(!counter.try_acquire(0.0));

        // New window
        assert!(counter.try_acquire(6.0));
        assert_eq!(counter.remaining(), 1);
    }

    #[test]
    fn test_keyed_rate_limiter() {
        let config = LimiterConfig {
            max_requests: 2,
            window_secs: 10.0,
        };
        let mut limiter = KeyedRateLimiter::new_fixed_window(config);

        assert!(limiter.try_acquire("user-1", 0.0));
        assert!(limiter.try_acquire("user-1", 0.0));
        assert!(!limiter.try_acquire("user-1", 0.0));

        // Different user
        assert!(limiter.try_acquire("user-2", 0.0));
        assert_eq!(limiter.remaining("user-2"), 1);
    }

    #[test]
    fn test_keyed_rate_limiter_active_keys() {
        let config = LimiterConfig {
            max_requests: 10,
            window_secs: 60.0,
        };
        let mut limiter = KeyedRateLimiter::new_fixed_window(config);

        limiter.try_acquire("a", 0.0);
        limiter.try_acquire("b", 0.0);
        limiter.try_acquire("c", 0.0);
        assert_eq!(limiter.active_keys(), 3);
    }

    #[test]
    fn test_rate_limit_headers() {
        let headers = RateLimitHeaders::new(100, 75, 30);
        let h = headers.to_headers();
        assert_eq!(h.get("ratelimit-limit").unwrap(), "100");
        assert_eq!(h.get("ratelimit-remaining").unwrap(), "75");
        assert_eq!(h.get("ratelimit-reset").unwrap(), "30");
    }

    #[test]
    fn test_rate_limit_result_allowed() {
        let result = RateLimitResult::Allowed {
            remaining: 50,
            limit: 100,
            reset_at: 1000,
        };
        assert!(result.is_allowed());
        let headers = result.headers();
        assert_eq!(headers.get("ratelimit-remaining").unwrap(), "50");
    }

    #[test]
    fn test_rate_limit_result_denied() {
        let result = RateLimitResult::Denied {
            retry_after: 30,
            limit: 100,
        };
        assert!(!result.is_allowed());
        let headers = result.headers();
        assert_eq!(headers.get("retry-after").unwrap(), "30");
        assert_eq!(headers.get("ratelimit-remaining").unwrap(), "0");
    }

    #[test]
    fn test_tiered_rate_limiter() {
        let mut limiter = TieredRateLimiter::new()
            .add_tier("per-second", 5, 1.0)
            .add_tier("per-minute", 100, 60.0);

        // Should allow up to per-second limit
        for _ in 0..5 {
            assert!(limiter.try_acquire(0.0).is_ok());
        }
        assert!(limiter.try_acquire(0.0).is_err());
    }

    #[test]
    fn test_tiered_rate_limiter_minute_limit() {
        let mut limiter = TieredRateLimiter::new()
            .add_tier("per-second", 1000, 1.0)
            .add_tier("per-minute", 3, 60.0);

        assert!(limiter.try_acquire(0.0).is_ok());
        assert!(limiter.try_acquire(0.0).is_ok());
        assert!(limiter.try_acquire(0.0).is_ok());
        assert!(limiter.try_acquire(0.0).is_err());
    }

    #[test]
    fn test_token_bucket_capacity_preserved_after_refill() {
        let mut bucket = TokenBucket::new(5, 10.0);
        // Use 3
        bucket.try_consume(3, 0.0);
        assert_eq!(bucket.available_tokens(), 2);

        // Refill way past capacity
        bucket.try_acquire(100.0);
        assert!(bucket.available_tokens() <= 5); // capped at capacity
    }
}
