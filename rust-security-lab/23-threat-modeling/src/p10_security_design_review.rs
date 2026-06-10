//! # Lesson 10: Security Design Review
//!
//! ## The Problem
//!
//! A threat model is only as good as its validation. If the threat model is incomplete,
//! outdated, or wrong, the system has blind spots. Security design review is the process
//! of validating that the threat model is accurate, complete, and that all mitigations
//! are properly implemented.
//!
//! ## The Solution: Structured Design Review
//!
//! A security design review checks:
//!
//! ```text
//! 1. Completeness    — Are all components modeled?
//! 2. Accuracy        — Does the model match the actual architecture?
//! 3. Threat Coverage — Are all STRIDE categories addressed?
//! 4. Mitigation      — Are mitigations implemented and tested?
//! 5. Risk Acceptance — Are residual risks formally accepted?
//! 6. Freshness       — When was the model last updated?
//! ```
//!
//! The review produces a checklist of findings, each with a severity and recommendation.
//!
//! ## Attack Example: Stale Threat Model
//!
//! A team creates a threat model at launch. Six months later, they add a new microservice
//! and a third-party integration. The threat model was never updated. The new service has
//! no authentication, and the third-party integration exposes internal APIs. Neither was
//! caught because the threat model was stale.
//!
//! Defense: Review and update the threat model with every architecture change.

use serde::{Deserialize, Serialize};

/// Severity of a design review finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FindingSeverity {
    /// Informational observation
    Info,
    /// Should be addressed in next release
    Warning,
    /// Must be addressed before release
    Critical,
    /// Blocks release
    Blocker,
}

/// Status of a design review finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingStatus {
    /// Newly identified
    Open,
    /// Being worked on
    InProgress,
    /// Resolved and verified
    Resolved,
    /// Accepted as-is with justification
    Accepted,
}

/// A single finding from a security design review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub area: String,
    pub description: String,
    pub severity: FindingSeverity,
    pub status: FindingStatus,
    pub recommendation: String,
}

/// A checklist item for the design review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub question: String,
    pub passed: bool,
    pub notes: String,
}

/// A complete security design review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignReview {
    pub system_name: String,
    pub reviewer: String,
    pub checklist: Vec<ChecklistItem>,
    pub findings: Vec<Finding>,
}

impl DesignReview {
    /// Create a new empty design review.
    pub fn new(system_name: &str, reviewer: &str) -> Self {
        todo!("Create empty review")
    }

    /// Add a checklist item.
    pub fn add_checklist_item(&mut self, item: ChecklistItem) {
        todo!("Add checklist item")
    }

    /// Add a finding.
    pub fn add_finding(&mut self, finding: Finding) {
        todo!("Add finding")
    }

    /// Return all findings with a given severity.
    pub fn findings_by_severity(&self, severity: FindingSeverity) -> Vec<&Finding> {
        todo!("Filter findings by severity")
    }

    /// Return all findings with a given status.
    pub fn findings_by_status(&self, status: FindingStatus) -> Vec<&Finding> {
        todo!("Filter findings by status")
    }

    /// Return checklist items that failed.
    pub fn failed_checks(&self) -> Vec<&ChecklistItem> {
        todo!("Filter where passed is false")
    }

    /// Return checklist items that passed.
    pub fn passed_checks(&self) -> Vec<&ChecklistItem> {
        todo!("Filter where passed is true")
    }

    /// Check if the review is clean: no open Critical or Blocker findings.
    pub fn is_clean(&self) -> bool {
        todo!("Check that no findings with Critical or Blocker severity have Open status")
    }

    /// Compute the checklist pass rate as a percentage.
    pub fn pass_rate(&self) -> f64 {
        todo!("Return passed / total * 100, or 100 if no items")
    }

    /// Return all open findings sorted by severity (most severe first).
    pub fn open_findings_sorted(&self) -> Vec<&Finding> {
        todo!("Filter by Open/InProgress status, sort by severity descending")
    }

    /// Count total findings.
    pub fn finding_count(&self) -> usize {
        todo!("Return finding count")
    }

    /// Count total checklist items.
    pub fn checklist_count(&self) -> usize {
        todo!("Return checklist count")
    }
}

/// Build a design review for a web application with checklist items and findings.
pub fn build_webapp_design_review() -> DesignReview {
    todo!("Build a review with at least 6 checklist items and 4 findings covering different severities")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_review() -> DesignReview {
        let mut review = DesignReview::new("test-system", "security-team");

        review.add_checklist_item(ChecklistItem {
            id: "C001".into(),
            question: "Are all external inputs validated?".into(),
            passed: true,
            notes: "All API endpoints use serde validation".into(),
        });
        review.add_checklist_item(ChecklistItem {
            id: "C002".into(),
            question: "Is authentication required on all endpoints?".into(),
            passed: false,
            notes: "Health check endpoint is unauthenticated".into(),
        });
        review.add_checklist_item(ChecklistItem {
            id: "C003".into(),
            question: "Are secrets stored securely?".into(),
            passed: true,
            notes: "Using HashiCorp Vault for secret management".into(),
        });

        review.add_finding(Finding {
            id: "F001".into(),
            area: "Authentication".into(),
            description: "Health check endpoint exposes internal service information".into(),
            severity: FindingSeverity::Warning,
            status: FindingStatus::Open,
            recommendation: "Remove internal details from health check response".into(),
        });
        review.add_finding(Finding {
            id: "F002".into(),
            area: "Data Protection".into(),
            description: "Database credentials in environment variables without encryption".into(),
            severity: FindingSeverity::Critical,
            status: FindingStatus::InProgress,
            recommendation: "Migrate to encrypted secrets management".into(),
        });
        review.add_finding(Finding {
            id: "F003".into(),
            area: "Logging".into(),
            description: "PII appearing in application logs".into(),
            severity: FindingSeverity::Blocker,
            status: FindingStatus::Open,
            recommendation: "Implement log sanitization for all PII fields".into(),
        });

        review
    }

    #[test]
    fn test_review_creation() {
        let review = DesignReview::new("my-system", "alice");
        assert_eq!(review.system_name, "my-system");
        assert_eq!(review.reviewer, "alice");
        assert_eq!(review.finding_count(), 0);
        assert_eq!(review.checklist_count(), 0);
    }

    #[test]
    fn test_add_items() {
        let review = make_test_review();
        assert_eq!(review.checklist_count(), 3);
        assert_eq!(review.finding_count(), 3);
    }

    #[test]
    fn test_findings_by_severity() {
        let review = make_test_review();
        let warnings = review.findings_by_severity(FindingSeverity::Warning);
        assert_eq!(warnings.len(), 1);
        let blockers = review.findings_by_severity(FindingSeverity::Blocker);
        assert_eq!(blockers.len(), 1);
    }

    #[test]
    fn test_findings_by_status() {
        let review = make_test_review();
        let open = review.findings_by_status(FindingStatus::Open);
        assert_eq!(open.len(), 2);
        let in_progress = review.findings_by_status(FindingStatus::InProgress);
        assert_eq!(in_progress.len(), 1);
    }

    #[test]
    fn test_failed_checks() {
        let review = make_test_review();
        let failed = review.failed_checks();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].id, "C002");
    }

    #[test]
    fn test_passed_checks() {
        let review = make_test_review();
        let passed = review.passed_checks();
        assert_eq!(passed.len(), 2);
    }

    #[test]
    fn test_is_clean() {
        let review = make_test_review();
        // Has open Blocker finding
        assert!(!review.is_clean());
    }

    #[test]
    fn test_pass_rate() {
        let review = make_test_review();
        // 2 passed out of 3
        assert!((review.pass_rate() - (2.0 / 3.0 * 100.0)).abs() < 0.01);
    }

    #[test]
    fn test_open_findings_sorted() {
        let review = make_test_review();
        let open = review.open_findings_sorted();
        // F001=Open(Warning), F002=InProgress(Critical), F003=Open(Blocker)
        assert_eq!(open.len(), 3);
        // Blocker should come first
        assert_eq!(open[0].severity, FindingSeverity::Blocker);
        assert_eq!(open[1].severity, FindingSeverity::Critical);
        assert_eq!(open[2].severity, FindingSeverity::Warning);
    }

    #[test]
    fn test_build_webapp_design_review() {
        let review = build_webapp_design_review();
        assert!(review.checklist_count() >= 6, "Should have at least 6 checklist items");
        assert!(review.finding_count() >= 4, "Should have at least 4 findings");
    }
}
