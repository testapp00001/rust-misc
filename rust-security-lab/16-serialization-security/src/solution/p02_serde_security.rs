//! # Lesson 02: Serde Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::Deserialize;

/// A login request with strict field requirements.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Parse a login request with deny_unknown_fields.
pub fn parse_login_request(json: &str) -> Result<LoginRequest, String> {
    serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))
}

/// Check if a JSON string would be rejected due to unknown fields.
pub fn has_unknown_fields(json: &str) -> Result<bool, String> {
    let result: Result<LoginRequest, _> = serde_json::from_str(json);
    Ok(result.is_err())
}

/// Validate a LoginRequest after deserialization.
pub fn validate_login(request: &LoginRequest) -> Result<(), String> {
    if request.username.is_empty() {
        return Err("username must not be empty".to_string());
    }
    if request.username.len() > 64 {
        return Err("username must be at most 64 characters".to_string());
    }
    if request.password.len() < 8 {
        return Err("password must be at least 8 characters".to_string());
    }
    if request.password.len() > 128 {
        return Err("password must be at most 128 characters".to_string());
    }
    Ok(())
}

/// Combined parse and validate.
pub fn parse_and_validate_login(json: &str) -> Result<LoginRequest, String> {
    let request = parse_login_request(json)?;
    validate_login(&request)?;
    Ok(request)
}

/// A registration request with more fields and validation.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegistrationRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

/// Validate a registration request.
pub fn validate_registration(request: &RegistrationRequest) -> Result<(), String> {
    if request.username.len() < 3 || request.username.len() > 64 {
        return Err("username must be 3-64 characters".to_string());
    }
    if !request.username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("username must be alphanumeric + underscore only".to_string());
    }
    if !request.email.contains('@') || !request.email.contains('.') {
        return Err("email must contain '@' and '.'".to_string());
    }
    if request.password.len() < 12 {
        return Err("password must be at least 12 characters".to_string());
    }
    if request.password != request.confirm_password {
        return Err("passwords do not match".to_string());
    }
    Ok(())
}

/// Parse and validate a registration request from JSON.
pub fn parse_and_validate_registration(json: &str) -> Result<RegistrationRequest, String> {
    let request: RegistrationRequest =
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))?;
    validate_registration(&request)?;
    Ok(request)
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
