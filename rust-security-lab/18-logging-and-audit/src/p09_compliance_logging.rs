//! # Lesson 09: Compliance Logging
//!
//! ## The Problem
//!
//! Different regulatory frameworks have specific logging requirements. Failing to
//! meet these requirements can result in massive fines (GDPR: up to 4% of global
//! revenue), loss of certifications (SOC2), or criminal liability (SOX).
//!
//! ## Framework Requirements
//!
//! ### GDPR (General Data Protection Regulation)
//! - Must log all access to personal data
//! - Must be able to produce data access records for a specific user
//! - Must support right to erasure (delete user data from logs)
//! - Must log data breaches within 72 hours
//!
//! ### SOC2 (Service Organization Control 2)
//! - Audit trail for all access to sensitive data
//! - Tamper-evident logs
//! - Log retention for minimum 1 year
//! - Access controls on log storage
//!
//! ### HIPAA (Health Insurance Portability and Accountability Act)
//! - Access logs for all PHI (Protected Health Information)
//! - 6-year retention minimum
//! - Encryption of logs at rest
//! - Audit trail for all modifications
//!
//! ### PCI-DSS (Payment Card Industry Data Security Standard)
//! - Never log full PAN (Primary Account Number)
//! - Audit trail for all access to cardholder data
//! - 1-year retention (3 months immediately available)
//! - Daily review of logs
//!
//! ## What You'll Implement
//!
//! 1. Compliance-aware log entry creation
//! 2. Data classification for automatic compliance tagging
//! 3. Right-to-erasure support (GDPR Article 17)
//! 4. Compliance report generation
//! 5. Retention policy enforcement
//! 6. Compliance violation detection

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data classification levels for compliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataClassification {
    /// Public data, no special handling
    Public,
    /// Internal data, standard logging
    Internal,
    /// Confidential data, must be protected
    Confidential,
    /// Restricted data (PII, PHI, financial), maximum protection
    Restricted,
}

/// Compliance frameworks.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceFramework {
    Gdpr,
    Soc2,
    Hipaa,
    PciDss,
    Sox,
}

/// A compliance-tagged log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceLogEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub user_id: Option<String>,
    pub data_classification: DataClassification,
    pub resource: String,
    pub action: String,
    pub result: String,
    pub applicable_frameworks: Vec<ComplianceFramework>,
    pub contains_pii: bool,
    pub retention_days: u32,
}

/// Exercise 1: Create a compliance log entry.
///
/// Automatically set:
/// - timestamp to Utc::now()
/// - applicable_frameworks based on data_classification:
///   - Public: none
///   - Internal: [Soc2]
///   - Confidential: [Soc2, Gdpr]
///   - Restricted: [Soc2, Gdpr, Hipaa, PciDss]
/// - retention_days based on framework requirements:
///   - Gdpr: 365 days
///   - Soc2: 365 days
///   - Hipaa: 2190 days (6 years)
///   - PciDss: 365 days
///   - Sox: 2555 days (7 years)
///   - Default: 365 days
/// - contains_pii: true if Restricted
pub fn create_compliance_entry(
    event_type: &str,
    user_id: Option<&str>,
    data_classification: DataClassification,
    resource: &str,
    action: &str,
    result: &str,
) -> ComplianceLogEntry {
    todo!("Implement compliance entry creation")
}

/// Exercise 2: Check if a log entry is within its retention period.
///
/// Return true if the entry has not expired.
///
/// Hints:
/// - Compute expiry: `entry.timestamp + Duration::days(entry.retention_days as i64)`
/// - Compare with `Utc::now()`
pub fn is_within_retention(entry: &ComplianceLogEntry) -> bool {
    todo!("Implement retention check")
}

/// Exercise 3: Simulate GDPR right to erasure.
///
/// Given a list of entries and a user_id, return entries with that user's
/// PII redacted (user_id replaced with "[REDACTED]", and any PII fields
/// replaced with "[GDPR_ERASED]").
///
/// This is the "right to be forgotten" -- the entries remain for audit
/// purposes but the user's identity is removed.
///
/// Hints:
/// - Clone entries
/// - Replace matching user_id with "[REDACTED]"
/// - Set contains_pii to false
pub fn gdpr_erase_user(
    entries: &mut [ComplianceLogEntry],
    user_id: &str,
) {
    todo!("Implement GDPR right to erasure")
}

/// Exercise 4: Generate a compliance report for a specific framework.
///
/// Return a String containing:
/// - Framework name
/// - Total entries applicable to this framework
/// - Entries within retention
/// - Entries expired
/// - PII-containing entries
/// - Any violations detected
pub fn compliance_report(
    entries: &[ComplianceLogEntry],
    framework: &ComplianceFramework,
) -> String {
    todo!("Implement compliance report generation")
}

/// A compliance violation.
#[derive(Debug, Clone)]
pub struct ComplianceViolation {
    pub framework: ComplianceFramework,
    pub description: String,
    pub severity: String,
}

/// Exercise 5: Detect compliance violations in a set of log entries.
///
/// Check for:
/// - Expired entries that should have been deleted (retention violation)
/// - Restricted data without all applicable frameworks tagged
/// - PII entries without user_id (can't support right to erasure)
///
/// Return a list of violations found.
pub fn detect_violations(entries: &[ComplianceLogEntry]) -> Vec<ComplianceViolation> {
    todo!("Implement violation detection")
}

/// Exercise 6: Check if an action on data requires audit logging.
///
/// Actions that require audit:
/// - Read on Confidential or Restricted data
/// - Write on any non-Public data
/// - Delete on any data
/// - Export on any data
///
/// Hints:
/// - Match on (action, classification) pairs
pub fn requires_audit(action: &str, classification: &DataClassification) -> bool {
    todo!("Implement audit requirement check")
}

/// Exercise 7: Get applicable frameworks for a data classification.
pub fn applicable_frameworks(classification: &DataClassification) -> Vec<ComplianceFramework> {
    todo!("Implement framework lookup")
}

/// Exercise 8: Calculate the required retention period for a set of frameworks.
///
/// Return the MAXIMUM required retention across all frameworks.
pub fn required_retention_days(frameworks: &[ComplianceFramework]) -> u32 {
    todo!("Implement retention calculation")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_compliance_entry_restricted() {
        let entry = create_compliance_entry(
            "data_access",
            Some("user-1"),
            DataClassification::Restricted,
            "/api/phi",
            "read",
            "success",
        );
        assert!(entry.applicable_frameworks.contains(&ComplianceFramework::Gdpr));
        assert!(entry.applicable_frameworks.contains(&ComplianceFramework::Hipaa));
        assert!(entry.contains_pii);
    }

    #[test]
    fn test_create_compliance_entry_public() {
        let entry = create_compliance_entry(
            "page_view",
            None,
            DataClassification::Public,
            "/about",
            "read",
            "success",
        );
        assert!(entry.applicable_frameworks.is_empty());
        assert!(!entry.contains_pii);
    }

    #[test]
    fn test_retention_check() {
        let entry = ComplianceLogEntry {
            timestamp: Utc::now() - Duration::days(100),
            event_type: "test".to_string(),
            user_id: None,
            data_classification: DataClassification::Internal,
            resource: "/api/test".to_string(),
            action: "read".to_string(),
            result: "success".to_string(),
            applicable_frameworks: vec![ComplianceFramework::Soc2],
            contains_pii: false,
            retention_days: 365,
        };
        assert!(is_within_retention(&entry));
    }

    #[test]
    fn test_retention_expired() {
        let entry = ComplianceLogEntry {
            timestamp: Utc::now() - Duration::days(400),
            event_type: "test".to_string(),
            user_id: None,
            data_classification: DataClassification::Internal,
            resource: "/api/test".to_string(),
            action: "read".to_string(),
            result: "success".to_string(),
            applicable_frameworks: vec![ComplianceFramework::Soc2],
            contains_pii: false,
            retention_days: 365,
        };
        assert!(!is_within_retention(&entry));
    }

    #[test]
    fn test_gdpr_erase() {
        let mut entries = vec![
            create_compliance_entry(
                "data_access", Some("user-1"), DataClassification::Restricted,
                "/api/phi", "read", "success",
            ),
            create_compliance_entry(
                "data_access", Some("user-2"), DataClassification::Restricted,
                "/api/phi", "read", "success",
            ),
        ];
        gdpr_erase_user(&mut entries, "user-1");
        assert_eq!(entries[0].user_id, Some("[REDACTED]".to_string()));
        assert_eq!(entries[1].user_id, Some("user-2".to_string()));
    }

    #[test]
    fn test_compliance_report() {
        let entries = vec![
            create_compliance_entry(
                "data_access", Some("u1"), DataClassification::Restricted,
                "/api/phi", "read", "success",
            ),
        ];
        let report = compliance_report(&entries, &ComplianceFramework::Hipaa);
        assert!(report.contains("HIPAA"));
        assert!(report.contains("1")); // 1 entry
    }

    #[test]
    fn test_requires_audit() {
        assert!(requires_audit("read", &DataClassification::Confidential));
        assert!(requires_audit("write", &DataClassification::Internal));
        assert!(requires_audit("delete", &DataClassification::Public));
        assert!(!requires_audit("read", &DataClassification::Public));
    }

    #[test]
    fn test_retention_calculation() {
        let frameworks = vec![ComplianceFramework::Soc2, ComplianceFramework::Hipaa];
        let days = required_retention_days(&frameworks);
        assert_eq!(days, 2190); // HIPAA's 6 years is the max
    }

    #[test]
    fn test_detect_violations_pii_without_user() {
        let entry = ComplianceLogEntry {
            timestamp: Utc::now(),
            event_type: "data_access".to_string(),
            user_id: None,
            data_classification: DataClassification::Restricted,
            resource: "/api/phi".to_string(),
            action: "read".to_string(),
            result: "success".to_string(),
            applicable_frameworks: vec![ComplianceFramework::Gdpr],
            contains_pii: true,
            retention_days: 365,
        };
        let violations = detect_violations(&[entry]);
        assert!(!violations.is_empty());
    }
}
