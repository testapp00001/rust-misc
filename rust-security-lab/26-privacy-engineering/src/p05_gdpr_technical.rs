//! # Lesson 05: GDPR Technical Implementation -- Rights and Compliance
//!
//! ## What is GDPR?
//!
//! The General Data Protection Regulation (EU 2016/679) grants individuals rights
//! over their personal data. This lesson implements the technical mechanisms needed
//! to comply with the most impactful articles.
//!
//! ## Key Technical Requirements
//!
//! | GDPR Article | Right | Technical Implementation |
//! |-------------|-------|------------------------|
//! | Art. 17 | Right to erasure | Delete all user data, propagate to backups |
//! | Art. 20 | Data portability | Export in machine-readable format (JSON) |
//! | Art. 7 | Consent | Record consent with timestamp, allow withdrawal |
//! | Art. 30 | Records of processing | Audit log of all data operations |
//! | Art. 33 | Breach notification | Detect and report within 72 hours |
//!
//! ## Right to Erasure (Art. 17)
//!
//! Users can request deletion of their personal data. You must:
//! 1. Delete the data from primary storage
//! 2. Propagate deletion to backups (within reasonable timeframe)
//! 3. Inform third parties who received the data
//! 4. Confirm deletion to the user
//!
//! ## Data Portability (Art. 20)
//!
//! Users can request their data in a structured, machine-readable format.
//! JSON is the standard format. The export must include all personal data.
//!
//! ## Attack: Incomplete Erasure
//!
//! If erasure doesn't propagate to all systems (backups, analytics, caches),
//! the user's data persists and can be exposed in a future breach. A proper
//! erasure must be tracked and verified.

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
    pub action: String,     // "collect", "export", "erase", "consent_withdraw"
    pub timestamp: u64,
    pub details: String,
}

/// Exercise 1: Export user data as JSON (Data Portability - Art. 20).
///
/// Serialize the UserRecord to a JSON string.
///
/// Hints:
/// - Use `serde_json::to_string_pretty(&record)`
/// - Return the JSON string or an error message
pub fn export_user_data(record: &UserRecord) -> Result<String, String> {
    todo!("Export user data as JSON for portability")
}

/// Exercise 2: Erase all personal fields from a user record (Right to Erasure - Art. 17).
///
/// Replace all personal data fields with "[ERASED]" while keeping the user_id
/// for audit purposes. The preferences map should be cleared.
///
/// Hints:
/// - Clone the record
/// - Set name, email, phone to "[ERASED]"
/// - Clear the preferences HashMap
pub fn erase_user_data(record: &UserRecord) -> UserRecord {
    todo!("Erase personal data from user record")
}

/// Exercise 3: Propagate erasure across multiple storage systems.
///
/// Given a list of "systems" (as strings) and a user_id, return a list of
/// (system, success) tuples indicating whether erasure succeeded on each system.
///
/// For this exercise, simulate: erasure "succeeds" on systems that do NOT
/// contain the word "readonly" or "backup_locked" in their name.
///
/// Hints:
/// - For each system, check if it contains "readonly" or "backup_locked"
/// - If yes, (system, false); otherwise (system, true)
pub fn propagate_erasure(systems: &[&str], user_id: &str) -> Vec<(String, bool)> {
    todo!("Propagate erasure across storage systems")
}

/// Exercise 4: Record a consent decision.
///
/// Create an AuditEntry for a consent action.
///
/// Hints:
/// - action is "consent_granted" or "consent_withdrawn"
/// - details should describe what was consented to
pub fn record_consent(
    user_id: &str,
    granted: bool,
    purpose: &str,
    timestamp: u64,
) -> AuditEntry {
    todo!("Create a consent audit entry")
}

/// Exercise 5: Validate that all required erasure targets were successful.
///
/// Given the results from `propagate_erasure`, return Ok if all succeeded,
/// or Err with the list of systems that failed.
///
/// Hints:
/// - Filter for systems where success == false
/// - If any failures, return Err with their names
/// - Otherwise, return Ok(())
pub fn validate_erasure(results: &[(String, bool)]) -> Result<(), Vec<String>> {
    todo!("Validate that all erasure operations succeeded")
}

/// Exercise 6: Generate a GDPR compliance report for a user.
///
/// Given a user record and their audit trail, produce a JSON report containing:
/// - The user's data (from export_user_data)
/// - All audit entries for this user
/// - Whether the user's data has been erased
///
/// Hints:
/// - Filter audit entries by user_id
/// - Check if any entry has action "erase"
/// - Serialize the combined report to JSON
pub fn compliance_report(
    record: &UserRecord,
    audit_log: &[AuditEntry],
) -> Result<String, String> {
    todo!("Generate GDPR compliance report")
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
        // Must be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["user_id"], "u1");
    }

    #[test]
    fn test_erase_user_data() {
        let user = make_user("u1", "Alice", "alice@example.com");
        let erased = erase_user_data(&user);
        assert_eq!(erased.user_id, "u1", "user_id should be preserved");
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
        assert!(!report.contains("u2"), "Should not include other users' audit entries");
    }
}
