//! # Lesson 01: Deserialization Attacks
//!
//! ## The Threat
//!
//! Deserialization attacks exploit the gap between what the sender intended and
//! what the receiver constructs. When your application deserializes untrusted
//! input into rich objects, the attacker controls:
//!
//! 1. **Type confusion** -- supplying a string where a number is expected, or
//!    an object where a primitive is expected.
//! 2. **Extra fields** -- adding fields the application does not expect, which
//!    may be interpreted differently by different components.
//! 3. **Missing fields** -- omitting required fields to trigger default values
//!    that bypass security checks.
//!
//! ## Why Rust + serde Is (Mostly) Safe
//!
//! Unlike Java, Python, or PHP, serde does not execute arbitrary code during
//! deserialization. There is no `__reduce__` or `readObject()` equivalent. But
//! serde still trusts the shape of the data unless you configure it otherwise:
//!
//! - Without `deny_unknown_fields`, extra fields are silently ignored.
//! - Without explicit validation, garbage values pass through.
//! - Without size limits, a 10 GB JSON blob will fill your memory.
//!
//! ## Attack Scenario
//!
//! An API endpoint accepts `{"user_id": 42, "role": "admin"}`. The developer
//! forgot `deny_unknown_fields`. An attacker sends:
//!
//! ```json
//! {"user_id": 42, "role": "user", "is_admin": true}
//! ```
//!
//! If a downstream service reads `is_admin` (a field the first service ignored),
//! the attacker gains admin access.

use serde::Deserialize;

/// A user profile that should only contain id and name.
/// Without deny_unknown_fields, extra fields are silently ignored.
#[derive(Deserialize, Debug, PartialEq)]
pub struct UserProfile {
    pub id: u64,
    pub name: String,
}

/// Exercise 1: Demonstrate type confusion attack.
///
/// Given a JSON string where the `id` field is a string instead of a number,
/// return `Ok(true)` if serde rejects it, `Ok(false)` if serde accepts it.
///
/// Use `serde_json::from_str::<UserProfile>(json)`.
pub fn rejects_type_confusion(json: &str) -> Result<bool, String> {
    todo!("Demonstrate type confusion rejection")
}

/// Exercise 2: Demonstrate extra field acceptance.
///
/// Given a JSON string with an extra field `is_admin: true`, return `Ok(true)`
/// if the extra field was accepted (parsed without error), `Ok(false)` if rejected.
///
/// This shows why `deny_unknown_fields` matters.
pub fn accepts_extra_fields(json: &str) -> Result<bool, String> {
    todo!("Demonstrate extra field acceptance")
}

/// Exercise 3: Parse and return the UserProfile from valid JSON.
///
/// Return the parsed UserProfile. If parsing fails, return an error message.
pub fn parse_user_profile(json: &str) -> Result<UserProfile, String> {
    todo!("Parse UserProfile from JSON")
}

/// Exercise 4: A safe UserProfile that rejects unknown fields.
///
/// Implement the same struct but with `#[serde(deny_unknown_fields)]`.
/// Parse the JSON and return `Ok(true)` if it correctly rejects a JSON
/// string with an extra `is_admin` field, `Ok(false)` otherwise.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SafeUserProfile {
    pub id: u64,
    pub name: String,
}

pub fn safe_profile_rejects_extra(json: &str) -> Result<bool, String> {
    todo!("Demonstrate deny_unknown_fields rejection")
}

/// Exercise 5: Demonstrate missing field attack.
///
/// When a required field is missing, serde returns an error.
/// Given JSON with the `name` field omitted, return `Ok(true)` if serde
/// correctly rejects it, `Ok(false)` otherwise.
pub fn rejects_missing_field(json: &str) -> Result<bool, String> {
    todo!("Demonstrate missing field rejection")
}

/// Exercise 6: Demonstrate integer overflow defense.
///
/// Given a JSON string with an extremely large number for `id` (larger than
/// u64::MAX), return `Ok(true)` if serde rejects it, `Ok(false)` otherwise.
pub fn rejects_integer_overflow(json: &str) -> Result<bool, String> {
    todo!("Demonstrate integer overflow rejection")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_confusion_rejected() {
        // id should be a number, not a string
        let json = r#"{"id": "not_a_number", "name": "Alice"}"#;
        assert!(rejects_type_confusion(json).unwrap());
    }

    #[test]
    fn test_type_confusion_string_number() {
        // A number provided as string should still be rejected by strict parsing
        let json = r#"{"id": "42", "name": "Alice"}"#;
        assert!(rejects_type_confusion(json).unwrap());
    }

    #[test]
    fn test_extra_fields_accepted_by_default() {
        // Without deny_unknown_fields, extra fields are silently ignored
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
