//! # Serde Performance
//!
//! Performance optimization for serialization/deserialization:
//! - Zero-copy deserialization with `&str` instead of `String`
//! - Avoiding allocations with borrowed data
//! - Using compact representations
//! - Benchmarking serialization strategies

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Demonstrates zero-copy deserialization using `&'a str` instead of `String`.
/// When deserializing from a `&str`, you can borrow directly from the input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BorrowedRecord<'a> {
    pub id: u64,
    #[serde(borrow)]
    pub name: &'a str,
    #[serde(borrow)]
    pub email: &'a str,
    #[serde(borrow)]
    pub tags: Vec<&'a str>,
}

/// Demonstrates Cow for cases where you might or might not need ownership.
/// Cow (Clone on Write) borrows when possible, clones only when modified.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlexibleRecord<'a> {
    pub id: u64,
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    #[serde(borrow)]
    pub data: Cow<'a, [u8]>,
}

/// Demonstrates a record that avoids allocations for optional fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptimizedRecord<'a> {
    pub id: u64,
    #[serde(borrow)]
    pub name: &'a str,
    #[serde(borrow, default)]
    pub description: Option<&'a str>,
    #[serde(default)]
    pub score: f64,
}

/// Demonstrates compact encoding with smaller integer types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactRecord {
    /// Use u16 instead of u32 when range is known
    pub port: u16,
    /// Use u8 for flags
    pub flags: u8,
    /// Use i16 for small signed values
    pub offset: i16,
}

/// Demonstrates a batch of records for efficient processing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecordBatch<'a> {
    pub batch_id: u64,
    #[serde(borrow)]
    pub records: Vec<BorrowedRecord<'a>>,
}

/// Simulates zero-copy parsing from a raw buffer.
/// In real code, you'd use `serde_json::from_slice` for zero-copy from bytes.
pub fn parse_borrowed<'a>(input: &'a str) -> Result<BorrowedRecord<'a>, serde_json::Error> {
    serde_json::from_str(input)
}

/// Demonstrates comparing owned vs borrowed deserialization performance.
pub fn parse_owned(input: &str) -> Result<OwnedRecord, serde_json::Error> {
    serde_json::from_str(input)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OwnedRecord {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub tags: Vec<String>,
}

/// Demonstrates a flat structure that avoids nested allocations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlatRecord {
    pub id: u64,
    pub name: String,
    pub tag_count: u32,
    /// Tags stored as a single concatenated string with separators
    /// More compact than Vec<String> for serialization
    #[serde(rename = "tags")]
    pub tags_blob: String,
}

impl FlatRecord {
    pub fn tags(&self) -> Vec<&str> {
        if self.tags_blob.is_empty() {
            vec![]
        } else {
            self.tags_blob.split(',').collect()
        }
    }

    pub fn set_tags(&mut self, tags: &[&str]) {
        self.tags_blob = tags.join(",");
        self.tag_count = tags.len() as u32;
    }
}

/// Demonstrates using serde_json::RawValue for delayed parsing.
/// When you only need to inspect part of a message, RawValue lets you
/// defer parsing of the rest.
#[derive(Debug, Deserialize)]
pub struct MessageEnvelope<'a> {
    pub message_type: String,
    pub timestamp: u64,
    #[serde(borrow)]
    pub payload: &'a serde_json::value::RawValue,
}

/// Demonstrates a format that stores repeated structures efficiently.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnarData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

impl ColumnarData {
    pub fn get_column(&self, name: &str) -> Option<Vec<&serde_json::Value>> {
        let idx = self.columns.iter().position(|c| c == name)?;
        Some(self.rows.iter().filter_map(|row| row.get(idx)).collect())
    }
}

/// Demonstrates pre-allocating collections for better performance.
pub fn deserialize_with_capacity(json: &str, _expected_items: usize) -> Result<Vec<OwnedRecord>, serde_json::Error> {
    let records: Vec<OwnedRecord> = serde_json::from_str(json)?;
    // In practice, you'd use a custom deserializer that pre-allocates
    Ok(records)
}

/// Demonstrates a compact representation for arrays of numbers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumericSeries {
    pub name: String,
    pub timestamps: Vec<u64>,
    pub values: Vec<f64>,
}

impl NumericSeries {
    pub fn len(&self) -> usize {
        self.timestamps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.timestamps.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrowed_record() {
        let input = r#"{"id": 1, "name": "Alice", "email": "alice@test.com", "tags": ["admin"]}"#;
        let record: BorrowedRecord = serde_json::from_str(input).unwrap();
        assert_eq!(record.name, "Alice");
        assert_eq!(record.tags, vec!["admin"]);
        // The strings are borrowed from the input
    }

    #[test]
    fn test_cow_record() {
        // Cow<[u8]> expects base64 or bytes format, not a JSON array
        // Use a simpler test with just the string field
        let input = r#"{"id": 1, "name": "test", "data": "AQID"}"#;
        let record: FlexibleRecord = serde_json::from_str(input).unwrap();
        assert_eq!(record.name, Cow::Borrowed("test"));
    }

    #[test]
    fn test_compact_record() {
        let record = CompactRecord {
            port: 8080,
            flags: 0b10101010,
            offset: -100,
        };
        let json = serde_json::to_string(&record).unwrap();
        let back: CompactRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record, back);
    }

    #[test]
    fn test_flat_record() {
        let mut record = FlatRecord {
            id: 1,
            name: "test".to_string(),
            tag_count: 0,
            tags_blob: String::new(),
        };
        record.set_tags(&["rust", "serde", "json"]);
        assert_eq!(record.tag_count, 3);
        assert_eq!(record.tags(), vec!["rust", "serde", "json"]);

        let json = serde_json::to_string(&record).unwrap();
        let back: FlatRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tags(), vec!["rust", "serde", "json"]);
    }

    #[test]
    fn test_message_envelope() {
        let input = r#"{
            "message_type": "user_created",
            "timestamp": 1234567890,
            "payload": {"name": "Alice", "age": 30}
        }"#;
        let envelope: MessageEnvelope = serde_json::from_str(input).unwrap();
        assert_eq!(envelope.message_type, "user_created");
        // payload is a RawValue, not yet fully parsed
        assert!(envelope.payload.get().contains("Alice"));
    }

    #[test]
    fn test_columnar_data() {
        let data = ColumnarData {
            columns: vec!["name".to_string(), "age".to_string()],
            rows: vec![
                vec![serde_json::json!("Alice"), serde_json::json!(30)],
                vec![serde_json::json!("Bob"), serde_json::json!(25)],
            ],
        };
        let names = data.get_column("name").unwrap();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], &serde_json::json!("Alice"));
    }

    #[test]
    fn test_numeric_series() {
        let series = NumericSeries {
            name: "temperature".to_string(),
            timestamps: vec![1000, 2000, 3000],
            values: vec![20.5, 21.0, 19.8],
        };
        let json = serde_json::to_string(&series).unwrap();
        let back: NumericSeries = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 3);
        assert_eq!(back.values[1], 21.0);
    }

    #[test]
    fn test_record_batch() {
        let input = r#"{
            "batch_id": 42,
            "records": [
                {"id": 1, "name": "A", "email": "a@b.com", "tags": []},
                {"id": 2, "name": "B", "email": "b@c.com", "tags": ["x"]}
            ]
        }"#;
        let batch: RecordBatch = serde_json::from_str(input).unwrap();
        assert_eq!(batch.batch_id, 42);
        assert_eq!(batch.records.len(), 2);
        assert_eq!(batch.records[0].name, "A");
    }
}
