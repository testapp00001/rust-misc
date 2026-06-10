//! # Lesson 10: Security Design Review (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FindingSeverity {
    Info,
    Warning,
    Critical,
    Blocker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingStatus {
    Open,
    InProgress,
    Resolved,
    Accepted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub area: String,
    pub description: String,
    pub severity: FindingSeverity,
    pub status: FindingStatus,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub question: String,
    pub passed: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignReview {
    pub system_name: String,
    pub reviewer: String,
    pub checklist: Vec<ChecklistItem>,
    pub findings: Vec<Finding>,
}

impl DesignReview {
    pub fn new(system_name: &str, reviewer: &str) -> Self {
        Self {
            system_name: system_name.to_string(),
            reviewer: reviewer.to_string(),
            checklist: Vec::new(),
            findings: Vec::new(),
        }
    }

    pub fn add_checklist_item(&mut self, item: ChecklistItem) {
        self.checklist.push(item);
    }

    pub fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    pub fn findings_by_severity(&self, severity: FindingSeverity) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.severity == severity).collect()
    }

    pub fn findings_by_status(&self, status: FindingStatus) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.status == status).collect()
    }

    pub fn failed_checks(&self) -> Vec<&ChecklistItem> {
        self.checklist.iter().filter(|c| !c.passed).collect()
    }

    pub fn passed_checks(&self) -> Vec<&ChecklistItem> {
        self.checklist.iter().filter(|c| c.passed).collect()
    }

    pub fn is_clean(&self) -> bool {
        !self.findings.iter().any(|f| {
            (f.severity == FindingSeverity::Critical || f.severity == FindingSeverity::Blocker)
                && f.status == FindingStatus::Open
        })
    }

    pub fn pass_rate(&self) -> f64 {
        if self.checklist.is_empty() {
            return 100.0;
        }
        let passed = self.checklist.iter().filter(|c| c.passed).count();
        (passed as f64 / self.checklist.len() as f64) * 100.0
    }

    pub fn open_findings_sorted(&self) -> Vec<&Finding> {
        let mut open: Vec<&Finding> = self
            .findings
            .iter()
            .filter(|f| f.status == FindingStatus::Open || f.status == FindingStatus::InProgress)
            .collect();
        open.sort_by(|a, b| b.severity.cmp(&a.severity));
        open
    }

    pub fn finding_count(&self) -> usize {
        self.findings.len()
    }

    pub fn checklist_count(&self) -> usize {
        self.checklist.len()
    }
}

pub fn build_webapp_design_review() -> DesignReview {
    let mut review = DesignReview::new("web-application", "security-review-board");

    // Checklist items
    review.add_checklist_item(ChecklistItem {
        id: "C001".into(),
        question: "Are all external inputs validated and sanitized?".into(),
        passed: true,
        notes: "All API endpoints use serde deserialization with validation".into(),
    });
    review.add_checklist_item(ChecklistItem {
        id: "C002".into(),
        question: "Is authentication required on all data-modifying endpoints?".into(),
        passed: true,
        notes: "JWT middleware applied to all non-public routes".into(),
    });
    review.add_checklist_item(ChecklistItem {
        id: "C003".into(),
        question: "Is authorization checked at the resource level?".into(),
        passed: false,
        notes: "Some admin endpoints only check role, not resource ownership".into(),
    });
    review.add_checklist_item(ChecklistItem {
        id: "C004".into(),
        question: "Are all secrets stored in a secrets manager?".into(),
        passed: true,
        notes: "AWS Secrets Manager with automatic rotation".into(),
    });
    review.add_checklist_item(ChecklistItem {
        id: "C005".into(),
        question: "Is TLS enforced on all connections?".into(),
        passed: true,
        notes: "HSTS enabled, all internal services use mTLS".into(),
    });
    review.add_checklist_item(ChecklistItem {
        id: "C006".into(),
        question: "Are audit logs immutable and tamper-evident?".into(),
        passed: false,
        notes: "Audit logs stored in same database as application data".into(),
    });
    review.add_checklist_item(ChecklistItem {
        id: "C007".into(),
        question: "Is there a documented incident response plan?".into(),
        passed: true,
        notes: "Runbook in Confluence, quarterly tabletop exercises".into(),
    });
    review.add_checklist_item(ChecklistItem {
        id: "C008".into(),
        question: "Are all dependencies scanned for known vulnerabilities?".into(),
        passed: true,
        notes: "Dependabot + Snyk integration in CI pipeline".into(),
    });

    // Findings
    review.add_finding(Finding {
        id: "F001".into(),
        area: "Authorization".into(),
        description: "Admin endpoints check role but not resource-level ownership, allowing horizontal privilege escalation".into(),
        severity: FindingSeverity::Critical,
        status: FindingStatus::Open,
        recommendation: "Implement resource-level authorization checks using ABAC or ownership verification".into(),
    });

    review.add_finding(Finding {
        id: "F002".into(),
        area: "Logging".into(),
        description: "Audit logs stored in application database; compromised DB allows log tampering".into(),
        severity: FindingSeverity::Critical,
        status: FindingStatus::InProgress,
        recommendation: "Migrate audit logs to append-only storage (e.g., AWS CloudTrail, immutable S3)".into(),
    });

    review.add_finding(Finding {
        id: "F003".into(),
        area: "Configuration".into(),
        description: "Debug mode enabled in production configuration template".into(),
        severity: FindingSeverity::Warning,
        status: FindingStatus::Open,
        recommendation: "Remove debug flag from production config; use environment-based config".into(),
    });

    review.add_finding(Finding {
        id: "F004".into(),
        area: "Data Protection".into(),
        description: "User email addresses returned in API responses without filtering".into(),
        severity: FindingSeverity::Info,
        status: FindingStatus::Accepted,
        recommendation: "Document as accepted risk; email addresses are semi-public in this application context".into(),
    });

    review.add_finding(Finding {
        id: "F005".into(),
        area: "Authentication".into(),
        description: "No account lockout after repeated failed login attempts".into(),
        severity: FindingSeverity::Critical,
        status: FindingStatus::Open,
        recommendation: "Implement progressive rate limiting and account lockout with CAPTCHA".into(),
    });

    review
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
        assert!(!review.is_clean());
    }

    #[test]
    fn test_pass_rate() {
        let review = make_test_review();
        assert!((review.pass_rate() - (2.0 / 3.0 * 100.0)).abs() < 0.01);
    }

    #[test]
    fn test_open_findings_sorted() {
        let review = make_test_review();
        let open = review.open_findings_sorted();
        // F001=Open(Warning), F002=InProgress(Critical), F003=Open(Blocker)
        assert_eq!(open.len(), 3);
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
