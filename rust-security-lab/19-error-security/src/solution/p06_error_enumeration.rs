//! # Lesson 06: Error Enumeration Defense (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;

/// Generic error class that reveals nothing about the specific failure.
#[derive(Debug, Clone, PartialEq)]
pub enum SecureErrorClass {
    Auth,
    Permission,
    Input,
    Resource,
    RateLimit,
}

impl SecureErrorClass {
    pub fn user_message(&self) -> &'static str {
        match self {
            SecureErrorClass::Auth => "Authentication failed",
            SecureErrorClass::Permission => "Access denied",
            SecureErrorClass::Input => "Invalid request",
            SecureErrorClass::Resource => "Resource not available",
            SecureErrorClass::RateLimit => "Too many requests",
        }
    }
}

/// Classify internal error reason into generic error class.
pub fn classify_error(reason: &str) -> SecureErrorClass {
    let lower = reason.to_lowercase();

    if lower.contains("password") || lower.contains("credential")
        || lower.contains("token") || lower.contains("session")
        || lower.contains("api_key")
    {
        SecureErrorClass::Auth
    } else if lower.contains("permission") || lower.contains("role")
        || lower.contains("admin") || lower.contains("owner")
    {
        SecureErrorClass::Permission
    } else if lower.contains("invalid") || lower.contains("format")
        || lower.contains("length") || lower.contains("range")
        || lower.contains("required")
    {
        SecureErrorClass::Input
    } else if lower.contains("not found") || lower.contains("deleted")
        || lower.contains("locked") || lower.contains("expired")
    {
        SecureErrorClass::Resource
    } else if lower.contains("rate") || lower.contains("throttle")
        || lower.contains("quota")
    {
        SecureErrorClass::RateLimit
    } else {
        SecureErrorClass::Auth // Fail closed
    }
}

/// Return unified auth error response.
pub fn unified_auth_error(_reason: &str) -> String {
    r#"{"error": "Authentication failed", "code": "AUTH_FAILED"}"#.to_string()
}

/// Check if all error responses are identical.
pub fn validate_no_enumeration(responses: &[String]) -> Result<(), String> {
    if responses.len() < 2 {
        return Ok(());
    }
    let first = &responses[0];
    for (i, response) in responses.iter().enumerate().skip(1) {
        if response != first {
            return Err(format!(
                "Enumeration leak: response[0]='{}' differs from response[{}]='{}'",
                first, i, response
            ));
        }
    }
    Ok(())
}

/// Secure registration with uniform error handling.
pub fn secure_register(
    db: &mut HashMap<String, String>,
    username: &str,
    email: &str,
    _password: &str,
) -> (bool, String) {
    let message = "Registration could not be completed".to_string();

    // Check if email already registered
    if db.contains_key(email) {
        return (false, message);
    }

    // Check if username already taken (check all values)
    if db.values().any(|u| u == username) {
        return (false, message);
    }

    // All validation passed -- register
    // Always return the same generic message to prevent enumeration
    db.insert(email.to_string(), username.to_string());
    (true, message)
}

/// Validate API key with uniform error handling.
pub fn validate_api_key(
    keys: &HashMap<String, (bool, bool)>, // key -> (is_valid, is_expired)
    api_key: &str,
) -> Result<&'static str, String> {
    match keys.get(api_key) {
        Some((is_valid, is_expired)) => {
            if *is_valid && !*is_expired {
                Ok("Key valid")
            } else {
                Err("Invalid API key".to_string())
            }
        }
        None => Err("Invalid API key".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_password_error() {
        assert_eq!(classify_error("Invalid password"), SecureErrorClass::Auth);
    }

    #[test]
    fn test_classify_token_error() {
        assert_eq!(classify_error("Token expired"), SecureErrorClass::Auth);
    }

    #[test]
    fn test_classify_permission_error() {
        assert_eq!(classify_error("Insufficient role"), SecureErrorClass::Permission);
    }

    #[test]
    fn test_classify_input_error() {
        assert_eq!(classify_error("Invalid format"), SecureErrorClass::Input);
    }

    #[test]
    fn test_classify_not_found_error() {
        assert_eq!(classify_error("Resource not found"), SecureErrorClass::Resource);
    }

    #[test]
    fn test_classify_unknown_defaults_to_auth() {
        assert_eq!(classify_error("Something weird happened"), SecureErrorClass::Auth);
    }

    #[test]
    fn test_unified_auth_error_same_for_all_reasons() {
        let r1 = unified_auth_error("user not found");
        let r2 = unified_auth_error("wrong password");
        let r3 = unified_auth_error("account disabled");
        assert_eq!(r1, r2, "Auth errors must be identical");
        assert_eq!(r2, r3, "Auth errors must be identical");
    }

    #[test]
    fn test_validate_no_enumeration_secure() {
        let responses = vec![
            "Authentication failed".to_string(),
            "Authentication failed".to_string(),
            "Authentication failed".to_string(),
        ];
        assert!(validate_no_enumeration(&responses).is_ok());
    }

    #[test]
    fn test_validate_no_enumeration_detects_leak() {
        let responses = vec![
            "User not found".to_string(),
            "Wrong password".to_string(),
        ];
        assert!(validate_no_enumeration(&responses).is_err());
    }

    #[test]
    fn test_secure_register_same_message() {
        let mut db = HashMap::new();
        db.insert("alice@example.com".to_string(), "alice".to_string());

        let (_, msg1) = secure_register(&mut db, "alice", "alice@example.com", "password123");
        let (_, msg2) = secure_register(&mut db, "bob", "bob@example.com", "password123");
        assert_eq!(msg1, msg2, "Registration messages must be identical");
    }

    #[test]
    fn test_api_key_uniform_error() {
        let mut keys = HashMap::new();
        keys.insert("valid_key".to_string(), (true, false));
        keys.insert("expired_key".to_string(), (true, true));
        keys.insert("revoked_key".to_string(), (false, false));

        let r1 = validate_api_key(&keys, "nonexistent_key").unwrap_err();
        let r2 = validate_api_key(&keys, "expired_key").unwrap_err();
        let r3 = validate_api_key(&keys, "revoked_key").unwrap_err();
        assert_eq!(r1, r2, "API key errors must be identical");
        assert_eq!(r2, r3, "API key errors must be identical");
    }

    #[test]
    fn test_api_key_valid_still_works() {
        let mut keys = HashMap::new();
        keys.insert("valid_key".to_string(), (true, false));

        let result = validate_api_key(&keys, "valid_key");
        assert!(result.is_ok(), "Valid key must succeed");
    }
}
