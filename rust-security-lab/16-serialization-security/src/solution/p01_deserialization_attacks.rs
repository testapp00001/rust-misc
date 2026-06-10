//! # Lesson 01: Deserialization Attacks (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::Deserialize;

/// A user profile that should only contain id and name.
#[derive(Deserialize, Debug, PartialEq)]
pub struct UserProfile {
    pub id: u64,
    pub name: String,
}

/// Demonstrate type confusion attack.
pub fn rejects_type_confusion(json: &str) -> Result<bool, String> {
    let result: Result<UserProfile, _> = serde_json::from_str(json);
    Ok(result.is_err())
}

/// Demonstrate extra field acceptance.
pub fn accepts_extra_fields(json: &str) -> Result<bool, String> {
    let result: Result<UserProfile, _> = serde_json::from_str(json);
    Ok(result.is_ok())
}

/// Parse and return the UserProfile from valid JSON.
pub fn parse_user_profile(json: &str) -> Result<UserProfile, String> {
    serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))
}

/// A safe UserProfile that rejects unknown fields.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SafeUserProfile {
    pub id: u64,
    pub name: String,
}

pub fn safe_profile_rejects_extra(json: &str) -> Result<bool, String> {
    let result: Result<SafeUserProfile, _> = serde_json::from_str(json);
    Ok(result.is_err())
}

/// Demonstrate missing field attack.
pub fn rejects_missing_field(json: &str) -> Result<bool, String> {
    let result: Result<UserProfile, _> = serde_json::from_str(json);
    Ok(result.is_err())
}

/// Demonstrate integer overflow defense.
pub fn rejects_integer_overflow(json: &str) -> Result<bool, String> {
    let result: Result<UserProfile, _> = serde_json::from_str(json);
    Ok(result.is_err())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_confusion_rejected() {
        let json = r#"{"id": "not_a_number", "name": "Alice"}"#;
        assert!(rejects_type_confusion(json).unwrap());
    }

    #[test]
    fn test_type_confusion_string_number() {
        let json = r#"{"id": "42", "name": "Alice"}"#;
        assert!(rejects_type_confusion(json).unwrap());
    }

    #[test]
    fn test_extra_fields_accepted_by_default() {
        let json = r#"{"id": 1, "name": "Alice", "is_admin": true}"#;
        assert!(accepts_extra_fields(json).unwrap());
    }

    #[test]
    fn test_parse_valid_profile() {
        let json = r#"{"id": 42, "name": "Bob"}"#;
        let profile = parse_user_profile(json).unwrap();
        assert_eq!(profile.id, 42);
        assert_eq!(profile.name, "Bob");
    }

    #[test]
    fn test_safe_profile_rejects_extra() {
        let json = r#"{"id": 1, "name": "Alice", "is_admin": true}"#;
        assert!(safe_profile_rejects_extra(json).unwrap());
    }

    #[test]
    fn test_safe_profile_accepts_valid() {
        let json = r#"{"id": 1, "name": "Alice"}"#;
        let result: Result<SafeUserProfile, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_name_rejected() {
        let json = r#"{"id": 42}"#;
        assert!(rejects_missing_field(json).unwrap());
    }

    #[test]
    fn test_integer_overflow_rejected() {
        let json = r#"{"id": 99999999999999999999999999999999, "name": "Eve"}"#;
        assert!(rejects_integer_overflow(json).unwrap());
    }
}
