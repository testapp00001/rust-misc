//! # Lesson 05: Input Validation
//!
//! ## Why Validate Everything?
//!
//! Every input from the outside world is untrusted. Attackers craft malicious inputs
//! to exploit vulnerabilities:
//!
//! - **SQL Injection**: `' OR 1=1 --` in a username field
//! - **XSS**: `<script>alert('xss')</script>` in a comment field
//! - **Path Traversal**: `../../etc/passwd` in a file parameter
//! - **Buffer Overflow**: Oversized input that exceeds expected length
//! - **Type Confusion**: String where a number is expected
//!
//! ## Validation Layers
//!
//! 1. **Type validation**: Is it the right type? (string, number, boolean)
//! 2. **Format validation**: Does it match the expected pattern? (email, UUID, date)
//! 3. **Range validation**: Is it within acceptable bounds? (min/max length, value range)
//! 4. **Allowlist validation**: Is it one of the known-good values? (enum values)
//! 5. **Sanitization**: Strip or escape dangerous characters
//!
//! ## Principle: Validate on Input, Encode on Output
//!
//! - Validate all inputs at the API boundary before processing
//! - Encode all outputs when rendering (HTML encode, JSON encode, etc.)
//! - Never rely on client-side validation alone
//!
//! ## Attack Context
//!
//! - **Missing validation**: API trusts the client and processes raw input
//! - **Incomplete validation**: Checks length but not content (injection still possible)
//! - **Double encoding**: Attacker uses `%2527` which decodes to `%27` which decodes to `'`
//! - **Unicode tricks**: Homoglyphs, null bytes, overlong UTF-8 encodings

use serde::{Deserialize, Serialize};

/// Validation error with field-level detail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    /// The field that failed validation
    pub field: String,
    /// Description of the validation failure
    pub message: String,
}

/// Validation result.
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// A user registration request (for validation exercises).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub age: u32,
    pub role: String,
}

/// Exercise 1: Validate a string field's length.
///
/// Check that the value length is between min_len and max_len (inclusive).
/// Return Ok(()) if valid, or Err with a ValidationError describing the problem.
///
/// The field_name parameter is used in the error message.
pub fn validate_length(
    field_name: &str,
    value: &str,
    min_len: usize,
    max_len: usize,
) -> Result<(), ValidationError> {
    todo!("Validate string field length")
}

/// Exercise 2: Validate that a string matches a simple email pattern.
///
/// An email is valid if it:
/// - Contains exactly one '@' character
/// - Has at least one character before '@'
/// - Has at least one '.' after '@'
/// - Has at least one character between '@' and '.'
/// - Has at least one character after the last '.'
/// - Is no longer than 254 characters total
/// - Contains no spaces
///
/// This is intentionally simplified. Production systems should use a proper library.
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    todo!("Validate email format")
}

/// Exercise 3: Validate that a string contains only safe characters.
///
/// A username is safe if it contains only:
/// - ASCII alphanumeric characters (a-z, A-Z, 0-9)
/// - Underscores (_)
/// - Hyphens (-)
///
/// It must also be between 3 and 32 characters long.
pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    todo!("Validate username format and characters")
}

/// Exercise 4: Validate password strength.
///
/// A password is strong if it:
/// - Is at least 8 characters long
/// - Is no longer than 128 characters
/// - Contains at least one uppercase letter
/// - Contains at least one lowercase letter
/// - Contains at least one digit
///
/// Return all validation failures, not just the first one.
pub fn validate_password(password: &str) -> Result<(), Vec<ValidationError>> {
    todo!("Validate password strength")
}

/// Exercise 5: Validate a value is in an allowlist.
///
/// Check that the value is one of the allowed values.
/// This prevents attackers from injecting unexpected values (e.g., escalating role).
pub fn validate_allowlist(
    field_name: &str,
    value: &str,
    allowed: &[&str],
) -> Result<(), ValidationError> {
    todo!("Validate value against allowlist")
}

/// Exercise 6: Validate an entire CreateUserRequest.
///
/// Apply all validation rules:
/// - username: validate_username rules
/// - email: validate_email rules
/// - password: validate_password rules
/// - age: must be between 13 and 150
/// - role: must be in allowlist ["user", "editor", "viewer"]
///
/// Collect ALL errors across all fields and return them together.
pub fn validate_create_user_request(request: &CreateUserRequest) -> ValidationResult {
    todo!("Validate entire user creation request")
}

/// Exercise 7: Sanitize a string by removing potentially dangerous characters.
///
/// Remove:
/// - HTML tags (anything between < and >)
/// - Null bytes (\0)
/// - Control characters (ASCII 0-31, except newline \n and tab \t)
///
/// Return the sanitized string.
pub fn sanitize_string(input: &str) -> String {
    todo!("Sanitize a string by removing dangerous characters")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_length_ok() {
        assert!(validate_length("name", "hello", 1, 10).is_ok());
    }

    #[test]
    fn test_validate_length_too_short() {
        assert!(validate_length("name", "hi", 3, 10).is_err());
    }

    #[test]
    fn test_validate_length_too_long() {
        assert!(validate_length("name", "this is way too long", 1, 5).is_err());
    }

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.user@domain.co").is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(validate_email("").is_err());
        assert!(validate_email("no-at-sign").is_err());
        assert!(validate_email("@no-local.com").is_err());
        assert!(validate_email("user@no-dot").is_err());
        assert!(validate_email("user @example.com").is_err());
    }

    #[test]
    fn test_validate_username_valid() {
        assert!(validate_username("alice").is_ok());
        assert!(validate_username("user_123").is_ok());
        assert!(validate_username("my-name").is_ok());
    }

    #[test]
    fn test_validate_username_invalid() {
        assert!(validate_username("ab").is_err()); // too short
        assert!(validate_username("user name").is_err()); // space
        assert!(validate_username("user@name").is_err()); // special char
    }

    #[test]
    fn test_validate_password_strong() {
        assert!(validate_password("MyPass123").is_ok());
    }

    #[test]
    fn test_validate_password_weak() {
        let result = validate_password("weak");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Should have multiple errors: too short, no uppercase, no digit
        assert!(errors.len() >= 2);
    }

    #[test]
    fn test_validate_password_all_uppercase() {
        assert!(validate_password("ALLUPPERCASE1").is_err()); // no lowercase
    }

    #[test]
    fn test_validate_allowlist_ok() {
        assert!(validate_allowlist("role", "user", &["user", "admin"]).is_ok());
    }

    #[test]
    fn test_validate_allowlist_rejected() {
        assert!(validate_allowlist("role", "superadmin", &["user", "admin"]).is_err());
    }

    #[test]
    fn test_validate_create_user_request_valid() {
        let req = CreateUserRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "StrongPass1".to_string(),
            age: 25,
            role: "user".to_string(),
        };
        assert!(validate_create_user_request(&req).is_ok());
    }

    #[test]
    fn test_validate_create_user_request_multiple_errors() {
        let req = CreateUserRequest {
            username: "a".to_string(),           // too short
            email: "invalid".to_string(),         // no @
            password: "weak".to_string(),          // too short, no upper/digit
            age: 10,                               // too young
            role: "admin".to_string(),             // not in allowlist
        };
        let result = validate_create_user_request(&req);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 4, "Should catch multiple validation errors");
    }

    #[test]
    fn test_sanitize_html_tags() {
        let result = sanitize_string("Hello <script>alert('xss')</script> World");
        assert!(!result.contains("<script>"));
        assert!(!result.contains("</script>"));
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_sanitize_null_bytes() {
        let result = sanitize_string("hello\0world");
        assert!(!result.contains('\0'));
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_sanitize_preserves_newlines_and_tabs() {
        let result = sanitize_string("hello\nworld\t!");
        assert!(result.contains('\n'));
        assert!(result.contains('\t'));
    }
}
