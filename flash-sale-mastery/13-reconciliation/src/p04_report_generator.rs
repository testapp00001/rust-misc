//! # Exercise 04: Report Generator
//!
//! ## Learning Objective
//! Implement structured reconciliation report generation with both machine-readable
//! (JSON) and human-readable output formats.
//!
//! ## Flash Sale Context
//! After reconciliation runs, stakeholders need to know: how many products were
//! checked, what discrepancies were found, and what corrections were applied.
//! The report must be serializable to JSON for dashboards and APIs, and also
//! renderable as human-readable text for Slack alerts and incident reports.
//!
//! ## Instructions
//! 1. Implement `ReportGenerator::new()` to store sync results, discrepancies,
//!    correction count, and total products checked.
//! 2. Implement `generate()` to produce a `ReconciliationReport` with:
//!    - A unique report ID and timestamp
//!    - All collected data
//!    - A `ReportSummary` computed from the discrepancies
//! 3. Implement `build_summary()` to count each discrepancy type and determine
//!    overall health (healthy only if zero discrepancies).
//! 4. Implement `generate_json()` and `generate_human_readable()`.
//! 5. Implement the `ReportBuilder` for ergonomic construction.
//!
//! ## Hints
//! - Use `Uuid::new_v4()` for the report ID.
//! - The `healthy` flag should be `true` only when the discrepancy list is empty.
//! - `serde_json::to_string_pretty` for formatted JSON output.

use chrono::Utc;
use uuid::Uuid;

use crate::shared::{
    Discrepancy, DiscrepancyType, ReconciliationReport, ReportSummary, SyncAction, SyncResult,
};

/// Generates structured reconciliation reports from sync results and discrepancies.
pub struct ReportGenerator {
    // TODO: add fields
}

impl ReportGenerator {
    /// Create a new ReportGenerator with the collected reconciliation data.
    pub fn new(
        sync_results: Vec<SyncResult>,
        discrepancies: Vec<Discrepancy>,
        corrections_applied: usize,
        total_products_checked: usize,
    ) -> Self {
        todo!("Store all parameters")
    }

    /// Generate a full reconciliation report.
    pub fn generate(&self) -> ReconciliationReport {
        todo!("Build the report with a unique ID, timestamp, summary, and all data")
    }

    /// Build the high-level summary from the collected data.
    fn build_summary(&self) -> ReportSummary {
        todo!("Count each discrepancy type and set the healthy flag")
    }

    /// Render the report as a JSON string.
    pub fn generate_json(&self) -> Result<String, serde_json::Error> {
        todo!("Generate the report and serialize to pretty JSON")
    }

    /// Render the report as a human-readable string.
    pub fn generate_human_readable(&self) -> String {
        todo!("Generate the report and call to_human_readable()")
    }
}

/// Builder pattern for constructing a ReportGenerator step by step.
pub struct ReportBuilder {
    // TODO: add fields
}

impl ReportBuilder {
    pub fn new() -> Self {
        todo!("Initialize with empty/default values")
    }

    pub fn with_sync_results(mut self, results: Vec<SyncResult>) -> Self {
        todo!("Store sync results")
    }

    pub fn with_discrepancies(mut self, discrepancies: Vec<Discrepancy>) -> Self {
        todo!("Store discrepancies")
    }

    pub fn with_corrections(mut self, count: usize) -> Self {
        todo!("Store correction count")
    }

    pub fn with_products_checked(mut self, count: usize) -> Self {
        todo!("Store products checked count")
    }

    pub fn build(self) -> ReportGenerator {
        todo!("Construct a ReportGenerator from the stored values")
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
