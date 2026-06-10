//! # Lesson 04: Data Anonymization -- Removing PII and Pseudonymization
//!
//! ## What is Data Anonymization?
//!
//! Anonymization transforms personal data so that the individual cannot be identified.
//! Unlike pseudonymization, true anonymization is irreversible.
//!
//! ## Techniques
//!
//! 1. **Direct removal**: Delete names, SSNs, emails
//! 2. **Pseudonymization**: Replace identifiers with consistent pseudonyms (reversible with key)
//! 3. **Generalization**: Replace exact values with ranges (see Module 02)
//! 4. **Suppression**: Remove records or fields entirely
//! 5. **Masking**: Replace parts of values with placeholders (e.g., SSN: ***-**-1234)
//!
//! ## Pseudonymization vs. Anonymization (GDPR Recital 26)
//!
//! - **Pseudonymized data is STILL personal data** under GDPR -- it can be re-identified
//! - **Anonymized data is NOT personal data** -- GDPR does not apply
//! - Pseudonymization reduces risk but does not eliminate legal obligations
//!
//! ## Attack: Re-identification Risk Assessment
//!
//! Even after removing direct identifiers, records can be re-identified through:
//! - **Linkage attacks**: Combining with external datasets
//! - **Inference attacks**: Using unique patterns in the data
//! - **Differential attacks**: Comparing anonymized datasets from different times
//!
//! ## Re-identification Risk Metric
//!
//! Risk = (number of records uniquely identifiable by quasi-identifiers) / (total records)
//! A risk > 0 means some records can be uniquely singled out.

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

/// Exercise 1: Strip direct identifiers from a record.
///
/// Remove name, email, and SSN. Keep age, zip, and other data fields.
/// Generate a pseudonym using SHA-256 of the email with a salt.
///
/// Hints:
/// - Hash: format!("{}:{}", salt, email), then SHA-256 hex
/// - Use `sha2::Sha256` and `sha2::Digest`
/// - Take first 16 hex chars of the hash as the pseudonym
pub fn anonymize_record(record: &RawRecord, salt: &str) -> AnonymizedRecord {
    todo!("Strip PII and generate pseudonym from email+salt")
}

/// Exercise 2: Mask sensitive fields (partial display).
///
/// For a string value, show only the first `show` characters and
/// replace the rest with '*' characters.
///
/// Examples:
/// - mask("1234567890", 4) -> "1234******"
/// - mask("ab", 4) -> "ab" (shorter than show, return as-is)
///
/// Hints:
/// - If value.len() <= show, return the value unchanged
/// - Otherwise, take first `show` chars and append '*' * (len - show)
pub fn mask_value(value: &str, show: usize) -> String {
    todo!("Mask a sensitive value, showing only first N characters")
}

/// Exercise 3: Mask an SSN to show only last 4 digits.
///
/// Format: "***-**-XXXX"
///
/// Hints:
/// - If SSN length < 4, return "***"
/// - Otherwise, take the last 4 chars
pub fn mask_ssn(ssn: &str) -> String {
    todo!("Mask SSN to show only last 4 digits")
}

/// Exercise 4: Compute re-identification risk.
///
/// Given anonymized records (with age and zip as quasi-identifiers),
/// compute the fraction of records that are uniquely identifiable.
///
/// Risk = (count of records whose (age, zip) combo appears exactly once) / (total records)
///
/// Hints:
/// - Count occurrences of each (age, zip) pair
/// - Count how many pairs have count == 1
/// - Return count_unique / total as f64
pub fn reidentification_risk(records: &[AnonymizedRecord]) -> f64 {
    todo!("Compute re-identification risk based on quasi-identifiers")
}

/// Exercise 5: Suppress records that are uniquely identifiable.
///
/// Remove all records whose quasi-identifier combination (age, zip)
/// appears fewer than `min_group_size` times.
///
/// Hints:
/// - Count (age, zip) frequencies
/// - Keep only records where their combo appears >= min_group_size times
pub fn suppress_rare(records: &[AnonymizedRecord], min_group_size: usize) -> Vec<AnonymizedRecord> {
    todo!("Suppress records that don't meet minimum group size")
}

/// Exercise 6: Batch anonymize a collection of records.
///
/// Anonymize each record using the same salt, then suppress records
/// with re-identification risk (groups smaller than min_group_size).
///
/// Hints:
/// - Map each record through `anonymize_record`
/// - Then filter with `suppress_rare`
pub fn batch_anonymize(
    records: &[RawRecord],
    salt: &str,
    min_group_size: usize,
) -> Vec<AnonymizedRecord> {
    todo!("Batch anonymize and suppress risky records")
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
        assert_eq!(a1.pseudonym, a2.pseudonym, "Same salt+email should give same pseudonym");
    }

    #[test]
    fn test_anonymize_different_salt() {
        let raw = make_raw("Alice", "alice@example.com", "123-45-6789", "30", "02138");
        let a1 = anonymize_record(&raw, "salt1");
        let a2 = anonymize_record(&raw, "salt2");
        assert_ne!(a1.pseudonym, a2.pseudonym, "Different salts should give different pseudonyms");
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
        // All records share the same quasi-identifier combo
        let records = vec![
            make_anon("p1", "30", "02138"),
            make_anon("p2", "30", "02138"),
            make_anon("p3", "30", "02138"),
        ];
        assert_eq!(reidentification_risk(&records), 0.0);
    }

    #[test]
    fn test_reidentification_risk_high() {
        // All records are unique
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
            make_anon("p3", "25", "10001"), // unique, should be suppressed
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
        assert_eq!(result.len(), 2, "Eve's record should be suppressed (unique)");
        assert!(result.iter().all(|r| r.age == "30" && r.zip == "02138"));
    }
}
