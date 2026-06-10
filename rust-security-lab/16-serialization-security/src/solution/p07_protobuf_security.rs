//! # Lesson 07: Protobuf Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::Deserialize;

/// Simulates a protobuf message with fields that have default values.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UserMessage {
    pub user_id: u64,
    pub username: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub is_verified: bool,
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Detect when a field is set to its default value.
pub fn has_default_fields(msg: &UserMessage) -> Result<bool, String> {
    Ok(msg.role.is_empty() || !msg.is_verified || msg.permissions.is_empty())
}

/// Validate that required fields are explicitly set.
pub fn validate_required_fields(msg: &UserMessage) -> Result<(), String> {
    if msg.role.is_empty() {
        return Err("role must not be empty (default value detected)".to_string());
    }
    if !msg.is_verified {
        return Err("is_verified must be true (default value detected)".to_string());
    }
    if msg.permissions.is_empty() {
        return Err("permissions must not be empty (default value detected)".to_string());
    }
    Ok(())
}

/// Simulate protobuf size bomb detection.
pub fn is_size_bomb(claimed_size: u64, max_allowed_bytes: u64) -> Result<bool, String> {
    Ok(claimed_size > max_allowed_bytes)
}

/// Simulate unknown field detection by comparing top-level key count.
pub fn detect_unknown_fields(json: &str, expected_field_count: usize) -> Result<bool, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))?;

    let actual_count = match &value {
        serde_json::Value::Object(map) => map.len(),
        _ => return Err("expected a JSON object".to_string()),
    };

    Ok(actual_count > expected_field_count)
}

/// Simulate protobuf wire-type confusion.
pub fn detect_type_confusion(value: &serde_json::Value) -> Result<bool, String> {
    Ok(value.is_string())
}

/// Parse a protobuf-like message with full security checks.
pub fn parse_secure_user_message(json: &str) -> Result<UserMessage, String> {
    let msg: UserMessage =
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))?;

    if msg.user_id == 0 {
        return Err("user_id must be non-zero (protobuf default detected)".to_string());
    }

    if msg.username.is_empty() {
        return Err("username must not be empty".to_string());
    }

    if msg.username.len() > 64 {
        return Err(format!(
            "username length {} exceeds maximum 64",
            msg.username.len()
        ));
    }

    validate_required_fields(&msg)?;

    Ok(msg)
}

/// Detect suspicious unknown fields (keys starting with underscore).
pub fn has_suspicious_unknown_fields(json: &str) -> Result<bool, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))?;

    match &value {
        serde_json::Value::Object(map) => {
            for key in map.keys() {
                if key.starts_with('_') {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        _ => Err("expected a JSON object".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_fields_detected() {
        let msg = UserMessage {
            user_id: 1,
            username: "alice".to_string(),
            role: "".to_string(),
            is_verified: false,
            permissions: vec![],
        };
        assert!(has_default_fields(&msg).unwrap());
    }

    #[test]
    fn test_no_default_fields() {
        let msg = UserMessage {
            user_id: 1,
            username: "alice".to_string(),
            role: "admin".to_string(),
            is_verified: true,
            permissions: vec!["read".to_string()],
        };
        assert!(!has_default_fields(&msg).unwrap());
    }

    #[test]
    fn test_required_fields_missing_role() {
        let msg = UserMessage {
            user_id: 1,
            username: "alice".to_string(),
            role: "".to_string(),
            is_verified: true,
            permissions: vec!["read".to_string()],
        };
        assert!(validate_required_fields(&msg).is_err());
    }

    #[test]
    fn test_required_fields_all_set() {
        let msg = UserMessage {
            user_id: 1,
            username: "alice".to_string(),
            role: "admin".to_string(),
            is_verified: true,
            permissions: vec!["read".to_string()],
        };
        assert!(validate_required_fields(&msg).is_ok());
    }

    #[test]
    fn test_size_bomb_detected() {
        assert!(is_size_bomb(1_000_000_000, 1024).unwrap());
    }

    #[test]
    fn test_size_within_limit() {
        assert!(!is_size_bomb(512, 1024).unwrap());
    }

    #[test]
    fn test_unknown_fields_detected() {
        let json = r#"{"user_id": 1, "username": "alice", "secret_backdoor": true}"#;
        assert!(detect_unknown_fields(json, 2).unwrap());
    }

    #[test]
    fn test_type_confusion_string_user_id() {
        let val = serde_json::json!("42");
        assert!(detect_type_confusion(&val).unwrap());
    }

    #[test]
    fn test_type_confusion_numeric_user_id() {
        let val = serde_json::json!(42);
        assert!(!detect_type_confusion(&val).unwrap());
    }

    #[test]
    fn test_parse_secure_valid() {
        let json = r#"{
            "user_id": 1,
            "username": "alice",
            "role": "admin",
            "is_verified": true,
            "permissions": ["read"]
        }"#;
        assert!(parse_secure_user_message(json).is_ok());
    }

    #[test]
    fn test_parse_secure_zero_user_id() {
        let json = r#"{
            "user_id": 0,
            "username": "alice",
            "role": "admin",
            "is_verified": true,
            "permissions": ["read"]
        }"#;
        assert!(parse_secure_user_message(json).is_err());
    }

    #[test]
    fn test_suspicious_underscore_fields() {
        let json = r#"{"user_id": 1, "_internal_flag": true}"#;
        assert!(has_suspicious_unknown_fields(json).unwrap());
    }

    #[test]
    fn test_no_suspicious_fields() {
        let json = r#"{"user_id": 1, "username": "alice"}"#;
        assert!(!has_suspicious_unknown_fields(json).unwrap());
    }
}
