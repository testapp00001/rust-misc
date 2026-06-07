//! # Format Design
//!
//! Designing your own serialization format with serde. This lesson covers:
//! - Custom tagged enum representations
//! - Adjacently tagged enums
//! - Internally tagged enums
//! - Untagged enums
//! - Custom content negotiation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Demonstrates internally tagged enum: the type tag is inside the object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Shape {
    #[serde(rename = "circle")]
    Circle { radius: f64 },
    #[serde(rename = "rectangle")]
    Rectangle { width: f64, height: f64 },
    #[serde(rename = "triangle")]
    Triangle { base: f64, height: f64 },
}

/// Demonstrates adjacently tagged enum: type and content are sibling fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data")]
pub enum Command {
    #[serde(rename = "read")]
    Read { path: String },
    #[serde(rename = "write")]
    Write { path: String, content: String },
    #[serde(rename = "delete")]
    Delete { path: String },
    #[serde(rename = "list")]
    List { prefix: Option<String> },
}

/// Demonstrates externally tagged enum (the default): type is the key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Demonstrates untagged enum: serde tries each variant in order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConfigValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

/// Demonstrates a custom format with a version field and content negotiation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Envelope<T> {
    pub version: u32,
    pub encoding: Encoding,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Encoding {
    Json,
    Msgpack,
    Protobuf,
}

/// Demonstrates a polymorphic container that can hold different types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value")]
pub enum Metric {
    #[serde(rename = "counter")]
    Counter(u64),
    #[serde(rename = "gauge")]
    Gauge(f64),
    #[serde(rename = "histogram")]
    Histogram(Vec<f64>),
    #[serde(rename = "set")]
    Set(Vec<String>),
}

/// Demonstrates a request/response format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    pub id: u64,
    #[serde(flatten)]
    pub result: ResponseResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status")]
pub enum ResponseResult {
    #[serde(rename = "ok")]
    Success { data: serde_json::Value },
    #[serde(rename = "error")]
    Error { code: i32, message: String },
}

/// Demonstrates a format with optional fields and defaults.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryParams {
    pub q: String,
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub filters: HashMap<String, String>,
}

fn default_page_size() -> u32 {
    20
}

/// Demonstrates a hierarchical configuration format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HierarchicalConfig {
    pub name: String,
    #[serde(default)]
    pub settings: HashMap<String, ConfigValue>,
    #[serde(default)]
    pub children: Vec<HierarchicalConfig>,
}

/// Demonstrates a format that supports both inline and reference values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ValueOrRef {
    Inline(serde_json::Value),
    Reference { r#ref: String },
}

/// Demonstrates a format with custom key serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderedMap {
    #[serde(flatten)]
    pub entries: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub order: Vec<String>,
}

impl OrderedMap {
    pub fn get_ordered(&self) -> Vec<(&str, &serde_json::Value)> {
        self.order
            .iter()
            .filter_map(|key| {
                self.entries
                    .get(key.as_str())
                    .map(|value| (key.as_str(), value))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internally_tagged_shape() {
        let circle = Shape::Circle { radius: 5.0 };
        let json = serde_json::to_string(&circle).unwrap();
        assert!(json.contains("\"type\":\"circle\""));
        assert!(json.contains("\"radius\":5.0"));

        let back: Shape = serde_json::from_str(&json).unwrap();
        assert_eq!(back, circle);
    }

    #[test]
    fn test_adjacently_tagged_command() {
        let cmd = Command::Write {
            path: "/tmp/test".to_string(),
            content: "hello".to_string(),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"kind\":\"write\""));
        assert!(json.contains("\"data\""));

        let back: Command = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cmd);
    }

    #[test]
    fn test_externally_tagged_log_level() {
        let level = LogLevel::Error;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"Error\"");
    }

    #[test]
    fn test_untagged_config_value() {
        let val = ConfigValue::Integer(42);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "42");

        let val = ConfigValue::String("hello".to_string());
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "\"hello\"");

        let val = ConfigValue::Boolean(true);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "true");
    }

    #[test]
    fn test_envelope() {
        let env = Envelope {
            version: 1,
            encoding: Encoding::Json,
            payload: vec![1, 2, 3],
        };
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"encoding\":\"json\""));
    }

    #[test]
    fn test_metric() {
        let counter = Metric::Counter(42);
        let json = serde_json::to_string(&counter).unwrap();
        assert!(json.contains("\"kind\":\"counter\""));
        assert!(json.contains("\"value\":42"));

        let histogram = Metric::Histogram(vec![0.1, 0.5, 0.9]);
        let json = serde_json::to_string(&histogram).unwrap();
        let back: Metric = serde_json::from_str(&json).unwrap();
        assert_eq!(back, histogram);
    }

    #[test]
    fn test_request_response() {
        let req = Request {
            id: 1,
            method: "get_user".to_string(),
            params: serde_json::json!({"id": 42}),
            timeout_ms: Some(5000),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn test_response_success() {
        let resp = Response {
            id: 1,
            result: ResponseResult::Success {
                data: serde_json::json!({"name": "Alice"}),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"ok\""));

        let back: Response = serde_json::from_str(&json).unwrap();
        assert_eq!(back, resp);
    }

    #[test]
    fn test_response_error() {
        let resp = Response {
            id: 1,
            result: ResponseResult::Error {
                code: 404,
                message: "not found".to_string(),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"error\""));
        assert!(json.contains("\"code\":404"));
    }

    #[test]
    fn test_query_params() {
        let json = r#"{"q": "rust"}"#;
        let params: QueryParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.q, "rust");
        assert_eq!(params.page, 0);
        assert_eq!(params.page_size, 20);
    }

    #[test]
    fn test_value_or_ref() {
        let inline = ValueOrRef::Inline(serde_json::json!(42));
        let json = serde_json::to_string(&inline).unwrap();
        assert_eq!(json, "42");

        let reference = ValueOrRef::Reference {
            r#ref: "#/definitions/User".to_string(),
        };
        let json = serde_json::to_string(&reference).unwrap();
        // The field name is "ref" (raw identifier), serialized as "ref"
        assert!(json.contains("ref"));
    }

    #[test]
    fn test_ordered_map() {
        let map = OrderedMap {
            entries: {
                let mut m = HashMap::new();
                m.insert("b".to_string(), serde_json::json!(2));
                m.insert("a".to_string(), serde_json::json!(1));
                m
            },
            order: vec!["a".to_string(), "b".to_string()],
        };
        let ordered = map.get_ordered();
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0].0, "a");
        assert_eq!(ordered[1].0, "b");
    }
}
