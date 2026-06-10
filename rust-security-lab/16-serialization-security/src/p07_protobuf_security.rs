//! # Lesson 07: Protobuf Security
//!
//! ## The Threat
//!
//! Protocol Buffers (protobuf) are a binary serialization format used extensively
//! in gRPC, Kubernetes, and many microservice architectures. While protobuf's
//! schema (`.proto` files) provides structural validation, several security
//! pitfalls remain:
//!
//! 1. **Default values** -- Missing fields silently get default values (0, "",
//!    false). An attacker can omit a field to bypass validation that only runs
//!    on present fields.
//! 2. **Field type confusion** -- Protobuf allows wire-type mismatches. A field
//!    encoded as varint can be decoded as a different type without error.
//! 3. **Size bombs** -- A length-delimited field can claim a large size, causing
//!    the parser to allocate excessive memory.
//! 4. **Unknown fields** -- By default, protobuf preserves unknown fields and
//!    can re-serialize them, potentially smuggling data through a proxy.
//!
//! ## Defense
//!
//! - Always validate after deserializing, including checking for default values
//!   that should not be defaults.
//! - Set size limits on length-delimited fields.
//! - Reject or log unknown fields in security-critical contexts.
//! - Use `deny_unknown_fields` equivalents where available.
//!
//! ## Note
//!
//! This lesson simulates protobuf-like security concerns using JSON and plain
//! Rust structs, since adding prost/tonic as dependencies is out of scope.
//! The patterns apply directly to real protobuf usage.

use serde::Deserialize;

/// Simulates a protobuf message with fields that have default values.
///
/// In real protobuf, missing fields get zero/empty defaults.
/// An attacker can omit `role` to get the default "user" or omit
/// `is_verified` to get `false`.
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

/// Exercise 1: Detect when a field is set to its default value.
///
/// Return Ok(true) if ANY of these fields are at their default values:
/// - role is empty string
/// - is_verified is false
/// - permissions is empty
///
/// Return Ok(false) if all fields have non-default values.
/// This simulates checking whether an attacker omitted fields.
pub fn has_default_fields(msg: &UserMessage) -> Result<bool, String> {
    todo!("Detect default field values")
}

/// Exercise 2: Validate that required fields are explicitly set.
///
/// A "required" field must not be at its default value.
/// Check:
/// - role must be non-empty
/// - is_verified must be true
/// - permissions must be non-empty
///
/// Return Ok(()) if valid, Err(message) if any required field is missing/default.
pub fn validate_required_fields(msg: &UserMessage) -> Result<(), String> {
    todo!("Validate required fields are not defaults")
}

/// Exercise 3: Simulate protobuf size bomb detection.
///
/// Given a claimed size (as an attacker would provide in a length-delimited
/// field), return Ok(true) if it exceeds `max_allowed_bytes`.
/// Return Ok(false) if within limits.
pub fn is_size_bomb(claimed_size: u64, max_allowed_bytes: u64) -> Result<bool, String> {
    todo!("Detect size bomb")
}

/// Exercise 4: Simulate unknown field detection.
///
/// Parse JSON into the target struct. If parsing succeeds but the JSON
/// contained fields not in the struct, return Ok(true) for "has unknown fields".
///
/// Use `serde_json::from_str` -- with `deny_unknown_fields`, unknown fields
/// cause a parse error. Without it, they are silently ignored.
///
/// For this exercise, parse into `serde_json::Value` first, then count
/// how many top-level keys exist. Compare against the expected field count.
pub fn detect_unknown_fields(json: &str, expected_field_count: usize) -> Result<bool, String> {
    todo!("Detect unknown fields in protobuf-like message")
}

/// Exercise 5: Simulate protobuf wire-type confusion.
///
/// In protobuf, a field encoded as one wire type can sometimes be decoded
/// as another. Simulate this by checking if a JSON value for a "user_id"
/// field is actually a string disguised as a number (e.g., "42" instead of 42).
///
/// Return Ok(true) if the value is a string (confusion detected),
/// Ok(false) if it is a proper number.
pub fn detect_type_confusion(value: &serde_json::Value) -> Result<bool, String> {
    todo!("Detect wire-type confusion")
}

/// Exercise 6: Parse a protobuf-like message with full security checks.
///
/// Steps:
/// 1. Parse JSON into UserMessage (deny_unknown_fields handles structure)
/// 2. Validate required fields are not defaults
/// 3. Validate user_id is non-zero (zero means unset in protobuf)
/// 4. Validate username is non-empty and at most 64 chars
///
/// Return Ok(message) or Err(message).
pub fn parse_secure_user_message(json: &str) -> Result<UserMessage, String> {
    todo!("Parse protobuf-like message with full validation")
}

/// Exercise 7: Demonstrate re-serialization risk with unknown fields.
///
/// Parse a JSON string into serde_json::Value, then check if any key
/// starts with underscore `_` (simulating a field the proxy does not
/// understand but would forward). Return Ok(true) if any such field exists.
pub fn has_suspicious_unknown_fields(json: &str) -> Result<bool, String> {
    todo!("Detect suspicious unknown fields")
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
        // UserMessage has 5 fields, JSON has 3 -- the method checks top-level keys
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
