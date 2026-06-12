//! # Solution 09: Multi-Layer Rate Limiting
//!
//! Complete implementation of multi-layer rate limiting with per-IP, per-account,
//! per-product, and global token buckets.

use std::collections::HashMap;

/// The result of a multi-layer rate limit check.
#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitResult {
    Allowed,
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
    config: RateLimitConfig,
    ip_buckets: HashMap<String, MiniBucket>,
    account_buckets: HashMap<String, MiniBucket>,
    product_buckets: HashMap<String, MiniBucket>,
    global_bucket: MiniBucket,
}

impl MultiLayerRateLimiter {
    /// Create a new multi-layer rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        let global_bucket = MiniBucket::new(config.global_capacity, config.global_refill_per_sec);
        Self {
            config,
            ip_buckets: HashMap::new(),
            account_buckets: HashMap::new(),
            product_buckets: HashMap::new(),
            global_bucket,
        }
    }

    /// Check a request against all rate limit layers.
    pub fn check(&mut self, request: &RateLimitRequest) -> RateLimitResult {
        // Layer 1: Per-IP
        let ip_bucket = self.ip_buckets
            .entry(request.ip.clone())
            .or_insert_with(|| MiniBucket::new(self.config.ip_capacity, self.config.ip_refill_per_sec));
        if !ip_bucket.try_acquire() {
            return RateLimitResult::Rejected {
                layer: "ip".to_string(),
                retry_after_ms: 1000,
            };
        }

        // Layer 2: Per-account
        let account_bucket = self.account_buckets
            .entry(request.account_id.clone())
            .or_insert_with(|| MiniBucket::new(self.config.account_capacity, self.config.account_refill_per_sec));
        if !account_bucket.try_acquire() {
            return RateLimitResult::Rejected {
                layer: "account".to_string(),
                retry_after_ms: 1000,
            };
        }

        // Layer 3: Per-product
        let product_bucket = self.product_buckets
            .entry(request.product_id.clone())
            .or_insert_with(|| MiniBucket::new(self.config.product_capacity, self.config.product_refill_per_sec));
        if !product_bucket.try_acquire() {
            return RateLimitResult::Rejected {
                layer: "product".to_string(),
                retry_after_ms: 1000,
            };
        }

        // Layer 4: Global
        if !self.global_bucket.try_acquire() {
            return RateLimitResult::Rejected {
                layer: "global".to_string(),
                retry_after_ms: 1000,
            };
        }

        RateLimitResult::Allowed
    }

    /// Get the number of buckets currently tracked.
    pub fn bucket_count(&self) -> usize {
        self.ip_buckets.len() + self.account_buckets.len() + self.product_buckets.len() + 1
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

        for _ in 0..3 {
            assert_eq!(limiter.check(&req), RateLimitResult::Allowed);
        }
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

        assert_eq!(limiter.check(&make_request("10.0.0.1", "u1", "p1")), RateLimitResult::Allowed);
        assert_eq!(limiter.check(&make_request("10.0.0.2", "u2", "p1")), RateLimitResult::Allowed);
        assert_eq!(limiter.check(&make_request("10.0.0.1", "u1", "p1")), RateLimitResult::Allowed);
        assert_eq!(limiter.check(&make_request("10.0.0.2", "u2", "p1")), RateLimitResult::Allowed);
        assert!(matches!(limiter.check(&make_request("10.0.0.1", "u1", "p1")), RateLimitResult::Rejected { .. }));
        assert!(matches!(limiter.check(&make_request("10.0.0.2", "u2", "p1")), RateLimitResult::Rejected { .. }));
    }
}
