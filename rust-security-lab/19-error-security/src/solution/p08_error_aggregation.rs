//! # Lesson 08: Error Aggregation Patterns (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Validate all fields and collect all errors.
pub fn validate_all_fields(email: &str, password: &str, username: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // Email validation
    if !email.contains('@') || !email.contains('.') {
        errors.push("Email format is invalid".to_string());
    }

    // Password validation
    if password.len() < 8 {
        errors.push("Password must be at least 8 characters".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("Password must contain a digit".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_alphabetic()) {
        errors.push("Password must contain a letter".to_string());
    }

    // Username validation
    if username.len() < 3 {
        errors.push("Username must be at least 3 characters".to_string());
    }
    if !username.chars().all(|c| c.is_ascii_alphanumeric()) {
        errors.push("Username must be alphanumeric".to_string());
    }

    errors
}

/// Return generic validation error response.
pub fn generic_validation_response(errors: &[String]) -> Result<String, String> {
    if errors.is_empty() {
        Ok("Valid".to_string())
    } else {
        Err("Invalid request".to_string())
    }
}

/// Create masked validation response.
pub fn masked_validation_response(errors: &[String]) -> String {
    if errors.is_empty() {
        r#"{"status": "ok"}"#.to_string()
    } else {
        r#"{"error": "Invalid request", "code": "VALIDATION_ERROR"}"#.to_string()
    }
}

/// Validate with error budget -- limit information disclosure.
pub fn validate_with_budget(errors: &[String], budget: usize) -> Vec<String> {
    errors.iter().take(budget).cloned().collect()
}

/// Check if validation errors reveal field names.
pub fn errors_leak_field_info(errors: &[String]) -> bool {
    let field_names = ["email", "password", "username", "phone", "name", "address"];

    for error in errors {
        let lower = error.to_lowercase();
        for field in &field_names {
            if lower.contains(field) {
                return true;
            }
        }
    }
    false
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
