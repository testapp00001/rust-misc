//! # Lesson 05: GDPR Technical Implementation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json;

/// A user's personal data record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub user_id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub preferences: HashMap<String, String>,
    pub created_at: u64,
}

/// A log entry for GDPR audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub user_id: String,
    pub action: String,
    pub timestamp: u64,
    pub details: String,
}

/// Export user data as JSON (Data Portability - Art. 20).
pub fn export_user_data(record: &UserRecord) -> Result<String, String> {
    serde_json::to_string_pretty(record).map_err(|e| format!("JSON serialization failed: {}", e))
}

/// Erase personal data from user record (Right to Erasure - Art. 17).
pub fn erase_user_data(record: &UserRecord) -> UserRecord {
    UserRecord {
        user_id: record.user_id.clone(),
        name: "[ERASED]".to_string(),
        email: "[ERASED]".to_string(),
        phone: "[ERASED]".to_string(),
        preferences: HashMap::new(),
        created_at: record.created_at,
    }
}

/// Propagate erasure across storage systems.
pub fn propagate_erasure(systems: &[&str], _user_id: &str) -> Vec<(String, bool)> {
    systems
        .iter()
        .map(|sys| {
            let success = !sys.contains("readonly") && !sys.contains("backup_locked");
            (sys.to_string(), success)
        })
        .collect()
}

/// Record a consent decision as an audit entry.
pub fn record_consent(
    user_id: &str,
    granted: bool,
    purpose: &str,
    timestamp: u64,
) -> AuditEntry {
    AuditEntry {
        user_id: user_id.to_string(),
        action: if granted { "consent_granted" } else { "consent_withdrawn" }.to_string(),
        timestamp,
        details: format!("{} for {}", if granted { "Granted" } else { "Withdrawn" }, purpose),
    }
}

/// Validate that all erasure operations succeeded.
pub fn validate_erasure(results: &[(String, bool)]) -> Result<(), Vec<String>> {
    let failures: Vec<String> = results
        .iter()
        .filter(|(_, success)| !success)
        .map(|(system, _)| system.clone())
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

/// Generate GDPR compliance report.
pub fn compliance_report(
    record: &UserRecord,
    audit_log: &[AuditEntry],
) -> Result<String, String> {
    let user_data = export_user_data(record)?;
    let user_audit: Vec<&AuditEntry> = audit_log
        .iter()
        .filter(|e| e.user_id == record.user_id)
        .collect();
    let has_erasure = user_audit.iter().any(|e| e.action == "erase");

    let report = serde_json::json!({
        "user_data": serde_json::from_str::<serde_json::Value>(&user_data).unwrap_or_default(),
        "audit_entries": user_audit,
        "has_been_erased": has_erasure,
    });

    serde_json::to_string_pretty(&report).map_err(|e| format!("Report serialization failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user(uid: &str, name: &str, email: &str) -> UserRecord {
        let mut prefs = HashMap::new();
        prefs.insert("theme".to_string(), "dark".to_string());
        prefs.insert("lang".to_string(), "en".to_string());
        UserRecord {
            user_id: uid.to_string(),
            name: name.to_string(),
            email: email.to_string(),
            phone: "555-0100".to_string(),
            preferences: prefs,
            created_at: 1000,
        }
    }

    #[test]
    fn test_export_user_data_json() {
        let user = make_user("u1", "Alice", "alice@example.com");
        let json = export_user_data(&user).unwrap();
        assert!(json.contains("Alice"));
        assert!(json.contains("alice@example.com"));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["user_id"], "u1");
    }

    #[test]
    fn test_erase_user_data() {
        let user = make_user("u1", "Alice", "alice@example.com");
        let erased = erase_user_data(&user);
        assert_eq!(erased.user_id, "u1");
        assert_eq!(erased.name, "[ERASED]");
        assert_eq!(erased.email, "[ERASED]");
        assert_eq!(erased.phone, "[ERASED]");
        assert!(erased.preferences.is_empty());
    }

    #[test]
    fn test_propagate_erasure_success() {
        let results = propagate_erasure(&["primary_db", "cache", "search_index"], "u1");
        assert!(results.iter().all(|(_, success)| *success));
    }

    #[test]
    fn test_propagate_erasure_partial_failure() {
        let results = propagate_erasure(&["primary_db", "backup_locked_tape", "readonly_archive"], "u1");
        let successes: Vec<_> = results.iter().filter(|(_, s)| *s).collect();
        let failures: Vec<_> = results.iter().filter(|(_, s)| !*s).collect();
        assert_eq!(successes.len(), 1);
        assert_eq!(failures.len(), 2);
    }

    #[test]
    fn test_record_consent_granted() {
        let entry = record_consent("u1", true, "marketing_emails", 5000);
        assert_eq!(entry.user_id, "u1");
        assert_eq!(entry.action, "consent_granted");
        assert_eq!(entry.timestamp, 5000);
    }

    #[test]
    fn test_record_consent_withdrawn() {
        let entry = record_consent("u1", false, "analytics", 6000);
        assert_eq!(entry.action, "consent_withdrawn");
    }

    #[test]
    fn test_validate_erasure_ok() {
        let results = vec![
            ("primary".to_string(), true),
            ("cache".to_string(), true),
        ];
        assert!(validate_erasure(&results).is_ok());
    }

    #[test]
    fn test_validate_erasure_failure() {
        let results = vec![
            ("primary".to_string(), true),
            ("backup_locked".to_string(), false),
            ("readonly_archive".to_string(), false),
        ];
        let err = validate_erasure(&results).unwrap_err();
        assert_eq!(err.len(), 2);
        assert!(err.contains(&"backup_locked".to_string()));
    }

    #[test]
    fn test_compliance_report() {
        let user = make_user("u1", "Alice", "alice@example.com");
        let audit = vec![
            AuditEntry { user_id: "u1".into(), action: "collect".into(), timestamp: 100, details: "registration".into() },
            AuditEntry { user_id: "u2".into(), action: "collect".into(), timestamp: 200, details: "other user".into() },
        ];
        let report = compliance_report(&user, &audit).unwrap();
        assert!(report.contains("Alice"));
        assert!(!report.contains("u2"));
    }
}
