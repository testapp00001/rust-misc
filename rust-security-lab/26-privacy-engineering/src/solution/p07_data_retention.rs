//! # Lesson 07: Data Retention Policies (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A data record with retention metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetainedRecord {
    pub id: String,
    pub category: String,
    pub data: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,
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

/// Check if a record has expired.
pub fn is_record_expired(
    record: &RetainedRecord,
    policies: &HashMap<String, RetentionPolicy>,
    current_time: u64,
) -> bool {
    if let Some(expires_at) = record.expires_at {
        return current_time >= expires_at;
    }
    if let Some(policy) = policies.get(&record.category) {
        let age = current_time.saturating_sub(record.created_at);
        return age > policy.max_age_seconds;
    }
    // No policy and no explicit expiry -- treat as not expired
    false
}

/// Find indices of all expired records.
pub fn find_expired_records(
    records: &[RetainedRecord],
    policies: &HashMap<String, RetentionPolicy>,
    current_time: u64,
) -> Vec<usize> {
    records
        .iter()
        .enumerate()
        .filter(|(_, r)| is_record_expired(r, policies, current_time))
        .map(|(i, _)| i)
        .collect()
}

/// Remove expired records and return survivors + deleted IDs.
pub fn enforce_retention(
    records: Vec<RetainedRecord>,
    policies: &HashMap<String, RetentionPolicy>,
    current_time: u64,
) -> (Vec<RetainedRecord>, Vec<String>) {
    let mut survivors = Vec::new();
    let mut deleted = Vec::new();

    for record in records {
        if is_record_expired(&record, policies, current_time) {
            deleted.push(record.id.clone());
        } else {
            survivors.push(record);
        }
    }

    (survivors, deleted)
}

/// Generate retention compliance report by category.
pub fn retention_report(
    records: &[RetainedRecord],
    policies: &HashMap<String, RetentionPolicy>,
    current_time: u64,
) -> HashMap<String, (usize, usize, u64)> {
    let mut report: HashMap<String, (usize, usize, u64)> = HashMap::new();

    for record in records {
        let entry = report.entry(record.category.clone()).or_insert((0, 0, 0));
        entry.0 += 1; // total
        if is_record_expired(record, policies, current_time) {
            entry.1 += 1; // expired
        }
        let age = current_time.saturating_sub(record.created_at);
        if age > entry.2 {
            entry.2 = age; // oldest
        }
    }

    report
}

/// Validate a retention policy.
pub fn validate_policy(policy: &RetentionPolicy) -> Result<(), String> {
    if policy.category.is_empty() {
        return Err("Category cannot be empty".to_string());
    }
    if policy.max_age_seconds == 0 {
        return Err("Max age must be greater than 0".to_string());
    }
    let ten_years_seconds: u64 = 365 * 10 * 86400;
    if policy.max_age_seconds > ten_years_seconds {
        return Err(format!(
            "Max age {} seconds exceeds 10-year limit",
            policy.max_age_seconds
        ));
    }
    Ok(())
}

/// Compute time until expiry for each record.
pub fn time_until_expiry(
    records: &[RetainedRecord],
    policies: &HashMap<String, RetentionPolicy>,
    current_time: u64,
) -> Vec<(String, Option<u64>)> {
    records
        .iter()
        .map(|record| {
            let seconds = if let Some(expires_at) = record.expires_at {
                Some(expires_at.saturating_sub(current_time))
            } else if let Some(policy) = policies.get(&record.category) {
                let age = current_time.saturating_sub(record.created_at);
                Some(policy.max_age_seconds.saturating_sub(age))
            } else {
                None
            };
            (record.id.clone(), seconds)
        })
        .collect()
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
        assert!(!is_record_expired(&record, &policies, 1000 + 2592000 - 1));
        assert!(is_record_expired(&record, &policies, 1000 + 2592000 + 1));
    }

    #[test]
    fn test_is_expired_no_policy() {
        let policies = HashMap::new();
        let record = make_record("r1", "unknown", 1000, None);
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
        assert_eq!(expired, vec![0]);
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
        assert_eq!(*expired, 1);
        assert_eq!(*oldest, 3000 - 500);
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
            max_age_seconds: 86400 * 365 * 15,
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
        assert!(times[1].1.is_some());
    }
}
