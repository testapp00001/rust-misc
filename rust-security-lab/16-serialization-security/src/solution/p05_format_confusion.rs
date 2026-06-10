//! # Lesson 05: Format Confusion Attacks (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::Deserialize;

/// Demonstrate JSON numeric precision issues.
#[derive(Deserialize, Debug)]
pub struct NumericId {
    pub id: f64,
}

pub fn parse_large_number(json: &str) -> Result<f64, String> {
    let val: NumericId =
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))?;
    Ok(val.id)
}

/// Detect potential numeric precision loss.
///
/// f64 can exactly represent integers up to 2^53 (9007199254740992).
/// Beyond that, not all integers can be distinguished -- this is the
/// same limit that JavaScript's Number type has.
///
/// We check both: (a) whether the round-trip i64->f64 loses data,
/// and (b) whether the value exceeds the safe integer threshold.
pub fn has_precision_loss(value: f64) -> Result<bool, String> {
    const MAX_SAFE_INTEGER: f64 = 9007199254740992.0; // 2^53

    // Round-trip check
    let as_i64 = value as i64;
    let back_to_f64 = as_i64 as f64;
    if value != back_to_f64 {
        return Ok(true);
    }

    // Values at or beyond 2^53 cannot represent all consecutive integers
    if value.abs() >= MAX_SAFE_INTEGER {
        return Ok(true);
    }

    Ok(false)
}

/// Demonstrate boolean type confusion.
#[derive(Deserialize, Debug)]
pub struct BooleanField {
    pub enabled: serde_json::Value,
}

pub fn detect_boolean_confusion(json: &str) -> Result<bool, String> {
    let val: BooleanField =
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))?;
    // Confusion if it's a string instead of a boolean
    Ok(val.enabled.is_string())
}

/// Normalize and compare strings to prevent Unicode confusion.
pub fn strings_match_normalized(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// Validate that a JSON field is exactly the expected type.
pub fn value_is_type(value: &serde_json::Value, expected_type: &str) -> Result<bool, String> {
    let matches = match expected_type {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        _ => return Err(format!("unknown type '{}'", expected_type)),
    };
    Ok(matches)
}

/// Detect format confusion in a "port" field.
pub fn is_valid_port(value: &serde_json::Value) -> Result<bool, String> {
    match value.as_u64() {
        Some(port) => Ok(port >= 1 && port <= 65535),
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_number_parsed() {
        let json = r#"{"id": 9007199254740993}"#;
        let val = parse_large_number(json).unwrap();
        assert_ne!(val as i64, 9007199254740993_i64);
    }

    #[test]
    fn test_precision_loss_detection() {
        assert!(!has_precision_loss(42.0).unwrap());
        assert!(!has_precision_loss(0.0).unwrap());
        assert!(has_precision_loss(9007199254740993.0).unwrap());
    }

    #[test]
    fn test_boolean_true_is_not_confused() {
        let json = r#"{"enabled": true}"#;
        assert!(!detect_boolean_confusion(json).unwrap());
    }

    #[test]
    fn test_boolean_string_is_confused() {
        let json = r#"{"enabled": "true"}"#;
        assert!(detect_boolean_confusion(json).unwrap());
    }

    #[test]
    fn test_normalized_match() {
        assert!(strings_match_normalized("  Admin  ", "admin"));
        assert!(strings_match_normalized("ADMIN", "admin"));
        assert!(!strings_match_normalized("admin", "user"));
    }

    #[test]
    fn test_value_type_check() {
        let val = serde_json::json!("hello");
        assert!(value_is_type(&val, "string").unwrap());
        assert!(!value_is_type(&val, "number").unwrap());

        let num = serde_json::json!(42);
        assert!(value_is_type(&num, "number").unwrap());
    }

    #[test]
    fn test_valid_port_integer() {
        let val = serde_json::json!(8080);
        assert!(is_valid_port(&val).unwrap());
    }

    #[test]
    fn test_invalid_port_string() {
        let val = serde_json::json!("8080");
        assert!(!is_valid_port(&val).unwrap());
    }

    #[test]
    fn test_invalid_port_out_of_range() {
        let val = serde_json::json!(99999);
        assert!(!is_valid_port(&val).unwrap());
    }
}
