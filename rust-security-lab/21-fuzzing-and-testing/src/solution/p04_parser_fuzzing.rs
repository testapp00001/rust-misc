//! # Lesson 04: Fuzzing Parsers (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use proptest::prelude::*;
use std::collections::HashMap;

/// A simple JSON value type.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

/// Parse a simple JSON string value (between double quotes).
pub fn parse_string(s: &str) -> Result<(&str, &str), &'static str> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'"' {
        return Err("expected '\"'");
    }
    for i in 1..bytes.len() {
        if bytes[i] == b'"' {
            return Ok((&s[1..i], &s[i + 1..]));
        }
    }
    Err("unterminated string")
}

/// Parse a JSON integer (possibly negative).
pub fn parse_integer(s: &str) -> Result<(i64, &str), &'static str> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return Err("empty input");
    }
    let start = if bytes[0] == b'-' { 1 } else { 0 };
    if start >= bytes.len() || !bytes[start].is_ascii_digit() {
        return Err("expected digit");
    }
    let mut end = start;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    let num: i64 = s[..end].parse().map_err(|_| "integer overflow")?;
    Ok((num, &s[end..]))
}

/// Parse a JSON boolean literal.
pub fn parse_bool(s: &str) -> Result<(bool, &str), &'static str> {
    if s.starts_with("true") {
        Ok((true, &s[4..]))
    } else if s.starts_with("false") {
        Ok((false, &s[5..]))
    } else {
        Err("expected 'true' or 'false'")
    }
}

/// Parse a JSON null literal.
pub fn parse_null(s: &str) -> Result<(), &'static str> {
    if s.starts_with("null") {
        Ok(())
    } else {
        Err("expected 'null'")
    }
}

/// Parse a comma-separated list of JSON integers: "[1, 2, 3]"
pub fn parse_int_array(s: &str) -> Result<Vec<i64>, &'static str> {
    let s = skip_whitespace(s);
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'[' {
        return Err("expected '['");
    }
    let mut rest = &s[1..];
    rest = skip_whitespace(rest);
    let mut result = Vec::new();

    if rest.starts_with(']') {
        return Ok(result);
    }

    loop {
        let (val, new_rest) = parse_integer(rest)?;
        result.push(val);
        rest = skip_whitespace(new_rest);

        if rest.is_empty() {
            return Err("unexpected end of input");
        }
        if rest.starts_with(']') {
            return Ok(result);
        }
        if rest.starts_with(',') {
            rest = skip_whitespace(&rest[1..]);
        } else {
            return Err("expected ',' or ']'");
        }
    }
}

/// Depth-limited parser for nested JSON arrays.
pub fn parse_nested_arrays(s: &str, max_depth: usize) -> Result<Vec<JsonValue>, &'static str> {
    let s = skip_whitespace(s);
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'[' {
        return Err("expected '['");
    }
    if max_depth == 0 {
        return Err("maximum nesting depth exceeded");
    }
    let mut rest = &s[1..];
    rest = skip_whitespace(rest);
    let mut result = Vec::new();

    if rest.starts_with(']') {
        return Ok(result);
    }

    loop {
        rest = skip_whitespace(rest);
        if rest.starts_with('[') {
            // Nested array
            let (inner, new_rest) = parse_nested_arrays_inner(rest, max_depth - 1)?;
            result.push(JsonValue::Array(inner));
            rest = new_rest;
        } else {
            // Integer
            let (val, new_rest) = parse_integer(rest)?;
            result.push(JsonValue::Number(val as f64));
            rest = new_rest;
        }

        rest = skip_whitespace(rest);
        if rest.is_empty() {
            return Err("unexpected end of input");
        }
        if rest.starts_with(']') {
            return Ok(result);
        }
        if rest.starts_with(',') {
            rest = &rest[1..];
        } else {
            return Err("expected ',' or ']'");
        }
    }
}

fn parse_nested_arrays_inner(s: &str, remaining_depth: usize) -> Result<(Vec<JsonValue>, &str), &'static str> {
    if remaining_depth == 0 {
        return Err("maximum nesting depth exceeded");
    }
    let mut rest = &s[1..]; // skip '['
    rest = skip_whitespace(rest);
    let mut result = Vec::new();

    if rest.starts_with(']') {
        return Ok((result, &rest[1..]));
    }

    loop {
        rest = skip_whitespace(rest);
        if rest.starts_with('[') {
            let (inner, new_rest) = parse_nested_arrays_inner(rest, remaining_depth - 1)?;
            result.push(JsonValue::Array(inner));
            rest = new_rest;
        } else {
            let (val, new_rest) = parse_integer(rest)?;
            result.push(JsonValue::Number(val as f64));
            rest = new_rest;
        }

        rest = skip_whitespace(rest);
        if rest.is_empty() {
            return Err("unexpected end of input");
        }
        if rest.starts_with(']') {
            return Ok((result, &rest[1..]));
        }
        if rest.starts_with(',') {
            rest = &rest[1..];
        } else {
            return Err("expected ',' or ']'");
        }
    }
}

/// Parse a key-value pair: "key": value
pub fn parse_kv_pair(s: &str) -> Result<(&str, i64, &str), &'static str> {
    let (key, rest) = parse_string(s)?;
    let rest = skip_whitespace(rest);
    if rest.is_empty() || !rest.starts_with(':') {
        return Err("expected ':'");
    }
    let rest = skip_whitespace(&rest[1..]);
    let (val, rest) = parse_integer(rest)?;
    Ok((key, val, rest))
}

/// Skip whitespace characters from the start of a string.
pub fn skip_whitespace(s: &str) -> &str {
    let trimmed = s.trim_start();
    trimmed
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
