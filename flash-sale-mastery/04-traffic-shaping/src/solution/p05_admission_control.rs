//! # Solution 05: Admission Control
//!
//! Complete implementation of admission control with load, stock, and reputation checks.

use serde::{Deserialize, Serialize};

/// Represents the admission decision for an incoming request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AdmissionDecision {
    Admit,
    Queue,
    Reject(String),
}

/// An incoming purchase request for admission control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseRequest {
    pub account_id: String,
    pub product_id: String,
    pub account_reputation: f64,
}

/// Configuration for the admission controller.
#[derive(Debug, Clone)]
pub struct AdmissionConfig {
    pub queue_threshold: f64,
    pub reject_threshold: f64,
    pub min_reputation: f64,
}

impl Default for AdmissionConfig {
    fn default() -> Self {
        Self {
            queue_threshold: 0.7,
            reject_threshold: 0.9,
            min_reputation: 0.3,
        }
    }
}

/// Admission controller that decides whether to admit, queue, or reject requests.
pub struct AdmissionController {
    config: AdmissionConfig,
    current_load: f64,
    stock_available: bool,
}

impl AdmissionController {
    /// Create a new admission controller.
    pub fn new(config: AdmissionConfig) -> Self {
        Self {
            config,
            current_load: 0.0,
            stock_available: true,
        }
    }

    /// Update the current system load.
    pub fn set_load(&mut self, load: f64) {
        self.current_load = load.clamp(0.0, 1.0);
    }

    /// Set whether stock is available.
    pub fn set_stock_available(&mut self, available: bool) {
        self.stock_available = available;
    }

    /// Make an admission decision for a purchase request.
    pub fn should_admit(&self, request: &PurchaseRequest) -> AdmissionDecision {
        // 1. Fast path: sold out
        if !self.stock_available {
            return AdmissionDecision::Reject("Sold out".to_string());
        }

        // 2. Check account reputation
        if request.account_reputation < self.config.min_reputation {
            return AdmissionDecision::Reject("Insufficient reputation".to_string());
        }

        // 3. Check system load
        if self.current_load > self.config.reject_threshold {
            return AdmissionDecision::Reject("System overloaded".to_string());
        }

        if self.current_load > self.config.queue_threshold {
            return AdmissionDecision::Queue;
        }

        AdmissionDecision::Admit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(reputation: f64) -> PurchaseRequest {
        PurchaseRequest {
            account_id: "user-001".to_string(),
            product_id: "product-001".to_string(),
            account_reputation: reputation,
        }
    }

    #[test]
    fn test_normal_load_admission() {
        let mut controller = AdmissionController::new(AdmissionConfig::default());
        controller.set_load(0.5);
        controller.set_stock_available(true);

        let decision = controller.should_admit(&make_request(0.8));
        assert_eq!(decision, AdmissionDecision::Admit);
    }

    #[test]
    fn test_overload_queues() {
        let mut controller = AdmissionController::new(AdmissionConfig::default());
        controller.set_load(0.8);
        controller.set_stock_available(true);

        let decision = controller.should_admit(&make_request(0.8));
        assert_eq!(decision, AdmissionDecision::Queue);
    }

    #[test]
    fn test_severe_overload_rejects() {
        let mut controller = AdmissionController::new(AdmissionConfig::default());
        controller.set_load(0.95);
        controller.set_stock_available(true);

        let decision = controller.should_admit(&make_request(0.8));
        assert!(matches!(decision, AdmissionDecision::Reject(_)));
    }

    #[test]
    fn test_sold_out_fast_path() {
        let mut controller = AdmissionController::new(AdmissionConfig::default());
        controller.set_load(0.1);
        controller.set_stock_available(false);

        let decision = controller.should_admit(&make_request(1.0));
        assert!(matches!(decision, AdmissionDecision::Reject(_)));
    }

    #[test]
    fn test_low_reputation_rejected() {
        let mut controller = AdmissionController::new(AdmissionConfig::default());
        controller.set_load(0.1);
        controller.set_stock_available(true);

        let decision = controller.should_admit(&make_request(0.1));
        assert!(matches!(decision, AdmissionDecision::Reject(_)));
    }
}
