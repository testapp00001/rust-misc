//! # Lesson 08: Error Aggregation Patterns
//!
//! ## The Problem
//!
//! When validation returns errors one at a time ("email is invalid", then on the next
//! attempt "password too short"), attackers can enumerate which fields have constraints
//! and what those constraints are. This is a field-by-field oracle attack.
//!
//! ## Attack Walkthrough
//!
//! ```text
//! # VULNERABLE: Sequential validation
//! POST /register {"email": "x", "password": "short"}
//! Response: "Email format is invalid"          → attacker fixes email
//!
//! POST /register {"email": "a@b.com", "password": "short"}
//! Response: "Password must be at least 8 chars" → attacker learns password policy
//!
//! POST /register {"email": "a@b.com", "password": "12345678"}
//! Response: "Password must contain a number"   → attacker learns full policy
//!
//! Result: attacker has mapped all validation rules field by field
//! ```
//!
//! ## Defense: Aggregate All Errors
//!
//! Validate ALL fields, collect ALL errors, and return them as a single response.
//! This prevents field-by-field enumeration because the attacker learns everything
//! at once -- and can't tell which error corresponds to which field.
//!
//! Better yet: return a single generic "Invalid request" for all validation failures.

/// Exercise 1: Implement aggregated validation.
///
/// Given a struct with email, password, and username fields, validate ALL fields
/// and return ALL errors as a Vec. Do NOT short-circuit on the first error.
///
/// Validation rules:
/// - email: must contain '@' and '.'
/// - password: must be >= 8 chars, contain a digit, contain a letter
/// - username: must be >= 3 chars, alphanumeric only
///
/// Return a Vec of error strings, one per failed validation.
/// Return an empty Vec if all fields are valid.
///
/// Hints:
/// - Use `let mut errors = Vec::new()` and push each failure
/// - Don't return early -- check all fields
pub fn validate_all_fields(email: &str, password: &str, username: &str) -> Vec<String> {
    todo!("Validate all fields and collect all errors")
}

/// Exercise 2: Create a generic validation error response.
///
/// Instead of returning specific field errors, return a single generic message
/// for ALL validation failures. This prevents attackers from learning which
/// fields failed.
///
/// If errors is empty, return Ok("Valid").
/// If errors is non-empty, return Err("Invalid request").
///
/// The response must NOT include any of the specific error messages.
pub fn generic_validation_response(errors: &[String]) -> Result<String, String> {
    todo!("Return generic validation error response")
}

/// Exercise 3: Create a masked validation response.
///
/// Return a response that indicates validation failed but doesn't reveal which
/// specific fields or rules failed. Include a generic error code.
///
/// Format on failure: `{"error": "Invalid request", "code": "VALIDATION_ERROR"}`
/// Format on success: `{"status": "ok"}`
pub fn masked_validation_response(errors: &[String]) -> String {
    todo!("Create masked validation response")
}

/// Exercise 4: Implement validation with error budget.
///
/// An "error budget" limits how much information validation can reveal per request.
/// Instead of returning all errors, return at most `budget` errors, randomly selected.
///
/// This makes it harder for attackers to reliably probe specific fields.
///
/// Given a list of errors and a budget (max errors to return), return at most
/// `budget` errors. If budget is 0, return an empty Vec.
///
/// For deterministic testing, always take the FIRST `budget` errors (sorted).
pub fn validate_with_budget(errors: &[String], budget: usize) -> Vec<String> {
    todo!("Limit error information with a budget")
}

/// Exercise 5: Detect if validation errors leak field information.
///
/// Given a list of error messages, check if any of them mention specific field names.
/// Field names to check: "email", "password", "username", "phone", "name", "address".
///
/// Return true if any error message contains a field name (the errors are leaking info).
/// Return false if all errors are generic.
pub fn errors_leak_field_info(errors: &[String]) -> bool {
    todo!("Check if validation errors reveal field names")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_all_fields_returns_multiple_errors() {
        let errors = validate_all_fields("invalid", "short", "ab");
        assert!(errors.len() >= 2, "Should return multiple errors for multiple invalid fields: {:?}", errors);
    }

    #[test]
    fn test_validate_all_fields_valid_input() {
        let errors = validate_all_fields("test@example.com", "password1", "alice");
        assert!(errors.is_empty(), "Valid input should produce no errors: {:?}", errors);
    }

    #[test]
    fn test_validate_all_fields_does_not_short_circuit() {
        // Email is invalid, password is also invalid -- both should be caught
        let errors = validate_all_fields("invalid", "short", "alice");
        let has_email_error = errors.iter().any(|e| e.to_lowercase().contains("email"));
        let has_password_error = errors.iter().any(|e| e.to_lowercase().contains("password"));
        assert!(has_email_error, "Should catch email error");
        assert!(has_password_error, "Should catch password error");
    }

    #[test]
    fn test_generic_response_no_details() {
        let errors = vec!["email is invalid".to_string(), "password too short".to_string()];
        let response = generic_validation_response(&errors).unwrap_err();
        assert!(!response.contains("email"), "Generic response must not contain field names");
        assert!(!response.contains("password"), "Generic response must not contain field names");
    }

    #[test]
    fn test_generic_response_success() {
        let errors: Vec<String> = vec![];
        let response = generic_validation_response(&errors).unwrap();
        assert_eq!(response, "Valid");
    }

    #[test]
    fn test_masked_response_failure() {
        let errors = vec!["email invalid".to_string()];
        let response = masked_validation_response(&errors);
        assert!(response.contains("VALIDATION_ERROR"), "Should contain error code");
        assert!(!response.contains("email"), "Should not contain field names");
    }

    #[test]
    fn test_budget_limits_errors() {
        let errors = vec![
            "error1".to_string(),
            "error2".to_string(),
            "error3".to_string(),
            "error4".to_string(),
        ];
        let limited = validate_with_budget(&errors, 2);
        assert!(limited.len() <= 2, "Budget should limit error count");
    }

    #[test]
    fn test_budget_zero_returns_empty() {
        let errors = vec!["error1".to_string(), "error2".to_string()];
        let limited = validate_with_budget(&errors, 0);
        assert!(limited.is_empty(), "Budget of 0 should return no errors");
    }

    #[test]
    fn test_detects_field_name_leak() {
        let errors = vec![
            "email format is invalid".to_string(),
            "password too short".to_string(),
        ];
        assert!(errors_leak_field_info(&errors));
    }

    #[test]
    fn test_no_field_name_leak() {
        let errors = vec![
            "Invalid format".to_string(),
            "Too short".to_string(),
        ];
        assert!(!errors_leak_field_info(&errors));
    }
}
