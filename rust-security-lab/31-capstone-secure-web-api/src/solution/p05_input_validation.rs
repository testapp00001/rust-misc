//! # Lesson 05: Input Validation — Solution
//!
//! Request body, query params, header validation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub type ValidationResult = Result<(), Vec<ValidationError>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub age: u32,
    pub role: String,
}

pub fn validate_length(
    field_name: &str,
    value: &str,
    min_len: usize,
    max_len: usize,
) -> Result<(), ValidationError> {
    let len = value.len();
    if len < min_len {
        Err(ValidationError {
            field: field_name.to_string(),
            message: format!("Must be at least {} characters, got {}", min_len, len),
        })
    } else if len > max_len {
        Err(ValidationError {
            field: field_name.to_string(),
            message: format!("Must be at most {} characters, got {}", max_len, len),
        })
    } else {
        Ok(())
    }
}

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    let field = "email";

    if email.is_empty() {
        return Err(ValidationError {
            field: field.to_string(),
            message: "Email cannot be empty".to_string(),
        });
    }

    if email.len() > 254 {
        return Err(ValidationError {
            field: field.to_string(),
            message: "Email must be at most 254 characters".to_string(),
        });
    }

    if email.contains(' ') {
        return Err(ValidationError {
            field: field.to_string(),
            message: "Email cannot contain spaces".to_string(),
        });
    }

    let at_count = email.matches('@').count();
    if at_count != 1 {
        return Err(ValidationError {
            field: field.to_string(),
            message: "Email must contain exactly one '@'".to_string(),
        });
    }

    let parts: Vec<&str> = email.split('@').collect();
    let local = parts[0];
    let domain = parts[1];

    if local.is_empty() {
        return Err(ValidationError {
            field: field.to_string(),
            message: "Email must have a local part before '@'".to_string(),
        });
    }

    if domain.is_empty() {
        return Err(ValidationError {
            field: field.to_string(),
            message: "Email must have a domain after '@'".to_string(),
        });
    }

    let last_dot = domain.rfind('.');
    match last_dot {
        Some(pos) => {
            if pos == 0 || pos == domain.len() - 1 {
                return Err(ValidationError {
                    field: field.to_string(),
                    message: "Email domain must have valid format".to_string(),
                });
            }
        }
        None => {
            return Err(ValidationError {
                field: field.to_string(),
                message: "Email domain must contain a '.'".to_string(),
            });
        }
    }

    Ok(())
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    let field = "username";

    if username.len() < 3 || username.len() > 32 {
        return Err(ValidationError {
            field: field.to_string(),
            message: format!(
                "Username must be 3-32 characters, got {}",
                username.len()
            ),
        });
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ValidationError {
            field: field.to_string(),
            message: "Username can only contain letters, digits, underscores, and hyphens"
                .to_string(),
        });
    }

    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if password.len() < 8 {
        errors.push(ValidationError {
            field: "password".to_string(),
            message: "Password must be at least 8 characters".to_string(),
        });
    }

    if password.len() > 128 {
        errors.push(ValidationError {
            field: "password".to_string(),
            message: "Password must be at most 128 characters".to_string(),
        });
    }

    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        errors.push(ValidationError {
            field: "password".to_string(),
            message: "Password must contain at least one uppercase letter".to_string(),
        });
    }

    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        errors.push(ValidationError {
            field: "password".to_string(),
            message: "Password must contain at least one lowercase letter".to_string(),
        });
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push(ValidationError {
            field: "password".to_string(),
            message: "Password must contain at least one digit".to_string(),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_allowlist(
    field_name: &str,
    value: &str,
    allowed: &[&str],
) -> Result<(), ValidationError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(ValidationError {
            field: field_name.to_string(),
            message: format!(
                "'{}' is not allowed. Expected one of: {:?}",
                value, allowed
            ),
        })
    }
}

pub fn validate_create_user_request(request: &CreateUserRequest) -> ValidationResult {
    let mut errors = Vec::new();

    if let Err(e) = validate_username(&request.username) {
        errors.push(e);
    }

    if let Err(e) = validate_email(&request.email) {
        errors.push(e);
    }

    if let Err(password_errors) = validate_password(&request.password) {
        errors.extend(password_errors);
    }

    if request.age < 13 {
        errors.push(ValidationError {
            field: "age".to_string(),
            message: format!("Must be at least 13 years old, got {}", request.age),
        });
    }

    if request.age > 150 {
        errors.push(ValidationError {
            field: "age".to_string(),
            message: format!("Invalid age: {}", request.age),
        });
    }

    if let Err(e) = validate_allowlist("role", &request.role, &["user", "editor", "viewer"]) {
        errors.push(e);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn sanitize_string(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            '\0' => {}
            _c if in_tag => {}
            c if c.is_control() && c != '\n' && c != '\t' => {}
            c => result.push(c),
        }
    }

    result
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
        assert!(validate_username("ab").is_err());
        assert!(validate_username("user name").is_err());
        assert!(validate_username("user@name").is_err());
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
        assert!(errors.len() >= 2);
    }

    #[test]
    fn test_validate_password_all_uppercase() {
        assert!(validate_password("ALLUPPERCASE1").is_err());
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
            username: "a".to_string(),
            email: "invalid".to_string(),
            password: "weak".to_string(),
            age: 10,
            role: "admin".to_string(),
        };
        let result = validate_create_user_request(&req);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 4);
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
