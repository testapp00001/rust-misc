//! # Exercise 05: Admission Control
//!
//! ## Learning Objective
//! Implement an admission controller that makes intelligent decisions about
//! whether to admit, queue, or reject requests based on system load, stock
//! availability, and account reputation. This goes beyond simple rate limiting
//! to consider business context.
//!
//! ## Flash Sale Context
//! Not all requests should be treated equally. A flash sale system needs to:
//! - Fast-reject requests when stock is sold out (avoid wasted work)
//! - Queue requests when the system is near capacity (smooth the load)
//! - Reject suspicious accounts (potential scalpers/bots)
//! - Admit requests when everything looks healthy
//!
//! ## Instructions
//! 1. Define `AdmissionDecision` enum with variants `Admit`, `Queue`, `Reject`
//! 2. Implement `AdmissionController::new(config)` with load thresholds
//! 3. Implement `should_admit(request)` that considers:
//!    - Current system load (reject if >90%, queue if >70%)
//!    - Stock availability (fast-reject if sold out)
//!    - Account reputation score (reject if below threshold)
//!
//! ## Hints
//! - The sold-out check should be the fastest path (O(1))
//! - Load percentage can be a simple gauge (0.0 to 1.0)
//! - Reputation score can be a float from 0.0 (bad) to 1.0 (trusted)

use serde::{Deserialize, Serialize};

/// Represents the admission decision for an incoming request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AdmissionDecision {
    /// Request is admitted for processing.
    Admit,
    /// Request should be queued for later processing.
    Queue,
    /// Request should be rejected immediately.
    Reject(String),
}

/// An incoming purchase request for admission control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseRequest {
    pub account_id: String,
    pub product_id: String,
    pub account_reputation: f64, // 0.0 (bad) to 1.0 (trusted)
}

/// Configuration for the admission controller.
#[derive(Debug, Clone)]
pub struct AdmissionConfig {
    /// Load threshold above which requests are queued (0.0-1.0).
    pub queue_threshold: f64,
    /// Load threshold above which requests are rejected (0.0-1.0).
    pub reject_threshold: f64,
    /// Minimum reputation score to be admitted.
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
    // TODO: Add fields for config, current_load, and stock_available
}

impl AdmissionController {
    /// Create a new admission controller.
    ///
    /// # Arguments
    /// * `config` - Admission control thresholds
    pub fn new(config: AdmissionConfig) -> Self {
        todo!("Implement admission controller creation")
    }

    /// Update the current system load.
    ///
    /// # Arguments
    /// * `load` - Current load as a fraction (0.0 to 1.0)
    pub fn set_load(&mut self, load: f64) {
        todo!("Implement load update")
    }

    /// Set whether stock is available.
    ///
    /// # Arguments
    /// * `available` - Whether there is stock remaining
    pub fn set_stock_available(&mut self, available: bool) {
        todo!("Implement stock availability update")
    }

    /// Make an admission decision for a purchase request.
    ///
    /// Decision logic (in priority order):
    /// 1. If stock is sold out -> Reject("Sold out")
    /// 2. If account reputation < min_reputation -> Reject("Insufficient reputation")
    /// 3. If load > reject_threshold -> Reject("System overloaded")
    /// 4. If load > queue_threshold -> Queue
    /// 5. Otherwise -> Admit
    pub fn should_admit(&self, request: &PurchaseRequest) -> AdmissionDecision {
        todo!("Implement admission decision logic")
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
        controller.set_load(0.8); // Above queue threshold (0.7)
        controller.set_stock_available(true);

        let decision = controller.should_admit(&make_request(0.8));
        assert_eq!(decision, AdmissionDecision::Queue);
    }

    #[test]
    fn test_severe_overload_rejects() {
        let mut controller = AdmissionController::new(AdmissionConfig::default());
        controller.set_load(0.95); // Above reject threshold (0.9)
        controller.set_stock_available(true);

        let decision = controller.should_admit(&make_request(0.8));
        assert!(matches!(decision, AdmissionDecision::Reject(_)));
    }

    #[test]
    fn test_sold_out_fast_path() {
        let mut controller = AdmissionController::new(AdmissionConfig::default());
        controller.set_load(0.1); // Low load
        controller.set_stock_available(false); // But sold out

        let decision = controller.should_admit(&make_request(1.0));
        assert!(matches!(decision, AdmissionDecision::Reject(_)));
    }

    #[test]
    fn test_low_reputation_rejected() {
        let mut controller = AdmissionController::new(AdmissionConfig::default());
        controller.set_load(0.1);
        controller.set_stock_available(true);

        let decision = controller.should_admit(&make_request(0.1)); // Below threshold
        assert!(matches!(decision, AdmissionDecision::Reject(_)));
    }
}
