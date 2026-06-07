//! # JSON Mastery with serde_json
//!
//! This lesson covers advanced serde_json usage:
//! - Working with `serde_json::Value` for dynamic JSON
//! - Streaming large JSON arrays
//! - Pretty printing and formatting
//! - Arbitrary precision numbers
//! - JSON path queries

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Number, Value};
use std::collections::HashMap;

/// Demonstrates the `json!` macro for creating JSON values.
pub fn create_json_value() -> Value {
    json!({
        "name": "Alice",
        "age": 30,
        "scores": [95, 87, 92],
        "address": {
            "city": "NYC",
            "zip": "10001"
        },
        "active": true,
        "nickname": null
    })
}

/// Demonstrates querying JSON values dynamically.
pub fn extract_nested_value<'a>(root: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = root;
    for key in path {
        current = current.get(key)?;
    }
    Some(current)
}

/// Demonstrates modifying JSON values in place.
pub fn update_json_fields(root: &mut Value, updates: &[(&str, Value)]) {
    for (path, value) in updates {
        let keys: Vec<&str> = path.split('.').collect();
        if keys.is_empty() {
            continue;
        }

        // Use a helper function to handle the borrowing
        fn set_nested(current: &mut Value, keys: &[&str], value: &Value) {
            if keys.len() == 1 {
                if let Some(obj) = current.as_object_mut() {
                    obj.insert(keys[0].to_string(), value.clone());
                }
                return;
            }

            if !current.is_object() {
                *current = json!({});
            }
            set_nested(&mut (*current)[keys[0]], &keys[1..], value);
        }

        set_nested(root, &keys, value);
    }
}

/// Demonstrates merging two JSON objects.
pub fn merge_json(base: &mut Value, overlay: &Value) {
    if let (Some(base_map), Some(overlay_map)) = (base.as_object_mut(), overlay.as_object()) {
        for (key, value) in overlay_map {
            if let Some(base_value) = base_map.get_mut(key) {
                if base_value.is_object() && value.is_object() {
                    merge_json(base_value, value);
                } else {
                    base_map.insert(key.clone(), value.clone());
                }
            } else {
                base_map.insert(key.clone(), value.clone());
            }
        }
    }
}

/// Demonstrates collecting JSON into a typed structure.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub total: u64,
    pub page: u32,
    pub results: Vec<SearchHit>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub score: f64,
    #[serde(default)]
    pub highlights: Vec<String>,
}

/// Demonstrates streaming JSON writer for large outputs.
pub fn write_json_array<W: std::io::Write>(writer: W, items: &[Value]) -> serde_json::Result<()> {
    let mut serializer = serde_json::Serializer::new(writer);
    items.serialize(&mut serializer)?;
    Ok(())
}

/// Demonstrates pretty printing with custom indentation.
pub fn pretty_print(value: &Value, indent: usize) -> String {
    let indent_bytes = " ".repeat(indent);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent_bytes.as_bytes());
    let mut buf = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut buf, formatter);
    value.serialize(&mut serializer).unwrap();
    String::from_utf8(buf).unwrap()
}

/// Demonstrates JSON path-like query (simplified).
pub fn json_path_query<'a>(root: &'a Value, path: &str) -> Vec<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut results: Vec<&'a Value> = vec![root];

    for part in parts {
        let mut next: Vec<&'a Value> = Vec::new();
        for current in results {
            match current {
                Value::Object(map) => {
                    if let Some(val) = map.get(part) {
                        next.push(val);
                    }
                }
                Value::Array(arr) => {
                    if part == "*" {
                        next.extend(arr.iter());
                    } else if let Ok(idx) = part.parse::<usize>() {
                        if let Some(val) = arr.get(idx) {
                            next.push(val);
                        }
                    }
                }
                _ => {}
            }
        }
        results = next;
    }

    results
}

/// Demonstrates converting between Value and typed structs.
pub fn value_to_struct<T: for<'de> Deserialize<'de>>(value: &Value) -> Result<T, serde_json::Error> {
    serde_json::from_value(value.clone())
}

pub fn struct_to_value<T: Serialize>(value: &T) -> Result<Value, serde_json::Error> {
    serde_json::to_value(value)
}

/// Demonstrates working with JSON numbers (including large ones).
pub fn create_number_value(n: i64) -> Value {
    Value::Number(Number::from(n))
}

pub fn create_float_value(f: f64) -> Option<Value> {
    Number::from_f64(f).map(Value::Number)
}

/// Demonstrates filtering JSON arrays.
pub fn filter_array<F>(value: &Value, predicate: F) -> Value
where
    F: Fn(&Value) -> bool,
{
    match value {
        Value::Array(arr) => {
            let filtered: Vec<Value> = arr.iter().filter(|v| predicate(v)).cloned().collect();
            Value::Array(filtered)
        }
        _ => value.clone(),
    }
}

/// Demonstrates transforming JSON values with a map function.
pub fn map_array<F>(value: &Value, f: F) -> Value
where
    F: Fn(&Value) -> Value,
{
    match value {
        Value::Array(arr) => Value::Array(arr.iter().map(|v| f(v)).collect()),
        _ => value.clone(),
    }
}

/// Demonstrates flattening nested JSON.
pub fn flatten_json(value: &Value, prefix: &str) -> Map<String, Value> {
    let mut result = Map::new();
    flatten_recursive(value, prefix, &mut result);
    result
}

fn flatten_recursive(value: &Value, prefix: &str, result: &mut Map<String, Value>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let new_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten_recursive(val, &new_key, result);
            }
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let new_key = format!("{prefix}[{i}]");
                flatten_recursive(val, &new_key, result);
            }
        }
        _ => {
            result.insert(prefix.to_string(), value.clone());
        }
    }
}

/// Demonstrates JSON diff (simplified).
pub fn json_diff(old: &Value, new: &Value) -> Vec<DiffEntry> {
    let mut diffs = Vec::new();
    diff_recursive(old, new, "", &mut diffs);
    diffs
}

#[derive(Debug, PartialEq)]
pub struct DiffEntry {
    pub path: String,
    pub kind: DiffKind,
}

#[derive(Debug, PartialEq)]
pub enum DiffKind {
    Added(Value),
    Removed(Value),
    Changed { old: Value, new: Value },
}

fn diff_recursive(old: &Value, new: &Value, path: &str, diffs: &mut Vec<DiffEntry>) {
    match (old, new) {
        (Value::Object(old_map), Value::Object(new_map)) => {
            for (key, old_val) in old_map {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                if let Some(new_val) = new_map.get(key) {
                    diff_recursive(old_val, new_val, &new_path, diffs);
                } else {
                    diffs.push(DiffEntry {
                        path: new_path,
                        kind: DiffKind::Removed(old_val.clone()),
                    });
                }
            }
            for (key, new_val) in new_map {
                if !old_map.contains_key(key) {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    diffs.push(DiffEntry {
                        path: new_path,
                        kind: DiffKind::Added(new_val.clone()),
                    });
                }
            }
        }
        _ if old != new => {
            diffs.push(DiffEntry {
                path: path.to_string(),
                kind: DiffKind::Changed {
                    old: old.clone(),
                    new: new.clone(),
                },
            });
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_json_value() {
        let val = create_json_value();
        assert_eq!(val["name"], "Alice");
        assert_eq!(val["age"], 30);
        assert_eq!(val["scores"][0], 95);
        assert_eq!(val["address"]["city"], "NYC");
    }

    #[test]
    fn test_extract_nested_value() {
        let val = create_json_value();
        assert_eq!(extract_nested_value(&val, &["name"]), Some(&Value::String("Alice".into())));
        assert_eq!(extract_nested_value(&val, &["address", "city"]), Some(&Value::String("NYC".into())));
        assert_eq!(extract_nested_value(&val, &["nonexistent"]), None);
    }

    #[test]
    fn test_update_json_fields() {
        let mut val = json!({"name": "Alice"});
        update_json_fields(&mut val, &[("name", json!("Bob")), ("age", json!(31))]);
        assert_eq!(val["name"], "Bob");
        assert_eq!(val["age"], 31);
    }

    #[test]
    fn test_merge_json() {
        let mut base = json!({"a": 1, "b": {"x": 10}});
        let overlay = json!({"b": {"y": 20}, "c": 3});
        merge_json(&mut base, &overlay);
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["x"], 10);
        assert_eq!(base["b"]["y"], 20);
        assert_eq!(base["c"], 3);
    }

    #[test]
    fn test_pretty_print() {
        let val = json!({"key": "value"});
        let pretty = pretty_print(&val, 2);
        assert!(pretty.contains("  "));
        assert!(pretty.contains("key"));
    }

    #[test]
    fn test_json_path_query() {
        let val = json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        });

        let names = json_path_query(&val, "users.*.name");
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "Alice");
        assert_eq!(names[1], "Bob");
    }

    #[test]
    fn test_value_to_struct() {
        let val = json!({"id": "abc", "title": "Test", "score": 0.95, "highlights": []});
        let hit: SearchHit = value_to_struct(&val).unwrap();
        assert_eq!(hit.id, "abc");
        assert_eq!(hit.title, "Test");
    }

    #[test]
    fn test_struct_to_value() {
        let hit = SearchHit {
            id: "abc".to_string(),
            title: "Test".to_string(),
            score: 0.95,
            highlights: vec![],
        };
        let val = struct_to_value(&hit).unwrap();
        assert_eq!(val["id"], "abc");
    }

    #[test]
    fn test_filter_array() {
        let arr = json!([1, 2, 3, 4, 5]);
        let filtered = filter_array(&arr, |v| v.as_i64().map_or(false, |n| n > 3));
        assert_eq!(filtered, json!([4, 5]));
    }

    #[test]
    fn test_map_array() {
        let arr = json!([1, 2, 3]);
        let mapped = map_array(&arr, |v| {
            json!(v.as_i64().unwrap() * 2)
        });
        assert_eq!(mapped, json!([2, 4, 6]));
    }

    #[test]
    fn test_flatten_json() {
        let val = json!({"a": 1, "b": {"c": 2, "d": [3, 4]}});
        let flat = flatten_json(&val, "");
        assert_eq!(flat["a"], 1);
        assert_eq!(flat["b.c"], 2);
        assert_eq!(flat["b.d[0]"], 3);
        assert_eq!(flat["b.d[1]"], 4);
    }

    #[test]
    fn test_json_diff() {
        let old = json!({"a": 1, "b": 2, "c": 3});
        let new = json!({"a": 1, "b": 5, "d": 4});
        let diffs = json_diff(&old, &new);
        assert_eq!(diffs.len(), 3); // b changed, c removed, d added
    }

    #[test]
    fn test_number_value() {
        let val = create_number_value(i64::MAX);
        assert_eq!(val.as_i64(), Some(i64::MAX));
    }

    #[test]
    fn test_float_value() {
        let val = create_float_value(3.14).unwrap();
        assert!(val.as_f64().unwrap() > 3.0);
    }
}
