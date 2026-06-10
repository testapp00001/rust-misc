//! # Lesson 08: JSON Parsing Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde_json::Value;

pub fn json_depth(value: &Value) -> usize {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => 1,
        Value::Array(arr) => {
            let max_child = arr.iter().map(|v| json_depth(v)).max().unwrap_or(0);
            1 + max_child
        }
        Value::Object(map) => {
            let max_child = map.values().map(|v| json_depth(v)).max().unwrap_or(0);
            1 + max_child
        }
    }
}

pub fn count_json_keys(value: &Value) -> usize {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => 0,
        Value::Array(arr) => arr.iter().map(|v| count_json_keys(v)).sum(),
        Value::Object(map) => {
            map.len() + map.values().map(|v| count_json_keys(v)).sum::<usize>()
        }
    }
}

pub fn validate_json_security(
    value: &Value,
    max_depth: usize,
    max_keys: usize,
    max_bytes: usize,
) -> Result<(), &'static str> {
    if json_depth(value) > max_depth {
        return Err("too_deep");
    }

    if count_json_keys(value) > max_keys {
        return Err("too_many_keys");
    }

    let serialized = serde_json::to_string(value).map_err(|_| "serialization_error")?;
    if serialized.len() > max_bytes {
        return Err("payload_too_large");
    }

    Ok(())
}

pub fn detect_duplicate_keys(json_str: &str) -> Vec<String> {
    // Simplified detection: scan for repeated key patterns in the raw string
    // This is a heuristic approach since serde_json deduplicates on parse
    let mut duplicates = Vec::new();

    // Find all quoted strings that appear as keys (before a colon)
    let mut seen_keys: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    let chars: Vec<char> = json_str.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '"' {
            // Find the end of this string
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && chars[end] != '"' {
                if chars[end] == '\\' {
                    end += 1; // skip escaped char
                }
                end += 1;
            }
            if end < chars.len() {
                let key: String = chars[start..end].iter().collect();
                // Check if this is followed by a colon (making it a key)
                let mut j = end + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && chars[j] == ':' {
                    seen_keys.entry(key).or_default().push(i);
                }
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }

    for (key, positions) in &seen_keys {
        if positions.len() > 1 {
            duplicates.push(format!("$.{}", key));
        }
    }

    duplicates
}

pub fn sanitize_json(
    value: &Value,
    max_string_length: usize,
    max_array_length: usize,
    max_object_keys: usize,
    max_depth: usize,
) -> Value {
    sanitize_json_inner(value, max_string_length, max_array_length, max_object_keys, max_depth, 1)
}

fn sanitize_json_inner(
    value: &Value,
    max_string_length: usize,
    max_array_length: usize,
    max_object_keys: usize,
    max_depth: usize,
    current_depth: usize,
) -> Value {
    if current_depth > max_depth {
        return Value::Null;
    }
    // If we're at max_depth, replace nested containers with null
    if current_depth == max_depth {
        match value {
            Value::Array(_) | Value::Object(_) => return Value::Null,
            _ => {}
        }
    }

    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => value.clone(),
        Value::String(s) => {
            if s.len() > max_string_length {
                // Truncate carefully at UTF-8 boundary
                let truncated: String = s.chars().take(max_string_length).collect();
                Value::String(truncated)
            } else {
                value.clone()
            }
        }
        Value::Array(arr) => {
            let truncated: Vec<Value> = arr
                .iter()
                .take(max_array_length)
                .map(|v| sanitize_json_inner(v, max_string_length, max_array_length, max_object_keys, max_depth, current_depth + 1))
                .collect();
            Value::Array(truncated)
        }
        Value::Object(map) => {
            let truncated: serde_json::Map<String, Value> = map
                .iter()
                .take(max_object_keys)
                .map(|(k, v)| {
                    let sanitized = sanitize_json_inner(v, max_string_length, max_array_length, max_object_keys, max_depth, current_depth + 1);
                    (k.clone(), sanitized)
                })
                .collect();
            Value::Object(truncated)
        }
    }
}

pub fn safe_json_parse(
    json_str: &str,
    max_bytes: usize,
    max_depth: usize,
    max_keys: usize,
) -> Result<Value, String> {
    // Check string length first
    if json_str.len() > max_bytes {
        return Err("payload_too_large".to_string());
    }

    // Parse the JSON
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("parse_error:{}", e))?;

    // Validate depth
    if json_depth(&value) > max_depth {
        return Err("too_deep".to_string());
    }

    // Validate key count
    if count_json_keys(&value) > max_keys {
        return Err("too_many_keys".to_string());
    }

    Ok(value)
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
        assert_eq!(count_json_keys(&val), 3);
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
