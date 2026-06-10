//! # Lesson 02: k-Anonymity -- Indistinguishability Protection (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::{BTreeMap, HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// A record with quasi-identifiers and a sensitive attribute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Record {
    pub quasi_identifiers: HashMap<String, String>,
    pub sensitive: String,
}

/// Check if a dataset satisfies k-anonymity.
///
/// Groups records by quasi-identifier values and checks that all groups have >= k members.
pub fn is_k_anonymous(records: &[Record], k: usize) -> bool {
    if k == 0 {
        return true;
    }
    let groups = group_by_qi(records);
    groups.values().all(|group| group.len() >= k)
}

/// Compute the effective k-anonymity (minimum group size).
pub fn effective_k(records: &[Record]) -> usize {
    if records.is_empty() {
        return 0;
    }
    let groups = group_by_qi(records);
    groups.values().map(|group| group.len()).min().unwrap_or(0)
}

/// Find indices of records violating k-anonymity.
pub fn find_violations(records: &[Record], k: usize) -> Vec<usize> {
    let groups = group_by_qi_with_indices(records);
    let mut violations: Vec<usize> = groups
        .values()
        .filter(|indices| indices.len() < k)
        .flat_map(|indices| indices.iter().copied())
        .collect();
    violations.sort();
    violations
}

/// Generalize an age into a range string.
pub fn generalize_age(age: u32, width: u32) -> String {
    let start = (age / width) * width;
    let end = start + width - 1;
    format!("{}-{}", start, end)
}

/// Generalize a zip code by truncating digits.
pub fn generalize_zip(zip: &str, keep: usize) -> String {
    if keep >= zip.len() {
        return zip.to_string();
    }
    let mut result: String = zip.chars().take(keep).collect();
    for _ in 0..(zip.len() - keep) {
        result.push('X');
    }
    result
}

/// Apply k-anonymity via progressive generalization.
pub fn apply_k_anonymity(
    records: &[Record],
    k: usize,
    mut age_width: u32,
    mut zip_digits: usize,
) -> Option<Vec<Record>> {
    let max_iterations = 10;
    for _ in 0..max_iterations {
        let generalized: Vec<Record> = records
            .iter()
            .map(|r| generalize_record(r, age_width, zip_digits))
            .collect();

        if is_k_anonymous(&generalized, k) {
            return Some(generalized);
        }

        // Widen generalization
        age_width *= 2;
        if zip_digits > 1 {
            zip_digits -= 1;
        }
    }
    None
}

/// Check l-diversity within each equivalence class.
pub fn is_l_diverse(records: &[Record], l: usize) -> bool {
    if l == 0 {
        return true;
    }
    let groups = group_by_qi(records);
    groups.values().all(|group| {
        let distinct_sensitive: HashSet<&str> = group.iter().map(|r| r.sensitive.as_str()).collect();
        distinct_sensitive.len() >= l
    })
}

// --- Helper functions ---

fn qi_key(record: &Record) -> BTreeMap<String, String> {
    record.quasi_identifiers.clone().into_iter().collect()
}

fn group_by_qi(records: &[Record]) -> HashMap<BTreeMap<String, String>, Vec<&Record>> {
    let mut groups: HashMap<BTreeMap<String, String>, Vec<&Record>> = HashMap::new();
    for record in records {
        groups.entry(qi_key(record)).or_default().push(record);
    }
    groups
}

fn group_by_qi_with_indices(records: &[Record]) -> HashMap<BTreeMap<String, String>, Vec<usize>> {
    let mut groups: HashMap<BTreeMap<String, String>, Vec<usize>> = HashMap::new();
    for (i, record) in records.iter().enumerate() {
        groups.entry(qi_key(record)).or_default().push(i);
    }
    groups
}

fn generalize_record(record: &Record, age_width: u32, zip_digits: usize) -> Record {
    let mut qi = record.quasi_identifiers.clone();
    if let Some(age) = qi.get("age").and_then(|a| a.parse::<u32>().ok()) {
        qi.insert("age".to_string(), generalize_age(age, age_width));
    }
    if let Some(zip) = qi.get("zip") {
        qi.insert("zip".to_string(), generalize_zip(zip, zip_digits));
    }
    Record {
        quasi_identifiers: qi,
        sensitive: record.sensitive.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(age: &str, zip: &str, disease: &str) -> Record {
        let mut qi = HashMap::new();
        qi.insert("age".to_string(), age.to_string());
        qi.insert("zip".to_string(), zip.to_string());
        Record {
            quasi_identifiers: qi,
            sensitive: disease.to_string(),
        }
    }

    #[test]
    fn test_is_k_anonymous_k2() {
        let records = vec![
            make_record("25", "02138", "flu"),
            make_record("25", "02138", "cold"),
            make_record("30", "10001", "flu"),
            make_record("30", "10001", "cold"),
        ];
        assert!(is_k_anonymous(&records, 2));
        assert!(!is_k_anonymous(&records, 3));
    }

    #[test]
    fn test_is_k_anonymous_violation() {
        let records = vec![
            make_record("25", "02138", "flu"),
            make_record("25", "02138", "cold"),
            make_record("30", "10001", "flu"),
        ];
        assert!(!is_k_anonymous(&records, 2), "Group of 1 violates k=2");
        assert!(!is_k_anonymous(&records, 3));
    }

    #[test]
    fn test_effective_k() {
        let records = vec![
            make_record("25", "02138", "flu"),
            make_record("25", "02138", "cold"),
            make_record("30", "10001", "flu"),
            make_record("30", "10001", "cold"),
            make_record("30", "10001", "headache"),
        ];
        assert_eq!(effective_k(&records), 2);
    }

    #[test]
    fn test_find_violations() {
        let records = vec![
            make_record("25", "02138", "flu"),
            make_record("25", "02138", "cold"),
            make_record("30", "10001", "flu"),
        ];
        let violations = find_violations(&records, 2);
        assert_eq!(violations, vec![2]);
    }

    #[test]
    fn test_generalize_age() {
        assert_eq!(generalize_age(23, 10), "20-29");
        assert_eq!(generalize_age(45, 10), "40-49");
        assert_eq!(generalize_age(30, 10), "30-39");
        assert_eq!(generalize_age(25, 5), "25-29");
    }

    #[test]
    fn test_generalize_zip() {
        assert_eq!(generalize_zip("02138", 3), "021XX");
        assert_eq!(generalize_zip("10001", 4), "1000X");
        assert_eq!(generalize_zip("10001", 5), "10001");
        assert_eq!(generalize_zip("02138", 1), "0XXXX");
    }

    #[test]
    fn test_is_l_diverse() {
        let records = vec![
            make_record("25", "02138", "flu"),
            make_record("25", "02138", "cold"),
            make_record("25", "02138", "headache"),
            make_record("30", "10001", "flu"),
            make_record("30", "10001", "cold"),
        ];
        assert!(is_l_diverse(&records, 2));
        assert!(!is_l_diverse(&records, 3), "Second group has only 2 distinct values");
        assert!(!is_l_diverse(&records, 4));
    }

    #[test]
    fn test_l_diversity_homogeneity_attack() {
        let records = vec![
            make_record("25", "02138", "flu"),
            make_record("25", "02138", "flu"),
            make_record("30", "10001", "cold"),
            make_record("30", "10001", "cold"),
        ];
        assert!(is_k_anonymous(&records, 2));
        assert!(!is_l_diverse(&records, 2), "Homogeneous groups fail l-diversity");
    }
}
