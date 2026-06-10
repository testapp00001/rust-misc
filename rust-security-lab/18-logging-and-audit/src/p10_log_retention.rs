//! # Lesson 10: Log Retention Policies
//!
//! ## The Problem
//!
//! Logs can't be kept forever -- storage costs money, and keeping data beyond its
//! legal retention period creates liability. But deleting logs too early violates
//! compliance requirements and destroys evidence. You need a policy that balances
//! these concerns.
//!
//! ## Retention Policy Considerations
//!
//! | Factor | Keep Shorter | Keep Longer |
//! |--------|-------------|-------------|
//! | Data sensitivity | Highly sensitive | Public/internal |
//! | Legal requirements | No mandate | GDPR, HIPAA, SOX |
//! | Incident investigation | Routine | Under investigation |
//! | Storage cost | High volume | Low volume |
//! | Age | Very old | Recent |
//!
//! ## Secure Deletion
//!
//! Simply calling `delete` on a file doesn't actually remove the data from disk.
//! For compliance, you may need:
//! - **Cryptographic erasure**: Delete the encryption key (data becomes unreadable)
//! - **Secure overwrite**: Write random data over the file before deletion
//! - **Audit the deletion**: Log that data was deleted and why
//!
//! ## What You'll Implement
//!
//! 1. A retention policy engine
//! 2. Log entry lifecycle (active -> archived -> expired -> deleted)
//! 3. Secure deletion with audit trail
//! 4. Retention policy evaluation
//! 5. Archive before delete
//! 6. Retention compliance checker

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Lifecycle states for log entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLifecycle {
    /// Currently active, being written to
    Active,
    /// Archived (read-only, compressed)
    Archived,
    /// Past retention period, pending deletion
    Expired,
    /// Securely deleted
    Deleted,
}

/// A log entry with retention metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetainedLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub data: String,
    pub lifecycle: LogLifecycle,
    pub retention_days: u32,
    pub archive_date: Option<DateTime<Utc>>,
    pub delete_date: Option<DateTime<Utc>>,
    pub deletion_hash: Option<String>,
}

/// Retention policy rules.
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// Default retention in days
    pub default_retention_days: u32,
    /// Archive after this many days
    pub archive_after_days: u32,
    /// Delete after this many days (must be > archive_after_days)
    pub delete_after_days: u32,
    /// Whether to compute deletion hash for audit
    pub audit_deletion: bool,
}

impl RetentionPolicy {
    /// Exercise 1: Create a retention policy with validation.
    ///
    /// Rules:
    /// - delete_after_days must be > archive_after_days
    /// - all values must be > 0
    /// - default_retention_days should be <= delete_after_days
    ///
    /// Return Ok(policy) or Err(description).
    pub fn new(
        default_retention_days: u32,
        archive_after_days: u32,
        delete_after_days: u32,
        audit_deletion: bool,
    ) -> Result<Self, String> {
        todo!("Implement policy creation with validation")
    }

    /// Exercise 2: Evaluate the current lifecycle state for an entry.
    ///
    /// Based on the entry's timestamp and the policy:
    /// - If age < archive_after_days: Active
    /// - If archive_after_days <= age < delete_after_days: Archived
    /// - If age >= delete_after_days: Expired
    ///
    /// Hints:
    /// - Calculate age: `(Utc::now() - entry.timestamp).num_days()`
    pub fn evaluate_lifecycle(&self, entry: &RetainedLogEntry) -> LogLifecycle {
        todo!("Implement lifecycle evaluation")
    }
}

/// Exercise 3: Create a retained log entry.
///
/// Set lifecycle to Active, compute an ID from the data hash.
///
/// Hints:
/// - Hash: SHA256 of data, take first 16 hex chars
pub fn create_retained_entry(data: &str, retention_days: u32) -> RetainedLogEntry {
    todo!("Implement retained entry creation")
}

/// Exercise 4: Archive an entry.
///
/// Update lifecycle to Archived, set archive_date to now.
/// Only works if entry is currently Active.
///
/// Return Ok(()) or Err if entry is not Active.
pub fn archive_entry(entry: &mut RetainedLogEntry) -> Result<(), String> {
    todo!("Implement entry archival")
}

/// Exercise 5: Securely delete an entry.
///
/// Steps:
/// 1. Verify entry is Expired (not Active or Archived)
/// 2. Compute deletion_hash: SHA256 of the data (for audit)
/// 3. Clear the data field (replace with "[DELETED]")
/// 4. Set lifecycle to Deleted
/// 5. Set delete_date to now
///
/// Return Ok(deletion_hash) or Err if entry is not Expired.
///
/// Hints:
/// - Check lifecycle == Expired
/// - Hash the data BEFORE clearing it
pub fn secure_delete(entry: &mut RetainedLogEntry) -> Result<String, String> {
    todo!("Implement secure deletion")
}

/// Exercise 6: Process a batch of entries according to the retention policy.
///
/// For each entry:
/// - Evaluate its lifecycle state
/// - Archive if it should be archived
/// - Mark as expired if past retention
///
/// Return a summary: (archived_count, expired_count).
pub fn process_retention_batch(
    entries: &mut [RetainedLogEntry],
    policy: &RetentionPolicy,
) -> (usize, usize) {
    todo!("Implement batch retention processing")
}

/// Exercise 7: Generate a retention compliance report.
///
/// Count entries by lifecycle state and return a summary string.
pub fn retention_report(entries: &[RetainedLogEntry]) -> String {
    todo!("Implement retention report")
}

/// Exercise 8: Check if any entries violate the retention policy.
///
/// Violations:
/// - Active entry older than archive_after_days
/// - Archived entry older than delete_after_days
/// - Any entry with data still present after delete_date
///
/// Return list of entry IDs that are in violation.
pub fn check_retention_compliance(
    entries: &[RetainedLogEntry],
    policy: &RetentionPolicy,
) -> Vec<String> {
    todo!("Implement retention compliance check")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_creation_valid() {
        let policy = RetentionPolicy::new(365, 90, 365, true);
        assert!(policy.is_ok());
    }

    #[test]
    fn test_policy_creation_invalid() {
        let policy = RetentionPolicy::new(365, 400, 365, true);
        assert!(policy.is_err());
    }

    #[test]
    fn test_create_retained_entry() {
        let entry = create_retained_entry("test log data", 365);
        assert_eq!(entry.lifecycle, LogLifecycle::Active);
        assert_eq!(entry.retention_days, 365);
        assert!(!entry.id.is_empty());
    }

    #[test]
    fn test_archive_entry() {
        let mut entry = create_retained_entry("test data", 365);
        assert!(archive_entry(&mut entry).is_ok());
        assert_eq!(entry.lifecycle, LogLifecycle::Archived);
        assert!(entry.archive_date.is_some());
    }

    #[test]
    fn test_archive_already_archived() {
        let mut entry = create_retained_entry("test data", 365);
        archive_entry(&mut entry).unwrap();
        assert!(archive_entry(&mut entry).is_err());
    }

    #[test]
    fn test_secure_delete() {
        let mut entry = create_retained_entry("sensitive data", 365);
        // Force to expired state
        entry.lifecycle = LogLifecycle::Expired;
        let hash = secure_delete(&mut entry).unwrap();
        assert_eq!(entry.lifecycle, LogLifecycle::Deleted);
        assert_eq!(entry.data, "[DELETED]");
        assert!(entry.delete_date.is_some());
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_secure_delete_not_expired() {
        let mut entry = create_retained_entry("data", 365);
        assert!(secure_delete(&mut entry).is_err());
    }

    #[test]
    fn test_retention_report() {
        let entries = vec![
            create_retained_entry("active", 365),
            {
                let mut e = create_retained_entry("archived", 365);
                e.lifecycle = LogLifecycle::Archived;
                e
            },
        ];
        let report = retention_report(&entries);
        assert!(report.contains("Active: 1"));
        assert!(report.contains("Archived: 1"));
    }

    #[test]
    fn test_lifecycle_evaluation() {
        let policy = RetentionPolicy::new(365, 30, 365, true).unwrap();
        let mut entry = create_retained_entry("test", 365);
        // Entry just created, should be Active
        assert_eq!(policy.evaluate_lifecycle(&entry), LogLifecycle::Active);

        // Simulate old entry
        entry.timestamp = Utc::now() - Duration::days(400);
        assert_eq!(policy.evaluate_lifecycle(&entry), LogLifecycle::Expired);
    }

    #[test]
    fn test_process_batch() {
        let policy = RetentionPolicy::new(365, 0, 0, true).unwrap();
        let mut entries = vec![
            create_retained_entry("data1", 365),
            create_retained_entry("data2", 365),
        ];
        // All entries are brand new, should stay Active
        let (archived, expired) = process_retention_batch(&mut entries, &policy);
        // With archive_after_days=0 and delete_after_days=0, all should be expired
        // but the entries are fresh so they should be processed
        assert!(archived + expired <= 2);
    }
}
