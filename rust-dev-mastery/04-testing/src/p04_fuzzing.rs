//! # Lesson 4: Fuzzing
//!
//! Fuzzing tests code with random, unexpected, and malformed inputs.
//! This lesson covers cargo-fuzz, libfuzzer targets, the arbitrary crate,
//! and corpus management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Code that's good fuzz targets: parsers, decoders, serializers
// ---------------------------------------------------------------------------

/// A simple JSON-like value parser.
/// This is the kind of code that benefits enormously from fuzzing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

/// Parse a simplified JSON value from a string.
/// Fuzzing would find edge cases like malformed UTF-8, deeply nested
/// structures, and unexpected characters.
pub fn parse_value(input: &str) -> Result<JsonValue, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError::UnexpectedEof);
    }

    match input.chars().next().unwrap() {
        'n' => {
            if input.starts_with("null") {
                Ok(JsonValue::Null)
            } else {
                Err(ParseError::UnexpectedToken)
            }
        }
        't' => {
            if input.starts_with("true") {
                Ok(JsonValue::Bool(true))
            } else {
                Err(ParseError::UnexpectedToken)
            }
        }
        'f' => {
            if input.starts_with("false") {
                Ok(JsonValue::Bool(false))
            } else {
                Err(ParseError::UnexpectedToken)
            }
        }
        '"' => parse_string(input),
        '0'..='9' | '-' => parse_number(input),
        '[' => parse_array(input),
        '{' => parse_object(input),
        _ => Err(ParseError::UnexpectedToken),
    }
}

fn parse_string(input: &str) -> Result<JsonValue, ParseError> {
    if !input.starts_with('"') || !input.ends_with('"') || input.len() < 2 {
        return Err(ParseError::UnterminatedString);
    }
    let inner = &input[1..input.len() - 1];
    Ok(JsonValue::Str(inner.to_string()))
}

fn parse_number(input: &str) -> Result<JsonValue, ParseError> {
    let num: f64 = input
        .parse()
        .map_err(|_| ParseError::InvalidNumber)?;
    Ok(JsonValue::Number(num))
}

fn parse_array(input: &str) -> Result<JsonValue, ParseError> {
    if !input.starts_with('[') || !input.ends_with(']') {
        return Err(ParseError::UnexpectedToken);
    }
    let inner = input[1..input.len() - 1].trim();
    if inner.is_empty() {
        return Ok(JsonValue::Array(vec![]));
    }
    let items: Result<Vec<_>, _> = inner
        .split(',')
        .map(|s| parse_value(s.trim()))
        .collect();
    Ok(JsonValue::Array(items?))
}

fn parse_object(input: &str) -> Result<JsonValue, ParseError> {
    if !input.starts_with('{') || !input.ends_with('}') {
        return Err(ParseError::UnexpectedToken);
    }
    let inner = input[1..input.len() - 1].trim();
    if inner.is_empty() {
        return Ok(JsonValue::Object(HashMap::new()));
    }
    let mut map = HashMap::new();
    for pair in inner.split(',') {
        let parts: Vec<&str> = pair.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(ParseError::UnexpectedToken);
        }
        let key = parts[0].trim();
        let key = key.trim_matches('"');
        let value = parse_value(parts[1].trim())?;
        map.insert(key.to_string(), value);
    }
    Ok(JsonValue::Object(map))
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedEof,
    UnexpectedToken,
    UnterminatedString,
    InvalidNumber,
}

// ---------------------------------------------------------------------------
// Fuzz target function (what cargo-fuzz would call)
// ---------------------------------------------------------------------------

/// The function that a fuzz target would call.
/// In a real project, this would be in fuzz/fuzz_targets/parse_value.rs:
///
/// ```rust,ignore
/// #![no_main]
/// use libfuzzer_sys::fuzz_target;
///
/// fuzz_target!(|data: &str| {
///     let _ = testing_mastery::p04_fuzzing::fuzz_entrypoint(data);
/// });
/// ```
pub fn fuzz_entrypoint(data: &str) -> Result<(), String> {
    // The key invariant: parse_value must never panic
    let result = parse_value(data);

    // Additional checks on successful parses
    if let Ok(ref value) = result {
        // Serialization should not panic
        let _ = serde_json::to_string(value);
    }

    result.map(|_| ()).map_err(|e| format!("{:?}", e))
}

// ---------------------------------------------------------------------------
// Arbitrary trait implementation for structured fuzzing
// ---------------------------------------------------------------------------

/// Demonstrates the arbitrary crate pattern for structured fuzzing.
/// In real fuzzing, you'd derive Arbitrary on your types.
#[derive(Debug, Clone, PartialEq)]
pub enum FuzzCommand {
    Insert(String, String),
    Get(String),
    Delete(String),
    List,
    Clear,
}

/// Generate a sequence of commands from raw bytes.
/// This is how libfuzzer converts random bytes into structured input.
pub fn commands_from_bytes(bytes: &[u8]) -> Vec<FuzzCommand> {
    let mut commands = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let cmd = match bytes[i] % 5 {
            0 => {
                let key = extract_string(bytes, i + 1, 8);
                let val = extract_string(bytes, i + 9, 8);
                i += 17;
                FuzzCommand::Insert(key, val)
            }
            1 => {
                let key = extract_string(bytes, i + 1, 8);
                i += 9;
                FuzzCommand::Get(key)
            }
            2 => {
                let key = extract_string(bytes, i + 1, 8);
                i += 9;
                FuzzCommand::Delete(key)
            }
            3 => {
                i += 1;
                FuzzCommand::List
            }
            _ => {
                i += 1;
                FuzzCommand::Clear
            }
        };
        commands.push(cmd);
    }

    commands
}

fn extract_string(bytes: &[u8], start: usize, max_len: usize) -> String {
    let end = (start + max_len).min(bytes.len());
    if start >= bytes.len() {
        return String::new();
    }
    bytes[start..end]
        .iter()
        .map(|&b| b as char)
        .collect()
}

/// Execute a sequence of fuzz commands against a HashMap.
/// The key invariant: this must never panic.
pub fn execute_fuzz_commands(commands: &[FuzzCommand]) {
    let mut store: HashMap<String, String> = HashMap::new();

    for cmd in commands {
        match cmd {
            FuzzCommand::Insert(k, v) => {
                store.insert(k.clone(), v.clone());
            }
            FuzzCommand::Get(k) => {
                let _ = store.get(k);
            }
            FuzzCommand::Delete(k) => {
                store.remove(k);
            }
            FuzzCommand::List => {
                let _: Vec<_> = store.keys().collect();
            }
            FuzzCommand::Clear => {
                store.clear();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Corpus management
// ---------------------------------------------------------------------------

/// Represents a corpus of test inputs for fuzzing.
pub struct Corpus {
    entries: Vec<CorpusEntry>,
}

#[derive(Debug, Clone)]
pub struct CorpusEntry {
    pub name: String,
    pub data: Vec<u8>,
    pub found_bug: bool,
}

impl Corpus {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, name: &str, data: &[u8]) {
        self.entries.push(CorpusEntry {
            name: name.to_string(),
            data: data.to_vec(),
            found_bug: false,
        });
    }

    pub fn add_seed_inputs(&mut self) {
        self.add("empty", b"");
        self.add("null", b"null");
        self.add("bool_true", b"true");
        self.add("bool_false", b"false");
        self.add("number", b"42");
        self.add("string", b"\"hello\"");
        self.add("array", b"[1,2,3]");
        self.add("object", b"{\"key\":\"value\"}");
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[CorpusEntry] {
        &self.entries
    }

    /// Run all corpus entries through the fuzz target.
    pub fn run_all(&mut self, target: fn(&[u8])) {
        for entry in &mut self.entries {
            target(&entry.data);
        }
    }
}

impl Default for Corpus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        assert_eq!(parse_value("null").unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_value("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse_value("false").unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_value("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse_value("-3.14").unwrap(), JsonValue::Number(-3.14));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_value("\"hello\"").unwrap(),
            JsonValue::Str("hello".to_string())
        );
    }

    #[test]
    fn test_parse_array() {
        let result = parse_value("[1,2,3]").unwrap();
        match result {
            JsonValue::Array(items) => assert_eq!(items.len(), 3),
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn test_parse_empty_array() {
        assert_eq!(parse_value("[]").unwrap(), JsonValue::Array(vec![]));
    }

    #[test]
    fn test_parse_object() {
        let result = parse_value("{\"key\":\"value\"}").unwrap();
        match result {
            JsonValue::Object(map) => {
                assert_eq!(map.get("key").unwrap(), &JsonValue::Str("value".to_string()));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_parse_empty_object() {
        assert_eq!(
            parse_value("{}").unwrap(),
            JsonValue::Object(HashMap::new())
        );
    }

    #[test]
    fn test_parse_errors() {
        assert_eq!(parse_value("").unwrap_err(), ParseError::UnexpectedEof);
        assert_eq!(
            parse_value("xyz").unwrap_err(),
            ParseError::UnexpectedToken
        );
    }

    // Fuzz-style: verify parser doesn't panic on arbitrary input
    #[test]
    fn fuzz_parser_no_panic() {
        let inputs = vec![
            "", " ", "null", "true", "false", "42", "-1", "3.14",
            "\"hello\"", "\"\"", "\"unclosed", "[1,2,3]", "[]",
            "[1,", "{\"a\":1}", "{}", "{", "}", "nul", "tru",
            "123abc", "null\x00", "[[[[]]]]",
        ];
        for input in inputs {
            // Must not panic
            let _ = fuzz_entrypoint(input);
        }
    }

    #[test]
    fn test_commands_from_bytes() {
        let bytes = vec![0, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112];
        let commands = commands_from_bytes(&bytes);
        assert!(!commands.is_empty());
    }

    #[test]
    fn test_execute_fuzz_commands_no_panic() {
        let commands = vec![
            FuzzCommand::Insert("key1".into(), "val1".into()),
            FuzzCommand::Get("key1".into()),
            FuzzCommand::Delete("key1".into()),
            FuzzCommand::List,
            FuzzCommand::Clear,
            FuzzCommand::Get("nonexistent".into()),
        ];
        execute_fuzz_commands(&commands);
    }

    #[test]
    fn test_corpus() {
        let mut corpus = Corpus::new();
        corpus.add_seed_inputs();
        assert_eq!(corpus.len(), 8);
        assert!(!corpus.is_empty());
    }

    #[test]
    fn test_corpus_run_all() {
        let mut corpus = Corpus::new();
        corpus.add_seed_inputs();
        corpus.run_all(|_data| {
            // Just verify no panic
        });
        assert_eq!(corpus.len(), 8);
    }

    #[test]
    fn test_extract_string() {
        let bytes = b"hello world";
        assert_eq!(extract_string(bytes, 0, 5), "hello");
        assert_eq!(extract_string(bytes, 6, 5), "world");
        assert_eq!(extract_string(bytes, 100, 5), "");
    }

    #[test]
    fn fuzz_deeply_nested() {
        let input = "[[[[[[[[[[[]]]]]]]]]]]";
        let _ = fuzz_entrypoint(input);
    }

    #[test]
    fn fuzz_special_chars() {
        let inputs = vec!["\"\n\"", "\"\t\"", "\"\\\\\"", "\"\\\"\""];
        for input in inputs {
            let _ = fuzz_entrypoint(input);
        }
    }
}
