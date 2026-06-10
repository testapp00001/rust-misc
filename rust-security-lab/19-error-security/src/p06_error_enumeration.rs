//! # Lesson 06: Error Enumeration Defense
//!
//! ## The Problem
//!
//! Error enumeration attacks go beyond just "user not found" vs "wrong password."
//! Attackers probe multiple endpoints and failure modes to enumerate system state:
//!
//! - Registration: "email already taken" reveals registered emails
//! - Password reset: "no account with this email" reveals email ownership
//! - Login: different delays for valid/invalid usernames
//! - API keys: "key expired" vs "key invalid" reveals key format
//! - Roles: "insufficient permissions (need admin)" reveals role structure
//!
//! ## Defense: Generic Error Classes
//!
//! Group all security-sensitive errors into broad categories that reveal nothing:
//!
//! | Category | Covers | User Message |
//! |----------|--------|-------------|
//! | Auth | Login, token, API key, session | "Authentication failed" |
//! | Permission | Role check, ACL, ownership | "Access denied" |
//! | Input | Validation, format, bounds | "Invalid request" |
//! | Resource | Not found, deleted, locked | "Resource not available" |
//! | Rate | Throttle, quota, burst | "Too many requests" |

use std::collections::HashMap;

/// Generic error class that reveals nothing about the specific failure.
#[derive(Debug, Clone, PartialEq)]
pub enum SecureErrorClass {
    /// All authentication failures (bad password, bad token, expired session, etc.)
    Auth,
    /// All authorization failures (wrong role, not owner, etc.)
    Permission,
    /// All input validation failures (format, length, range, etc.)
    Input,
    /// All resource access failures (not found, deleted, locked, etc.)
    Resource,
    /// Rate limiting
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

/// Exercise 1: Map internal error reasons to generic error classes.
///
/// Given an internal error reason string, classify it into the correct
/// `SecureErrorClass`. The mapping must be:
///
/// - Contains "password" or "credential" or "token" or "session" or "api_key" → Auth
/// - Contains "permission" or "role" or "admin" or "owner" → Permission
/// - Contains "invalid" or "format" or "length" or "range" or "required" → Input
/// - Contains "not found" or "deleted" or "locked" or "expired" (resource) → Resource
/// - Contains "rate" or "throttle" or "quota" → RateLimit
/// - Default → Auth (fail closed -- assume auth issue if unknown)
///
/// Hints:
/// - Convert to lowercase for matching
/// - Use `contains()` on the string
/// - The order of checks doesn't matter since categories don't overlap
pub fn classify_error(reason: &str) -> SecureErrorClass {
    todo!("Classify internal error reason into generic error class")
}

/// Exercise 2: Create a unified authentication error response.
///
/// Given ANY authentication-related failure, return the same response structure:
/// `{"error": "Authentication failed", "code": "AUTH_FAILED"}`
///
/// The function must ignore the specific reason and always return the same JSON.
pub fn unified_auth_error(_reason: &str) -> String {
    todo!("Return unified auth error response")
}

/// Exercise 3: Validate that error responses don't leak enumeration info.
///
/// Given a list of error responses (strings), check if ALL responses are identical.
/// If any two differ, the system is leaking enumeration information.
///
/// Return Ok(()) if all identical, Err(pair_description) if any differ.
pub fn validate_no_enumeration(responses: &[String]) -> Result<(), String> {
    todo!("Check if all error responses are identical")
}

/// Exercise 4: Implement secure registration error handling.
///
/// Registration must handle these cases without leaking which one occurred:
/// - Email already registered
/// - Username already taken
/// - Invalid email format
/// - Password too short
///
/// ALL cases must return: "Registration could not be completed"
/// Return Ok(message) always. The bool in the tuple indicates if registration actually happened.
pub fn secure_register(
    db: &mut HashMap<String, String>,
    username: &str,
    email: &str,
    password: &str,
) -> (bool, String) {
    todo!("Implement secure registration with uniform error handling")
}

/// Exercise 5: Implement secure API key validation.
///
/// API key validation must return the same error for ALL failure modes:
/// - Key not found
/// - Key expired
/// - Key revoked
/// - Key format invalid
///
/// Always return Err("Invalid API key") regardless of the specific failure.
pub fn validate_api_key(
    keys: &HashMap<String, (bool, bool)>, // key -> (is_valid, is_expired)
    api_key: &str,
) -> Result<&'static str, String> {
    todo!("Validate API key with uniform error handling")
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
