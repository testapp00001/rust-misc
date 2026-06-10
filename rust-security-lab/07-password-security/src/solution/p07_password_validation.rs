//! # Lesson 07: Password Strength Validation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Check password length requirements.
pub fn check_length(password: &str, min_length: usize, max_length: usize) -> Result<(), String> {
    if password.len() < min_length {
        return Err(format!("Password too short: {} chars (minimum {})", password.len(), min_length));
    }
    if password.len() > max_length {
        return Err(format!("Password too long: {} chars (maximum {})", password.len(), max_length));
    }
    Ok(())
}

/// Check if a password is in a list of common passwords.
pub fn is_common_password(password: &str) -> bool {
    const COMMON: &[&str] = &[
        "password", "123456", "12345678", "qwerty", "abc123",
        "monkey", "master", "dragon", "111111", "baseball",
        "iloveyou", "trustno1", "sunshine", "princess", "football",
        "shadow", "superman", "michael", "letmein", "welcome",
    ];
    let lower = password.to_lowercase();
    COMMON.iter().any(|&c| c == lower)
}

/// Estimate the entropy of a password in bits.
pub fn estimate_entropy(password: &str) -> f64 {
    if password.is_empty() {
        return 0.0;
    }
    let mut charset_size = 0u32;
    let mut has_lower = false;
    let mut has_upper = false;
    let mut has_digit = false;
    let mut has_symbol = false;

    for ch in password.chars() {
        if ch.is_ascii_lowercase() && !has_lower {
            charset_size += 26;
            has_lower = true;
        } else if ch.is_ascii_uppercase() && !has_upper {
            charset_size += 26;
            has_upper = true;
        } else if ch.is_ascii_digit() && !has_digit {
            charset_size += 10;
            has_digit = true;
        } else if !ch.is_ascii_alphanumeric() && !has_symbol {
            charset_size += 33;
            has_symbol = true;
        }
    }

    if charset_size == 0 {
        charset_size = 26; // fallback
    }

    (charset_size as f64).log2() * password.len() as f64
}

/// Compute SHA-1 of a password (for HIBP k-anonymity check).
pub fn sha1_hex_upper(data: &[u8]) -> String {
    let hash = ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, data);
    hex::encode_upper(hash.as_ref())
}

/// Check if a password hash appears in a simulated breach database.
pub fn check_breach_database(password: &str, breach_hashes: &[String]) -> bool {
    let hash = sha1_hex_upper(password.as_bytes());
    breach_hashes.iter().any(|bh| {
        // Constant-time comparison
        if bh.len() != hash.len() {
            return false;
        }
        let mut result = 0u8;
        for (a, b) in bh.bytes().zip(hash.bytes()) {
            result |= a ^ b;
        }
        result == 0
    })
}

/// Implement k-anonymity prefix extraction for HIBP.
pub fn hibp_prefix(password: &str) -> String {
    let hash = sha1_hex_upper(password.as_bytes());
    hash[..5].to_string()
}

/// Full password strength validation.
pub fn validate_password(password: &str) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Err(e) = check_length(password, 8, 128) {
        errors.push(e);
    }

    if is_common_password(password) {
        errors.push("Password is in the list of common passwords".to_string());
    }

    let entropy = estimate_entropy(password);
    if entropy < 40.0 {
        errors.push(format!("Password entropy too low: {:.1} bits (minimum 40)", entropy));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
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
