//! # Lesson 07: Code Review Checklist (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Validate input length against security constraints.
pub fn validate_input_length(input: &str, min_len: usize, max_len: usize) -> Result<(), String> {
    let len = input.len();
    if len < min_len {
        Err(format!("Input too short: {} chars (minimum {})", len, min_len))
    } else if len > max_len {
        Err(format!("Input too long: {} chars (maximum {})", len, max_len))
    } else {
        Ok(())
    }
}

/// Sanitize error message to remove sensitive information.
pub fn sanitize_error_message(message: &str) -> String {
    let mut result = String::with_capacity(message.len());
    let chars: Vec<char> = message.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for file path (contains / or \)
        if chars[i] == '/' || chars[i] == '\\' {
            // Find the extent of the path-like segment
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '/' || chars[i] == '\\' || chars[i] == '.' || chars[i] == '_' || chars[i] == '-') {
                i += 1;
            }
            if i - start > 2 {
                result.push_str("[REDACTED]");
            } else {
                for ch in &chars[start..i] {
                    result.push(*ch);
                }
            }
            continue;
        }

        // Check for IP address pattern (digits and dots)
        if chars[i].is_ascii_digit() {
            let mut dot_count = 0;
            let mut digit_groups = 0;
            let mut current_digits = 0;

            let mut j = i;

            while j < chars.len() {
                if chars[j].is_ascii_digit() {
                    current_digits += 1;
                    j += 1;
                } else if chars[j] == '.' && current_digits > 0 && current_digits <= 3 {
                    dot_count += 1;
                    digit_groups += 1;
                    current_digits = 0;
                    j += 1;
                } else {
                    break;
                }
            }
            if current_digits > 0 {
                digit_groups += 1;
            }

            if dot_count == 3 && digit_groups == 4 {
                result.push_str("[REDACTED]");
                i = j;
            } else {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Validate password meets complexity requirements.
pub fn validate_password_complexity(password: &str, min_length: usize) -> Result<(), String> {
    let mut errors = Vec::new();

    if password.len() < min_length {
        errors.push(format!("at least {} characters", min_length));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push("at least one uppercase letter".to_string());
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push("at least one lowercase letter".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("at least one digit".to_string());
    }
    if !password.chars().any(|c| "!@#$%^&*".contains(c)) {
        errors.push("at least one special character (!@#$%^&*)".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("Password requirements not met: {}", errors.join("; ")))
    }
}

/// Validate token format.
pub fn is_valid_token_format(token: &str) -> bool {
    token.len() >= 32
        && !token.contains(' ')
        && token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Detect potential SQL injection patterns.
pub fn has_sql_injection_risk(query: &str) -> bool {
    let upper = query.to_uppercase();
    upper.contains("DROP TABLE")
        || upper.contains("UNION SELECT")
        || upper.contains("OR 1=1")
        || upper.contains("--")
        || upper.contains("/*")
        || upper.contains("*/")
}

/// Validate cryptographic parameter minimums.
pub fn validate_crypto_params(algorithm: &str, key_bits: u32, min_key_bits: u32) -> Result<(), String> {
    let upper = algorithm.to_uppercase();

    if upper.contains("RSA") && key_bits < 2048 {
        return Err(format!(
            "RSA key size {} is below minimum 2048 bits",
            key_bits
        ));
    }

    if (upper.contains("ECDSA") || upper.contains("ED25519")) && key_bits < 256 {
        return Err(format!(
            "EC key size {} is below minimum 256 bits",
            key_bits
        ));
    }

    if key_bits < min_key_bits {
        return Err(format!(
            "Key size {} is below minimum {} bits",
            key_bits, min_key_bits
        ));
    }

    Ok(())
}

/// Calculate security review score.
pub fn security_review_score(critical: u32, high: u32, medium: u32, low: u32) -> u32 {
    let deduction = critical.saturating_mul(25)
        + high.saturating_mul(10)
        + medium.saturating_mul(3)
        + low.saturating_mul(1);
    100u32.saturating_sub(deduction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_input_length_valid() {
        assert!(validate_input_length("hello", 1, 100).is_ok());
    }

    #[test]
    fn test_validate_input_length_too_short() {
        assert!(validate_input_length("hi", 8, 100).is_err());
    }

    #[test]
    fn test_validate_input_length_too_long() {
        assert!(validate_input_length(&"a".repeat(200), 1, 100).is_err());
    }

    #[test]
    fn test_sanitize_error_message_path() {
        let msg = "Error reading /etc/passwd";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("/etc/passwd"));
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_error_message_ip() {
        let msg = "Connection to 192.168.1.1 failed";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("192.168.1.1"));
    }

    #[test]
    fn test_sanitize_error_message_clean() {
        let msg = "Invalid input provided";
        let sanitized = sanitize_error_message(msg);
        assert_eq!(sanitized, msg);
    }

    #[test]
    fn test_validate_password_complexity_strong() {
        assert!(validate_password_complexity("MyP@ssw0rd!", 8).is_ok());
    }

    #[test]
    fn test_validate_password_complexity_no_uppercase() {
        assert!(validate_password_complexity("myp@ssw0rd!", 8).is_err());
    }

    #[test]
    fn test_validate_password_complexity_too_short() {
        assert!(validate_password_complexity("Ab1!", 8).is_err());
    }

    #[test]
    fn test_validate_password_complexity_no_special() {
        assert!(validate_password_complexity("MyPassw0rd1", 8).is_err());
    }

    #[test]
    fn test_is_valid_token_format_valid() {
        assert!(is_valid_token_format("abc123def456ghi789jkl012mno345pq"));
    }

    #[test]
    fn test_is_valid_token_format_with_dashes() {
        assert!(is_valid_token_format("abc-def-ghi-jkl-mno-pqr-stu-vwx-yz0"));
    }

    #[test]
    fn test_is_valid_token_format_too_short() {
        assert!(!is_valid_token_format("short"));
    }

    #[test]
    fn test_is_valid_token_format_with_spaces() {
        assert!(!is_valid_token_format("abc def ghi jkl mno pqr stu vwx yz01"));
    }

    #[test]
    fn test_has_sql_injection_risk_union() {
        assert!(has_sql_injection_risk("SELECT * FROM users UNION SELECT * FROM passwords"));
    }

    #[test]
    fn test_has_sql_injection_risk_comment() {
        assert!(has_sql_injection_risk("SELECT * FROM users -- WHERE id = 1"));
    }

    #[test]
    fn test_has_sql_injection_risk_or_1_1() {
        assert!(has_sql_injection_risk("SELECT * FROM users WHERE id = 1 OR 1=1"));
    }

    #[test]
    fn test_has_sql_injection_risk_clean() {
        assert!(!has_sql_injection_risk("SELECT * FROM users WHERE id = ?"));
    }

    #[test]
    fn test_validate_crypto_params_aes() {
        assert!(validate_crypto_params("AES-256-GCM", 256, 128).is_ok());
    }

    #[test]
    fn test_validate_crypto_params_aes_too_small() {
        assert!(validate_crypto_params("AES-64", 64, 128).is_err());
    }

    #[test]
    fn test_validate_crypto_params_rsa() {
        assert!(validate_crypto_params("RSA", 2048, 128).is_ok());
    }

    #[test]
    fn test_validate_crypto_params_rsa_too_small() {
        assert!(validate_crypto_params("RSA", 1024, 128).is_err());
    }

    #[test]
    fn test_security_review_score_clean() {
        assert_eq!(security_review_score(0, 0, 0, 0), 100);
    }

    #[test]
    fn test_security_review_score_critical() {
        assert_eq!(security_review_score(1, 0, 0, 0), 75);
    }

    #[test]
    fn test_security_review_score_clamped() {
        assert_eq!(security_review_score(10, 10, 10, 10), 0);
    }

    #[test]
    fn test_security_review_score_mixed() {
        // 100 - (1*25) - (2*10) - (3*3) - (4*1) = 100 - 25 - 20 - 9 - 4 = 42
        assert_eq!(security_review_score(1, 2, 3, 4), 42);
    }
}
