//! # Testing Serialization
//!
//! Testing serialization code requires special strategies:
//! - Round-trip testing (serialize then deserialize)
//! - Snapshot testing for wire format stability
//! - Fuzzing serialized data
//! - Testing edge cases and error handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A type used for round-trip testing demonstrations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestData {
    pub name: String,
    pub value: i64,
    pub items: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Demonstrates round-trip testing: serialize then deserialize.
pub fn round_trip_json<T>(value: &T) -> Result<T, Box<dyn std::error::Error>>
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(value)?;
    let back: T = serde_json::from_str(&json)?;
    Ok(back)
}

/// Demonstrates round-trip testing with pretty printing.
pub fn round_trip_json_pretty<T>(value: &T) -> Result<T, Box<dyn std::error::Error>>
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string_pretty(value)?;
    let back: T = serde_json::from_str(&json)?;
    Ok(back)
}

/// Demonstrates testing that serialization produces expected output.
pub fn assert_serializes_to<T: Serialize>(value: &T, expected: &str) {
    let json = serde_json::to_string(value).unwrap();
    assert_eq!(json, expected, "serialization mismatch");
}

/// Demonstrates testing that deserialization produces expected value.
pub fn assert_deserializes_from<'de, T: Deserialize<'de> + PartialEq + std::fmt::Debug>(
    json: &'de str,
    expected: &T,
) {
    let value: T = serde_json::from_str(json).unwrap();
    assert_eq!(&value, expected, "deserialization mismatch");
}

/// Demonstrates snapshot testing for wire format stability.
pub fn assert_snapshot<T: Serialize>(name: &str, value: &T) -> String {
    let json = serde_json::to_string_pretty(value).unwrap();
    // In real code, you'd compare against a stored snapshot file
    // For demonstration, we just return the JSON
    let _ = name;
    json
}

/// Demonstrates testing error cases.
pub fn assert_deserialize_error<'de, T: Deserialize<'de> + std::fmt::Debug>(json: &'de str) {
    let result: Result<T, _> = serde_json::from_str(json);
    assert!(result.is_err(), "expected deserialization to fail, got: {:?}", result.unwrap());
}

/// Demonstrates a fuzzing strategy for serialization.
pub fn fuzz_deserialize<T>(inputs: &[&str]) -> Vec<FuzzResult>
where
    T: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    inputs
        .iter()
        .map(|input| {
            let result: Result<T, _> = serde_json::from_str(input);
            FuzzResult {
                input: input.to_string(),
                success: result.is_ok(),
                error: result.err().map(|e| e.to_string()),
            }
        })
        .collect()
}

#[derive(Debug)]
pub struct FuzzResult {
    pub input: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Demonstrates testing with generated test data.
pub fn generate_test_data(count: usize) -> Vec<TestData> {
    (0..count)
        .map(|i| TestData {
            name: format!("item_{i}"),
            value: i as i64 * 100,
            items: (0..3).map(|j| format!("sub_{i}_{j}")).collect(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("index".to_string(), i.to_string());
                map
            },
        })
        .collect()
}

/// Demonstrates testing serialization of collections.
pub fn round_trip_collection<T>(items: &[T]) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(items)?;
    let back: Vec<T> = serde_json::from_str(&json)?;
    Ok(back)
}

/// Demonstrates testing HashMap serialization (order-independent).
pub fn assert_map_serializes_to(
    map: &HashMap<String, i32>,
    expected_keys: &[&str],
    expected_values: &[i32],
) {
    let json = serde_json::to_string(map).unwrap();
    for key in expected_keys {
        assert!(json.contains(key), "missing key: {key}");
    }
    let back: HashMap<String, i32> = serde_json::from_str(&json).unwrap();
    assert_eq!(back.len(), expected_keys.len());
    for (key, expected) in expected_keys.iter().zip(expected_values.iter()) {
        assert_eq!(back.get(*key), Some(expected));
    }
}

/// Demonstrates testing with special float values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FloatData {
    pub value: f64,
    #[serde(default)]
    pub optional: Option<f64>,
}

/// Demonstrates testing schema evolution compatibility.
pub fn test_backward_compatibility<T: Serialize, U: for<'de> Deserialize<'de> + std::fmt::Debug>(
    old_value: &T,
) -> Result<U, Box<dyn std::error::Error>> {
    let json = serde_json::to_string(old_value)?;
    let new_value: U = serde_json::from_str(&json)?;
    Ok(new_value)
}

/// Demonstrates testing with custom error messages.
pub fn assert_serde_roundtrip_with_message<T>(value: &T, message: &str)
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect(&format!("{message}: serialization failed"));
    let back: T = serde_json::from_str(&json).expect(&format!("{message}: deserialization failed"));
    assert_eq!(*value, back, "{message}: round-trip mismatch");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
            items: vec!["a".to_string(), "b".to_string()],
            metadata: {
                let mut map = HashMap::new();
                map.insert("key".to_string(), "val".to_string());
                map
            },
        };
        let back = round_trip_json(&data).unwrap();
        assert_eq!(data, back);
    }

    #[test]
    fn test_round_trip_pretty() {
        let data = TestData {
            name: "test".to_string(),
            value: 1,
            items: vec![],
            metadata: HashMap::new(),
        };
        let back = round_trip_json_pretty(&data).unwrap();
        assert_eq!(data, back);
    }

    #[test]
    fn test_assert_serializes_to() {
        assert_serializes_to(&42, "42");
        assert_serializes_to(&"hello", "\"hello\"");
        assert_serializes_to(&true, "true");
    }

    #[test]
    fn test_assert_deserializes_from() {
        assert_deserializes_from("42", &42i32);
        assert_deserializes_from("\"hello\"", &"hello".to_string());
    }

    #[test]
    fn test_assert_deserialize_error() {
        assert_deserialize_error::<i32>("\"not a number\"");
        assert_deserialize_error::<String>("42");
    }

    #[test]
    fn test_fuzz_deserialize() {
        let inputs = vec![
            "42",
            "\"hello\"",
            "null",
            "true",
            "[1,2,3]",
            "{\"key\":\"value\"}",
            "invalid json",
            "",
        ];
        let results = fuzz_deserialize::<serde_json::Value>(&inputs);
        // Most should succeed except "invalid json" and ""
        let successes = results.iter().filter(|r| r.success).count();
        assert!(successes >= 6);
    }

    #[test]
    fn test_generate_and_roundtrip() {
        let data = generate_test_data(10);
        let back = round_trip_collection(&data).unwrap();
        assert_eq!(data, back);
    }

    #[test]
    fn test_map_serializes_to() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);
        assert_map_serializes_to(&map, &["a", "b"], &[1, 2]);
    }

    #[test]
    fn test_snapshot() {
        let data = TestData {
            name: "snapshot_test".to_string(),
            value: 99,
            items: vec!["x".to_string()],
            metadata: HashMap::new(),
        };
        let snapshot = assert_snapshot("test_data", &data);
        assert!(snapshot.contains("snapshot_test"));
        assert!(snapshot.contains("99"));
    }

    #[test]
    fn test_backwards_compatibility() {
        // Old format has fewer fields
        let old_data = TestData {
            name: "old".to_string(),
            value: 1,
            items: vec![],
            metadata: HashMap::new(),
        };

        #[derive(Deserialize, Debug)]
        struct NewData {
            name: String,
            value: i64,
            #[serde(default)]
            items: Vec<String>,
            #[serde(default)]
            metadata: HashMap<String, String>,
            #[serde(default)]
            new_field: Option<String>,
        }

        let new: NewData = test_backward_compatibility(&old_data).unwrap();
        assert_eq!(new.name, "old");
        assert_eq!(new.new_field, None);
    }

    #[test]
    fn test_serde_roundtrip_with_message() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
            items: vec![],
            metadata: HashMap::new(),
        };
        assert_serde_roundtrip_with_message(&data, "basic test data");
    }
}
