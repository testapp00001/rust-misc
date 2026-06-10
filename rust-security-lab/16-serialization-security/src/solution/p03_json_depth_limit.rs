//! # Lesson 03: JSON Depth Limiting (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Count the maximum nesting depth of a JSON string.
///
/// Depth counts the number of enclosing non-empty containers.
/// An empty container `[]` or `{}` does not increase depth.
pub fn max_json_depth(json: &str) -> usize {
    let bytes: Vec<u8> = json.bytes().collect();
    let len = bytes.len();
    let mut max_depth = 0usize;
    let mut current_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;

    while i < len {
        let byte = bytes[i];
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if byte == b'\\' && in_string {
            escaped = true;
            i += 1;
            continue;
        }
        if byte == b'"' {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if in_string {
            i += 1;
            continue;
        }
        match byte {
            b'{' | b'[' => {
                let close = if byte == b'[' { b']' } else { b'}' };
                // Check if next non-whitespace char is the closing bracket (empty container)
                let mut j = i + 1;
                while j < len && bytes[j] == b' ' {
                    j += 1;
                }
                if j < len && bytes[j] == close {
                    // Empty container -- skip it, don't count as depth
                    i = j + 1;
                    continue;
                }
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
        i += 1;
    }

    max_depth
}

/// Check if a JSON string exceeds a maximum depth.
pub fn exceeds_depth(json: &str, max_depth: usize) -> Result<bool, String> {
    Ok(max_json_depth(json) > max_depth)
}

/// Parse JSON only if depth is within limit.
pub fn parse_with_depth_limit(json: &str, max_depth: usize) -> Result<serde_json::Value, String> {
    if max_json_depth(json) > max_depth {
        return Err(format!("JSON depth exceeds maximum {}", max_depth));
    }
    serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))
}

/// A safe JSON parser that enforces both depth and size limits.
pub fn safe_parse_json(
    json: &str,
    max_bytes: usize,
    max_depth: usize,
) -> Result<serde_json::Value, String> {
    if json.len() > max_bytes {
        return Err(format!("JSON size {} exceeds maximum {}", json.len(), max_bytes));
    }
    parse_with_depth_limit(json, max_depth)
}

/// Parse JSON into a typed struct with depth checking.
pub fn parse_typed_with_depth<T: serde::de::DeserializeOwned>(
    json: &str,
    max_depth: usize,
) -> Result<T, String> {
    if max_json_depth(json) > max_depth {
        return Err(format!("JSON depth exceeds maximum {}", max_depth));
    }
    serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))
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
