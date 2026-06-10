//! # Lesson 04: Fuzzing Parsers — JSON, XML, and Custom Formats
//!
//! ## Why Parsers are the #1 Fuzzing Target
//!
//! Parsers are the boundary between trusted code and untrusted input. Every network
//! packet, file format, and API request goes through a parser. A bug in a parser can
//! lead to:
//! - Denial of service (infinite loop, stack overflow)
//! - Memory corruption (buffer overflow, use-after-free)
//! - Logic bugs (incorrect data interpretation)
//!
//! ## The OWASP Parser Fuzzing Checklist
//!
//! 1. Empty input
//! 2. Extremely long input (megabytes)
//! 3. Invalid UTF-8 sequences
//! 4. Nested structures (deeply nested JSON objects)
//! 5. Mismatched delimiters
//! 6. Null bytes in strings
//! 7. Integer overflow in length fields
//! 8. Duplicate keys
//!
//! ## Security Perspective
//!
//! ### Attack: Billion Laughs (XML Bomb)
//! Nested entity expansion can consume exponential memory:
//! ```xml
//! <!DOCTYPE bomb [
//!   <!ENTITY a "1234567890">
//!   <!ENTITY b "&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;">
//!   <!ENTITY c "&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;">
//! ]>
//! <bomb>&c;</bomb>
//! ```
//!
//! ### Defense: Limit Depth, Length, and Recursion
//! - Set maximum nesting depth (e.g., 64)
//! - Set maximum string length
//! - Set maximum total input size
//! - Never use recursive descent without depth limits

use std::collections::HashMap;

/// A simple JSON value type for our custom parser.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

/// Parse a "simple JSON" string value (without escape handling).
///
/// Reads characters between two double-quote characters.
/// Returns the string content and the remaining unparsed input.
///
/// Rules:
/// - Input must start with '"'
/// - Read until the next unescaped '"'
/// - If no closing quote found, return Err
///
/// Hints:
/// - Check first char is '"'
/// - Iterate from index 1, looking for '"'
/// - Return (&str, remaining_slice)
pub fn parse_string(s: &str) -> Result<(&str, &str), &'static str> {
    todo!("Implement simple JSON string parser")
}

/// Parse a JSON integer (possibly negative).
///
/// Reads digits (and optional leading '-') from the start of the string.
/// Returns the parsed i64 and remaining input.
///
/// Hints:
/// - Track if the first char is '-'
/// - Read digits into a slice
/// - Parse with `str::parse::<i64>()`
/// - Return Err if no digits found
pub fn parse_integer(s: &str) -> Result<(i64, &str), &'static str> {
    todo!("Implement JSON integer parser")
}

/// Parse a JSON boolean literal ("true" or "false").
///
/// Returns the bool value and remaining input.
///
/// Hints:
/// - Check if input starts with "true" or "false"
/// - Return the corresponding bool and the rest of the string
pub fn parse_bool(s: &str) -> Result<(bool, &str), &'static str> {
    todo!("Implement JSON boolean parser")
}

/// Parse a JSON null literal.
///
/// Returns () and remaining input.
///
/// Hints:
/// - Check if input starts with "null"
/// - Return ((), rest) where rest is input[4..]
pub fn parse_null(s: &str) -> Result<(), &'static str> {
    todo!("Implement JSON null parser")
}

/// Parse a comma-separated list of JSON integers.
///
/// Format: "[1, 2, 3]" or "[]"
///
/// Rules:
/// - Must start with '[' and end with ']'
/// - Elements are integers separated by commas
/// - Optional whitespace around commas and brackets
/// - Empty array "[]" is valid
///
/// Hints:
/// - Strip '[' and whitespace
/// - If next char is ']', return empty vec
/// - Loop: parse integer, skip whitespace, expect ',' or ']'
/// - Handle trailing comma? No -- strict JSON
pub fn parse_int_array(s: &str) -> Result<Vec<i64>, &'static str> {
    todo!("Implement JSON integer array parser")
}

/// Depth-limited parser for nested JSON arrays.
///
/// Format: "[[1, 2], [3, [4, 5]]]"
///
/// This parser enforces a maximum nesting depth to prevent stack overflow
/// attacks (billion laughs style).
///
/// Hints:
/// - Accept a `max_depth` parameter
/// - If depth reaches 0 and we see '[', return error
/// - Recursively parse inner arrays with depth - 1
/// - Parse integers as leaf values
pub fn parse_nested_arrays(s: &str, max_depth: usize) -> Result<Vec<JsonValue>, &'static str> {
    todo!("Implement depth-limited nested array parser")
}

/// Parse a key-value pair: "key": value
///
/// The key is a string, followed by ':', followed by an integer value.
/// Returns (key_string, value, remaining_input).
///
/// Hints:
/// - Use `parse_string` to get the key
/// - Skip whitespace and ':'
/// - Use `parse_integer` to get the value
pub fn parse_kv_pair(s: &str) -> Result<(&str, i64, &str), &'static str> {
    todo!("Implement key-value pair parser")
}

/// Skip whitespace characters from the start of a string.
///
/// Returns the remaining string after all leading whitespace.
///
/// Hints:
/// - Use `s.trim_start()` or iterate and skip ' ', '\t', '\n', '\r'
pub fn skip_whitespace(s: &str) -> &str {
    todo!("Implement whitespace skipping")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_string ---

    #[test]
    fn test_parse_string_basic() {
        let (val, rest) = parse_string("\"hello\"").unwrap();
        assert_eq!(val, "hello");
        assert_eq!(rest, "");
    }

    #[test]
    fn test_parse_string_with_remainder() {
        let (val, rest) = parse_string("\"abc\", 42").unwrap();
        assert_eq!(val, "abc");
        assert_eq!(rest, ", 42");
    }

    #[test]
    fn test_parse_string_empty() {
        let (val, _) = parse_string("\"\"").unwrap();
        assert_eq!(val, "");
    }

    #[test]
    fn test_parse_string_no_closing_quote() {
        assert!(parse_string("\"unclosed").is_err());
    }

    #[test]
    fn test_parse_string_no_opening_quote() {
        assert!(parse_string("hello").is_err());
    }

    // --- parse_integer ---

    #[test]
    fn test_parse_integer_positive() {
        let (val, rest) = parse_integer("42abc").unwrap();
        assert_eq!(val, 42);
        assert_eq!(rest, "abc");
    }

    #[test]
    fn test_parse_integer_negative() {
        let (val, _) = parse_integer("-7").unwrap();
        assert_eq!(val, -7);
    }

    #[test]
    fn test_parse_integer_no_digits() {
        assert!(parse_integer("abc").is_err());
    }

    // --- parse_bool ---

    #[test]
    fn test_parse_bool_true() {
        let (val, rest) = parse_bool("true, false").unwrap();
        assert!(val);
        assert_eq!(rest, ", false");
    }

    #[test]
    fn test_parse_bool_false() {
        let (val, rest) = parse_bool("false]").unwrap();
        assert!(!val);
        assert_eq!(rest, "]");
    }

    #[test]
    fn test_parse_bool_invalid() {
        assert!(parse_bool("maybe").is_err());
    }

    // --- parse_null ---

    #[test]
    fn test_parse_null_valid() {
        assert!(parse_null("null").is_ok());
    }

    #[test]
    fn test_parse_null_invalid() {
        assert!(parse_null("nada").is_err());
    }

    // --- parse_int_array ---

    #[test]
    fn test_parse_int_array_empty() {
        let arr = parse_int_array("[]").unwrap();
        assert!(arr.is_empty());
    }

    #[test]
    fn test_parse_int_array_basic() {
        let arr = parse_int_array("[1, 2, 3]").unwrap();
        assert_eq!(arr, vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_int_array_no_brackets() {
        assert!(parse_int_array("1, 2, 3").is_err());
    }

    // --- parse_nested_arrays ---

    #[test]
    fn test_nested_arrays_flat() {
        let result = parse_nested_arrays("[1, 2, 3]", 10).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_nested_arrays_depth_limit() {
        // Deeply nested array should fail with depth limit of 2
        let result = parse_nested_arrays("[[[[1]]]]", 2);
        assert!(result.is_err(), "Should reject nesting beyond max_depth");
    }

    // --- parse_kv_pair ---

    #[test]
    fn test_kv_pair_basic() {
        let (key, val, rest) = parse_kv_pair("\"name\": 42, ...").unwrap();
        assert_eq!(key, "name");
        assert_eq!(val, 42);
        assert_eq!(rest, ", ...");
    }

    // --- skip_whitespace ---

    #[test]
    fn test_skip_whitespace() {
        assert_eq!(skip_whitespace("  hello"), "hello");
        assert_eq!(skip_whitespace("\t\n\r test"), "test");
        assert_eq!(skip_whitespace("no_ws"), "no_ws");
    }

    // --- Fuzzing-style: no panics on arbitrary input ---

    proptest::proptest! {
        #[test]
        fn test_parse_string_never_panics(s in ".*") {
            let _ = parse_string(&s);
        }

        #[test]
        fn test_parse_integer_never_panics(s in ".*") {
            let _ = parse_integer(&s);
        }

        #[test]
        fn test_parse_int_array_never_panics(s in ".*") {
            let _ = parse_int_array(&s);
        }

        #[test]
        fn test_parse_nested_arrays_never_panics(
            s in ".*",
            depth in 0usize..100
        ) {
            let _ = parse_nested_arrays(&s, depth);
        }

        #[test]
        fn test_parse_bool_never_panics(s in ".*") {
            let _ = parse_bool(&s);
        }

        #[test]
        fn test_parse_null_never_panics(s in ".*") {
            let _ = parse_null(&s);
        }
    }
}
