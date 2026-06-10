//! # Lesson 05: Format Confusion Attacks
//!
//! ## The Attack
//!
//! The same bytes can be valid in multiple formats, each interpreting the data
//! differently. An attacker sends data that parses correctly in one format
//! (JSON) but is interpreted differently in another (XML), or vice versa.
//!
//! ## Real-World Examples
//!
//! ### JSON vs Protobuf ambiguity
//! A JSON field `"enabled": "true"` is a string. In Protobuf, a bool field
//! accepts `1`/`0` but not `"true"`. If your gateway parses JSON and your
//! backend parses Protobuf, the same bytes mean different things.
//!
//! ### Numeric precision
//! JSON has no integer type -- `9007199254740993` (2^53 + 1) loses precision
//! in JavaScript but is exact in Rust i64. If your frontend and backend
//! disagree on precision, authorization IDs can collide.
//!
//! ### Unicode normalization
//! `"admin"` and `"admin"` (with a zero-width space) look identical but are
//! different strings. JSON preserves them; some XML parsers normalize them.
//!
//! ## Defense
//!
//! - Use ONE canonical format per trust boundary
//! - Validate types and ranges after parsing, regardless of format
//! - Never assume format-specific behavior (like JSON numeric precision)

use serde::Deserialize;

/// Exercise 1: Demonstrate JSON numeric precision issues.
///
/// Parse the JSON string `{"id": 9007199254740993}` (2^53 + 1) into this struct.
/// Return the parsed id value.
///
/// This demonstrates that JSON does not distinguish integers from floats,
/// and large integers may lose precision.
#[derive(Deserialize, Debug)]
pub struct NumericId {
    pub id: f64,
}

pub fn parse_large_number(json: &str) -> Result<f64, String> {
    todo!("Parse large number from JSON")
}

/// Exercise 2: Detect potential numeric precision loss.
///
/// Given a JSON number as an f64, check if it can be exactly represented
/// as an i64. Return Ok(true) if there would be precision loss,
/// Ok(false) if it converts exactly.
pub fn has_precision_loss(value: f64) -> Result<bool, String> {
    todo!("Check for precision loss")
}

/// Exercise 3: Demonstrate boolean type confusion.
///
/// In JSON, `true` is a boolean and `"true"` is a string.
/// Parse the JSON into this struct. If the value is the string "true"
/// instead of the boolean `true`, return Ok(true) to indicate confusion.
#[derive(Deserialize, Debug)]
pub struct BooleanField {
    pub enabled: serde_json::Value,
}

pub fn detect_boolean_confusion(json: &str) -> Result<bool, String> {
    todo!("Detect boolean type confusion")
}

/// Exercise 4: Normalize and compare strings to prevent Unicode confusion.
///
/// Two strings that look the same may have different byte representations.
/// Implement a comparison that:
/// 1. Trims whitespace
/// 2. Converts to lowercase
/// 3. Compares the result
///
/// Return true if they match after normalization.
pub fn strings_match_normalized(a: &str, b: &str) -> bool {
    todo!("Normalized string comparison")
}

/// Exercise 5: Validate that a JSON field is exactly the expected type.
///
/// Given a `serde_json::Value`, check that it matches the expected type tag:
/// - "string" -> Value::String
/// - "number" -> Value::Number
/// - "boolean" -> Value::Bool
/// - "null" -> Value::Null
/// - "array" -> Value::Array
/// - "object" -> Value::Object
///
/// Return Ok(true) if the type matches, Ok(false) otherwise.
pub fn value_is_type(value: &serde_json::Value, expected_type: &str) -> Result<bool, String> {
    todo!("Check JSON value type")
}

/// Exercise 6: Detect format confusion in a "port" field.
///
/// A port number should be an integer, not a string.
/// Given a JSON value, return:
/// - Ok(true) if the value is a proper integer in 1-65535
/// - Ok(false) if it is something else (string, float, out of range)
pub fn is_valid_port(value: &serde_json::Value) -> Result<bool, String> {
    todo!("Validate port as strict integer")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_number_parsed() {
        let json = r#"{"id": 9007199254740993}"#;
        let val = parse_large_number(json).unwrap();
        // f64 cannot represent 2^53 + 1 exactly
        assert_ne!(val as i64, 9007199254740993_i64);
    }

    #[test]
    fn test_precision_loss_detection() {
        assert!(!has_precision_loss(42.0).unwrap());
        assert!(!has_precision_loss(0.0).unwrap());
        // 2^53 + 1 cannot be exactly represented as f64
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
