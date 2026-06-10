//! # Lesson 10: Safe Defaults (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::Deserialize;

/// Build a secure JSON deserialization pipeline.
pub fn secure_deserialize<T: serde::de::DeserializeOwned>(
    json: &str,
    max_bytes: usize,
    max_depth: usize,
) -> Result<T, String> {
    // Step 1: Size check
    if json.len() > max_bytes {
        return Err(format!(
            "Payload size {} exceeds maximum {}",
            json.len(),
            max_bytes
        ));
    }

    // Step 2: Depth check
    let depth = max_json_depth(json);
    if depth > max_depth {
        return Err(format!(
            "JSON nesting depth {} exceeds maximum {}",
            depth, max_depth
        ));
    }

    // Step 3: Parse
    serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))
}

/// Helper: count maximum JSON nesting depth.
fn max_json_depth(json: &str) -> usize {
    let mut max_depth = 0usize;
    let mut current_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for byte in json.bytes() {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' && in_string {
            escaped = true;
            continue;
        }
        if byte == b'"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match byte {
            b'{' | b'[' => {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            }
            b'}' | b']' => {
                if current_depth > 0 {
                    current_depth -= 1;
                }
            }
            _ => {}
        }
    }

    max_depth
}

/// A bounded string that enforces maximum length.
#[derive(Deserialize, Debug, PartialEq)]
pub struct BoundedString(pub String);

impl BoundedString {
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

/// A range-limited u64.
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

/// A validated email field.
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

/// An allowlist-based permission field.
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

/// A complete secure API request type.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SecureApiRequest {
    pub username: BoundedString,
    pub email: EmailField,
    pub age: RangedU64,
    pub permission: Permission,
}

/// Parse and validate a SecureApiRequest from JSON.
pub fn parse_secure_request(json: &str, max_bytes: usize) -> Result<SecureApiRequest, String> {
    // Step 1: Size limit
    if json.len() > max_bytes {
        return Err(format!(
            "Payload size {} exceeds maximum {}",
            json.len(),
            max_bytes
        ));
    }

    // Step 2: Depth check
    let depth = max_json_depth(json);
    if depth > 10 {
        return Err(format!("JSON nesting depth {} exceeds maximum 10", depth));
    }

    // Step 3: Deserialize (deny_unknown_fields is on the struct)
    let req: SecureApiRequest =
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))?;

    // Step 4: Validate bounded types
    req.username.validate(64)?;
    req.age.validate(0, 200)?;

    Ok(req)
}

/// Audit a struct definition for security best practices.
pub fn audit_serde_struct(json_definition: &str) -> Vec<String> {
    let mut warnings = Vec::new();

    if !json_definition.contains("deny_unknown_fields") {
        warnings.push(
            "Missing #[serde(deny_unknown_fields)] -- extra fields will be silently ignored"
                .to_string(),
        );
    }

    if json_definition.contains("#[serde(default)]") {
        warnings.push(
            "Uses #[serde(default)] -- verify these fields should accept missing values"
                .to_string(),
        );
    }

    // Check for String fields without apparent length validation
    if json_definition.contains("String") && !json_definition.contains("BoundedString") {
        warnings.push(
            "Contains unbounded String fields -- consider adding length limits".to_string(),
        );
    }

    warnings
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
