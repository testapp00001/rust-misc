//! # Lesson 07: Password Strength Validation
//!
//! ## Why Validate Passwords?
//!
//! Even the best hashing algorithm cannot protect a password that is "123456".
//! Password validation ensures that users choose passwords with sufficient
//! entropy to resist offline brute-force attacks.
//!
//! ## Modern Best Practices (NIST SP 800-63B)
//!
//! The old rules ("must have uppercase, lowercase, digit, special char") are
//! **deprecated** by NIST. They lead to predictable patterns like "P@ssw0rd1!"
//! and frustrate users.
//!
//! Modern guidelines:
//! 1. **Minimum length**: 8 characters (12+ recommended)
//! 2. **Maximum length**: At least 64 characters (support passphrases)
//! 3. **No composition rules**: Don't require specific character classes
//! 4. **Check against breach databases**: Reject passwords found in known breaches
//! 5. **Check against common passwords**: Reject "password", "123456", etc.
//! 6. **Allow all characters**: Including Unicode, spaces, emoji
//!
//! ## Breach Database Check (k-Anonymity)
//!
//! The Have I Been Pwned (HIBP) API uses k-anonymity:
//! 1. Compute SHA-1 of the password
//! 2. Send only the first 5 hex characters (prefix) to the API
//! 3. API returns all hashes with that prefix
//! 4. Check locally if the full hash is in the response
//!
//! The API never sees the full hash. The server learns nothing about the password.
//!
//! ## Entropy Estimation
//!
//! Password entropy is measured in bits. A password with 40 bits of entropy
//! requires 2^40 (~1 trillion) attempts to brute-force.
//!
//! Common entropy estimates:
//! - Random from 26 lowercase: log2(26) * length = 4.7 * length
//! - Random from 62 alphanumeric: log2(62) * length = 5.95 * length
//! - Random from 95 printable ASCII: log2(95) * length = 6.57 * length
//! - Dictionary word: ~10-12 bits per word
//!
//! A 12-character random alphanumeric password has ~71 bits of entropy.
//! A 4-word passphrase from a 7776-word list has ~52 bits (Diceware standard).

/// Exercise 1: Check password length requirements.
///
/// Returns Ok(()) if the password meets minimum and maximum length requirements.
/// Returns Err with a description if it doesn't.
///
/// Hints:
/// - Check `password.len() >= min_length`
/// - Check `password.len() <= max_length`
/// - Use descriptive error messages
pub fn check_length(password: &str, min_length: usize, max_length: usize) -> Result<(), String> {
    todo!("Implement length check")
}

/// Exercise 2: Check if a password is in a list of common passwords.
///
/// The common list should include at least: "password", "123456", "12345678",
/// "qwerty", "abc123", "monkey", "master", "dragon", "111111", "baseball".
///
/// Hints:
/// - Convert password to lowercase for comparison
/// - Check against a hardcoded list
/// - Return true if found (is common), false if not
pub fn is_common_password(password: &str) -> bool {
    todo!("Implement common password check")
}

/// Exercise 3: Estimate the entropy of a password in bits.
///
/// Use character set size estimation:
/// - Has lowercase? Add 26 to charset
/// - Has uppercase? Add 26
/// - Has digits? Add 10
/// - Has symbols? Add 33
/// - Entropy = log2(charset_size) * length
///
/// Hints:
/// - Check each character class with `.is_ascii_lowercase()`, etc.
/// - Use `charset_size as f64).log2() * password.len() as f64`
pub fn estimate_entropy(password: &str) -> f64 {
    todo!("Implement entropy estimation")
}

/// Exercise 4: Compute SHA-1 of a password (for HIBP k-anonymity check).
///
/// Returns the uppercase hex string of the SHA-1 hash.
///
/// Hints:
/// - Use `ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, data)`
/// - Convert to uppercase hex with `hex::encode_upper`
pub fn sha1_hex_upper(data: &[u8]) -> String {
    todo!("Implement SHA-1 for HIBP check")
}

/// Exercise 5: Check if a password hash appears in a simulated breach database.
///
/// The "breach database" is provided as a list of SHA-1 hashes (uppercase hex).
/// This simulates the client-side check after receiving HIBP API results.
///
/// Hints:
/// - Compute SHA-1 of the password
/// - Check if it's in the provided list (use constant-time comparison for each)
pub fn check_breach_database(password: &str, breach_hashes: &[String]) -> bool {
    todo!("Implement breach database check")
}

/// Exercise 6: Implement k-anonymity prefix extraction for HIBP.
///
/// The HIBP API expects the first 5 characters of the uppercase SHA-1 hex.
///
/// Hints:
/// - Compute SHA-1 hex uppercase
/// - Take the first 5 characters
pub fn hibp_prefix(password: &str) -> String {
    todo!("Implement HIBP prefix extraction")
}

/// Exercise 7: Full password strength validation.
///
/// Combine all checks into a single validation function.
/// Returns Ok(()) if the password passes all checks, or Err with all failures.
///
/// Requirements:
/// - Minimum 8 characters, maximum 128
/// - Not in common password list
/// - At least 40 bits of entropy
///
/// Hints:
/// - Call each check function
/// - Collect all errors (don't short-circuit on first failure)
pub fn validate_password(password: &str) -> Result<(), Vec<String>> {
    todo!("Implement full password validation")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_valid() {
        assert!(check_length("password123", 8, 128).is_ok());
    }

    #[test]
    fn test_length_too_short() {
        assert!(check_length("short", 8, 128).is_err());
    }

    #[test]
    fn test_length_too_long() {
        let long = "a".repeat(200);
        assert!(check_length(&long, 8, 128).is_err());
    }

    #[test]
    fn test_common_passwords() {
        assert!(is_common_password("password"));
        assert!(is_common_password("123456"));
        assert!(is_common_password("PASSWORD")); // case-insensitive
        assert!(!is_common_password("xK9!mZ2@qR7#"));
    }

    #[test]
    fn test_entropy_estimation() {
        // Lowercase only: ~4.7 bits per char * 10 = ~47 bits
        let entropy = estimate_entropy("abcdefghij");
        assert!(entropy > 40.0 && entropy < 55.0);

        // Mixed case + digits + symbols: high entropy
        let entropy2 = estimate_entropy("xK9!mZ2@qR7#");
        assert!(entropy2 > 60.0);
    }

    #[test]
    fn test_sha1_hex_upper() {
        // Known SHA-1 of empty string
        let hash = sha1_hex_upper(b"");
        assert_eq!(hash, "DA39A3EE5E6B4B0D3255BFEF95601890AFD80709");
    }

    #[test]
    fn test_hibp_prefix() {
        let prefix = hibp_prefix("password");
        assert_eq!(prefix.len(), 5, "HIBP prefix should be 5 characters");
        // Known: SHA-1 of "password" = 5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8
        assert_eq!(prefix, "5BAA6");
    }

    #[test]
    fn test_breach_database_check() {
        // SHA-1 of "password" = 5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8
        let breach_hashes = vec![
            "5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8".to_string(),
        ];
        assert!(check_breach_database("password", &breach_hashes));
        assert!(!check_breach_database("xK9!mZ2@qR7#", &breach_hashes));
    }

    #[test]
    fn test_full_validation_strong_password() {
        assert!(validate_password("xK9!mZ2@qR7#wP4$").is_ok());
    }

    #[test]
    fn test_full_validation_weak_password() {
        let result = validate_password("password");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() > 0, "Should report at least one error");
    }
}
