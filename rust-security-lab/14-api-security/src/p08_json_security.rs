//! # Lesson 08: JSON Parsing Security
//!
//! ## Why JSON Security?
//!
//! JSON is the de facto data format for APIs. But JSON parsers can be exploited:
//!
//! ### Deep Nesting Attack
//!
//! ```json
//! {"a":{"b":{"c":{"d":{"e":{"f":{"g":...}}}}}}}
//! ```
//!
//! Recursive parsers use stack space for each nesting level. Deeply nested
//! JSON can cause a stack overflow, crashing the server.
//!
//! ### Large Payload Attack
//!
//! A 100 MB JSON object exhausts memory when parsed into a DOM tree.
//!
//! ### Key Collision Attack
//!
//! ```json
//! {"role": "user", "role": "admin"}
//! ```
//!
//! Different parsers handle duplicates differently. Some keep the first value,
//! some keep the last. This inconsistency can be exploited for privilege escalation.
//!
//! ### Type Confusion
//!
//! ```json
//! {"age": "twenty"}  // Expecting a number, got a string
//! {"items": "none"}  // Expecting an array, got a string
//! ```
//!
//! ## Defense
//!
//! 1. Limit JSON nesting depth (5-10 levels for most APIs)
//! 2. Limit total JSON size before parsing
//! 3. Limit the number of keys in an object
//! 4. Reject or explicitly handle duplicate keys
//! 5. Validate types strictly — don't silently coerce
//! 6. Use streaming parsers for large payloads

use serde_json::Value;

/// Exercise 1: Calculate the nesting depth of a JSON value.
///
/// A scalar (string, number, bool, null) has depth 1.
/// An array or object has depth = 1 + max(child depths).
/// An empty array or object has depth 1.
///
/// Examples:
/// - `"hello"` → 1
/// - `[1, 2, 3]` → 2
/// - `{"a": [1, {"b": 2}]}` → 4
///
/// Hints:
/// - Match on the Value type
/// - For arrays/objects, recursively compute child depths
/// - Use `.iter().map(|v| depth(v)).max().unwrap_or(0)`
pub fn json_depth(value: &Value) -> usize {
    todo!("Calculate the nesting depth of a JSON value")
}

/// Exercise 2: Count the total number of keys in a JSON value.
///
/// Count all keys across all nested objects. Arrays contribute 0 keys
/// but their elements are recursively counted.
///
/// Examples:
/// - `"hello"` → 0
/// - `{"a": 1}` → 1
/// - `{"a": {"b": 1, "c": 2}}` → 3 (a, b, c)
/// - `{"a": [1, {"b": 2}]}` → 2 (a, b)
///
/// Hints:
/// - For objects: keys.len() + sum(child key counts)
/// - For arrays: sum(child key counts)
/// - For scalars: 0
pub fn count_json_keys(value: &Value) -> usize {
    todo!("Count total keys in a JSON value recursively")
}

/// Exercise 3: Validate JSON against security constraints.
///
/// Check a parsed JSON value against multiple security constraints.
/// Returns `Ok(())` if all pass, `Err(description)` for the first failure.
///
/// Constraints (check in this order):
/// 1. Nesting depth must not exceed `max_depth`
/// 2. Total key count must not exceed `max_keys`
/// 3. The serialized size must not exceed `max_bytes` (check `serde_json::to_string` length)
///
/// Hints:
/// - Use `json_depth` and `count_json_keys`
/// - Serialize to string and check `.len()`
/// - Return early on first failure
pub fn validate_json_security(
    value: &Value,
    max_depth: usize,
    max_keys: usize,
    max_bytes: usize,
) -> Result<(), &'static str> {
    todo!("Validate JSON against security constraints")
}

/// Exercise 4: Detect duplicate keys in a JSON string.
///
/// Parse a JSON string and check if any object has duplicate keys.
/// Returns a list of paths where duplicates were found.
///
/// Example:
/// ```json
/// {"a": 1, "a": 2, "b": {"c": 1, "c": 2}}
/// ```
/// Returns: vec!["$.a", "$.b.c"]
///
/// Hints:
/// - Parse the JSON string to a serde_json::Value
/// - For serde_json, duplicate keys in objects are handled by keeping the last one
/// - Instead, parse the JSON as a raw string and manually track keys
/// - Or use a custom deserializer
///
/// For this exercise, use a simpler approach: parse the JSON string and
/// compare the number of unique keys vs total keys at each level.
/// Actually, serde_json removes duplicates on parse, so we can't detect them
/// after parsing. Instead, implement a manual scanner.
///
/// Simpler approach: scan the raw JSON string for duplicate keys at each
/// nesting level. This is a simplified version that works for flat objects.
///
/// For the test cases, focus on detecting duplicates in the raw string format.
pub fn detect_duplicate_keys(json_str: &str) -> Vec<String> {
    todo!("Detect duplicate keys in a JSON string")
}

/// Exercise 5: Sanitize a JSON value by removing dangerous content.
///
/// Create a "safe" version of a JSON value:
/// 1. Truncate strings longer than `max_string_length`
/// 2. Limit arrays to `max_array_length` elements
/// 3. Limit objects to `max_object_keys` keys (keep first N)
/// 4. Cap nesting depth at `max_depth` (replace deeper values with null)
///
/// Return the sanitized Value.
///
/// Hints:
/// - Recursively process the Value
/// - For strings: `value[..max_string_length]` (be careful with UTF-8)
/// - For arrays: `.take(max_array_length)`
/// - For objects: `.take(max_object_keys)`
/// - Track current depth, return Value::Null when depth exceeded
pub fn sanitize_json(
    value: &Value,
    max_string_length: usize,
    max_array_length: usize,
    max_object_keys: usize,
    max_depth: usize,
) -> Value {
    todo!("Sanitize a JSON value by applying size limits")
}

/// Exercise 6: Parse JSON safely with all protections.
///
/// A one-stop function that parses a JSON string with all security checks:
/// 1. Check raw string length <= `max_bytes`
/// 2. Parse the string into a Value
/// 3. Validate depth <= `max_depth`
/// 4. Validate key count <= `max_keys`
///
/// Returns:
/// - `Ok(value)` if all checks pass
/// - `Err("payload_too_large")` if string is too long
/// - `Err("parse_error:...")` if JSON is invalid
/// - `Err("too_deep")` if nesting exceeds limit
/// - `Err("too_many_keys")` if key count exceeds limit
///
/// Hints:
/// - Check string length first (cheapest check)
/// - Parse with `serde_json::from_str`
/// - Use `json_depth` and `count_json_keys`
pub fn safe_json_parse(
    json_str: &str,
    max_bytes: usize,
    max_depth: usize,
    max_keys: usize,
) -> Result<Value, String> {
    todo!("Parse JSON with all security protections")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_depth_scalar() {
        assert_eq!(json_depth(&json!("hello")), 1);
        assert_eq!(json_depth(&json!(42)), 1);
        assert_eq!(json_depth(&json!(null)), 1);
    }

    #[test]
    fn test_depth_array() {
        assert_eq!(json_depth(&json!([1, 2, 3])), 2);
    }

    #[test]
    fn test_depth_nested() {
        let val = json!({"a": {"b": {"c": 1}}});
        assert_eq!(json_depth(&val), 4);
    }

    #[test]
    fn test_depth_empty() {
        assert_eq!(json_depth(&json!({})), 1);
        assert_eq!(json_depth(&json!([])), 1);
    }

    #[test]
    fn test_count_keys_scalar() {
        assert_eq!(count_json_keys(&json!("hello")), 0);
    }

    #[test]
    fn test_count_keys_nested() {
        let val = json!({"a": {"b": 1, "c": 2}});
        assert_eq!(count_json_keys(&val), 3);
    }

    #[test]
    fn test_count_keys_array() {
        let val = json!({"items": [{"x": 1}, {"y": 2}]});
        assert_eq!(count_json_keys(&val), 3); // items, x, y
    }

    #[test]
    fn test_validate_json_security_ok() {
        let val = json!({"name": "Alice", "age": 30});
        assert!(validate_json_security(&val, 10, 100, 1000).is_ok());
    }

    #[test]
    fn test_validate_json_security_too_deep() {
        let val = json!({"a": {"b": {"c": {"d": 1}}}});
        assert_eq!(validate_json_security(&val, 2, 100, 10000).unwrap_err(), "too_deep");
    }

    #[test]
    fn test_validate_json_security_too_many_keys() {
        let val = json!({"a":1,"b":2,"c":3,"d":4,"e":5});
        assert_eq!(validate_json_security(&val, 10, 3, 10000).unwrap_err(), "too_many_keys");
    }

    #[test]
    fn test_validate_json_security_too_large() {
        let big_string = "x".repeat(2000);
        let val = json!({"data": big_string});
        assert_eq!(validate_json_security(&val, 10, 10, 100).unwrap_err(), "payload_too_large");
    }

    #[test]
    fn test_sanitize_truncates_strings() {
        let val = json!({"name": "a very long string here"});
        let sanitized = sanitize_json(&val, 10, 100, 100, 10);
        assert!(sanitized["name"].as_str().unwrap().len() <= 10);
    }

    #[test]
    fn test_sanitize_limits_arrays() {
        let val = json!({"items": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]});
        let sanitized = sanitize_json(&val, 100, 3, 100, 10);
        assert_eq!(sanitized["items"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_sanitize_limits_objects() {
        let val = json!({"a":1,"b":2,"c":3,"d":4,"e":5});
        let sanitized = sanitize_json(&val, 100, 100, 3, 10);
        assert!(sanitized.as_object().unwrap().len() <= 3);
    }

    #[test]
    fn test_sanitize_limits_depth() {
        let val = json!({"a": {"b": {"c": {"d": "deep"}}}});
        let sanitized = sanitize_json(&val, 100, 100, 100, 2);
        // Depth beyond 2 should be replaced with null
        assert_eq!(json_depth(&sanitized), 2);
    }

    #[test]
    fn test_safe_parse_valid() {
        let result = safe_json_parse(r#"{"name":"Alice"}"#, 1024, 10, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_parse_too_large() {
        let big = format!("{{\"data\":\"{}\"}}", "x".repeat(2000));
        assert!(safe_json_parse(&big, 100, 10, 100).unwrap_err().contains("payload_too_large"));
    }

    #[test]
    fn test_safe_parse_invalid_json() {
        assert!(safe_json_parse("{invalid json}", 1024, 10, 100).unwrap_err().contains("parse_error"));
    }

    #[test]
    fn test_safe_parse_too_deep() {
        let result = safe_json_parse(r#"{"a":{"b":{"c":1}}}"#, 1024, 2, 100);
        assert!(result.unwrap_err().contains("too_deep"));
    }
}
