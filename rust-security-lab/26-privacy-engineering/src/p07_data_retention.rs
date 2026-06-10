//! # Lesson 07: Data Retention Policies -- Automated Deletion and Compliance
//!
//! ## What is Data Retention?
//!
//! Data retention defines how long personal data is kept before deletion.
//! GDPR does not specify exact periods but requires that data be kept only
//! as long as necessary for the stated purpose.
//!
//! ## Why It Matters
//!
//! - **Legal compliance**: GDPR Art. 5(1)(e) -- storage limitation
//! - **Risk reduction**: Less stored data = smaller breach impact
//! - **User trust**: Demonstrates responsible data stewardship
//!
//! ## Retention Periods (Industry Guidelines)
//!
//! | Data Type | Typical Retention |
//! |-----------|------------------|
//! | Session logs | 30 days |
//! | Transaction records | 7 years (financial regulation) |
//! | Marketing consent | Until withdrawal |
//! | Account data | Until account deletion + 30 days |
//! | Analytics data | 26 months (Google Analytics default) |
//!
//! ## Attack: Zombie Data
//!
//! Data past its retention period that was never deleted. It sits in backups,
//! analytics databases, and log files. A breach exposes this "dead" data that
//! should have been destroyed.
//!
//! ## Best Practices
//!
//! 1. Tag every data record with its category and collection timestamp
//! 2. Define retention policies per category
//! 3. Run automated deletion jobs on a schedule
//! 4. Log all deletions for audit
//! 5. Verify deletion propagated to all storage systems

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A data record with retention metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetainedRecord {
    pub id: String,
    pub category: String,
    pub data: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,  // None = use category policy
}

/// A retention policy for a data category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub category: String,
    pub max_age_seconds: u64,
    pub description: String,
}

/// Result of a retention enforcement run.
#[derive(Debug, Clone)]
pub struct RetentionReport {
    pub total_records: usize,
    pub expired_records: usize,
    pub deleted_ids: Vec<String>,
    pub errors: Vec<String>,
}

/// Exercise 1: Check if a record has expired.
///
/// A record is expired if:
/// - `expires_at` is Some and current_time >= expires_at, OR
/// - `expires_at` is None and current_time - created_at > policy's max_age_seconds
///
/// Hints:
/// - Check expires_at first
/// - If None, look up the category policy and compute age
pub fn is_record_expired(
    record: &RetainedRecord,
    policies: &HashMap<String, RetentionPolicy>,
    current_time: u64,
) -> bool {
    todo!("Check if a record has exceeded its retention period")
}

/// Exercise 2: Find all expired records in a collection.
///
/// Return the indices of expired records.
///
/// Hints:
/// - Iterate with enumerate()
/// - Use is_record_expired for each
pub fn find_expired_records(
    records: &[RetainedRecord],
    policies: &HashMap<String, RetentionPolicy>,
    current_time: u64,
) -> Vec<usize> {
    todo!("Find indices of all expired records")
}

/// Exercise 3: Enforce retention -- remove expired records.
///
/// Return (surviving_records, deleted_ids).
///
/// Hints:
/// - Partition records into expired and non-expired
/// - Collect IDs of expired records
pub fn enforce_retention(
    records: Vec<RetainedRecord>,
    policies: &HashMap<String, RetentionPolicy>,
    current_time: u64,
) -> (Vec<RetainedRecord>, Vec<String>) {
    todo!("Remove expired records and return survivors + deleted IDs")
}

/// Exercise 4: Generate a retention compliance report.
///
/// Count records per category, how many are expired, and the oldest record age.
///
/// Returns a HashMap of category -> (total, expired, oldest_age_seconds).
pub fn retention_report(
    records: &[RetainedRecord],
    policies: &HashMap<String, RetentionPolicy>,
    current_time: u64,
) -> HashMap<String, (usize, usize, u64)> {
    todo!("Generate retention compliance report by category")
}

/// Exercise 5: Validate that a retention policy is sensible.
///
/// Rules:
/// - max_age_seconds > 0
/// - max_age_seconds <= 10 years (365 * 10 * 86400 seconds)
/// - category is not empty
///
/// Returns Ok(()) or Err with description of the issue.
pub fn validate_policy(policy: &RetentionPolicy) -> Result<(), String> {
    todo!("Validate a retention policy")
}

/// Exercise 6: Compute how long until each record expires.
///
/// For each record, compute seconds until expiry. If already expired, return 0.
/// If no policy exists for the category, return None.
///
/// Returns Vec of (record_id, Option<seconds_remaining>).
pub fn time_until_expiry(
    records: &[RetainedRecord],
    policies: &HashMap<String, RetentionPolicy>,
    current_time: u64,
) -> Vec<(String, Option<u64>)> {
    todo!("Compute time until expiry for each record")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(id: &str, category: &str, created_at: u64, expires_at: Option<u64>) -> RetainedRecord {
        RetainedRecord {
            id: id.to_string(),
            category: category.to_string(),
            data: "test data".to_string(),
            created_at,
            expires_at,
        }
    }

    fn make_policy(category: &str, max_age: u64) -> (String, RetentionPolicy) {
        (
            category.to_string(),
            RetentionPolicy {
                category: category.to_string(),
                max_age_seconds: max_age,
                description: format!("{} retention", category),
            },
        )
    }

    fn default_policies() -> HashMap<String, RetentionPolicy> {
        vec![
            make_policy("logs", 30 * 86400),
            make_policy("accounts", 365 * 86400),
        ].into_iter().collect()
    }

    #[test]
    fn test_is_expired_with_expires_at() {
        let record = make_record("r1", "logs", 1000, Some(5000));
        assert!(!is_record_expired(&record, &default_policies(), 4000));
        assert!(is_record_expired(&record, &default_policies(), 5000));
        assert!(is_record_expired(&record, &default_policies(), 6000));
    }

    #[test]
    fn test_is_expired_with_policy() {
        let policies = default_policies();
        let record = make_record("r1", "logs", 1000, None);
        // logs max_age = 30 * 86400 = 2592000
        assert!(!is_record_expired(&record, &policies, 1000 + 2592000 - 1));
        assert!(is_record_expired(&record, &policies, 1000 + 2592000 + 1));
    }

    #[test]
    fn test_is_expired_no_policy() {
        let policies = HashMap::new();
        let record = make_record("r1", "unknown", 1000, None);
        // No policy -> should handle gracefully (not panic)
        let _ = is_record_expired(&record, &policies, 99999);
    }

    #[test]
    fn test_find_expired_records() {
        let policies = default_policies();
        let records = vec![
            make_record("r1", "logs", 1000, Some(2000)),
            make_record("r2", "logs", 1000, Some(5000)),
            make_record("r3", "accounts", 1000, None),
        ];
        let expired = find_expired_records(&records, &policies, 3000);
        assert_eq!(expired, vec![0]); // only r1 is expired
    }

    #[test]
    fn test_enforce_retention() {
        let policies = default_policies();
        let records = vec![
            make_record("r1", "logs", 1000, Some(2000)),
            make_record("r2", "logs", 1000, Some(5000)),
            make_record("r3", "accounts", 1000, None),
        ];
        let (surviving, deleted) = enforce_retention(records, &policies, 3000);
        assert_eq!(surviving.len(), 2);
        assert_eq!(deleted, vec!["r1"]);
    }

    #[test]
    fn test_retention_report() {
        let policies = default_policies();
        let records = vec![
            make_record("r1", "logs", 1000, Some(2000)),
            make_record("r2", "logs", 1000, Some(5000)),
            make_record("r3", "logs", 500, None),
        ];
        let report = retention_report(&records, &policies, 3000);
        let (total, expired, oldest) = report.get("logs").unwrap();
        assert_eq!(*total, 3);
        assert_eq!(*expired, 1); // r1 expired
        assert_eq!(*oldest, 3000 - 500); // r3 is oldest
    }

    #[test]
    fn test_validate_policy_ok() {
        let policy = RetentionPolicy {
            category: "logs".into(),
            max_age_seconds: 86400 * 30,
            description: "30 day logs".into(),
        };
        assert!(validate_policy(&policy).is_ok());
    }

    #[test]
    fn test_validate_policy_too_long() {
        let policy = RetentionPolicy {
            category: "logs".into(),
            max_age_seconds: 86400 * 365 * 15, // 15 years
            description: "too long".into(),
        };
        assert!(validate_policy(&policy).is_err());
    }

    #[test]
    fn test_validate_policy_zero() {
        let policy = RetentionPolicy {
            category: "logs".into(),
            max_age_seconds: 0,
            description: "zero".into(),
        };
        assert!(validate_policy(&policy).is_err());
    }

    #[test]
    fn test_validate_policy_empty_category() {
        let policy = RetentionPolicy {
            category: "".into(),
            max_age_seconds: 86400,
            description: "empty".into(),
        };
        assert!(validate_policy(&policy).is_err());
    }

    #[test]
    fn test_time_until_expiry() {
        let policies = default_policies();
        let records = vec![
            make_record("r1", "logs", 1000, Some(5000)),
            make_record("r2", "logs", 1000, None),
        ];
        let times = time_until_expiry(&records, &policies, 2000);
        assert_eq!(times[0], ("r1".to_string(), Some(3000)));
        // r2 has policy-based expiry
        assert!(times[1].1.is_some());
    }
}
