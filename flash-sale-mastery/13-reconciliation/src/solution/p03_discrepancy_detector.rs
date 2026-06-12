//! # Solution 03: Discrepancy Detector
//!
//! Complete implementation of cross-system discrepancy detection.

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
/// In production these would be Redis, PostgreSQL, and the voucher service.
pub struct DetectionDataSources {
    /// Orders in the database.
    pub db_orders: Vec<OrderRecord>,
    /// Orders in Redis (the fast-path log).
    pub redis_orders: Vec<OrderRecord>,
    /// Claims/reservations in the system.
    pub claims: Vec<ClaimRecord>,
    /// Voucher usage records: (voucher_code, user_id) pairs.
    pub voucher_uses: Vec<(String, String)>,
    /// Voucher usage limits: voucher_code -> max_uses.
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

/// Scans multiple data sources to find discrepancies that indicate
/// data inconsistency between Redis, PostgreSQL, and other services.
pub struct DiscrepancyDetector {
    data: DetectionDataSources,
}

impl DiscrepancyDetector {
    pub fn new(data: DetectionDataSources) -> Self {
        Self { data }
    }

    /// Run all detection checks and return every discrepancy found.
    pub fn scan_all(&self) -> Result<Vec<Discrepancy>, ReconciliationError> {
        let mut discrepancies = Vec::new();

        discrepancies.extend(self.detect_stock_mismatches()?);
        discrepancies.extend(self.detect_missing_orders()?);
        discrepancies.extend(self.detect_orphaned_claims()?);
        discrepancies.extend(self.detect_duplicate_vouchers()?);

        Ok(discrepancies)
    }

    /// Detect orders that exist in one system but not the other.
    fn detect_missing_orders(&self) -> Result<Vec<Discrepancy>, ReconciliationError> {
        let mut discrepancies = Vec::new();

        let db_ids: HashSet<&str> = self
            .data
            .db_orders
            .iter()
            .map(|o| o.order_id.as_str())
            .collect();
        let redis_ids: HashSet<&str> = self
            .data
            .redis_orders
            .iter()
            .map(|o| o.order_id.as_str())
            .collect();

        // Orders in DB but not in Redis.
        for order in &self.data.db_orders {
            if !redis_ids.contains(order.order_id.as_str()) {
                discrepancies.push(Discrepancy {
                    discrepancy_id: Uuid::new_v4(),
                    discrepancy_type: DiscrepancyType::MissingOrder,
                    product_id: Some(order.product_id.clone()),
                    description: format!(
                        "Order {} exists in DB but not in Redis",
                        order.order_id
                    ),
                    severity: Severity::High,
                    detected_at: Utc::now(),
                });
            }
        }

        // Orders in Redis but not in DB.
        for order in &self.data.redis_orders {
            if !db_ids.contains(order.order_id.as_str()) {
                discrepancies.push(Discrepancy {
                    discrepancy_id: Uuid::new_v4(),
                    discrepancy_type: DiscrepancyType::MissingOrder,
                    product_id: Some(order.product_id.clone()),
                    description: format!(
                        "Order {} exists in Redis but not in DB",
                        order.order_id
                    ),
                    severity: Severity::Critical,
                    detected_at: Utc::now(),
                });
            }
        }

        Ok(discrepancies)
    }

    /// Detect claims that have no corresponding order.
    fn detect_orphaned_claims(&self) -> Result<Vec<Discrepancy>, ReconciliationError> {
        let mut discrepancies = Vec::new();

        let db_order_ids: HashSet<&str> = self
            .data
            .db_orders
            .iter()
            .map(|o| o.order_id.as_str())
            .collect();
        let redis_order_ids: HashSet<&str> = self
            .data
            .redis_orders
            .iter()
            .map(|o| o.order_id.as_str())
            .collect();

        for claim in &self.data.claims {
            if let Some(ref order_id) = claim.order_id {
                if !db_order_ids.contains(order_id.as_str())
                    && !redis_order_ids.contains(order_id.as_str())
                {
                    discrepancies.push(Discrepancy {
                        discrepancy_id: Uuid::new_v4(),
                        discrepancy_type: DiscrepancyType::OrphanedClaim,
                        product_id: Some(claim.product_id.clone()),
                        description: format!(
                            "Claim {} references order {} which does not exist",
                            claim.claim_id, order_id
                        ),
                        severity: Severity::High,
                        detected_at: Utc::now(),
                    });
                }
            } else {
                // Claim with no order reference at all.
                discrepancies.push(Discrepancy {
                    discrepancy_id: Uuid::new_v4(),
                    discrepancy_type: DiscrepancyType::OrphanedClaim,
                    product_id: Some(claim.product_id.clone()),
                    description: format!(
                        "Claim {} has no associated order",
                        claim.claim_id
                    ),
                    severity: Severity::Medium,
                    detected_at: Utc::now(),
                });
            }
        }

        Ok(discrepancies)
    }

    /// Detect vouchers used more times than their limit allows.
    fn detect_duplicate_vouchers(&self) -> Result<Vec<Discrepancy>, ReconciliationError> {
        let mut discrepancies = Vec::new();

        // Count unique users per voucher.
        let mut usage_counts: HashMap<&str, HashSet<&str>> = HashMap::new();
        for (voucher_code, user_id) in &self.data.voucher_uses {
            usage_counts
                .entry(voucher_code.as_str())
                .or_default()
                .insert(user_id.as_str());
        }

        for (voucher_code, users) in &usage_counts {
            if let Some(&limit) = self.data.voucher_limits.get(*voucher_code) {
                if users.len() > limit {
                    discrepancies.push(Discrepancy {
                        discrepancy_id: Uuid::new_v4(),
                        discrepancy_type: DiscrepancyType::DuplicateVoucher,
                        product_id: None,
                        description: format!(
                            "Voucher {} used by {} users but limit is {}",
                            voucher_code,
                            users.len(),
                            limit
                        ),
                        severity: Severity::Critical,
                        detected_at: Utc::now(),
                    });
                }
            }
        }

        Ok(discrepancies)
    }

    /// Detect stock mismatches (delegates to inventory sync data).
    /// Here we implement a lightweight version that compares two stock maps.
    fn detect_stock_mismatches(&self) -> Result<Vec<Discrepancy>, ReconciliationError> {
        // This is a placeholder that would integrate with InventorySync.
        // In practice, the caller passes in stock comparison data.
        // For this exercise, we return an empty vec -- stock mismatch detection
        // is handled by p01_inventory_sync.
        Ok(Vec::new())
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
