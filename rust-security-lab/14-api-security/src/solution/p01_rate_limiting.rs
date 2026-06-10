//! # Lesson 01: Rate Limiting (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;

// ── Fixed Window Limiter ──

pub struct FixedWindowLimiter {
    windows: HashMap<String, (u64, u32)>,
    window_seconds: u64,
    max_requests: u32,
}

impl FixedWindowLimiter {
    pub fn new(window_seconds: u64, max_requests: u32) -> Self {
        Self {
            windows: HashMap::new(),
            window_seconds,
            max_requests,
        }
    }

    pub fn allow_request(&mut self, client_id: &str, now_secs: u64) -> bool {
        let window_start = (now_secs / self.window_seconds) * self.window_seconds;
        let entry = self.windows
            .entry(client_id.to_string())
            .or_insert((window_start, 0));

        if entry.0 != window_start {
            *entry = (window_start, 0);
        }

        if entry.1 < self.max_requests {
            entry.1 += 1;
            true
        } else {
            false
        }
    }

    pub fn remaining(&self, client_id: &str, now_secs: u64) -> u32 {
        let window_start = (now_secs / self.window_seconds) * self.window_seconds;
        match self.windows.get(client_id) {
            Some((start, count)) if *start == window_start => {
                self.max_requests.saturating_sub(*count)
            }
            _ => self.max_requests,
        }
    }
}

// ── Token Bucket Limiter ──

pub struct TokenBucketLimiter {
    buckets: HashMap<String, (u64, f64)>,
    capacity: f64,
    refill_rate: f64,
}

impl TokenBucketLimiter {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            buckets: HashMap::new(),
            capacity,
            refill_rate,
        }
    }

    pub fn allow_request(&mut self, client_id: &str, now_millis: u64) -> bool {
        let entry = self.buckets
            .entry(client_id.to_string())
            .or_insert((now_millis, self.capacity));

        let elapsed = now_millis.saturating_sub(entry.0);
        let refill = (elapsed as f64) * self.refill_rate;
        entry.1 = (entry.1 + refill).min(self.capacity);
        entry.0 = now_millis;

        if entry.1 >= 1.0 {
            entry.1 -= 1.0;
            true
        } else {
            false
        }
    }

    pub fn available_tokens(&mut self, client_id: &str, now_millis: u64) -> f64 {
        let entry = self.buckets
            .entry(client_id.to_string())
            .or_insert((now_millis, self.capacity));

        let elapsed = now_millis.saturating_sub(entry.0);
        let refill = (elapsed as f64) * self.refill_rate;
        let tokens = (entry.1 + refill).min(self.capacity);
        tokens
    }
}

// ── Sliding Window Limiter ──

pub struct SlidingWindowLimiter {
    requests: HashMap<String, Vec<u64>>,
    window_seconds: u64,
    max_requests: usize,
}

impl SlidingWindowLimiter {
    pub fn new(window_seconds: u64, max_requests: usize) -> Self {
        Self {
            requests: HashMap::new(),
            window_seconds,
            max_requests,
        }
    }

    pub fn allow_request(&mut self, client_id: &str, now_secs: u64) -> bool {
        let entries = self.requests
            .entry(client_id.to_string())
            .or_insert_with(Vec::new);

        entries.retain(|&ts| now_secs.saturating_sub(ts) <= self.window_seconds);

        if entries.len() < self.max_requests {
            entries.push(now_secs);
            true
        } else {
            false
        }
    }

    pub fn current_count(&mut self, client_id: &str, now_secs: u64) -> usize {
        if let Some(entries) = self.requests.get_mut(client_id) {
            entries.retain(|&ts| now_secs.saturating_sub(ts) <= self.window_seconds);
            entries.len()
        } else {
            0
        }
    }
}

// ── Multi-Tier Limiter ──

pub struct MultiTierLimiter {
    tiers: Vec<FixedWindowLimiter>,
}

impl MultiTierLimiter {
    pub fn new(tiers: &[(u64, u32)]) -> Self {
        Self {
            tiers: tiers.iter().map(|(w, m)| FixedWindowLimiter::new(*w, *m)).collect(),
        }
    }

    pub fn allow_request(&mut self, client_id: &str, now_secs: u64) -> bool {
        self.tiers.iter_mut().all(|tier| tier.allow_request(client_id, now_secs))
    }
}

// ── Retry-After ──

pub fn calculate_retry_after(window_start: u64, window_seconds: u64, now_secs: u64) -> Option<u64> {
    let window_end = window_start + window_seconds;
    if now_secs > window_end {
        None
    } else {
        Some(window_end - now_secs)
    }
}

// ── IP Extraction ──

pub fn extract_client_ip(headers: &HashMap<String, String>, remote_addr: &str) -> String {
    if let Some(real_ip) = headers.get("X-Real-IP") {
        let trimmed = real_ip.trim();
        if !trimmed.is_empty() && (trimmed.contains('.') || trimmed.contains(':')) {
            return trimmed.to_string();
        }
    }

    if let Some(forwarded) = headers.get("X-Forwarded-For") {
        if let Some(first) = forwarded.split(',').next() {
            let trimmed = first.trim();
            if !trimmed.is_empty() && (trimmed.contains('.') || trimmed.contains(':')) {
                return trimmed.to_string();
            }
        }
    }

    remote_addr.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(!limiter.allow_request("user1", 1003));
    }

    #[test]
    fn test_fixed_window_resets_on_new_window() {
        let mut limiter = FixedWindowLimiter::new(60, 2);
        assert!(limiter.allow_request("user1", 1000));
        assert!(limiter.allow_request("user1", 1001));
        assert!(!limiter.allow_request("user1", 1002));
        assert!(limiter.allow_request("user1", 1020));
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
        let tokens = bucket.available_tokens("user1", 10_000);
        assert!(tokens <= 3.0, "Tokens should be capped at capacity, got {}", tokens);
    }

    #[test]
    fn test_token_bucket_separate_clients() {
        let mut bucket = TokenBucketLimiter::new(2.0, 1.0);
        assert!(bucket.allow_request("a", 1000));
        assert!(bucket.allow_request("a", 1000));
        assert!(!bucket.allow_request("a", 1000));
        assert!(bucket.allow_request("b", 1000));
    }

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
        assert!(limiter.allow_request("user1", 1061));
    }

    #[test]
    fn test_sliding_window_count() {
        let mut limiter = SlidingWindowLimiter::new(60, 10);
        limiter.allow_request("user1", 1000);
        limiter.allow_request("user1", 1001);
        limiter.allow_request("user1", 1030);
        assert_eq!(limiter.current_count("user1", 1030), 3);
        assert_eq!(limiter.current_count("user1", 1061), 2);
    }

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
        assert!(!limiter.allow_request("user1", 1003));
    }

    #[test]
    fn test_retry_after_calculation() {
        assert_eq!(calculate_retry_after(1000, 60, 1040), Some(20));
    }

    #[test]
    fn test_retry_after_zero_when_at_boundary() {
        assert_eq!(calculate_retry_after(1000, 60, 1060), Some(0));
    }

    #[test]
    fn test_retry_after_none_when_in_past() {
        assert_eq!(calculate_retry_after(1000, 60, 1061), None);
    }

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
