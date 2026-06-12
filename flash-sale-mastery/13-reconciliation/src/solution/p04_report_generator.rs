//! # Solution 04: Report Generator
//!
//! Complete implementation of reconciliation report generation.

use chrono::Utc;
use uuid::Uuid;

#[allow(unused_imports)]
use crate::shared::{
    Discrepancy, DiscrepancyType, ReconciliationReport, ReportSummary, SyncAction, SyncResult,
};

/// Generates structured reconciliation reports from sync results and discrepancies.
pub struct ReportGenerator {
    sync_results: Vec<SyncResult>,
    discrepancies: Vec<Discrepancy>,
    corrections_applied: usize,
    total_products_checked: usize,
}

impl ReportGenerator {
    /// Create a new ReportGenerator with the collected reconciliation data.
    pub fn new(
        sync_results: Vec<SyncResult>,
        discrepancies: Vec<Discrepancy>,
        corrections_applied: usize,
        total_products_checked: usize,
    ) -> Self {
        Self {
            sync_results,
            discrepancies,
            corrections_applied,
            total_products_checked,
        }
    }

    /// Generate a full reconciliation report.
    pub fn generate(&self) -> ReconciliationReport {
        let summary = self.build_summary();

        ReconciliationReport {
            report_id: Uuid::new_v4(),
            generated_at: Utc::now(),
            total_products_checked: self.total_products_checked,
            discrepancies_found: self.discrepancies.clone(),
            corrections_applied: self.corrections_applied,
            sync_results: self.sync_results.clone(),
            summary,
        }
    }

    /// Build the high-level summary from the collected data.
    fn build_summary(&self) -> ReportSummary {
        let mut stock_mismatches = 0;
        let mut missing_orders = 0;
        let mut orphaned_claims = 0;
        let mut duplicate_vouchers = 0;

        for d in &self.discrepancies {
            match d.discrepancy_type {
                DiscrepancyType::StockMismatch => stock_mismatches += 1,
                DiscrepancyType::MissingOrder => missing_orders += 1,
                DiscrepancyType::OrphanedClaim => orphaned_claims += 1,
                DiscrepancyType::DuplicateVoucher => duplicate_vouchers += 1,
            }
        }

        let total_corrections = self.corrections_applied;
        // System is healthy if there are no discrepancies and no pending corrections.
        let healthy = self.discrepancies.is_empty();

        ReportSummary {
            healthy,
            stock_mismatches,
            missing_orders,
            orphaned_claims,
            duplicate_vouchers,
            total_corrections,
        }
    }

    /// Render the report as a JSON string.
    pub fn generate_json(&self) -> Result<String, serde_json::Error> {
        let report = self.generate();
        serde_json::to_string_pretty(&report)
    }

    /// Render the report as a human-readable string.
    pub fn generate_human_readable(&self) -> String {
        let report = self.generate();
        report.to_human_readable()
    }
}

/// Builder pattern for constructing a ReportGenerator step by step.
pub struct ReportBuilder {
    sync_results: Vec<SyncResult>,
    discrepancies: Vec<Discrepancy>,
    corrections_applied: usize,
    total_products_checked: usize,
}

impl ReportBuilder {
    pub fn new() -> Self {
        Self {
            sync_results: Vec::new(),
            discrepancies: Vec::new(),
            corrections_applied: 0,
            total_products_checked: 0,
        }
    }

    pub fn with_sync_results(mut self, results: Vec<SyncResult>) -> Self {
        self.sync_results = results;
        self
    }

    pub fn with_discrepancies(mut self, discrepancies: Vec<Discrepancy>) -> Self {
        self.discrepancies = discrepancies;
        self
    }

    pub fn with_corrections(mut self, count: usize) -> Self {
        self.corrections_applied = count;
        self
    }

    pub fn with_products_checked(mut self, count: usize) -> Self {
        self.total_products_checked = count;
        self
    }

    pub fn build(self) -> ReportGenerator {
        ReportGenerator::new(
            self.sync_results,
            self.discrepancies,
            self.corrections_applied,
            self.total_products_checked,
        )
    }
}

impl Default for ReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::Severity;

    fn make_sync_result(product_id: &str, mismatch: bool) -> SyncResult {
        SyncResult {
            product_id: product_id.to_string(),
            redis_stock: 100,
            db_stock: if mismatch { 80 } else { 100 },
            mismatch,
            action_taken: if mismatch {
                SyncAction::UpdatedDb
            } else {
                SyncAction::NoOp
            },
        }
    }

    fn make_discrepancy(dtype: DiscrepancyType) -> Discrepancy {
        Discrepancy {
            discrepancy_id: Uuid::new_v4(),
            discrepancy_type: dtype,
            product_id: Some("SKU-1".into()),
            description: "test discrepancy".into(),
            severity: Severity::High,
            detected_at: Utc::now(),
        }
    }

    #[test]
    fn test_report_structure() {
        let report_gen = ReportBuilder::new()
            .with_sync_results(vec![
                make_sync_result("A", false),
                make_sync_result("B", true),
            ])
            .with_discrepancies(vec![make_discrepancy(DiscrepancyType::StockMismatch)])
            .with_corrections(1)
            .with_products_checked(10)
            .build();

        let report = report_gen.generate();

        assert_eq!(report.total_products_checked, 10);
        assert_eq!(report.discrepancies_found.len(), 1);
        assert_eq!(report.corrections_applied, 1);
        assert_eq!(report.sync_results.len(), 2);
        assert!(!report.summary.healthy);
        assert_eq!(report.summary.stock_mismatches, 1);
    }

    #[test]
    fn test_healthy_report() {
        let report_gen = ReportBuilder::new()
            .with_sync_results(vec![make_sync_result("A", false)])
            .with_discrepancies(vec![])
            .with_corrections(0)
            .with_products_checked(5)
            .build();

        let report = report_gen.generate();

        assert!(report.summary.healthy);
        assert_eq!(report.summary.stock_mismatches, 0);
        assert_eq!(report.summary.missing_orders, 0);
        assert_eq!(report.summary.orphaned_claims, 0);
        assert_eq!(report.summary.duplicate_vouchers, 0);
    }

    #[test]
    fn test_json_serialization() {
        let report_gen = ReportBuilder::new()
            .with_sync_results(vec![make_sync_result("SKU-1", true)])
            .with_discrepancies(vec![
                make_discrepancy(DiscrepancyType::MissingOrder),
                make_discrepancy(DiscrepancyType::DuplicateVoucher),
            ])
            .with_corrections(1)
            .with_products_checked(3)
            .build();

        let json = report_gen.generate_json().unwrap();
        assert!(json.contains("\"total_products_checked\": 3"));
        assert!(json.contains("\"corrections_applied\": 1"));
        assert!(json.contains("MissingOrder"));
        assert!(json.contains("DuplicateVoucher"));

        // Verify it round-trips.
        let parsed: ReconciliationReport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.total_products_checked, 3);
        assert_eq!(parsed.discrepancies_found.len(), 2);
    }

    #[test]
    fn test_human_readable_output() {
        let report_gen = ReportBuilder::new()
            .with_sync_results(vec![])
            .with_discrepancies(vec![])
            .with_corrections(0)
            .with_products_checked(5)
            .build();

        let text = report_gen.generate_human_readable();
        assert!(text.contains("Reconciliation Report"));
        assert!(text.contains("Products checked: 5"));
        assert!(text.contains("HEALTHY"));
    }

    #[test]
    fn test_summary_counts_all_types() {
        let report_gen = ReportBuilder::new()
            .with_sync_results(vec![])
            .with_discrepancies(vec![
                make_discrepancy(DiscrepancyType::StockMismatch),
                make_discrepancy(DiscrepancyType::StockMismatch),
                make_discrepancy(DiscrepancyType::MissingOrder),
                make_discrepancy(DiscrepancyType::OrphanedClaim),
                make_discrepancy(DiscrepancyType::DuplicateVoucher),
                make_discrepancy(DiscrepancyType::DuplicateVoucher),
            ])
            .with_corrections(3)
            .with_products_checked(20)
            .build();

        let report = report_gen.generate();
        assert_eq!(report.summary.stock_mismatches, 2);
        assert_eq!(report.summary.missing_orders, 1);
        assert_eq!(report.summary.orphaned_claims, 1);
        assert_eq!(report.summary.duplicate_vouchers, 2);
        assert_eq!(report.summary.total_corrections, 3);
        assert!(!report.summary.healthy);
    }
}
