//! # Lesson 03: JSON Depth Limiting
//!
//! ## The Attack
//!
//! A deeply nested JSON payload can cause a stack overflow in parsers that
//! use recursion. A 10 KB JSON string like `[[[[[[...]]]]]]` nested 100,000
//! levels deep will crash most JSON parsers.
//!
//! ## The Defense
//!
//! Limit the maximum nesting depth before parsing. serde_json supports this
//! via a custom `Deserializer` with a depth counter, or you can pre-scan the
//! raw bytes to count nesting before handing them to serde.
//!
//! ## Why serde_json Alone Is Not Enough
//!
//! `serde_json::from_str` does NOT enforce a depth limit by default. It will
//! happily recurse until the stack overflows. You must add this check yourself.

use serde::Deserialize;

/// Exercise 1: Count the maximum nesting depth of a JSON string.
///
/// Walk the raw JSON bytes and track the maximum nesting level:
/// - Increment depth on `{` or `[`
/// - Decrement depth on `}` or `]`
/// - Skip strings (content between `"` and `"`, handling `\"` escapes)
///
/// Return the maximum depth encountered.
///
/// Examples:
/// - `"42"` -> 0 (no nesting)
/// - `"[1,2,3]"` -> 1
/// - `"[[1]]"` -> 2
/// - `"[{\"a\":[1]}]"` -> 3
pub fn max_json_depth(json: &str) -> usize {
    todo!("Count maximum JSON nesting depth")
}

/// Exercise 2: Check if a JSON string exceeds a maximum depth.
///
/// Return `Ok(true)` if the JSON exceeds `max_depth`, `Ok(false)` otherwise.
/// Use `max_json_depth` internally.
pub fn exceeds_depth(json: &str, max_depth: usize) -> Result<bool, String> {
    todo!("Check if JSON exceeds max depth")
}

/// Exercise 3: Parse JSON only if depth is within limit.
///
/// If the depth exceeds `max_depth`, return an error.
/// Otherwise, parse into `serde_json::Value` and return it.
pub fn parse_with_depth_limit(json: &str, max_depth: usize) -> Result<serde_json::Value, String> {
    todo!("Parse JSON with depth limit")
}

/// Exercise 4: A safe JSON parser that enforces both depth and size limits.
///
/// - Input must not exceed `max_bytes` in length
/// - Depth must not exceed `max_depth`
/// - Only then parse into `serde_json::Value`
pub fn safe_parse_json(
    json: &str,
    max_bytes: usize,
    max_depth: usize,
) -> Result<serde_json::Value, String> {
    todo!("Parse JSON with depth and size limits")
}

/// Exercise 5: Parse JSON into a typed struct with depth checking.
///
/// Check depth first, then parse into type T. Return error if depth exceeded.
pub fn parse_typed_with_depth<T: serde::de::DeserializeOwned>(
    json: &str,
    max_depth: usize,
) -> Result<T, String> {
    todo!("Parse typed JSON with depth limit")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_number() {
        assert_eq!(max_json_depth("42"), 0);
    }

    #[test]
    fn test_simple_array() {
        assert_eq!(max_json_depth("[1,2,3]"), 1);
    }

    #[test]
    fn test_nested_arrays() {
        assert_eq!(max_json_depth("[[1],[2]]"), 2);
    }

    #[test]
    fn test_deeply_nested() {
        assert_eq!(max_json_depth("[[[[[]]]]]"), 4);
    }

    #[test]
    fn test_object_nesting() {
        assert_eq!(max_json_depth(r#"{"a":{"b":{"c":1}}}"#), 3);
    }

    #[test]
    fn test_string_not_counted() {
        // Brackets inside strings should not count
        assert_eq!(max_json_depth(r#"{"key":"[[["}"#), 1);
    }

    #[test]
    fn test_escaped_quote_in_string() {
        assert_eq!(max_json_depth(r#"{"key":"he\"llo"}"#), 1);
    }

    #[test]
    fn test_exceeds_depth_true() {
        assert!(exceeds_depth("[[[[[]]]]]", 3).unwrap());
    }

    #[test]
    fn test_exceeds_depth_false() {
        assert!(!exceeds_depth("[[[[[]]]]]", 5).unwrap());
    }

    #[test]
    fn test_parse_within_limit() {
        let json = r#"[1, [2, [3]]]"#;
        let val = parse_with_depth_limit(json, 3).unwrap();
        assert!(val.is_array());
    }

    #[test]
    fn test_parse_exceeds_limit() {
        let json = r#"[1, [2, [3]]]"#;
        assert!(parse_with_depth_limit(json, 2).is_err());
    }

    #[test]
    fn test_safe_parse_size_limit() {
        let json = r#"[1,2,3]"#;
        assert!(safe_parse_json(json, 100, 5).is_ok());
        assert!(safe_parse_json(json, 3, 5).is_err());
    }

    #[test]
    fn test_typed_parse_with_depth() {
        let json = r#"[1,2,3]"#;
        let val: Vec<i64> = parse_typed_with_depth(json, 2).unwrap();
        assert_eq!(val, vec![1, 2, 3]);
    }
}
