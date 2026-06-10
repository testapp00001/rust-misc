//! # Lesson 08: Breach Detection (Reference Solution)
//!
//! See the exercise file for full documentation on k-anonymity breach detection.

/// A simulated breach database entry.
#[derive(Debug, Clone)]
pub struct BreachEntry {
    pub hash: String,
    pub count: u32,
}

/// Compute SHA-1 hash of a password as uppercase hex.
pub fn sha1_hash(password: &str) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, password.as_bytes());
    hex::encode(digest.as_ref()).to_uppercase()
}

/// Split a SHA-1 hash into prefix (5 chars) and suffix (35 chars).
pub fn split_hash(hash: &str) -> (String, String) {
    (hash[..5].to_string(), hash[5..].to_string())
}

/// Check if a password appears in a simulated breach database.
pub fn check_password_breach(password: &str, breach_db: &[BreachEntry]) -> Option<u32> {
    let hash = sha1_hash(password);
    let (prefix, _suffix) = split_hash(&hash);
    breach_db
        .iter()
        .filter(|e| e.hash.starts_with(&prefix))
        .find(|e| e.hash == hash)
        .map(|e| e.count)
}

/// Check multiple passwords and return breach results.
pub fn check_multiple_passwords(passwords: &[&str], breach_db: &[BreachEntry]) -> Vec<(usize, u32)> {
    passwords
        .iter()
        .enumerate()
        .filter_map(|(i, pw)| check_password_breach(pw, breach_db).map(|count| (i, count)))
        .collect()
}

/// Generate a simulated breach database from a list of passwords.
pub fn generate_breach_db(passwords: &[&str]) -> Vec<BreachEntry> {
    passwords
        .iter()
        .map(|pw| BreachEntry {
            hash: sha1_hash(pw),
            count: 1,
        })
        .collect()
}

/// Calculate the password strength score (0-100).
pub fn password_strength_score(password: &str, breach_db: &[BreachEntry]) -> u32 {
    let mut score: u32 = 0;

    // Length: +2 per character (max 40)
    score += (password.len() as u32 * 2).min(40);

    // Has uppercase
    if password.chars().any(|c| c.is_ascii_uppercase()) {
        score += 10;
    }

    // Has lowercase
    if password.chars().any(|c| c.is_ascii_lowercase()) {
        score += 10;
    }

    // Has digits
    if password.chars().any(|c| c.is_ascii_digit()) {
        score += 10;
    }

    // Has special chars
    if password.chars().any(|c| !c.is_ascii_alphanumeric()) {
        score += 10;
    }

    // Not in breach DB
    if check_password_breach(password, breach_db).is_none() {
        score += 20;
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_breach_db() -> Vec<BreachEntry> {
        vec![
            BreachEntry { hash: "5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8".into(), count: 3_000_000 },
            BreachEntry { hash: "7C4A8D09CA3762AF61E59520943DC26494F8941B".into(), count: 10_000_000 },
            BreachEntry { hash: "E38AD214943DAAD1D64C102FAEC29DE4AFE9DA3D".into(), count: 500_000 },
        ]
    }

    #[test]
    fn test_sha1_hash_format() {
        let hash = sha1_hash("password");
        assert_eq!(hash.len(), 40, "SHA-1 hash should be 40 hex chars");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hash, "5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8");
    }

    #[test]
    fn test_split_hash() {
        let (prefix, suffix) = split_hash("5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8");
        assert_eq!(prefix, "5BAA6");
        assert_eq!(suffix, "1E4C9B93F3F0682250B6CF8331B7EE68FD8");
        assert_eq!(prefix.len(), 5);
        assert_eq!(suffix.len(), 35);
    }

    #[test]
    fn test_check_breached_password() {
        let db = test_breach_db();
        let result = check_password_breach("password", &db);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 3_000_000);
    }

    #[test]
    fn test_check_safe_password() {
        let db = test_breach_db();
        let result = check_password_breach("my-very-unique-p4ssphr4se!2024", &db);
        assert!(result.is_none());
    }

    #[test]
    fn test_check_multiple_passwords() {
        let db = test_breach_db();
        let passwords = vec!["password", "safe_password_123!@#", "123456"];
        let results = check_multiple_passwords(&passwords, &db);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_generate_breach_db() {
        let passwords = vec!["password", "123456"];
        let db = generate_breach_db(&passwords);
        assert_eq!(db.len(), 2);
        assert!(db.iter().all(|e| e.hash.len() == 40));
    }

    #[test]
    fn test_password_strength_score() {
        let db = test_breach_db();
        let score = password_strength_score("MyS3cur3P@ssw0rd!2024", &db);
        assert!(score >= 80, "Strong password should score >= 80, got {}", score);
    }

    #[test]
    fn test_password_strength_weak() {
        let db = test_breach_db();
        let score = password_strength_score("password", &db);
        assert!(score < 50, "Weak breached password should score < 50, got {}", score);
    }
}
