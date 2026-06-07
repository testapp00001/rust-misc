//! # Node.js Native Modules with Rust
//!
//! napi-rs allows building Node.js native modules in Rust. This module
//! covers the patterns for creating high-performance Node.js addons.
//!
//! ## Setup:
//!
//! ```toml
//! [dependencies]
//! napi = "2"
//! napi-derive = "2"
//!
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! ## Key Concepts:
//!
//! - `#[napi]`: Macro to expose functions/classes to Node.js
//! - Async support via Tokio
//! - Buffer sharing between Node.js and Rust

/// Simulated napi-compatible function signatures.
/// In real napi-rs code, these would use #[napi] macro.
pub mod node_api {
    use std::collections::HashMap;

    /// A Node.js-compatible object.
    #[derive(Debug, Clone)]
    pub struct NodeObject {
        properties: HashMap<String, NodeValue>,
    }

    #[derive(Debug, Clone)]
    pub enum NodeValue {
        Number(f64),
        String(String),
        Boolean(bool),
        Array(Vec<NodeValue>),
        Object(NodeObject),
        Null,
    }

    impl NodeObject {
        pub fn new() -> Self {
            Self {
                properties: HashMap::new(),
            }
        }

        pub fn set(&mut self, key: &str, value: NodeValue) {
            self.properties.insert(key.into(), value);
        }

        pub fn get(&self, key: &str) -> Option<&NodeValue> {
            self.properties.get(key)
        }

        pub fn keys(&self) -> Vec<&str> {
            self.properties.keys().map(|s| s.as_str()).collect()
        }

        pub fn len(&self) -> usize {
            self.properties.len()
        }
    }

    /// Buffer type compatible with Node.js Buffer.
    pub struct NodeBuffer {
        data: Vec<u8>,
    }

    impl NodeBuffer {
        pub fn new(data: Vec<u8>) -> Self {
            Self { data }
        }

        pub fn from_string(s: &str) -> Self {
            Self {
                data: s.as_bytes().to_vec(),
            }
        }

        pub fn as_bytes(&self) -> &[u8] {
            &self.data
        }

        pub fn len(&self) -> usize {
            self.data.len()
        }

        pub fn is_empty(&self) -> bool {
            self.data.is_empty()
        }

        pub fn to_string_lossy(&self) -> String {
            String::from_utf8_lossy(&self.data).into_owned()
        }
    }

    /// Async task result for Node.js async/await.
    pub struct AsyncTask<T> {
        result: Option<T>,
        error: Option<String>,
    }

    impl<T> AsyncTask<T> {
        pub fn ok(value: T) -> Self {
            Self {
                result: Some(value),
                error: None,
            }
        }

        pub fn err(message: &str) -> Self {
            Self {
                result: None,
                error: Some(message.into()),
            }
        }

        pub fn is_ok(&self) -> bool {
            self.result.is_some()
        }

        pub fn unwrap(self) -> T {
            self.result.unwrap()
        }

        pub fn error_message(&self) -> Option<&str> {
            self.error.as_deref()
        }
    }

    /// Error type compatible with Node.js exceptions.
    #[derive(Debug)]
    pub struct NodeError {
        pub code: String,
        pub message: String,
    }

    impl NodeError {
        pub fn new(code: &str, message: &str) -> Self {
            Self {
                code: code.into(),
                message: message.into(),
            }
        }

        pub fn type_error(message: &str) -> Self {
            Self::new("ERR_INVALID_ARG_TYPE", message)
        }

        pub fn range_error(message: &str) -> Self {
            Self::new("ERR_OUT_OF_RANGE", message)
        }
    }

    impl std::fmt::Display for NodeError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{}] {}", self.code, self.message)
        }
    }

    impl std::error::Error for NodeError {}
}

/// Example: High-performance string processing module.
pub struct StringProcessor;

impl StringProcessor {
    pub fn reverse(s: &str) -> String {
        s.chars().rev().collect()
    }

    pub fn count_chars(s: &str) -> std::collections::HashMap<char, usize> {
        let mut counts = std::collections::HashMap::new();
        for c in s.chars() {
            *counts.entry(c).or_insert(0) += 1;
        }
        counts
    }

    pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0usize; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                    .min(matrix[i + 1][j] + 1)
                    .min(matrix[i][j] + cost);
            }
        }

        matrix[len1][len2]
    }
}

/// Example: JSON processing module for Node.js.
pub struct JsonProcessor;

impl JsonProcessor {
    /// Pretty-print a JSON string with indentation.
    /// This is a simplified formatter — in production you'd use serde_json.
    pub fn pretty_print(json: &str) -> Result<String, String> {
        let trimmed = json.trim();
        // Validate it looks like JSON (basic check)
        if trimmed.is_empty()
            || (!trimmed.starts_with('{')
                && !trimmed.starts_with('[')
                && !trimmed.starts_with('"')
                && !trimmed.starts_with("null")
                && !trimmed.starts_with("true")
                && !trimmed.starts_with("false")
                && !trimmed.chars().next().map_or(false, |c| c.is_ascii_digit() || c == '-'))
        {
            return Err("Invalid JSON".into());
        }
        // Simple pretty-print: add newlines and indentation after , and :
        let mut result = String::new();
        let mut indent: usize = 0;
        let mut in_string = false;
        let mut prev_char = '\0';

        for ch in trimmed.chars() {
            if ch == '"' && prev_char != '\\' {
                in_string = !in_string;
                result.push(ch);
            } else if in_string {
                result.push(ch);
            } else {
                match ch {
                    '{' | '[' => {
                        indent += 1;
                        result.push(ch);
                        result.push('\n');
                        result.push_str(&"  ".repeat(indent));
                    }
                    '}' | ']' => {
                        indent = indent.saturating_sub(1);
                        result.push('\n');
                        result.push_str(&"  ".repeat(indent));
                        result.push(ch);
                    }
                    ',' => {
                        result.push(ch);
                        result.push('\n');
                        result.push_str(&"  ".repeat(indent));
                    }
                    ':' => {
                        result.push(ch);
                        result.push(' ');
                    }
                    ' ' | '\n' | '\r' | '\t' => {} // skip existing whitespace
                    _ => result.push(ch),
                }
            }
            prev_char = ch;
        }
        Ok(result)
    }

    /// Extract top-level keys from a JSON object string.
    pub fn extract_keys(json: &str) -> Result<Vec<String>, String> {
        let trimmed = json.trim();
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            return Err("Not a JSON object".into());
        }
        let inner = trimmed[1..trimmed.len() - 1].trim();
        if inner.is_empty() {
            return Ok(Vec::new());
        }
        let mut keys = Vec::new();
        let mut in_string = false;
        let mut in_key = false;
        let mut current_key = String::new();
        let mut depth = 0i32;
        let mut prev_char = '\0';

        for ch in inner.chars() {
            if ch == '"' && prev_char != '\\' {
                if !in_string {
                    if depth == 0 && !in_key {
                        in_key = true;
                        current_key.clear();
                    }
                    in_string = true;
                } else {
                    in_string = false;
                    if in_key {
                        keys.push(current_key.clone());
                        in_key = false;
                    }
                }
            } else if in_string {
                if in_key {
                    current_key.push(ch);
                }
            } else {
                match ch {
                    '{' | '[' => depth += 1,
                    '}' | ']' => depth -= 1,
                    _ => {}
                }
            }
            prev_char = ch;
        }
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::node_api::*;
    use super::*;

    #[test]
    fn test_node_object() {
        let mut obj = NodeObject::new();
        obj.set("name", NodeValue::String("test".into()));
        obj.set("value", NodeValue::Number(42.0));

        assert_eq!(obj.len(), 2);
        match obj.get("name") {
            Some(NodeValue::String(s)) => assert_eq!(s, "test"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_node_object_keys() {
        let mut obj = NodeObject::new();
        obj.set("a", NodeValue::Number(1.0));
        obj.set("b", NodeValue::Number(2.0));

        let mut keys = obj.keys();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn test_node_buffer() {
        let buf = NodeBuffer::from_string("hello");
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.to_string_lossy(), "hello");
    }

    #[test]
    fn test_node_buffer_empty() {
        let buf = NodeBuffer::new(vec![]);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_async_task_ok() {
        let task = AsyncTask::ok(42);
        assert!(task.is_ok());
        assert_eq!(task.unwrap(), 42);
    }

    #[test]
    fn test_async_task_err() {
        let task: AsyncTask<i32> = AsyncTask::err("something failed");
        assert!(!task.is_ok());
        assert_eq!(task.error_message(), Some("something failed"));
    }

    #[test]
    fn test_node_error() {
        let err = NodeError::type_error("Expected string");
        assert_eq!(err.code, "ERR_INVALID_ARG_TYPE");
        assert!(format!("{}", err).contains("Expected string"));
    }

    #[test]
    fn test_node_error_range() {
        let err = NodeError::range_error("Index out of bounds");
        assert_eq!(err.code, "ERR_OUT_OF_RANGE");
    }

    #[test]
    fn test_string_processor_reverse() {
        assert_eq!(StringProcessor::reverse("hello"), "olleh");
        assert_eq!(StringProcessor::reverse(""), "");
    }

    #[test]
    fn test_string_processor_count_chars() {
        let counts = StringProcessor::count_chars("aab");
        assert_eq!(counts.get(&'a'), Some(&2));
        assert_eq!(counts.get(&'b'), Some(&1));
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(StringProcessor::levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(StringProcessor::levenshtein_distance("", "abc"), 3);
        assert_eq!(StringProcessor::levenshtein_distance("abc", "abc"), 0);
    }

    #[test]
    fn test_json_processor_pretty_print() {
        let json = r#"{"name":"test","value":42}"#;
        let pretty = JsonProcessor::pretty_print(json).unwrap();
        assert!(pretty.contains("name"));
        assert!(pretty.contains("42"));
    }

    #[test]
    fn test_json_processor_extract_keys() {
        let json = r#"{"a":1,"b":2,"c":3}"#;
        let mut keys = JsonProcessor::extract_keys(json).unwrap();
        keys.sort();
        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_json_processor_invalid() {
        let result = JsonProcessor::pretty_print("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_processor_non_object() {
        let result = JsonProcessor::extract_keys("[1,2,3]");
        assert!(result.is_err());
    }
}
