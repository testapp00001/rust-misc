//! # Lesson 07: Security Code Review Checklist
//!
//! ## The Problem
//!
//! Code reviews catch bugs, but security bugs require a different lens. A typical
//! reviewer checks "does it work?" A security reviewer asks "how can it fail
//! under adversarial conditions?"
//!
//! ## The Security Review Checklist
//!
//! ### Input Validation
//! - [ ] All external input is validated before use
//! - [ ] Validation happens on the server, not just the client
//! - [ ] Length limits are enforced on all string/blob inputs
//!
//! ### Error Handling
//! - [ ] No `.unwrap()` or `.expect()` in production code paths
//! - [ ] Error messages do not leak sensitive information
//! - [ ] Errors are logged with context but without secrets
//!
//! ### Authentication & Authorization
//! - [ ] All endpoints check authentication
//! - [ ] Authorization is checked AFTER authentication
//! - [ ] Token validation is constant-time
//!
//! ### Cryptography
//! - [ ] No homegrown crypto algorithms
//! - [ ] Keys are generated with CSPRNG
//! - [ ] Nonces/IVs are never reused
//! - [ ] Secrets are zeroized after use
//!
//! ### Memory Safety
//! - [ ] `unsafe` blocks have safety comments
//! - [ ] `unsafe` is minimized and audited
//! - [ ] Buffers are bounds-checked
//!
//! ## Exercise
//!
//! Implement functions that perform automated checks from the security review
//! checklist. In a real CI pipeline, these would be custom lint rules or
//! pre-commit hooks.

/// Exercise 1: Check if a string input meets security length constraints.
///
/// Requirements:
/// - Return `Ok(())` if `input.len()` is between `min_len` and `max_len` (inclusive)
/// - Return `Err(String)` with a descriptive message if violated
/// - The error message should include the actual length and the allowed range
pub fn validate_input_length(input: &str, min_len: usize, max_len: usize) -> Result<(), String> {
    todo!("Validate input length against security constraints")
}

/// Exercise 2: Sanitize an error message to remove sensitive information.
///
/// Requirements:
/// - Remove any substring that looks like a file path (contains '/' or '\')
/// - Remove any substring that looks like an IP address (digits and dots pattern)
/// - Replace removed content with "[REDACTED]"
/// - This prevents error messages from leaking internal paths or IPs
pub fn sanitize_error_message(message: &str) -> String {
    todo!("Sanitize error message to remove sensitive information")
}

/// Exercise 3: Check if a password meets complexity requirements.
///
/// Requirements:
/// - Must be at least `min_length` characters
/// - Must contain at least one uppercase letter
/// - Must contain at least one lowercase letter
/// - Must contain at least one digit
/// - Must contain at least one special character (!@#$%^&*)
/// - Return `Ok(())` if all requirements met, `Err(String)` listing what is missing
pub fn validate_password_complexity(password: &str, min_length: usize) -> Result<(), String> {
    todo!("Validate password meets complexity requirements")
}

/// Exercise 4: Check that a token format is valid (basic structure check).
///
/// Requirements:
/// - Token must be at least 32 characters
/// - Token must contain only alphanumeric characters, '-', '_', and '.'
/// - Token must not contain spaces
/// - Return `true` if valid format, `false` otherwise
pub fn is_valid_token_format(token: &str) -> bool {
    todo!("Validate token format")
}

/// Exercise 5: Detect potential SQL injection patterns in a query string.
///
/// Requirements:
/// - Return `true` if the query contains suspicious patterns:
///   - String concatenation with "+" followed by a quote
///   - Comment sequences: "--", "/*", "*/"
///   - Common injection keywords: "DROP TABLE", "UNION SELECT", "OR 1=1"
/// - Case-insensitive matching
/// - Return `false` if no suspicious patterns found
pub fn has_sql_injection_risk(query: &str) -> bool {
    todo!("Detect potential SQL injection patterns")
}

/// Exercise 6: Check that cryptographic parameters meet minimum requirements.
///
/// Requirements:
/// - Key size must be at least `min_key_bits` (e.g., 128)
/// - If `algorithm` contains "RSA", key size must be at least 2048
/// - If `algorithm` contains "ECDSA" or "Ed25519", key size must be at least 256
/// - Return `Ok(())` if requirements met, `Err(String)` if not
pub fn validate_crypto_params(algorithm: &str, key_bits: u32, min_key_bits: u32) -> Result<(), String> {
    todo!("Validate cryptographic parameter minimums")
}

/// Exercise 7: Generate a security review score.
///
/// Requirements:
/// - Take counts of: `critical`, `high`, `medium`, `low` findings
/// - Score = 100 - (critical * 25) - (high * 10) - (medium * 3) - (low * 1)
/// - Minimum score is 0
/// - Return the score clamped to [0, 100]
pub fn security_review_score(critical: u32, high: u32, medium: u32, low: u32) -> u32 {
    todo!("Calculate security review score")
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
