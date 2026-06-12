//! # Exercise 09: Multi-Layer Rate Limiting
//!
//! ## Learning Objective
//! Compose multiple rate limiters into a layered defense system. Each layer
//! protects a different dimension: per-IP, per-account, per-product, and
//! global. A request must pass ALL layers to be admitted.
//!
//! ## Flash Sale Context
//! A real flash sale system needs rate limiting at multiple levels:
//! - **Per-IP**: Prevent a single machine from flooding the API (bot detection)
//! - **Per-Account**: Prevent a single user from hogging all the capacity
//! - **Per-Product**: Protect each product's purchase pipeline independently
//! - **Global**: Protect the entire system from total overload
//!
//! ## Instructions
//! 1. Implement `RateLimitConfig` with per-layer configuration
//! 2. Implement `MultiLayerRateLimiter::new(config)` with token buckets for
//!    each layer
//! 3. Implement `check(request)` that tests all layers in order (cheapest
//!    first) and returns the first rejection, or `Allowed` if all pass
//!
//! ## Hints
//! - Use the `TokenBucket` from exercise 01 (or a simplified version)
//! - Check layers in order: IP -> Account -> Product -> Global
//! - Short-circuit on first rejection (don't waste work checking further)
//! - Each layer needs its own bucket per key (use a HashMap)

use std::collections::HashMap;

/// The result of a multi-layer rate limit check.
#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitResult {
    /// Request passed all rate limit layers.
    Allowed,
    /// Request was rejected by a specific layer.
    Rejected {
        layer: String,
        retry_after_ms: u64,
    },
}

/// A request to be rate-checked.
#[derive(Debug, Clone)]
pub struct RateLimitRequest {
    pub ip: String,
    pub account_id: String,
    pub product_id: String,
}

/// Configuration for each rate limit layer.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub ip_capacity: u64,
    pub ip_refill_per_sec: f64,
    pub account_capacity: u64,
    pub account_refill_per_sec: f64,
    pub product_capacity: u64,
    pub product_refill_per_sec: f64,
    pub global_capacity: u64,
    pub global_refill_per_sec: f64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            ip_capacity: 20,
            ip_refill_per_sec: 10.0,
            account_capacity: 10,
            account_refill_per_sec: 5.0,
            product_capacity: 1000,
            product_refill_per_sec: 100.0,
            global_capacity: 5000,
            global_refill_per_sec: 1000.0,
        }
    }
}

/// A simplified token bucket for use in the multi-layer limiter.
pub struct MiniBucket {
    pub capacity: f64,
    pub tokens: f64,
    pub refill_per_sec: f64,
    pub last_refill: std::time::Instant,
}

impl MiniBucket {
    pub fn new(capacity: u64, refill_per_sec: f64) -> Self {
        Self {
            capacity: capacity as f64,
            tokens: capacity as f64,
            refill_per_sec,
            last_refill: std::time::Instant::now(),
        }
    }

    pub fn try_acquire(&mut self) -> bool {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Multi-layer rate limiter that checks IP, account, product, and global limits.
pub struct MultiLayerRateLimiter {
    // TODO: Add fields for config, per-IP buckets, per-account buckets,
    // per-product buckets, and a global bucket
}

impl MultiLayerRateLimiter {
    /// Create a new multi-layer rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        todo!("Implement multi-layer rate limiter with HashMaps of buckets")
    }

    /// Check a request against all rate limit layers.
    ///
    /// Layers are checked in order: IP -> Account -> Product -> Global.
    /// Returns `Allowed` if all layers pass, or `Rejected` with the name
    /// of the first layer that rejected.
    pub fn check(&mut self, request: &RateLimitRequest) -> RateLimitResult {
        todo!("Implement layered rate limit check with short-circuit")
    }

    /// Get the number of buckets currently tracked (for monitoring).
    pub fn bucket_count(&self) -> usize {
        todo!("Count total buckets across all layers")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(ip: &str, account: &str, product: &str) -> RateLimitRequest {
        RateLimitRequest {
            ip: ip.to_string(),
            account_id: account.to_string(),
            product_id: product.to_string(),
        }
    }

    #[test]
    fn test_ip_layer_blocks_flood() {
        let config = RateLimitConfig {
            ip_capacity: 3,
            ip_refill_per_sec: 1.0,
            ..Default::default()
        };
        let mut limiter = MultiLayerRateLimiter::new(config);
        let req = make_request("10.0.0.1", "user-1", "prod-1");

        // First 3 should pass
        for _ in 0..3 {
            assert_eq!(limiter.check(&req), RateLimitResult::Allowed);
        }
        // 4th should be rejected at IP layer
        let result = limiter.check(&req);
        assert!(matches!(result, RateLimitResult::Rejected { ref layer, .. } if layer == "ip"));
    }

    #[test]
    fn test_account_layer_limits() {
        let config = RateLimitConfig {
            ip_capacity: 100,
            ip_refill_per_sec: 100.0,
            account_capacity: 2,
            account_refill_per_sec: 1.0,
            ..Default::default()
        };
        let mut limiter = MultiLayerRateLimiter::new(config);

        // Same account from different IPs
        assert_eq!(limiter.check(&make_request("10.0.0.1", "user-1", "p1")), RateLimitResult::Allowed);
        assert_eq!(limiter.check(&make_request("10.0.0.2", "user-1", "p1")), RateLimitResult::Allowed);
        let result = limiter.check(&make_request("10.0.0.3", "user-1", "p1"));
        assert!(matches!(result, RateLimitResult::Rejected { ref layer, .. } if layer == "account"));
    }

    #[test]
    fn test_product_layer_limits() {
        let config = RateLimitConfig {
            ip_capacity: 100,
            ip_refill_per_sec: 100.0,
            account_capacity: 100,
            account_refill_per_sec: 100.0,
            product_capacity: 2,
            product_refill_per_sec: 1.0,
            global_capacity: 10000,
            global_refill_per_sec: 10000.0,
        };
        let mut limiter = MultiLayerRateLimiter::new(config);

        // Different users, same product
        assert_eq!(limiter.check(&make_request("10.0.0.1", "user-1", "prod-1")), RateLimitResult::Allowed);
        assert_eq!(limiter.check(&make_request("10.0.0.2", "user-2", "prod-1")), RateLimitResult::Allowed);
        let result = limiter.check(&make_request("10.0.0.3", "user-3", "prod-1"));
        assert!(matches!(result, RateLimitResult::Rejected { ref layer, .. } if layer == "product"));
    }

    #[test]
    fn test_global_layer_limits() {
        let config = RateLimitConfig {
            ip_capacity: 100,
            ip_refill_per_sec: 100.0,
            account_capacity: 100,
            account_refill_per_sec: 100.0,
            product_capacity: 100,
            product_refill_per_sec: 100.0,
            global_capacity: 2,
            global_refill_per_sec: 1.0,
        };
        let mut limiter = MultiLayerRateLimiter::new(config);

        assert_eq!(limiter.check(&make_request("10.0.0.1", "user-1", "p1")), RateLimitResult::Allowed);
        assert_eq!(limiter.check(&make_request("10.0.0.2", "user-2", "p2")), RateLimitResult::Allowed);
        let result = limiter.check(&make_request("10.0.0.3", "user-3", "p3"));
        assert!(matches!(result, RateLimitResult::Rejected { ref layer, .. } if layer == "global"));
    }

    #[test]
    fn test_different_limits_independent() {
        let config = RateLimitConfig {
            ip_capacity: 2,
            ip_refill_per_sec: 1.0,
            ..Default::default()
        };
        let mut limiter = MultiLayerRateLimiter::new(config);

        // Different IPs should have independent limits
        assert_eq!(limiter.check(&make_request("10.0.0.1", "u1", "p1")), RateLimitResult::Allowed);
        assert_eq!(limiter.check(&make_request("10.0.0.2", "u2", "p1")), RateLimitResult::Allowed);
        assert_eq!(limiter.check(&make_request("10.0.0.1", "u1", "p1")), RateLimitResult::Allowed);
        assert_eq!(limiter.check(&make_request("10.0.0.2", "u2", "p1")), RateLimitResult::Allowed);
        // Now both IPs should be at their limits
        assert!(matches!(limiter.check(&make_request("10.0.0.1", "u1", "p1")), RateLimitResult::Rejected { .. }));
        assert!(matches!(limiter.check(&make_request("10.0.0.2", "u2", "p1")), RateLimitResult::Rejected { .. }));
    }
}
