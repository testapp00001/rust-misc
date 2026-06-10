//! # Lesson 02: k-Anonymity -- Indistinguishability Protection
//!
//! ## What is k-Anonymity?
//!
//! A dataset satisfies k-anonymity if every combination of quasi-identifier values
//! (e.g., age, zip code, gender) appears in at least k records. This ensures that
//! each person's record is indistinguishable from at least k-1 others.
//!
//! ## Quasi-Identifiers vs. Identifiers
//!
//! - **Direct identifiers**: Name, SSN, email -- must be removed or pseudonymized
//! - **Quasi-identifiers**: Age, zip code, gender, occupation -- can be combined
//!   to re-identify individuals (linkage attack)
//! - **Sensitive attributes**: Disease, salary, vote -- the data we want to study
//!
//! ## Generalization and Suppression
//!
//! To achieve k-anonymity:
//! - **Generalization**: Replace precise values with ranges (age 25 -> age 20-30)
//! - **Suppression**: Remove records that cannot be generalized enough
//!
//! ## Attack: Linkage Attack (Re-identification)
//!
//! In 1997, Massachusetts Governor William Weld's health records were "anonymized"
//! but re-identified by combining zip code + birth date + gender with voter rolls.
//! 87% of Americans are uniquely identifiable by these 3 fields alone.
//!
//! ## Limitations of k-Anonymity
//!
//! - Does not protect against **homogeneity attack**: if all k records in a group
//!   have the same sensitive value, the adversary learns it regardless
//! - Does not protect against **background knowledge attack**: if the adversary
//!   knows the target is not in certain groups
//! - Solution: l-diversity (each group has diverse sensitive values) and t-closeness

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A record with quasi-identifiers and a sensitive attribute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Record {
    pub quasi_identifiers: HashMap<String, String>,
    pub sensitive: String,
}

/// Exercise 1: Check if a dataset satisfies k-anonymity.
///
/// Group records by their quasi-identifier values. If every group has
/// at least k members, the dataset is k-anonymous.
///
/// Hints:
/// - Use a HashMap with quasi-identifier tuples as keys
/// - The key should be a BTreeMap or sorted Vec of (key, value) pairs
/// - Check that all group sizes >= k
pub fn is_k_anonymous(records: &[Record], k: usize) -> bool {
    todo!("Check if dataset satisfies k-anonymity")
}

/// Exercise 2: Count the minimum group size (effective k).
///
/// Returns the size of the smallest equivalence class.
/// If the dataset is empty, returns 0.
///
/// Hints:
/// - Group by quasi-identifiers (same approach as `is_k_anonymous`)
/// - Return the minimum group size
pub fn effective_k(records: &[Record]) -> usize {
    todo!("Compute the effective k-anonymity of the dataset")
}

/// Exercise 3: Find records that violate k-anonymity.
///
/// Return the indices of records belonging to equivalence classes
/// with fewer than k members.
///
/// Hints:
/// - First build groups with record indices
/// - Then collect indices from all groups with size < k
pub fn find_violations(records: &[Record], k: usize) -> Vec<usize> {
    todo!("Find indices of records violating k-anonymity")
}

/// Exercise 4: Generalize age values into ranges.
///
/// Given a list of ages and a range width, generalize each age to its range.
/// For example, with width=10: age 23 -> "20-29", age 45 -> "40-49".
///
/// Hints:
/// - The range start is (age / width) * width
/// - The range end is start + width - 1
/// - Format as "start-end" string
pub fn generalize_age(age: u32, width: u32) -> String {
    todo!("Generalize an age into a range")
}

/// Exercise 5: Generalize zip codes by truncating digits.
///
/// Given a 5-digit zip code and the number of digits to keep,
/// replace the remaining digits with 'X'.
///
/// Examples:
/// - zip "02138", keep 3 -> "021XX"
/// - zip "10001", keep 4 -> "1000X"
///
/// Hints:
/// - Convert to string, keep first `keep` chars, append 'X' for the rest
/// - If keep >= zip length, return as-is
pub fn generalize_zip(zip: &str, keep: usize) -> String {
    todo!("Generalize a zip code by truncating digits")
}

/// Exercise 6: Apply k-anonymity via generalization.
///
/// Given records with "age" and "zip" quasi-identifiers, generalize them
/// and check if the result is k-anonymous. Try progressively wider
/// generalizations until k-anonymity is achieved.
///
/// Parameters:
/// - age_width: starting age range width (try 10, then 20, then 50...)
/// - zip_digits: starting zip digits to keep (try 4, then 3, then 2...)
///
/// Returns the generalized records, or None if k-anonymity cannot be achieved
/// even with maximum generalization.
///
/// Hints:
/// - Clone records and update quasi-identifiers with generalized values
/// - Use `generalize_age` and `generalize_zip`
/// - Check `is_k_anonymous` after each generalization step
/// - Try doubling age_width and decreasing zip_digits until k-anonymity holds
pub fn apply_k_anonymity(
    records: &[Record],
    k: usize,
    mut age_width: u32,
    mut zip_digits: usize,
) -> Option<Vec<Record>> {
    todo!("Apply generalization to achieve k-anonymity")
}

/// Exercise 7: Check l-diversity within each equivalence class.
///
/// l-diversity requires that each equivalence class has at least l
/// distinct values of the sensitive attribute.
///
/// Hints:
/// - Group records by quasi-identifiers
/// - For each group, count distinct sensitive values
/// - Return true if all groups have >= l distinct values
pub fn is_l_diverse(records: &[Record], l: usize) -> bool {
    todo!("Check if dataset satisfies l-diversity")
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
        assert_eq!(violations, vec![2]); // index 2 is alone in its group
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
        // l-diversity: each group must have >= l distinct sensitive values
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
        // All records in group have same sensitive value -- l-diversity fails
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
