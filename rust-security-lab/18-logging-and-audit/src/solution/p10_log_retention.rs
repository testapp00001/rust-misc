//! # Lesson 10: Log Retention Policies (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLifecycle {
    Active,
    Archived,
    Expired,
    Deleted,
}

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

#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    pub default_retention_days: u32,
    pub archive_after_days: u32,
    pub delete_after_days: u32,
    pub audit_deletion: bool,
}

impl RetentionPolicy {
    pub fn new(
        default_retention_days: u32,
        archive_after_days: u32,
        delete_after_days: u32,
        audit_deletion: bool,
    ) -> Result<Self, String> {
        if default_retention_days == 0 || archive_after_days == 0 || delete_after_days == 0 {
            return Err("All retention values must be > 0".to_string());
        }
        if delete_after_days <= archive_after_days {
            return Err("delete_after_days must be > archive_after_days".to_string());
        }
        if default_retention_days > delete_after_days {
            return Err("default_retention_days should be <= delete_after_days".to_string());
        }

        Ok(RetentionPolicy {
            default_retention_days,
            archive_after_days,
            delete_after_days,
            audit_deletion,
        })
    }

    pub fn evaluate_lifecycle(&self, entry: &RetainedLogEntry) -> LogLifecycle {
        let age_days = (Utc::now() - entry.timestamp).num_days() as u32;

        if age_days >= self.delete_after_days {
            LogLifecycle::Expired
        } else if age_days >= self.archive_after_days {
            LogLifecycle::Archived
        } else {
            LogLifecycle::Active
        }
    }
}

fn compute_id(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8]) // First 16 hex chars
}

pub fn create_retained_entry(data: &str, retention_days: u32) -> RetainedLogEntry {
    RetainedLogEntry {
        id: compute_id(data),
        timestamp: Utc::now(),
        data: data.to_string(),
        lifecycle: LogLifecycle::Active,
        retention_days,
        archive_date: None,
        delete_date: None,
        deletion_hash: None,
    }
}

pub fn archive_entry(entry: &mut RetainedLogEntry) -> Result<(), String> {
    if entry.lifecycle != LogLifecycle::Active {
        return Err(format!(
            "Cannot archive entry in {:?} state, must be Active",
            entry.lifecycle
        ));
    }
    entry.lifecycle = LogLifecycle::Archived;
    entry.archive_date = Some(Utc::now());
    Ok(())
}

pub fn secure_delete(entry: &mut RetainedLogEntry) -> Result<String, String> {
    if entry.lifecycle != LogLifecycle::Expired {
        return Err(format!(
            "Cannot delete entry in {:?} state, must be Expired",
            entry.lifecycle
        ));
    }

    // Compute hash of data BEFORE clearing (for audit trail)
    let mut hasher = Sha256::new();
    hasher.update(entry.data.as_bytes());
    let hash = hex::encode(hasher.finalize());

    entry.deletion_hash = Some(hash.clone());
    entry.data = "[DELETED]".to_string();
    entry.lifecycle = LogLifecycle::Deleted;
    entry.delete_date = Some(Utc::now());

    Ok(hash)
}

pub fn process_retention_batch(
    entries: &mut [RetainedLogEntry],
    policy: &RetentionPolicy,
) -> (usize, usize) {
    let mut archived_count = 0;
    let mut expired_count = 0;

    for entry in entries.iter_mut() {
        let target_lifecycle = policy.evaluate_lifecycle(entry);

        match target_lifecycle {
            LogLifecycle::Archived if entry.lifecycle == LogLifecycle::Active => {
                if archive_entry(entry).is_ok() {
                    archived_count += 1;
                }
            }
            LogLifecycle::Expired if entry.lifecycle == LogLifecycle::Active
                || entry.lifecycle == LogLifecycle::Archived =>
            {
                // First archive if still active
                if entry.lifecycle == LogLifecycle::Active {
                    let _ = archive_entry(entry);
                }
                entry.lifecycle = LogLifecycle::Expired;
                expired_count += 1;
            }
            _ => {}
        }
    }

    (archived_count, expired_count)
}

pub fn retention_report(entries: &[RetainedLogEntry]) -> String {
    let mut active = 0;
    let mut archived = 0;
    let mut expired = 0;
    let mut deleted = 0;

    for entry in entries {
        match entry.lifecycle {
            LogLifecycle::Active => active += 1,
            LogLifecycle::Archived => archived += 1,
            LogLifecycle::Expired => expired += 1,
            LogLifecycle::Deleted => deleted += 1,
        }
    }

    format!(
        "Retention Report\n\
         ================\n\
         Total entries: {}\n\
         Active: {}\n\
         Archived: {}\n\
         Expired: {}\n\
         Deleted: {}",
        entries.len(),
        active,
        archived,
        expired,
        deleted
    )
}

pub fn check_retention_compliance(
    entries: &[RetainedLogEntry],
    policy: &RetentionPolicy,
) -> Vec<String> {
    let mut violations = Vec::new();

    for entry in entries {
        let age_days = (Utc::now() - entry.timestamp).num_days() as u32;

        // Active entry older than archive_after_days
        if entry.lifecycle == LogLifecycle::Active && age_days >= policy.archive_after_days {
            violations.push(format!(
                "{}: Active entry is {} days old (should be archived at {} days)",
                entry.id, age_days, policy.archive_after_days
            ));
        }

        // Archived entry older than delete_after_days
        if entry.lifecycle == LogLifecycle::Archived && age_days >= policy.delete_after_days {
            violations.push(format!(
                "{}: Archived entry is {} days old (should be expired at {} days)",
                entry.id, age_days, policy.delete_after_days
            ));
        }

        // Entry with data still present after delete_date
        if entry.lifecycle == LogLifecycle::Deleted && entry.data != "[DELETED]" {
            violations.push(format!(
                "{}: Deleted entry still contains data",
                entry.id
            ));
        }
    }

    violations
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
        assert_eq!(policy.evaluate_lifecycle(&entry), LogLifecycle::Active);

        entry.timestamp = Utc::now() - Duration::days(400);
        assert_eq!(policy.evaluate_lifecycle(&entry), LogLifecycle::Expired);
    }

    #[test]
    fn test_process_batch() {
        let policy = RetentionPolicy::new(365, 30, 365, true).unwrap();
        let mut entries = vec![
            create_retained_entry("data1", 365),
            create_retained_entry("data2", 365),
        ];
        let (archived, expired) = process_retention_batch(&mut entries, &policy);
        assert_eq!(archived, 0); // Entries are brand new
        assert_eq!(expired, 0);
    }

    #[test]
    fn test_compliance_check() {
        let policy = RetentionPolicy::new(365, 30, 365, true).unwrap();
        let mut entry = create_retained_entry("old data", 365);
        entry.timestamp = Utc::now() - Duration::days(60);
        // Entry is 60 days old, should be Archived but is still Active
        let violations = check_retention_compliance(&[entry], &policy);
        assert!(!violations.is_empty());
    }
}
