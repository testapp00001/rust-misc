//! # Lesson 07: Data Masking (Reference Solution)
//!
//! See the exercise file for full documentation.

use sha2::{Digest, Sha256};

const FAKE_NAMES: &[&str] = &[
    "Jane Smith",
    "John Doe",
    "Alex Johnson",
    "Sam Williams",
    "Chris Brown",
    "Pat Davis",
    "Taylor Miller",
    "Jordan Wilson",
    "Casey Moore",
    "Riley Anderson",
    "Morgan Taylor",
    "Drew Thomas",
    "Quinn Jackson",
    "Avery White",
    "Skyler Harris",
    "Dakota Martin",
];

/// Mask a Social Security Number, showing only last 4 digits.
pub fn mask_ssn(ssn: &str) -> String {
    if ssn.len() >= 4 {
        let last4 = &ssn[ssn.len() - 4..];
        format!("***-**-{}", last4)
    } else {
        "***-**-****".to_string()
    }
}

/// Mask a credit card number, showing only last 4 digits.
pub fn mask_credit_card(cc: &str) -> String {
    let parts: Vec<&str> = cc.split('-').collect();
    if parts.len() >= 2 {
        let last_group = parts.last().unwrap();
        let masked_groups: Vec<String> = parts[..parts.len() - 1]
            .iter()
            .map(|_| "****".to_string())
            .collect();
        format!("{}-{}", masked_groups.join("-"), last_group)
    } else {
        "****-****-****-****".to_string()
    }
}

/// Mask an email address, showing only first character and domain.
pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let local = &email[..at_pos];
        let domain = &email[at_pos..];
        let first_char = local.chars().next().unwrap_or('*');
        format!("{}***{}", first_char, domain)
    } else {
        "***@***".to_string()
    }
}

/// Mask a phone number, showing only last 4 digits.
pub fn mask_phone(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        let last4 = &digits[digits.len() - 4..];
        format!("(***) ***-{}", last4)
    } else {
        "(***) ***-****".to_string()
    }
}

/// Generate a deterministic fake name from a real name.
pub fn mask_name(name: &str) -> String {
    let hash = Sha256::digest(name.as_bytes());
    let index = hash[0] as usize % FAKE_NAMES.len();
    FAKE_NAMES[index].to_string()
}

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

/// Apply masking to all fields of a SensitiveRecord.
pub fn mask_record(record: &SensitiveRecord) -> MaskedRecord {
    MaskedRecord {
        name: mask_name(&record.name),
        email: mask_email(&record.email),
        ssn: mask_ssn(&record.ssn),
        phone: mask_phone(&record.phone),
        credit_card: mask_credit_card(&record.credit_card),
    }
}

/// Demonstrate that masking is deterministic.
pub fn demonstrate_deterministic_masking() -> (bool, bool) {
    let name1 = mask_name("Alice Johnson");
    let name2 = mask_name("Alice Johnson");
    let name_consistent = name1 == name2;

    let ssn1 = mask_ssn("123-45-6789");
    let ssn2 = mask_ssn("123-45-6789");
    let ssn_consistent = ssn1 == ssn2;

    (name_consistent, ssn_consistent)
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
