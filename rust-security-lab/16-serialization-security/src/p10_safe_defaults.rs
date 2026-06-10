//! # Lesson 10: Safe Defaults
//!
//! ## The Principle
//!
//! Security should be the default, not an afterthought. Every serialization
//! boundary in your application should ship with safe defaults:
//!
//! 1. **Deny unknown fields** -- Always add `#[serde(deny_unknown_fields)]`.
//! 2. **Size limits** -- Reject payloads before parsing.
//! 3. **Depth limits** -- Prevent stack overflow from nested structures.
//! 4. **Required fields** -- Use `Option<T>` only when a field is truly optional.
//! 5. **Validation** -- Check business rules after deserialization.
//! 6. **Allowlists** -- Prefer allowlists over denylists for values.
//!
//! ## Building a Secure Deserialization Pipeline
//!
//! A secure pipeline looks like this:
//!
//! ```text
//! raw bytes -> size check -> depth check -> parse -> validate -> use
//! ```
//!
//! Each stage rejects bad input before the next stage sees it.
//!
//! ## Exercise
//!
//! Build a complete secure deserialization pipeline that combines all the
//! techniques from this module into a single, auditable function.

use serde::Deserialize;

/// Exercise 1: Build a secure JSON deserialization pipeline.
///
/// Steps:
/// 1. Reject if input exceeds `max_bytes`
/// 2. Reject if nesting depth exceeds `max_depth`
/// 3. Parse into the target struct (which has deny_unknown_fields)
/// 4. Validate the parsed value
///
/// Return Ok(value) or Err(message).
pub fn secure_deserialize<T: serde::de::DeserializeOwned>(
    json: &str,
    max_bytes: usize,
    max_depth: usize,
) -> Result<T, String> {
    todo!("Build secure deserialization pipeline")
}

/// Exercise 2: Build a size-limited string type.
///
/// A String that enforces a maximum length after deserialization.
/// Validation happens in `validate()` rather than during deserialization,
/// because serde does not support const generics in `deserialize_with`.
#[derive(Deserialize, Debug, PartialEq)]
pub struct BoundedString(pub String);

impl BoundedString {
    /// Create a new BoundedString, checking length at construction time.
    pub fn new(s: String, max_len: usize) -> Result<Self, String> {
        if s.len() > max_len {
            return Err(format!(
                "string length {} exceeds maximum {}",
                s.len(),
                max_len
            ));
        }
        Ok(Self(s))
    }

    /// Validate the string is within bounds.
    pub fn validate(&self, max_len: usize) -> Result<(), String> {
        if self.0.len() > max_len {
            return Err(format!(
                "string length {} exceeds maximum {}",
                self.0.len(),
                max_len
            ));
        }
        Ok(())
    }
}

/// Exercise 3: Build a range-limited integer type.
///
/// A u64 that enforces min/max after deserialization.
#[derive(Deserialize, Debug, PartialEq)]
pub struct RangedU64(pub u64);

impl RangedU64 {
    pub fn new(v: u64, min: u64, max: u64) -> Result<Self, String> {
        if v < min || v > max {
            return Err(format!("value {} out of range [{}, {}]", v, min, max));
        }
        Ok(Self(v))
    }

    pub fn validate(&self, min: u64, max: u64) -> Result<(), String> {
        if self.0 < min || self.0 > max {
            return Err(format!(
                "value {} out of range [{}, {}]",
                self.0, min, max
            ));
        }
        Ok(())
    }
}

/// Exercise 4: Build a validated email field.
///
/// Deserialize a string and validate it looks like an email:
/// - Contains exactly one `@`
/// - Has at least one `.` after the `@`
/// - Is not empty
/// - Is at most 254 characters (RFC 5321 limit)
#[derive(Deserialize, Debug, PartialEq)]
pub struct EmailField(
    #[serde(deserialize_with = "validate_email_de")]
    pub String,
);

fn validate_email_de<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let s = String::deserialize(deserializer)?;
    validate_email_inner(&s).map_err(serde::de::Error::custom)?;
    Ok(s)
}

fn validate_email_inner(s: &str) -> Result<(), String> {
    if s.is_empty() {
        return Err("email must not be empty".to_string());
    }
    if s.len() > 254 {
        return Err("email exceeds 254 characters".to_string());
    }
    let at_count = s.matches('@').count();
    if at_count != 1 {
        return Err("email must contain exactly one '@'".to_string());
    }
    let domain = s.split('@').nth(1).unwrap_or("");
    if !domain.contains('.') {
        return Err("email domain must contain a dot".to_string());
    }
    Ok(())
}

/// Exercise 5: Build an allowlist-based enum validator.
///
/// Deserialize a string and validate it against a set of allowed values.
/// For simplicity, validate against a hardcoded list: ["read", "write", "admin"].
#[derive(Deserialize, Debug, PartialEq)]
pub struct Permission(
    #[serde(deserialize_with = "validate_permission_de")]
    pub String,
);

fn validate_permission_de<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let s = String::deserialize(deserializer)?;
    const ALLOWED: &[&str] = &["read", "write", "admin"];
    if !ALLOWED.contains(&s.as_str()) {
        return Err(serde::de::Error::custom(format!(
            "'{}' is not a valid permission. Allowed: {:?}",
            s, ALLOWED
        )));
    }
    Ok(s)
}

/// Exercise 6: Build a complete secure API request type.
///
/// Combine all the safe types into one request struct.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SecureApiRequest {
    pub username: BoundedString,
    pub email: EmailField,
    pub age: RangedU64,
    pub permission: Permission,
}

/// Parse and validate a SecureApiRequest from JSON.
/// Apply size limit, then deserialize, then validate bounded types.
pub fn parse_secure_request(json: &str, max_bytes: usize) -> Result<SecureApiRequest, String> {
    todo!("Parse secure API request")
}

/// Exercise 7: Audit a struct for security best practices.
///
/// Given a JSON representation of a struct definition, check for these
/// security issues and return a list of warnings:
///
/// - No `deny_unknown_fields` -> warning
/// - Fields with `#[serde(default)]` that should be required -> warning
/// - String fields without length limits -> warning
///
/// For this exercise, just check if the JSON string contains these patterns.
/// Return a Vec of warning strings.
pub fn audit_serde_struct(json_definition: &str) -> Vec<String> {
    todo!("Audit serde struct for security issues")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_deserialize_valid() {
        let json = r#"{"name": "test"}"#;
        let val: serde_json::Value = secure_deserialize(json, 1024, 10).unwrap();
        assert!(val.is_object());
    }

    #[test]
    fn test_secure_deserialize_too_large() {
        let json = r#"{"name": "test"}"#;
        assert!(secure_deserialize::<serde_json::Value>(json, 5, 10).is_err());
    }

    #[test]
    fn test_secure_deserialize_too_deep() {
        let json = r#"[[[[[1]]]]]"#;
        assert!(secure_deserialize::<serde_json::Value>(json, 1024, 3).is_err());
    }

    #[test]
    fn test_bounded_string_ok() {
        let bs = BoundedString::new("hello".to_string(), 10).unwrap();
        assert_eq!(bs.0, "hello");
    }

    #[test]
    fn test_bounded_string_too_long() {
        assert!(BoundedString::new("this is a very long string".to_string(), 5).is_err());
    }

    #[test]
    fn test_bounded_string_validate() {
        let bs = BoundedString("hello".to_string());
        assert!(bs.validate(10).is_ok());
        assert!(bs.validate(3).is_err());
    }

    #[test]
    fn test_ranged_u64_ok() {
        let r = RangedU64::new(50, 0, 100).unwrap();
        assert_eq!(r.0, 50);
    }

    #[test]
    fn test_ranged_u64_out_of_range() {
        assert!(RangedU64::new(150, 0, 100).is_err());
    }

    #[test]
    fn test_email_valid() {
        let json = r#""user@example.com""#;
        let val: EmailField = serde_json::from_str(json).unwrap();
        assert_eq!(val.0, "user@example.com");
    }

    #[test]
    fn test_email_no_at() {
        let json = r#""userexample.com""#;
        let result: Result<EmailField, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_email_no_dot_in_domain() {
        let json = r#""user@localhost""#;
        let result: Result<EmailField, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_permission_valid() {
        let json = r#""read""#;
        let val: Permission = serde_json::from_str(json).unwrap();
        assert_eq!(val.0, "read");
    }

    #[test]
    fn test_permission_invalid() {
        let json = r#""delete""#;
        let result: Result<Permission, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_secure_request_valid() {
        let json = r#"{
            "username": "alice",
            "email": "alice@example.com",
            "age": 25,
            "permission": "read"
        }"#;
        let result = parse_secure_request(json, 1024);
        assert!(result.is_ok());
    }

    #[test]
    fn test_secure_request_too_large() {
        let json = r#"{
            "username": "alice",
            "email": "alice@example.com",
            "age": 25,
            "permission": "read"
        }"#;
        let result = parse_secure_request(json, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_audit_finds_no_deny_unknown() {
        let def = r#"#[derive(Deserialize)] struct Foo { bar: String }"#;
        let warnings = audit_serde_struct(def);
        assert!(warnings.iter().any(|w| w.contains("deny_unknown_fields")));
    }
}
