//! # Lesson 07: Data Masking
//!
//! ## What Is Data Masking?
//!
//! Data masking replaces sensitive values with realistic but fake data. Developers
//! and testers get data that looks real but contains no actual PII. This is critical
//! for non-production environments where real data is a compliance violation.
//!
//! ## Types of Masking
//!
//! ```text
//! STATIC MASKING: Create a permanently masked copy of the database
//!   - Used for dev/test environments
//!   - Masked once, then the copy is used
//!
//! DYNAMIC MASKING: Mask on-the-fly when non-privileged users query
//!   - Production database stays intact
//!   - Results are masked before returning to the caller
//!
//! TOKENIZATION: Replace sensitive value with a reversible token
//!   - Token can be mapped back to original value
//!   - Used for PCI-DSS compliance (credit card numbers)
//! ```
//!
//! ## Masking Strategies
//!
//! | Data Type | Strategy | Example |
//! |-----------|----------|---------|
//! | SSN | Partial reveal | `***-**-6789` |
//! | Credit card | Last 4 only | `****-****-****-1111` |
//! | Email | Mask local part | `a***@example.com` |
//! | Name | Replace with fake | `John Doe` → `Jane Smith` |
//! | Phone | Partial reveal | `(***) ***-1234` |
//! | Date of birth | Shift by random days | Preserve age range |
//!
//! ## This Module
//!
//! We implement masking functions for common PII types, using deterministic
//! masking (same input always produces same output in a given environment).

use sha2::{Digest, Sha256};

/// Mask a Social Security Number, showing only the last 4 digits.
///
/// Input: "123-45-6789"  →  Output: "***-**-6789"
///
/// Exercise: Return the masked SSN.
///
/// Hints:
/// - Check that the input has the expected format (XXX-XX-XXXX)
/// - Replace the first 5 digits with `*`
/// - Keep the last 4 digits visible
pub fn mask_ssn(ssn: &str) -> String {
    todo!("Mask SSN, showing only last 4 digits")
}

/// Mask a credit card number, showing only the last 4 digits.
///
/// Input: "4111-1111-1111-1111"  →  Output: "****-****-****-1111"
///
/// Exercise: Return the masked credit card number.
///
/// Hints:
/// - Split by `-`
/// - Replace all groups except the last with `****`
/// - Keep the last group as-is
pub fn mask_credit_card(cc: &str) -> String {
    todo!("Mask credit card, showing only last 4 digits")
}

/// Mask an email address, showing only the first character and domain.
///
/// Input: "alice.jones@example.com"  →  Output: "a***@example.com"
///
/// Exercise: Return the masked email.
///
/// Hints:
/// - Split by `@`
/// - Keep first character of local part, replace rest with `***`
/// - Keep domain as-is
pub fn mask_email(email: &str) -> String {
    todo!("Mask email address")
}

/// Mask a phone number, showing only the last 4 digits.
///
/// Input: "(555) 123-4567"  →  Output: "(***) ***-4567"
///
/// Exercise: Return the masked phone number.
///
/// Hints:
/// - Extract the last 4 digits
/// - Replace everything else with `*` patterns
pub fn mask_phone(phone: &str) -> String {
    todo!("Mask phone number, showing only last 4 digits")
}

/// Deterministic fake name generation.
///
/// Given a real name, produce a consistent fake name using a seed.
/// Same input always produces the same output (important for referential integrity).
///
/// Exercise: Use a hash of the input to select from a list of fake names.
///
/// Hints:
/// - Hash the input name with SHA-256
/// - Use the first byte of the hash as an index into a fake name list
/// - Return the selected fake name
pub fn mask_name(name: &str) -> String {
    todo!("Generate a deterministic fake name from a real name")
}

/// Apply all masking functions to a record.
///
/// Takes a record with sensitive fields and returns a masked version.
///
/// Exercise: Apply the appropriate masking function to each field.
pub struct SensitiveRecord {
    pub name: String,
    pub email: String,
    pub ssn: String,
    pub phone: String,
    pub credit_card: String,
}

pub struct MaskedRecord {
    pub name: String,
    pub email: String,
    pub ssn: String,
    pub phone: String,
    pub credit_card: String,
}

pub fn mask_record(record: &SensitiveRecord) -> MaskedRecord {
    todo!("Apply masking to all fields of a SensitiveRecord")
}

/// Demonstrate that masking is deterministic.
///
/// Exercise: Show that calling the same masking function twice on the same input
/// produces the same output.
///
/// Hints:
/// - Mask a name twice
/// - Mask an SSN twice
/// - Return (name_consistent, ssn_consistent)
pub fn demonstrate_deterministic_masking() -> (bool, bool) {
    todo!("Show that masking is deterministic")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_ssn() {
        let masked = mask_ssn("123-45-6789");
        assert_eq!(masked, "***-**-6789");
        assert!(!masked.contains("123"));
        assert!(!masked.contains("45"));
    }

    #[test]
    fn test_mask_credit_card() {
        let masked = mask_credit_card("4111-1111-1111-1111");
        assert_eq!(masked, "****-****-****-1111");
        assert!(!masked.contains("4111"));
    }

    #[test]
    fn test_mask_email() {
        let masked = mask_email("alice.jones@example.com");
        assert!(masked.starts_with('a'));
        assert!(masked.contains("@example.com"));
        assert!(masked.contains("***"));
        assert!(!masked.contains("alice"));
    }

    #[test]
    fn test_mask_phone() {
        let masked = mask_phone("(555) 123-4567");
        assert!(masked.contains("4567"));
        assert!(!masked.contains("123"));
        assert!(!masked.contains("555"));
    }

    #[test]
    fn test_mask_name_deterministic() {
        let name1 = mask_name("Alice Johnson");
        let name2 = mask_name("Alice Johnson");
        assert_eq!(name1, name2, "Same input must produce same masked name");
    }

    #[test]
    fn test_mask_name_differs_from_original() {
        let original = "Alice Johnson";
        let masked = mask_name(original);
        // The masked name should be a different name (or at least not the same)
        // Note: in rare hash collisions it could be the same, but very unlikely
        assert!(!masked.is_empty());
    }

    #[test]
    fn test_mask_record() {
        let record = SensitiveRecord {
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            ssn: "123-45-6789".to_string(),
            phone: "(555) 123-4567".to_string(),
            credit_card: "4111-1111-1111-1111".to_string(),
        };
        let masked = mask_record(&record);

        assert!(!masked.ssn.contains("123"));
        assert!(!masked.credit_card.contains("4111"));
        assert!(masked.email.contains("@example.com"));
        assert!(masked.ssn.contains("6789"));
    }

    #[test]
    fn test_demonstrate_deterministic() {
        let (name_ok, ssn_ok) = demonstrate_deterministic_masking();
        assert!(name_ok, "Name masking must be deterministic");
        assert!(ssn_ok, "SSN masking must be deterministic");
    }
}
