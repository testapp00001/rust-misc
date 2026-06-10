//! # Lesson 03: Data Minimization -- Collect Only What You Need (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// A data record with all possible fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub fields: HashMap<String, String>,
    pub purpose: String,
    pub collected_at: u64,
}

/// Filter a record to only include allowed fields.
pub fn filter_fields(record: &DataRecord, allowed_fields: &[&str]) -> DataRecord {
    let allowed: HashSet<&str> = allowed_fields.iter().copied().collect();
    let fields: HashMap<String, String> = record
        .fields
        .iter()
        .filter(|(k, _)| allowed.contains(k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    DataRecord {
        fields,
        purpose: record.purpose.clone(),
        collected_at: record.collected_at,
    }
}

/// Find fields that exceed what's needed for the purpose.
pub fn find_excess_fields<'a>(
    record: &'a DataRecord,
    purpose_fields: &HashMap<String, HashSet<String>>,
) -> Vec<&'a str> {
    match purpose_fields.get(&record.purpose) {
        Some(allowed) => record
            .fields
            .keys()
            .filter(|k| !allowed.contains(k.as_str()))
            .map(|k| k.as_str())
            .collect(),
        None => record.fields.keys().map(|k| k.as_str()).collect(),
    }
}

/// Redact fields not needed for the purpose.
pub fn redact_excess(record: &DataRecord, allowed_fields: &[&str]) -> DataRecord {
    let allowed: HashSet<&str> = allowed_fields.iter().copied().collect();
    let fields: HashMap<String, String> = record
        .fields
        .iter()
        .map(|(k, v)| {
            if allowed.contains(k.as_str()) {
                (k.clone(), v.clone())
            } else {
                (k.clone(), "[REDACTED]".to_string())
            }
        })
        .collect();
    DataRecord {
        fields,
        purpose: record.purpose.clone(),
        collected_at: record.collected_at,
    }
}

/// Check if data has exceeded its retention period.
pub fn is_expired(collected_at: u64, current_time: u64, max_age_seconds: u64) -> bool {
    current_time.saturating_sub(collected_at) > max_age_seconds
}

/// Apply data minimization to a batch: filter fields and remove expired.
pub fn minimize_batch(
    records: &[DataRecord],
    allowed_fields: &[&str],
    current_time: u64,
    max_age_seconds: u64,
) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| !is_expired(r.collected_at, current_time, max_age_seconds))
        .map(|r| filter_fields(r, allowed_fields))
        .collect()
}

/// Generate audit report of data collection.
pub fn audit_collection(records: &[DataRecord]) -> (HashMap<String, usize>, HashSet<String>) {
    let mut purpose_counts: HashMap<String, usize> = HashMap::new();
    let mut all_fields: HashSet<String> = HashSet::new();

    for record in records {
        *purpose_counts.entry(record.purpose.clone()).or_insert(0) += 1;
        for key in record.fields.keys() {
            all_fields.insert(key.clone());
        }
    }

    (purpose_counts, all_fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(purpose: &str, fields: Vec<(&str, &str)>, ts: u64) -> DataRecord {
        DataRecord {
            fields: fields.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            purpose: purpose.to_string(),
            collected_at: ts,
        }
    }

    #[test]
    fn test_filter_fields() {
        let record = make_record("weather", vec![
            ("location", "NYC"), ("temp", "72"), ("contacts", "alice,bob"),
        ], 1000);
        let filtered = filter_fields(&record, &["location", "temp"]);
        assert!(filtered.fields.contains_key("location"));
        assert!(filtered.fields.contains_key("temp"));
        assert!(!filtered.fields.contains_key("contacts"));
        assert_eq!(filtered.purpose, "weather");
    }

    #[test]
    fn test_filter_fields_preserves_all_when_allowed() {
        let record = make_record("weather", vec![
            ("location", "NYC"), ("temp", "72"),
        ], 1000);
        let filtered = filter_fields(&record, &["location", "temp"]);
        assert_eq!(filtered.fields.len(), 2);
    }

    #[test]
    fn test_find_excess_fields() {
        let record = make_record("weather", vec![
            ("location", "NYC"), ("temp", "72"), ("contacts", "alice"), ("photos", "img.jpg"),
        ], 1000);
        let mut purpose_fields = HashMap::new();
        purpose_fields.insert(
            "weather".to_string(),
            vec!["location".to_string(), "temp".to_string()].into_iter().collect(),
        );
        let excess = find_excess_fields(&record, &purpose_fields);
        assert!(excess.contains(&"contacts"));
        assert!(excess.contains(&"photos"));
        assert!(!excess.contains(&"location"));
    }

    #[test]
    fn test_find_excess_unknown_purpose() {
        let record = make_record("unknown_purpose", vec![("data", "val")], 1000);
        let purpose_fields = HashMap::new();
        let excess = find_excess_fields(&record, &purpose_fields);
        assert_eq!(excess.len(), 1);
    }

    #[test]
    fn test_redact_excess() {
        let record = make_record("weather", vec![
            ("location", "NYC"), ("temp", "72"), ("ssn", "123-45-6789"),
        ], 1000);
        let redacted = redact_excess(&record, &["location", "temp"]);
        assert_eq!(redacted.fields.get("location").unwrap(), "NYC");
        assert_eq!(redacted.fields.get("temp").unwrap(), "72");
        assert_eq!(redacted.fields.get("ssn").unwrap(), "[REDACTED]");
    }

    #[test]
    fn test_is_expired() {
        assert!(!is_expired(1000, 2000, 5000));
        assert!(is_expired(1000, 7000, 5000));
        assert!(!is_expired(1000, 6000, 5000));
        assert!(is_expired(1000, 6001, 5000));
    }

    #[test]
    fn test_minimize_batch_correct() {
        let records = vec![
            make_record("weather", vec![("location", "NYC"), ("contacts", "alice")], 1000),
            make_record("weather", vec![("location", "LA"), ("ssn", "123")], 3000),
        ];
        let result = minimize_batch(&records, &["location"], 4000, 5000);
        assert_eq!(result.len(), 2, "Both should survive (within retention)");
        for r in &result {
            assert!(r.fields.contains_key("location"));
            assert!(!r.fields.contains_key("contacts") || r.fields.get("contacts").map(|v| v.as_str()) != Some("alice"));
        }
    }

    #[test]
    fn test_audit_collection() {
        let records = vec![
            make_record("weather", vec![("location", "NYC"), ("temp", "72")], 1000),
            make_record("analytics", vec![("location", "LA"), ("clicks", "5")], 2000),
            make_record("weather", vec![("location", "CHI"), ("humidity", "80")], 3000),
        ];
        let (counts, fields) = audit_collection(&records);
        assert_eq!(counts.get("weather").unwrap(), &2);
        assert_eq!(counts.get("analytics").unwrap(), &1);
        assert!(fields.contains("location"));
        assert!(fields.contains("temp"));
        assert!(fields.contains("clicks"));
        assert!(fields.contains("humidity"));
        assert_eq!(fields.len(), 4);
    }
}
