//! # Lesson 01: Rate Limiting
//!
//! ## Why Rate Limiting?
//!
//! Without rate limiting, a single client can overwhelm your API with thousands
//! of requests per second, causing denial of service for everyone. Rate limiting
//! protects against:
//!
//! - **Brute force attacks**: Automated password guessing
//! - **DoS attacks**: Overwhelming the server with requests
//! - **Resource exhaustion**: Database connections, CPU, memory
//! - **API abuse**: Scraping, data harvesting
//!
//! ## Token Bucket Algorithm
//!
//! The token bucket algorithm models a bucket that holds tokens:
//!
//! ```text
//! Bucket capacity: 10 tokens
//! Refill rate: 2 tokens/second
//!
//! Time 0s:  10 tokens available
//! 5 requests → 5 tokens left
//! Time 1s:  7 tokens (refilled 2)
//! 8 requests → rejected after 7, bucket empty
//! Time 2s:  2 tokens (refilled 2)
//! ```
//!
//! ## Sliding Window Algorithm
//!
//! Tracks timestamps of requests within a rolling window:
//!
//! ```text
//! Window: 60 seconds, Max: 100 requests
//!
//! At T=100s, window is [T=40s, T=100s]
//! Count requests in window → if < 100, allow; else reject
//! ```
//!
//! More accurate than fixed windows, which can allow 2x burst at window boundaries.
//!
//! ## Attack: Rate Limit Bypass
//!
//! Attackers bypass rate limiting by:
//! 1. Rotating IP addresses via botnets
//! 2. Using multiple API keys or accounts
//! 3. Manipulating headers: `X-Forwarded-For`, `X-Real-IP`
//! 4. Sending requests at exactly the limit (low-and-slow)
//!
//! ## Defense
//!
//! 1. Rate limit by multiple dimensions: IP, user, API key, endpoint
//! 2. Use sliding window over fixed window for accuracy
//! 3. Don't trust client-provided IP headers without proxy validation
//! 4. Apply different limits to different endpoint sensitivity levels
//! 5. Return `429 Too Many Requests` with `Retry-After` header

use std::collections::HashMap;

/// Exercise 1: Implement a simple fixed-window rate limiter.
///
/// Track how many requests each client (identified by a string key) has made
/// within the current time window. A window is defined by `window_seconds` and
/// `max_requests`.
///
/// You get the current timestamp as `now_secs` (Unix epoch seconds).
///
/// Rules:
/// - Each client has a counter and a window start time
/// - If `now_secs` is beyond the current window, reset the counter and window start
/// - If the counter is below `max_requests`, increment and return `true` (allowed)
/// - If the counter has reached `max_requests`, return `false` (rejected)
///
/// Hints:
/// - Use `HashMap<String, (u64, u32)>` to store (window_start, request_count) per client
/// - The current window starts at: `(now_secs / window_seconds) * window_seconds`
pub struct FixedWindowLimiter {
    /// (window_start_epoch_secs, request_count) per client
    windows: HashMap<String, (u64, u32)>,
    window_seconds: u64,
    max_requests: u32,
}

impl FixedWindowLimiter {
    pub fn new(window_seconds: u64, max_requests: u32) -> Self {
        todo!("Initialize the fixed window rate limiter")
    }

    /// Returns true if the request is allowed, false if rate-limited.
    pub fn allow_request(&mut self, client_id: &str, now_secs: u64) -> bool {
        todo!("Check and update the rate limit for this client")
    }

    /// Returns the number of requests remaining for this client in the current window.
    pub fn remaining(&self, client_id: &str, now_secs: u64) -> u32 {
        todo!("Calculate remaining requests for this client")
    }
}

/// Exercise 2: Implement a token bucket rate limiter.
///
/// The token bucket holds up to `capacity` tokens. Tokens refill at `refill_rate`
/// per second. Each request consumes one token.
///
/// Rules:
/// - On each request, first calculate how many tokens to add based on elapsed time
/// - Cap the token count at `capacity`
/// - If tokens > 0, consume one and return `true`
/// - If tokens == 0, return `false`
///
/// Hints:
/// - Store (last_refill_time, available_tokens) per client
/// - Tokens to add = elapsed_seconds * refill_rate
/// - Use `f64` for fractional token accumulation, or `u64` for integer-only
pub struct TokenBucketLimiter {
    buckets: HashMap<String, (u64, f64)>,
    capacity: f64,
    refill_rate: f64,
}

impl TokenBucketLimiter {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        todo!("Initialize the token bucket limiter")
    }

    /// Returns true if the request is allowed, false if rate-limited.
    pub fn allow_request(&mut self, client_id: &str, now_millis: u64) -> bool {
        todo!("Refill tokens and consume one if available")
    }

    /// Returns the current number of available tokens for a client.
    pub fn available_tokens(&mut self, client_id: &str, now_millis: u64) -> f64 {
        todo!("Calculate current available tokens after refill")
    }
}

/// Exercise 3: Implement a sliding window rate limiter.
///
/// Instead of a fixed window, this tracks individual request timestamps and
/// counts how many fall within the last `window_seconds` from the current time.
///
/// Rules:
/// - Store a list of request timestamps per client
/// - On each request, remove timestamps older than `window_seconds` from now
/// - If the remaining count is below `max_requests`, add the new timestamp and return `true`
/// - Otherwise return `false`
///
/// Hints:
/// - Use `Vec<u64>` of timestamps per client
/// - Retain only timestamps where `now_secs - timestamp < window_seconds`
/// - Check `len()` against `max_requests`
pub struct SlidingWindowLimiter {
    requests: HashMap<String, Vec<u64>>,
    window_seconds: u64,
    max_requests: usize,
}

impl SlidingWindowLimiter {
    pub fn new(window_seconds: u64, max_requests: usize) -> Self {
        todo!("Initialize the sliding window limiter")
    }

    /// Returns true if the request is allowed, false if rate-limited.
    pub fn allow_request(&mut self, client_id: &str, now_secs: u64) -> bool {
        todo!("Slide the window and check the request count")
    }

    /// Returns the number of requests in the current window for this client.
    pub fn current_count(&mut self, client_id: &str, now_secs: u64) -> usize {
        todo!("Count requests in the current window")
    }
}

/// Exercise 4: Multi-tier rate limiting.
///
/// Apply multiple rate limits to the same client. A request is allowed only if
/// ALL tiers allow it. For example:
/// - Tier 1: 10 requests per second (burst protection)
/// - Tier 2: 100 requests per minute (sustained rate)
/// - Tier 3: 1000 requests per hour (daily abuse)
///
/// Implement this by composing multiple `FixedWindowLimiter` instances.
///
/// Hints:
/// - Store a Vec of `FixedWindowLimiter` with different window sizes
/// - A request passes only if all limiters return `true`
/// - Use mutable references carefully since each limiter needs to be updated
pub struct MultiTierLimiter {
    tiers: Vec<FixedWindowLimiter>,
}

impl MultiTierLimiter {
    /// Create a new multi-tier limiter. Each tier is (window_seconds, max_requests).
    pub fn new(tiers: &[(u64, u32)]) -> Self {
        todo!("Create a limiter with multiple tiers")
    }

    /// Returns true only if ALL tiers allow the request.
    pub fn allow_request(&mut self, client_id: &str, now_secs: u64) -> bool {
        todo!("Check all tiers and allow only if all pass")
    }
}

/// Exercise 5: Rate limit response with Retry-After.
///
/// When a request is rate-limited, calculate the number of seconds the client
/// should wait before retrying. This value goes in the `Retry-After` HTTP header.
///
/// Given the window start time, window duration, and current time, return
/// `Some(retry_after_seconds)` if rate-limited, or `None` if allowed.
///
/// For a fixed window: retry after = window_end - now
/// For a token bucket: retry after = time to accumulate 1 token
///
/// Hints:
/// - window_end = window_start + window_seconds
/// - retry_after = window_end.saturating_sub(now_secs)
pub fn calculate_retry_after(window_start: u64, window_seconds: u64, now_secs: u64) -> Option<u64> {
    todo!("Calculate Retry-After header value")
}

/// Exercise 6: IP-based rate limiting with header trust.
///
/// Extract the client IP from request headers, choosing the most trustworthy
/// source. This demonstrates the risk of blindly trusting proxy headers.
///
/// Given a map of headers, extract the client IP using this priority:
/// 1. If `X-Real-IP` exists and is a valid IP, use it (set by a trusted reverse proxy)
/// 2. If `X-Forwarded-For` exists, use the FIRST IP in the comma-separated list
/// 3. Otherwise, use the provided `remote_addr` (TCP connection IP)
///
/// Attack context: An attacker can forge `X-Forwarded-For` to bypass IP-based
/// rate limiting. Only trust these headers from known proxies.
///
/// Hints:
/// - `X-Forwarded-For` format: "client, proxy1, proxy2"
/// - Split on ',' and take the first, trim whitespace
/// - Validate it looks like an IP (basic check: contains '.' or ':')
pub fn extract_client_ip(headers: &HashMap<String, String>, remote_addr: &str) -> String {
    todo!("Extract the most trustworthy client IP from headers")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Fixed Window Tests ──

    #[test]
    fn test_fixed_window_allows_within_limit() {
        let mut limiter = FixedWindowLimiter::new(60, 5);
        for i in 0..5 {
            assert!(limiter.allow_request("user1", 1000 + i), "Request {} should be allowed", i);
        }
    }

    #[test]
    fn test_fixed_window_rejects_over_limit() {
        let mut limiter = FixedWindowLimiter::new(60, 3);
        assert!(limiter.allow_request("user1", 1000));
        assert!(limiter.allow_request("user1", 1001));
        assert!(limiter.allow_request("user1", 1002));
        assert!(!limiter.allow_request("user1", 1003), "4th request should be rejected");
    }

    #[test]
    fn test_fixed_window_resets_on_new_window() {
        let mut limiter = FixedWindowLimiter::new(60, 2);
        assert!(limiter.allow_request("user1", 1000));
        assert!(limiter.allow_request("user1", 1001));
        assert!(!limiter.allow_request("user1", 1002));
        // New window starts at 1020 (60-second boundary)
        assert!(limiter.allow_request("user1", 1020), "New window should allow requests");
    }

    #[test]
    fn test_fixed_window_separate_clients() {
        let mut limiter = FixedWindowLimiter::new(60, 2);
        assert!(limiter.allow_request("user1", 1000));
        assert!(limiter.allow_request("user2", 1000));
        assert!(limiter.allow_request("user1", 1001));
        assert!(limiter.allow_request("user2", 1001));
        assert!(!limiter.allow_request("user1", 1002));
        assert!(!limiter.allow_request("user2", 1002));
    }

    #[test]
    fn test_fixed_window_remaining() {
        let mut limiter = FixedWindowLimiter::new(60, 5);
        assert_eq!(limiter.remaining("user1", 1000), 5);
        limiter.allow_request("user1", 1000);
        assert_eq!(limiter.remaining("user1", 1000), 4);
        limiter.allow_request("user1", 1001);
        assert_eq!(limiter.remaining("user1", 1001), 3);
    }

    // ── Token Bucket Tests ──

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucketLimiter::new(5.0, 1.0);
        // 5 requests should be allowed immediately (capacity = 5)
        for _ in 0..5 {
            assert!(bucket.allow_request("user1", 1000));
        }
        // 6th should be rejected (same time, no refill)
        assert!(!bucket.allow_request("user1", 1000));
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucketLimiter::new(5.0, 1.0);
        // Drain all tokens at the same time (burst)
        for _ in 0..5 {
            bucket.allow_request("user1", 1000);
        }
        assert!(!bucket.allow_request("user1", 1000));
        // Wait 3 seconds -> 3 tokens refilled (capped at capacity 5)
        assert!(bucket.allow_request("user1", 1003));
        assert!(bucket.allow_request("user1", 1003));
        assert!(bucket.allow_request("user1", 1003));
        assert!(!bucket.allow_request("user1", 1003));
    }

    #[test]
    fn test_token_bucket_cap_at_capacity() {
        let mut bucket = TokenBucketLimiter::new(3.0, 10.0);
        // After 10 seconds, should be capped at 3, not 30
        let tokens = bucket.available_tokens("user1", 10_000);
        assert!(tokens <= 3.0, "Tokens should be capped at capacity, got {}", tokens);
    }

    #[test]
    fn test_token_bucket_separate_clients() {
        let mut bucket = TokenBucketLimiter::new(2.0, 1.0);
        assert!(bucket.allow_request("a", 1000));
        assert!(bucket.allow_request("a", 1000));
        assert!(!bucket.allow_request("a", 1000));
        // Different client has its own bucket
        assert!(bucket.allow_request("b", 1000));
    }

    // ── Sliding Window Tests ──

    #[test]
    fn test_sliding_window_allows_within_limit() {
        let mut limiter = SlidingWindowLimiter::new(60, 5);
        for i in 0..5 {
            assert!(limiter.allow_request("user1", 1000 + i));
        }
    }

    #[test]
    fn test_sliding_window_rejects_over_limit() {
        let mut limiter = SlidingWindowLimiter::new(60, 3);
        assert!(limiter.allow_request("user1", 1000));
        assert!(limiter.allow_request("user1", 1001));
        assert!(limiter.allow_request("user1", 1002));
        assert!(!limiter.allow_request("user1", 1003));
    }

    #[test]
    fn test_sliding_window_expires_old_entries() {
        let mut limiter = SlidingWindowLimiter::new(60, 3);
        assert!(limiter.allow_request("user1", 1000));
        assert!(limiter.allow_request("user1", 1001));
        assert!(limiter.allow_request("user1", 1002));
        assert!(!limiter.allow_request("user1", 1003));
        // After the first request falls out of the window (1000 + 60 = 1060)
        assert!(limiter.allow_request("user1", 1061));
    }

    #[test]
    fn test_sliding_window_count() {
        let mut limiter = SlidingWindowLimiter::new(60, 10);
        limiter.allow_request("user1", 1000);
        limiter.allow_request("user1", 1001);
        limiter.allow_request("user1", 1030);
        assert_eq!(limiter.current_count("user1", 1030), 3);
        // After window slides past 1000
        assert_eq!(limiter.current_count("user1", 1061), 2);
    }

    // ── Multi-Tier Tests ──

    #[test]
    fn test_multi_tier_allows_within_all_tiers() {
        let mut limiter = MultiTierLimiter::new(&[(1, 10), (60, 100)]);
        for i in 0..10 {
            assert!(limiter.allow_request("user1", 1000 + i / 10));
        }
    }

    #[test]
    fn test_multi_tier_rejects_when_first_tier_exceeded() {
        let mut limiter = MultiTierLimiter::new(&[(1, 3), (60, 100)]);
        assert!(limiter.allow_request("user1", 1000));
        assert!(limiter.allow_request("user1", 1000));
        assert!(limiter.allow_request("user1", 1000));
        assert!(!limiter.allow_request("user1", 1000));
    }

    #[test]
    fn test_multi_tier_rejects_when_second_tier_exceeded() {
        let mut limiter = MultiTierLimiter::new(&[(1, 100), (60, 3)]);
        assert!(limiter.allow_request("user1", 1000));
        assert!(limiter.allow_request("user1", 1001));
        assert!(limiter.allow_request("user1", 1002));
        // Second tier (60-second window) has a limit of 3
        assert!(!limiter.allow_request("user1", 1003));
    }

    // ── Retry-After Tests ──

    #[test]
    fn test_retry_after_calculation() {
        // Window starts at 1000, lasts 60 seconds, now is 1040
        // Retry-After should be 20 seconds
        assert_eq!(calculate_retry_after(1000, 60, 1040), Some(20));
    }

    #[test]
    fn test_retry_after_zero_when_at_boundary() {
        assert_eq!(calculate_retry_after(1000, 60, 1060), Some(0));
    }

    #[test]
    fn test_retry_after_none_when_in_past() {
        // If window already ended, no retry-after needed
        assert_eq!(calculate_retry_after(1000, 60, 1061), None);
    }

    // ── IP Extraction Tests ──

    #[test]
    fn test_extract_ip_from_x_real_ip() {
        let mut headers = HashMap::new();
        headers.insert("X-Real-IP".to_string(), "1.2.3.4".to_string());
        headers.insert("X-Forwarded-For".to_string(), "5.6.7.8".to_string());
        assert_eq!(extract_client_ip(&headers, "9.9.9.9"), "1.2.3.4");
    }

    #[test]
    fn test_extract_ip_from_x_forwarded_for() {
        let mut headers = HashMap::new();
        headers.insert("X-Forwarded-For".to_string(), "10.0.0.1, 10.0.0.2".to_string());
        assert_eq!(extract_client_ip(&headers, "9.9.9.9"), "10.0.0.1");
    }

    #[test]
    fn test_extract_ip_fallback_to_remote_addr() {
        let headers = HashMap::new();
        assert_eq!(extract_client_ip(&headers, "192.168.1.1"), "192.168.1.1");
    }
}
