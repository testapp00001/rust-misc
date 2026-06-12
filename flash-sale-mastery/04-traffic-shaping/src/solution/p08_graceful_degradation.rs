//! # Solution 08: Graceful Degradation
//!
//! Complete implementation of graceful degradation strategies.

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
    flags: FeatureFlags,
    current_load: f64,
}

impl DegradationController {
    /// Create a new degradation controller with default (all-enabled) flags.
    pub fn new() -> Self {
        Self {
            flags: FeatureFlags::default(),
            current_load: 0.0,
        }
    }

    /// Update degradation level based on current system load.
    pub fn update_load(&mut self, load: f64) {
        self.current_load = load.clamp(0.0, 1.0);

        if load >= 0.9 {
            self.flags.analytics_enabled = false;
            self.flags.recommendations_enabled = false;
            self.flags.reviews_enabled = false;
            self.flags.detailed_product_info = false;
        } else if load >= 0.7 {
            self.flags.analytics_enabled = false;
            self.flags.recommendations_enabled = false;
            self.flags.reviews_enabled = true;
            self.flags.detailed_product_info = true;
        } else if load >= 0.5 {
            self.flags.analytics_enabled = false;
            self.flags.recommendations_enabled = true;
            self.flags.reviews_enabled = true;
            self.flags.detailed_product_info = true;
        } else {
            self.flags = FeatureFlags::default();
        }
    }

    /// Check if a specific feature is enabled.
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "recommendations" => self.flags.recommendations_enabled,
            "reviews" => self.flags.reviews_enabled,
            "analytics" => self.flags.analytics_enabled,
            "detailed_product_info" => self.flags.detailed_product_info,
            _ => true,
        }
    }

    /// Get current feature flags.
    pub fn get_flags(&self) -> &FeatureFlags {
        &self.flags
    }
}

/// Serve a cached "sold out" response.
pub fn serve_sold_out() -> Response {
    let mut headers = HashMap::new();
    headers.insert("X-Cache".to_string(), "HIT".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    Response {
        status: 200,
        headers,
        body: r#"{"status":"sold_out","message":"This item has Sold Out. Please check back later."}"#.to_string(),
    }
}

/// Serve a 503 Service Unavailable response.
pub fn serve_503(retry_after: u32) -> Response {
    let mut headers = HashMap::new();
    headers.insert("Retry-After".to_string(), retry_after.to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    Response {
        status: 503,
        headers,
        body: r#"{"status":"service_unavailable","message":"System is under heavy load. Please retry later."}"#.to_string(),
    }
}

/// Serve stale product data from cache.
pub fn serve_stale_data(product_id: &str) -> Response {
    let mut headers = HashMap::new();
    headers.insert("X-Cache".to_string(), "STALE".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    Response {
        status: 200,
        headers,
        body: format!(
            r#"{{"product_id":"{product_id}","name":"Cached Product","price":99.99,"note":"Data may be stale"}}"#
        ),
    }
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
        assert!(response.headers.contains_key("Retry-After"));
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

        controller.update_load(0.6);
        assert!(!controller.is_feature_enabled("analytics"));
        assert!(controller.is_feature_enabled("reviews"));

        controller.update_load(0.8);
        assert!(!controller.is_feature_enabled("analytics"));
        assert!(!controller.is_feature_enabled("recommendations"));
        assert!(controller.is_feature_enabled("reviews"));

        controller.update_load(0.95);
        assert!(!controller.is_feature_enabled("analytics"));
        assert!(!controller.is_feature_enabled("recommendations"));
        assert!(!controller.is_feature_enabled("reviews"));
        assert!(!controller.is_feature_enabled("detailed_product_info"));
    }

    #[test]
    fn test_response_times_under_degradation() {
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
