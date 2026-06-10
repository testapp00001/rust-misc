//! # Lesson 08: Breach Detection
//!
//! ## Checking Passwords Against Known Breaches
//!
//! Even with a strong password manager, users sometimes choose weak passwords
//! or reuse passwords across sites. Breach detection checks each password
//! against databases of known-compromised passwords.
//!
//! ## The k-Anonymity Model (HaveIBeenPwned)
//!
//! The HaveIBeenPwned (HIBP) API uses a clever privacy-preserving approach:
//!
//! 1. Hash the password with SHA-1: `SHA1(password) = "5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8"`
//! 2. Take the first 5 hex chars (the "prefix"): `"5BAA6"`
//! 3. Send ONLY the prefix to the API
//! 4. The API returns all hashes starting with that prefix (~500-800 results)
//! 5. Check locally if the full hash is in the response
//!
//! This means the server NEVER sees your full password hash. It cannot
//! determine which of the ~500 results you were checking.
//!
//! ## Why SHA-1?
//!
//! HIBP uses SHA-1 for historical reasons (compatibility with existing breach
//! databases). It is NOT used for security -- only as a lookup key. The
//! k-anonymity model means even a broken hash is safe here.
//!
//! ## Implementation
//!
//! For this exercise, we simulate the HIBP API with a local breach database.
//! The real API would be called with: `GET https://api.pwnedpasswords.com/range/{prefix}`
//!
//! ## Attack Scenario
//!
//! A user sets their vault password to "password123". The breach checker
//! detects that this password appears in 100,000+ breach records and warns
//! the user before they proceed.

use sha2::{Sha256, Digest};

/// A simulated breach database entry.
/// In the real HIBP API, each entry is a SHA-1 suffix and count.
#[derive(Debug, Clone)]
pub struct BreachEntry {
    /// Full SHA-1 hash of the breached password (uppercase hex)
    pub hash: String,
    /// Number of times this password appeared in breaches
    pub count: u32,
}

/// Exercise 1: Compute the SHA-1 hash of a password and return it as uppercase hex.
///
/// Hints:
/// - `use sha1::Sha1;` -- wait, we use `sha2`. Use `ring::digest` instead:
///   `let digest = ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, password.as_bytes());`
///   `hex::encode(digest.as_ref()).to_uppercase()`
/// - Or use sha2's Sha1: `sha2::Sha1` (sha2 includes Sha1)
///
/// Since we don't have sha1 in dependencies, use ring:
/// - `ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, password.as_bytes())`
pub fn sha1_hash(password: &str) -> String {
    todo!("Compute SHA-1 hash of password as uppercase hex")
}

/// Exercise 2: Split a SHA-1 hash into prefix (5 chars) and suffix (35 chars).
///
/// Hints:
/// - `let prefix = &hash[..5];`
/// - `let suffix = &hash[5..];`
pub fn split_hash(hash: &str) -> (String, String) {
    todo!("Split SHA-1 hash into prefix and suffix")
}

/// Exercise 3: Check if a password appears in a simulated breach database.
///
/// Simulates the HIBP k-anonymity model:
/// 1. Hash the password with SHA-1
/// 2. Extract the 5-char prefix
/// 3. Find all entries in `breach_db` matching that prefix
/// 4. Check if the full hash appears in the matching entries
///
/// Returns Some(count) if breached, None if safe.
///
/// Hints:
/// - Hash the password
/// - Split into prefix/suffix
/// - Filter breach_db entries: `breach_db.iter().filter(|e| e.hash.starts_with(&prefix))`
/// - Check if any match the full hash
pub fn check_password_breach(password: &str, breach_db: &[BreachEntry]) -> Option<u32> {
    todo!("Check if password appears in breach database")
}

/// Exercise 4: Check multiple passwords and return breach results.
///
/// Returns a Vec of (password_index, breach_count) for breached passwords.
///
/// Hints:
/// - Iterate over passwords with enumerate
/// - Call check_password_breach for each
/// - Collect results where breach is Some
pub fn check_multiple_passwords(passwords: &[&str], breach_db: &[BreachEntry]) -> Vec<(usize, u32)> {
    todo!("Check multiple passwords against breach database")
}

/// Exercise 5: Generate a simulated breach database from a list of passwords.
///
/// This is for testing. In production, you'd use the HIBP API.
///
/// Hints:
/// - For each password, compute SHA-1 hash
/// - Create a BreachEntry with count 1 (or random count)
pub fn generate_breach_db(passwords: &[&str]) -> Vec<BreachEntry> {
    todo!("Generate a simulated breach database")
}

/// Exercise 6: Calculate the password strength score (0-100).
///
/// Scoring criteria:
/// - Length: +2 per character (max 40)
/// - Has uppercase: +10
/// - Has lowercase: +10
/// - Has digits: +10
/// - Has special chars: +10
/// - Not in breach DB: +20
///
/// Hints:
/// - Check each criterion with `password.chars().any(|c| c.is_...())`
/// - Use `check_password_breach` for breach check
pub fn password_strength_score(password: &str, breach_db: &[BreachEntry]) -> u32 {
    todo!("Calculate password strength score")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_breach_db() -> Vec<BreachEntry> {
        // Known SHA-1 hashes for common passwords
        // "password" -> 5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8
        // "123456" -> 7C4A8D09CA3762AF61E59520943DC26494F8941B
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
        // Known SHA-1 of "password"
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
        assert_eq!(results.len(), 2); // "password" and "123456" are breached
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
        // Strong password, not breached
        let score = password_strength_score("MyS3cur3P@ssw0rd!2024", &db);
        assert!(score >= 80, "Strong password should score >= 80, got {}", score);
    }

    #[test]
    fn test_password_strength_weak() {
        let db = test_breach_db();
        // Weak password, breached
        let score = password_strength_score("password", &db);
        assert!(score < 50, "Weak breached password should score < 50, got {}", score);
    }
}
