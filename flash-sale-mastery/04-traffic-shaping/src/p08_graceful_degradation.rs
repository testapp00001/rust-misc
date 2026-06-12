//! # Exercise 08: Graceful Degradation
//!
//! ## Learning Objective
//! Implement graceful degradation strategies that keep a flash sale system
//! serving useful responses even under extreme load. Instead of crashing or
//! returning 500 errors, the system sheds non-essential features and serves
//! fast, cached responses.
//!
//! ## Flash Sale Context
//! When traffic exceeds capacity, the system should degrade gracefully:
//! - Serve a cached "sold out" page in <1ms instead of checking the database
//! - Return 503 with Retry-After header instead of timing out
//! - Serve stale product data instead of fresh data from a slow database
//! - Disable non-essential features (recommendations, reviews, analytics)
//!
//! ## Instructions
//! 1. Implement `serve_sold_out()` -- return a fast, cached "sold out" response
//! 2. Implement `serve_503(retry_after)` -- return a 503 with Retry-After header
//! 3. Implement `serve_stale_data()` -- return cached/stale product data
//! 4. Implement `DegradationController` -- feature flag system that disables
//!    non-essential features under load
//!
//! ## Hints
//! - These functions should be fast (no I/O, no database calls)
//! - Use `serde_json` for JSON response bodies
//! - Feature flags can be simple boolean toggles

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A simplified HTTP response for demonstration purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Feature flags for graceful degradation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub recommendations_enabled: bool,
    pub reviews_enabled: bool,
    pub analytics_enabled: bool,
    pub detailed_product_info: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            recommendations_enabled: true,
            reviews_enabled: true,
            analytics_enabled: true,
            detailed_product_info: true,
        }
    }
}

/// Controller for managing graceful degradation.
pub struct DegradationController {
    // TODO: Add fields for feature flags and load level
}

impl DegradationController {
    /// Create a new degradation controller with default (all-enabled) flags.
    pub fn new() -> Self {
        todo!("Implement degradation controller")
    }

    /// Update degradation level based on current system load.
    ///
    /// - load < 0.5: all features enabled
    /// - 0.5 <= load < 0.7: disable analytics
    /// - 0.7 <= load < 0.9: disable analytics + recommendations
    /// - load >= 0.9: disable all non-essential features
    pub fn update_load(&mut self, load: f64) {
        todo!("Implement load-based degradation")
    }

    /// Check if a specific feature is enabled.
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        todo!("Implement feature check")
    }

    /// Get current feature flags.
    pub fn get_flags(&self) -> &FeatureFlags {
        todo!("Return feature flags reference")
    }
}

/// Serve a cached "sold out" response.
///
/// This should be extremely fast (<1ms) as it uses only pre-built data.
/// No database calls, no Redis calls -- pure in-memory response.
pub fn serve_sold_out() -> Response {
    todo!("Implement fast sold-out response")
}

/// Serve a 503 Service Unavailable response.
///
/// Includes a `Retry-After` header to tell the client when to retry.
///
/// # Arguments
/// * `retry_after` - Number of seconds the client should wait before retrying
pub fn serve_503(retry_after: u32) -> Response {
    todo!("Implement 503 response with Retry-After")
}

/// Serve stale product data from cache.
///
/// Returns a response with a `X-Cache: STALE` header to indicate the data
/// may not be fresh. This is preferable to a timeout or error.
pub fn serve_stale_data(product_id: &str) -> Response {
    todo!("Implement stale data response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sold_out_response() {
        let response = serve_sold_out();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("sold out") || response.body.contains("Sold Out"));
    }

    #[test]
    fn test_503_response() {
        let response = serve_503(30);
        assert_eq!(response.status, 503);
        assert!(
            response.headers.contains_key("Retry-After"),
            "Should include Retry-After header"
        );
        assert_eq!(response.headers["Retry-After"], "30");
    }

    #[test]
    fn test_stale_data_response() {
        let response = serve_stale_data("product-001");
        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get("X-Cache").unwrap(), "STALE");
    }

    #[test]
    fn test_degradation_triggers() {
        let mut controller = DegradationController::new();
        assert!(controller.is_feature_enabled("analytics"));

        // Medium load: disable analytics
        controller.update_load(0.6);
        assert!(!controller.is_feature_enabled("analytics"));
        assert!(controller.is_feature_enabled("reviews"));

        // High load: disable more features
        controller.update_load(0.8);
        assert!(!controller.is_feature_enabled("analytics"));
        assert!(!controller.is_feature_enabled("recommendations"));
        assert!(controller.is_feature_enabled("reviews"));

        // Critical load: disable all non-essential
        controller.update_load(0.95);
        assert!(!controller.is_feature_enabled("analytics"));
        assert!(!controller.is_feature_enabled("recommendations"));
        assert!(!controller.is_feature_enabled("reviews"));
        assert!(!controller.is_feature_enabled("detailed_product_info"));
    }

    #[test]
    fn test_response_times_under_degradation() {
        // These should all be pure in-memory operations, well under 1ms
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = serve_sold_out();
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "1000 sold-out responses should take <100ms, took {:?}",
            elapsed
        );
    }
}
