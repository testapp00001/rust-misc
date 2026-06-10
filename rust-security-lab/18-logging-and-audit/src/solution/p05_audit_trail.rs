//! # Lesson 05: Audit Trail Implementation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    Read,
    Write,
    Delete,
    Authenticate,
    Authorize,
    Admin,
    Export,
    ConfigChange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Denied,
    Error,
}

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

pub fn create_audit_entry(
    user_id: &str,
    ip_address: &str,
    action: AuditAction,
    resource: &str,
    result: AuditResult,
    details: &str,
) -> AuditEntry {
    AuditEntry {
        timestamp: Utc::now(),
        user_id: user_id.to_string(),
        ip_address: ip_address.to_string(),
        action,
        resource: resource.to_string(),
        result,
        details: details.to_string(),
        session_id: None,
    }
}

pub fn audit_to_json(entry: &AuditEntry) -> Result<String, String> {
    serde_json::to_string(entry).map_err(|e| e.to_string())
}

pub fn json_to_audit(json: &str) -> Result<AuditEntry, String> {
    serde_json::from_str::<AuditEntry>(json).map_err(|e| e.to_string())
}

pub struct AuditLog {
    pub entries: Vec<AuditEntry>,
}

impl AuditLog {
    pub fn new() -> Self {
        AuditLog { entries: Vec::new() }
    }

    pub fn record(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
    }

    pub fn entries_for_user(&self, user_id: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.user_id == user_id).collect()
    }

    pub fn entries_for_action(&self, action: &AuditAction) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| &e.action == action).collect()
    }

    pub fn failed_entries(&self) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.result == AuditResult::Denied || e.result == AuditResult::Error)
            .collect()
    }

    pub fn detect_brute_force(&self, threshold: usize) -> Vec<String> {
        let mut failure_counts: HashMap<&str, usize> = HashMap::new();
        for entry in &self.entries {
            if entry.action == AuditAction::Authenticate
                && (entry.result == AuditResult::Error || entry.result == AuditResult::Denied)
            {
                *failure_counts.entry(&entry.user_id).or_insert(0) += 1;
            }
        }
        failure_counts
            .iter()
            .filter(|(_, &count)| count > threshold)
            .map(|(&user, _)| user.to_string())
            .collect()
    }

    pub fn compliance_summary(&self) -> String {
        let total = self.entries.len();
        let mut action_counts: HashMap<String, usize> = HashMap::new();
        let mut success_count = 0;
        let mut users: std::collections::HashSet<&str> = std::collections::HashSet::new();
        let mut resources: std::collections::HashSet<&str> = std::collections::HashSet::new();

        for entry in &self.entries {
            let action_str = format!("{:?}", entry.action);
            *action_counts.entry(action_str).or_insert(0) += 1;
            if entry.result == AuditResult::Success {
                success_count += 1;
            }
            users.insert(&entry.user_id);
            resources.insert(&entry.resource);
        }

        let mut summary = format!("Total entries: {}\n", total);
        summary.push_str(&format!("Unique users: {}\n", users.len()));
        summary.push_str(&format!("Unique resources: {}\n", resources.len()));
        summary.push_str(&format!(
            "Success rate: {}/{}\n",
            success_count, total
        ));
        summary.push_str("Actions:\n");
        for (action, count) in &action_counts {
            summary.push_str(&format!("  {}: {}\n", action, count));
        }
        summary
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
