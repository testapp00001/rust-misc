//! # Exercise 03: Discrepancy Detector
//!
//! ## Learning Objective
//! Implement a multi-source scanner that detects data inconsistencies between
//! Redis, PostgreSQL, and the voucher service.
//!
//! ## Flash Sale Context
//! A flash sale touches many systems: inventory (Redis + DB), orders (Redis log + DB),
//! claims/reservations, and vouchers. Any of these can drift out of sync. The
//! discrepancy detector runs after the sale to find every inconsistency before
//! it becomes a customer complaint or financial error.
//!
//! ## Instructions
//! 1. Implement `DiscrepancyDetector::new()` with the combined data sources.
//! 2. Implement `scan_all()` which runs all four detection checks.
//! 3. Implement `detect_missing_orders()` -- find orders in DB but not Redis
//!    (and vice versa). DB-not-Redis is High severity; Redis-not-DB is Critical.
//! 4. Implement `detect_orphaned_claims()` -- find claims whose order_id does not
//!    exist in either system, or claims with no order_id at all.
//! 5. Implement `detect_duplicate_vouchers()` -- find vouchers used by more unique
//!    users than their limit allows.
//!
//! ## Hints
//! - Use `HashSet` for efficient set-difference operations on order IDs.
//! - A claim is orphaned if its `order_id` is `None` or references a nonexistent order.
//! - For vouchers, count unique `(voucher, user)` pairs, not total uses.

use chrono::Utc;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::shared::{
    Discrepancy, DiscrepancyType, ReconciliationError, Severity,
};

/// An order record for cross-system comparison.
#[derive(Debug, Clone)]
pub struct OrderRecord {
    pub order_id: String,
    pub product_id: String,
    pub user_id: String,
    pub voucher_code: Option<String>,
}

/// A reservation/claim record.
#[derive(Debug, Clone)]
pub struct ClaimRecord {
    pub claim_id: String,
    pub order_id: Option<String>,
    pub product_id: String,
    pub user_id: String,
}

/// Data sources for discrepancy detection.
pub struct DetectionDataSources {
    pub db_orders: Vec<OrderRecord>,
    pub redis_orders: Vec<OrderRecord>,
    pub claims: Vec<ClaimRecord>,
    pub voucher_uses: Vec<(String, String)>,
    pub voucher_limits: HashMap<String, usize>,
}

impl DetectionDataSources {
    pub fn new() -> Self {
        Self {
            db_orders: Vec::new(),
            redis_orders: Vec::new(),
            claims: Vec::new(),
            voucher_uses: Vec::new(),
            voucher_limits: HashMap::new(),
        }
    }
}

/// Scans multiple data sources to find discrepancies.
pub struct DiscrepancyDetector {
    // TODO: add field for data sources
}

impl DiscrepancyDetector {
    pub fn new(data: DetectionDataSources) -> Self {
        todo!("Store the data sources")
    }

    /// Run all detection checks and return every discrepancy found.
    pub fn scan_all(&self) -> Result<Vec<Discrepancy>, ReconciliationError> {
        todo!("Run all four detection methods and collect results")
    }

    /// Detect orders that exist in one system but not the other.
    fn detect_missing_orders(&self) -> Result<Vec<Discrepancy>, ReconciliationError> {
        todo!("Compare order IDs between DB and Redis using set operations")
    }

    /// Detect claims that have no corresponding order.
    fn detect_orphaned_claims(&self) -> Result<Vec<Discrepancy>, ReconciliationError> {
        todo!("Check each claim's order_id against both order sets")
    }

    /// Detect vouchers used more times than their limit allows.
    fn detect_duplicate_vouchers(&self) -> Result<Vec<Discrepancy>, ReconciliationError> {
        todo!("Count unique users per voucher and compare against limits")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_missing_order_in_redis() {
        let mut data = DetectionDataSources::new();
        data.db_orders.push(OrderRecord {
            order_id: "ORD-1".into(),
            product_id: "SKU-1".into(),
            user_id: "U1".into(),
            voucher_code: None,
        });
        // No corresponding Redis order.

        let detector = DiscrepancyDetector::new(data);
        let discrepancies = detector.scan_all().unwrap();

        assert_eq!(discrepancies.len(), 1);
        assert_eq!(
            discrepancies[0].discrepancy_type,
            DiscrepancyType::MissingOrder
        );
        assert!(discrepancies[0].description.contains("not in Redis"));
    }

    #[test]
    fn test_detect_missing_order_in_db() {
        let mut data = DetectionDataSources::new();
        data.redis_orders.push(OrderRecord {
            order_id: "ORD-2".into(),
            product_id: "SKU-1".into(),
            user_id: "U1".into(),
            voucher_code: None,
        });
        // No corresponding DB order.

        let detector = DiscrepancyDetector::new(data);
        let discrepancies = detector.scan_all().unwrap();

        assert_eq!(discrepancies.len(), 1);
        assert_eq!(
            discrepancies[0].discrepancy_type,
            DiscrepancyType::MissingOrder
        );
        assert!(discrepancies[0].description.contains("not in DB"));
    }

    #[test]
    fn test_detect_orphaned_claim() {
        let mut data = DetectionDataSources::new();
        data.claims.push(ClaimRecord {
            claim_id: "CLM-1".into(),
            order_id: Some("ORD-GHOST".into()),
            product_id: "SKU-1".into(),
            user_id: "U1".into(),
        });
        // ORD-GHOST does not exist in either orders list.

        let detector = DiscrepancyDetector::new(data);
        let discrepancies = detector.scan_all().unwrap();

        assert_eq!(discrepancies.len(), 1);
        assert_eq!(
            discrepancies[0].discrepancy_type,
            DiscrepancyType::OrphanedClaim
        );
    }

    #[test]
    fn test_detect_duplicate_voucher() {
        let mut data = DetectionDataSources::new();
        data.voucher_limits.insert("FLASH50".into(), 1);
        data.voucher_uses.push(("FLASH50".into(), "U1".into()));
        data.voucher_uses.push(("FLASH50".into(), "U2".into()));

        let detector = DiscrepancyDetector::new(data);
        let discrepancies = detector.scan_all().unwrap();

        assert_eq!(discrepancies.len(), 1);
        assert_eq!(
            discrepancies[0].discrepancy_type,
            DiscrepancyType::DuplicateVoucher
        );
        assert!(discrepancies[0].description.contains("limit is 1"));
    }

    #[test]
    fn test_clean_state_returns_empty() {
        let mut data = DetectionDataSources::new();
        // Matching orders in both systems.
        data.db_orders.push(OrderRecord {
            order_id: "ORD-1".into(),
            product_id: "SKU-1".into(),
            user_id: "U1".into(),
            voucher_code: None,
        });
        data.redis_orders.push(OrderRecord {
            order_id: "ORD-1".into(),
            product_id: "SKU-1".into(),
            user_id: "U1".into(),
            voucher_code: None,
        });
        // Claim references existing order.
        data.claims.push(ClaimRecord {
            claim_id: "CLM-1".into(),
            order_id: Some("ORD-1".into()),
            product_id: "SKU-1".into(),
            user_id: "U1".into(),
        });
        // Voucher within limits.
        data.voucher_limits.insert("CODE1".into(), 5);
        data.voucher_uses.push(("CODE1".into(), "U1".into()));

        let detector = DiscrepancyDetector::new(data);
        let discrepancies = detector.scan_all().unwrap();

        assert!(
            discrepancies.is_empty(),
            "Expected no discrepancies but found {}",
            discrepancies.len()
        );
    }
}
