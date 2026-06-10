//! # Lesson 05: Audit Trail Implementation
//!
//! ## The Problem
//!
//! When a security incident occurs, investigators need to answer: "Who did what,
//! when, where, and what was the result?" Without a proper audit trail, these
//! questions are impossible to answer. Compliance frameworks (SOC2, HIPAA, PCI-DSS)
//! REQUIRE audit trails for all access to sensitive data.
//!
//! ## The 5 W's of Audit Logging
//!
//! Every audit entry must answer:
//! - **WHO**: User ID, service account, IP address
//! - **WHAT**: Action performed (read, write, delete, admin action)
//! - **WHEN**: Precise timestamp (UTC, nanosecond precision)
//! - **WHERE**: Resource accessed (file, API endpoint, database table)
//! - **RESULT**: Success or failure, with reason if failed
//!
//! ## Attack Scenario
//!
//! An employee accesses customer records they shouldn't have access to. Without
//! an audit trail, you can't prove who accessed what. With a proper audit trail,
//! you can show: "User emp-456 queried customer_records table at 2024-01-15 14:30
//! from IP 10.0.0.50, accessed 500 records, result: success."
//!
//! ## What You'll Implement
//!
//! 1. An `AuditEntry` struct with all 5 W's
//! 2. Action types for common operations
//! 3. An audit logger that records entries
//! 4. Query functions for investigation
//! 5. Anomaly detection (unusual access patterns)
//! 6. Export for compliance reporting

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Actions that can be audited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    /// Read access to a resource
    Read,
    /// Write/modify a resource
    Write,
    /// Delete a resource
    Delete,
    /// Authentication attempt
    Authenticate,
    /// Authorization check
    Authorize,
    /// Administrative action
    Admin,
    /// Export/download data
    Export,
    /// Configuration change
    ConfigChange,
}

/// Result of an audited action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Denied,
    Error,
}

/// A single audit trail entry capturing the 5 W's.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub ip_address: String,
    pub action: AuditAction,
    pub resource: String,
    pub result: AuditResult,
    pub details: String,
    pub session_id: Option<String>,
}

/// Exercise 1: Create an AuditEntry.
///
/// Set timestamp to Utc::now(). All other fields come from parameters.
pub fn create_audit_entry(
    user_id: &str,
    ip_address: &str,
    action: AuditAction,
    resource: &str,
    result: AuditResult,
    details: &str,
) -> AuditEntry {
    todo!("Implement audit entry creation")
}

/// Exercise 2: Serialize an AuditEntry to JSON.
///
/// Hints:
/// - Use `serde_json::to_string(&entry).map_err(|e| e.to_string())`
pub fn audit_to_json(entry: &AuditEntry) -> Result<String, String> {
    todo!("Implement audit entry JSON serialization")
}

/// Exercise 3: Deserialize a JSON string to an AuditEntry.
pub fn json_to_audit(json: &str) -> Result<AuditEntry, String> {
    todo!("Implement audit entry JSON deserialization")
}

/// An audit log that stores entries and provides query capabilities.
pub struct AuditLog {
    pub entries: Vec<AuditEntry>,
}

impl AuditLog {
    pub fn new() -> Self {
        AuditLog { entries: Vec::new() }
    }

    /// Exercise 4: Record an audit entry.
    pub fn record(&mut self, entry: AuditEntry) {
        todo!("Implement entry recording")
    }

    /// Exercise 5: Query entries by user ID.
    pub fn entries_for_user(&self, user_id: &str) -> Vec<&AuditEntry> {
        todo!("Implement user-based query")
    }

    /// Exercise 6: Query entries by action type.
    pub fn entries_for_action(&self, action: &AuditAction) -> Vec<&AuditEntry> {
        todo!("Implement action-based query")
    }

    /// Exercise 7: Query failed actions (Denied or Error).
    pub fn failed_entries(&self) -> Vec<&AuditEntry> {
        todo!("Implement failed entries query")
    }

    /// Exercise 8: Detect anomalies -- users with more than threshold failed
    /// authentication attempts.
    ///
    Return a list of user_ids that exceed the threshold.
    ///
    /// Hints:
    /// - Count auth failures per user using a HashMap
    /// - Filter users exceeding threshold
    pub fn detect_brute_force(&self, threshold: usize) -> Vec<String> {
        todo!("Implement brute force detection")
    }

    /// Exercise 9: Generate a compliance report.
    ///
    /// Return a summary string with:
    /// - Total entries
    /// - Entries by action type
    /// - Success/failure ratio
    /// - Unique users
    /// - Unique resources accessed
    pub fn compliance_summary(&self) -> String {
        todo!("Implement compliance report generation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_audit_entry() {
        let entry = create_audit_entry(
            "user-1",
            "10.0.0.1",
            AuditAction::Read,
            "/api/customers",
            AuditResult::Success,
            "Read customer list",
        );
        assert_eq!(entry.user_id, "user-1");
        assert_eq!(entry.action, AuditAction::Read);
        assert_eq!(entry.result, AuditResult::Success);
    }

    #[test]
    fn test_audit_json_roundtrip() {
        let entry = create_audit_entry(
            "user-1",
            "10.0.0.1",
            AuditAction::Write,
            "/api/config",
            AuditResult::Success,
            "Updated config",
        );
        let json = audit_to_json(&entry).unwrap();
        let parsed = json_to_audit(&json).unwrap();
        assert_eq!(parsed.user_id, "user-1");
        assert_eq!(parsed.action, AuditAction::Write);
    }

    #[test]
    fn test_audit_log_record_and_query() {
        let mut log = AuditLog::new();
        log.record(create_audit_entry(
            "user-1", "10.0.0.1", AuditAction::Read, "/api/data",
            AuditResult::Success, "read",
        ));
        log.record(create_audit_entry(
            "user-2", "10.0.0.2", AuditAction::Write, "/api/data",
            AuditResult::Success, "write",
        ));
        log.record(create_audit_entry(
            "user-1", "10.0.0.1", AuditAction::Delete, "/api/data",
            AuditResult::Denied, "no permission",
        ));

        assert_eq!(log.entries_for_user("user-1").len(), 2);
        assert_eq!(log.entries_for_user("user-2").len(), 1);
    }

    #[test]
    fn test_query_by_action() {
        let mut log = AuditLog::new();
        log.record(create_audit_entry(
            "u1", "10.0.0.1", AuditAction::Read, "/api/x",
            AuditResult::Success, "",
        ));
        log.record(create_audit_entry(
            "u1", "10.0.0.1", AuditAction::Delete, "/api/x",
            AuditResult::Denied, "",
        ));

        assert_eq!(log.entries_for_action(&AuditAction::Read).len(), 1);
        assert_eq!(log.entries_for_action(&AuditAction::Delete).len(), 1);
    }

    #[test]
    fn test_failed_entries() {
        let mut log = AuditLog::new();
        log.record(create_audit_entry(
            "u1", "10.0.0.1", AuditAction::Authenticate, "/login",
            AuditResult::Error, "bad password",
        ));
        log.record(create_audit_entry(
            "u1", "10.0.0.1", AuditAction::Authenticate, "/login",
            AuditResult::Success, "ok",
        ));

        assert_eq!(log.failed_entries().len(), 1);
    }

    #[test]
    fn test_brute_force_detection() {
        let mut log = AuditLog::new();
        for _ in 0..5 {
            log.record(create_audit_entry(
                "attacker", "10.0.0.99", AuditAction::Authenticate, "/login",
                AuditResult::Error, "bad password",
            ));
        }
        log.record(create_audit_entry(
            "legit", "10.0.0.2", AuditAction::Authenticate, "/login",
            AuditResult::Error, "typo",
        ));

        let suspects = log.detect_brute_force(3);
        assert!(suspects.contains(&"attacker".to_string()));
        assert!(!suspects.contains(&"legit".to_string()));
    }

    #[test]
    fn test_compliance_summary() {
        let mut log = AuditLog::new();
        log.record(create_audit_entry(
            "u1", "10.0.0.1", AuditAction::Read, "/api/data",
            AuditResult::Success, "",
        ));
        log.record(create_audit_entry(
            "u2", "10.0.0.2", AuditAction::Write, "/api/data",
            AuditResult::Denied, "",
        ));

        let summary = log.compliance_summary();
        assert!(summary.contains("Total entries: 2"));
    }

    #[test]
    fn test_audit_entry_has_session_id() {
        let mut entry = create_audit_entry(
            "u1", "10.0.0.1", AuditAction::Read, "/api/x",
            AuditResult::Success, "",
        );
        entry.session_id = Some("sess-123".to_string());
        assert_eq!(entry.session_id, Some("sess-123".to_string()));
    }
}
