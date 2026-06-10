//! # Lesson 04: Data Anonymization -- Removing PII and Pseudonymization (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// A raw data record with PII fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRecord {
    pub name: String,
    pub email: String,
    pub ssn: String,
    pub age: String,
    pub zip: String,
    pub data: HashMap<String, String>,
}

/// An anonymized record (PII removed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedRecord {
    pub pseudonym: String,
    pub age: String,
    pub zip: String,
    pub data: HashMap<String, String>,
}

/// Strip direct identifiers and generate pseudonym from email+salt.
pub fn anonymize_record(record: &RawRecord, salt: &str) -> AnonymizedRecord {
    let input = format!("{}:{}", salt, record.email);
    let hash = Sha256::digest(input.as_bytes());
    let pseudonym = hex::encode(&hash)[..16].to_string();

    AnonymizedRecord {
        pseudonym,
        age: record.age.clone(),
        zip: record.zip.clone(),
        data: record.data.clone(),
    }
}

/// Mask a sensitive value, showing only first N characters.
pub fn mask_value(value: &str, show: usize) -> String {
    if value.len() <= show {
        return value.to_string();
    }
    let visible: String = value.chars().take(show).collect();
    let masked_len = value.len() - show;
    format!("{}{}", visible, "*".repeat(masked_len))
}

/// Mask SSN to show only last 4 digits.
pub fn mask_ssn(ssn: &str) -> String {
    if ssn.len() < 4 {
        return "***".to_string();
    }
    let last4: String = ssn.chars().rev().take(4).collect::<String>().chars().rev().collect();
    format!("***-**-{}", last4)
}

/// Compute re-identification risk based on quasi-identifiers.
pub fn reidentification_risk(records: &[AnonymizedRecord]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let mut qi_counts: HashMap<(&str, &str), usize> = HashMap::new();
    for record in records {
        *qi_counts.entry((record.age.as_str(), record.zip.as_str())).or_insert(0) += 1;
    }

    let unique_count = qi_counts.values().filter(|&&c| c == 1).count();
    unique_count as f64 / records.len() as f64
}

/// Suppress records with groups smaller than min_group_size.
pub fn suppress_rare(records: &[AnonymizedRecord], min_group_size: usize) -> Vec<AnonymizedRecord> {
    let mut qi_counts: HashMap<(&str, &str), usize> = HashMap::new();
    for record in records {
        *qi_counts.entry((record.age.as_str(), record.zip.as_str())).or_insert(0) += 1;
    }

    records
        .iter()
        .filter(|r| {
            let key = (r.age.as_str(), r.zip.as_str());
            qi_counts.get(&key).copied().unwrap_or(0) >= min_group_size
        })
        .cloned()
        .collect()
}

/// Batch anonymize and suppress risky records.
pub fn batch_anonymize(
    records: &[RawRecord],
    salt: &str,
    min_group_size: usize,
) -> Vec<AnonymizedRecord> {
    let anonymized: Vec<AnonymizedRecord> = records
        .iter()
        .map(|r| anonymize_record(r, salt))
        .collect();
    suppress_rare(&anonymized, min_group_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_raw(name: &str, email: &str, ssn: &str, age: &str, zip: &str) -> RawRecord {
        RawRecord {
            name: name.to_string(),
            email: email.to_string(),
            ssn: ssn.to_string(),
            age: age.to_string(),
            zip: zip.to_string(),
            data: HashMap::new(),
        }
    }

    fn make_anon(pseudonym: &str, age: &str, zip: &str) -> AnonymizedRecord {
        AnonymizedRecord {
            pseudonym: pseudonym.to_string(),
            age: age.to_string(),
            zip: zip.to_string(),
            data: HashMap::new(),
        }
    }

    #[test]
    fn test_anonymize_removes_pii() {
        let raw = make_raw("Alice", "alice@example.com", "123-45-6789", "30", "02138");
        let anon = anonymize_record(&raw, "salt123");
        assert_ne!(anon.pseudonym, "Alice");
        assert_ne!(anon.pseudonym, "alice@example.com");
        assert_eq!(anon.age, "30");
        assert_eq!(anon.zip, "02138");
    }

    #[test]
    fn test_anonymize_deterministic() {
        let raw = make_raw("Alice", "alice@example.com", "123-45-6789", "30", "02138");
        let a1 = anonymize_record(&raw, "salt123");
        let a2 = anonymize_record(&raw, "salt123");
        assert_eq!(a1.pseudonym, a2.pseudonym);
    }

    #[test]
    fn test_anonymize_different_salt() {
        let raw = make_raw("Alice", "alice@example.com", "123-45-6789", "30", "02138");
        let a1 = anonymize_record(&raw, "salt1");
        let a2 = anonymize_record(&raw, "salt2");
        assert_ne!(a1.pseudonym, a2.pseudonym);
    }

    #[test]
    fn test_mask_value() {
        assert_eq!(mask_value("1234567890", 4), "1234******");
        assert_eq!(mask_value("ab", 4), "ab");
        assert_eq!(mask_value("hello", 0), "*****");
    }

    #[test]
    fn test_mask_ssn() {
        assert_eq!(mask_ssn("123-45-6789"), "***-**-6789");
        assert_eq!(mask_ssn("000-00-0000"), "***-**-0000");
        assert_eq!(mask_ssn("12"), "***");
    }

    #[test]
    fn test_reidentification_risk_zero() {
        let records = vec![
            make_anon("p1", "30", "02138"),
            make_anon("p2", "30", "02138"),
            make_anon("p3", "30", "02138"),
        ];
        assert_eq!(reidentification_risk(&records), 0.0);
    }

    #[test]
    fn test_reidentification_risk_high() {
        let records = vec![
            make_anon("p1", "25", "02138"),
            make_anon("p2", "30", "10001"),
            make_anon("p3", "35", "90210"),
        ];
        assert_eq!(reidentification_risk(&records), 1.0);
    }

    #[test]
    fn test_suppress_rare() {
        let records = vec![
            make_anon("p1", "30", "02138"),
            make_anon("p2", "30", "02138"),
            make_anon("p3", "25", "10001"),
        ];
        let filtered = suppress_rare(&records, 2);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.age == "30" && r.zip == "02138"));
    }

    #[test]
    fn test_batch_anonymize() {
        let records = vec![
            make_raw("Alice", "a@x.com", "111-11-1111", "30", "02138"),
            make_raw("Bob", "b@x.com", "222-22-2222", "30", "02138"),
            make_raw("Eve", "e@x.com", "333-33-3333", "25", "10001"),
        ];
        let result = batch_anonymize(&records, "salt", 2);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|r| r.age == "30" && r.zip == "02138"));
    }
}
