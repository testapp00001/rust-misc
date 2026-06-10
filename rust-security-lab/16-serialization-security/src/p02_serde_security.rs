//! # Lesson 02: Serde Security
//!
//! ## deny_unknown_fields
//!
//! By default, serde silently ignores fields that are not present in your struct.
//! This is convenient but dangerous: an attacker can inject arbitrary fields that
//! a downstream consumer might interpret differently.
//!
//! ```rust
//! #[derive(Deserialize)]
//! #[serde(deny_unknown_fields)]
//! struct LoginRequest {
//!     username: String,
//!     password: String,
//! }
//! ```
//!
//! With this attribute, `{"username":"alice","password":"s3cret","admin":true}`
//! will be rejected instead of silently ignoring `admin`.
//!
//! ## Validate After Deserialization
//!
//! Deserialization should parse bytes into a struct. Validation should check
//! that the values make sense. These are separate concerns:
//!
//! ```rust
//! // Step 1: Deserialize (parse structure)
//! let input: LoginRequest = serde_json::from_str(&raw)?;
//!
//! // Step 2: Validate (check business rules)
//! validate_login(&input)?;
//! ```
//!
//! ## Why This Matters
//!
//! If you validate inside a custom `Deserialize` impl, the validation logic is
//! hidden from auditors and cannot be reused across formats. By validating
//! separately, you create a clear security boundary that can be tested, audited,
//! and shared across JSON, XML, and Protobuf inputs.

use serde::Deserialize;

/// A login request with strict field requirements.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Exercise 1: Parse a login request with deny_unknown_fields.
///
/// Parse the JSON string into a LoginRequest. Return Ok(request) on success,
/// or an error message string on failure.
pub fn parse_login_request(json: &str) -> Result<LoginRequest, String> {
    todo!("Parse LoginRequest with deny_unknown_fields")
}

/// Exercise 2: Check if a JSON string would be rejected due to unknown fields.
///
/// Return `Ok(true)` if the JSON contains unknown fields (parsing fails),
/// `Ok(false)` if it parses successfully.
pub fn has_unknown_fields(json: &str) -> Result<bool, String> {
    todo!("Check for unknown fields")
}

/// Exercise 3: Validate a LoginRequest after deserialization.
///
/// Business rules:
/// - username must not be empty
/// - username must be at most 64 characters
/// - password must be at least 8 characters
/// - password must be at most 128 characters
///
/// Return Ok(()) if valid, Err(message) if invalid.
pub fn validate_login(request: &LoginRequest) -> Result<(), String> {
    todo!("Validate LoginRequest business rules")
}

/// Exercise 4: Combined parse and validate.
///
/// Parse the JSON string into a LoginRequest, then validate it.
/// Return Ok(request) if both steps succeed, Err(message) otherwise.
pub fn parse_and_validate_login(json: &str) -> Result<LoginRequest, String> {
    todo!("Parse and validate login request")
}

/// Exercise 5: A registration request with more fields and validation.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegistrationRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

/// Validate a registration request:
/// - username: 3-64 chars, alphanumeric + underscore only
/// - email: must contain '@' and '.'
/// - password: at least 12 chars
/// - confirm_password must match password
pub fn validate_registration(request: &RegistrationRequest) -> Result<(), String> {
    todo!("Validate RegistrationRequest")
}

/// Parse and validate a registration request from JSON.
pub fn parse_and_validate_registration(json: &str) -> Result<RegistrationRequest, String> {
    todo!("Parse and validate registration request")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_login() {
        let json = r#"{"username": "alice", "password": "s3cretP@ss"}"#;
        let req = parse_login_request(json).unwrap();
        assert_eq!(req.username, "alice");
    }

    #[test]
    fn test_unknown_fields_detected() {
        let json = r#"{"username": "alice", "password": "s3cret", "admin": true}"#;
        assert!(has_unknown_fields(json).unwrap());
    }

    #[test]
    fn test_no_unknown_fields() {
        let json = r#"{"username": "alice", "password": "s3cretP@ss"}"#;
        assert!(!has_unknown_fields(json).unwrap());
    }

    #[test]
    fn test_validate_empty_username() {
        let req = LoginRequest {
            username: "".to_string(),
            password: "longenoughpass".to_string(),
        };
        assert!(validate_login(&req).is_err());
    }

    #[test]
    fn test_validate_short_password() {
        let req = LoginRequest {
            username: "alice".to_string(),
            password: "short".to_string(),
        };
        assert!(validate_login(&req).is_err());
    }

    #[test]
    fn test_parse_and_validate_success() {
        let json = r#"{"username": "alice", "password": "s3cretP@ssw0rd"}"#;
        let req = parse_and_validate_login(json).unwrap();
        assert_eq!(req.username, "alice");
    }

    #[test]
    fn test_parse_and_validate_fails_short_password() {
        let json = r#"{"username": "alice", "password": "short"}"#;
        assert!(parse_and_validate_login(json).is_err());
    }

    #[test]
    fn test_registration_password_mismatch() {
        let json = r#"{
            "username": "bob",
            "email": "bob@example.com",
            "password": "longpassword123",
            "confirm_password": "differentpassword"
        }"#;
        let req: RegistrationRequest = serde_json::from_str(json).unwrap();
        assert!(validate_registration(&req).is_err());
    }
}
