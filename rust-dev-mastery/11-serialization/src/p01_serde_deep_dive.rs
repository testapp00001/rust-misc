//! # Serde Deep Dive: Serialize/Deserialize Traits
//!
//! Serde is a framework for serializing and deserializing Rust data structures.
//! Its core insight: separate the data format from the data structure through
//! a data model with 29 types.
//!
//! Key concepts:
//! - `Serialize` trait: Converts Rust types to a data format
//! - `Deserialize` trait: Converts data format to Rust types
//! - Container attributes: `#[serde(rename_all, tag, ...)]`
//! - Field attributes: `#[serde(rename, default, skip, ...)]`
//! - The data model: bool, i8-i128, u8-u128, f32, f64, char, str, bytes,
//!   none, some, unit, unit_struct, unit_variant, newtype_struct, newtype_variant,
//!   seq, tuple, tuple_struct, tuple_variant, map, struct, struct_variant

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Demonstrates basic Serialize/Deserialize with attributes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserProfile {
    /// Renamed in serialized form
    #[serde(rename = "userName")]
    pub username: String,

    /// Skip this field during serialization
    #[serde(skip)]
    pub internal_id: u64,

    /// Use a default value if missing during deserialization
    #[serde(default)]
    pub bio: Option<String>,

    /// Skip serialization if the value is None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    /// Flatten nested struct into parent
    #[serde(flatten)]
    pub metadata: UserMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct UserMetadata {
    #[serde(default)]
    pub created_at: Option<String>,

    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Demonstrates rename_all for consistent naming conventions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApiRequest {
    pub request_id: String,
    pub user_id: u64,
    pub include_metadata: bool,
    pub max_results: Option<u32>,
}

/// Demonstrates tagged enums for representing sum types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Text(String),
    Image { url: String, width: u32, height: u32 },
    File { name: String, size: u64 },
}

/// Demonstrates externally tagged enum (default).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Color {
    Red,
    Green,
    Blue,
    Custom { r: u8, g: u8, b: u8 },
}

/// Demonstrates adjacently tagged enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value")]
pub enum Event {
    Click { x: f64, y: f64 },
    Scroll { delta: f64 },
    Resize { width: u32, height: u32 },
}

/// Demonstrates untagged enum for flexible deserialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FlexibleValue {
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    List(Vec<FlexibleValue>),
    Map(HashMap<String, FlexibleValue>),
}

/// Demonstrates container-level attributes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Inactive,
    PendingReview,
    Archived,
}

/// Demonstrates default values and custom deserialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default)]
    pub debug: bool,

    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_max_connections() -> usize {
    100
}

/// Demonstrates Vec-based serialization with custom element handling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Playlist {
    pub name: String,

    #[serde(default)]
    pub songs: Vec<Song>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Song {
    pub title: String,
    pub artist: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<u32>,
}

/// Demonstrates HashMap serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpHeaders {
    #[serde(flatten)]
    pub headers: HashMap<String, String>,
}

/// Demonstrates tuple struct serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Point(pub f64, pub f64, pub f64);

/// Demonstrates unit struct serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Marker;

/// Demonstrates newtype struct serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserId(pub u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile() {
        let profile = UserProfile {
            username: "alice".to_string(),
            internal_id: 12345,
            bio: Some("Hello world".to_string()),
            avatar_url: None,
            metadata: UserMetadata {
                created_at: Some("2024-01-01".to_string()),
                updated_at: None,
            },
        };

        let json = serde_json::to_string(&profile).unwrap();
        assert!(json.contains("userName"));
        assert!(!json.contains("internal_id")); // skipped

        let deserialized: UserProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.username, "alice");
        assert_eq!(deserialized.internal_id, 0); // default value
    }

    #[test]
    fn test_api_request_rename_all() {
        let req = ApiRequest {
            request_id: "abc".to_string(),
            user_id: 42,
            include_metadata: true,
            max_results: Some(10),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("requestId"));
        assert!(json.contains("userId"));
        assert!(json.contains("includeMetadata"));
        assert!(json.contains("maxResults"));
    }

    #[test]
    fn test_message_tagged() {
        let msg = Message::Text("hello".to_string());
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"Text\""));
        assert!(json.contains("\"data\":\"hello\""));

        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn test_color_externally_tagged() {
        let red = Color::Red;
        let json = serde_json::to_string(&red).unwrap();
        assert_eq!(json, "\"Red\"");

        let custom = Color::Custom { r: 255, g: 0, b: 128 };
        let json = serde_json::to_string(&custom).unwrap();
        assert!(json.contains("Custom"));
        assert!(json.contains("\"r\":255"));
    }

    #[test]
    fn test_event_adjacently_tagged() {
        let event = Event::Click { x: 10.0, y: 20.0 };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"kind\":\"Click\""));
        assert!(json.contains("\"value\""));
    }

    #[test]
    fn test_flexible_value_untagged() {
        let val = FlexibleValue::Integer(42);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "42");

        let val = FlexibleValue::Text("hello".to_string());
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "\"hello\"");
    }

    #[test]
    fn test_status_rename_all() {
        let status = Status::PendingReview;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"pending_review\"");
    }

    #[test]
    fn test_config_defaults() {
        let json = r#"{"host": "example.com"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 8080);
        assert!(!config.debug);
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_playlist() {
        let playlist = Playlist {
            name: "My Playlist".to_string(),
            songs: vec![
                Song {
                    title: "Song 1".to_string(),
                    artist: "Artist 1".to_string(),
                    duration_secs: Some(180),
                },
                Song {
                    title: "Song 2".to_string(),
                    artist: "Artist 2".to_string(),
                    duration_secs: None,
                },
            ],
            tags: vec!["rock".to_string()],
        };

        let json = serde_json::to_string_pretty(&playlist).unwrap();
        assert!(json.contains("My Playlist"));
        assert!(json.contains("Song 1"));

        let back: Playlist = serde_json::from_str(&json).unwrap();
        assert_eq!(back, playlist);
    }

    #[test]
    fn test_http_headers_flatten() {
        let headers = HttpHeaders {
            headers: {
                let mut map = HashMap::new();
                map.insert("content-type".to_string(), "application/json".to_string());
                map
            },
        };

        let json = serde_json::to_string(&headers).unwrap();
        assert!(json.contains("content-type"));
        assert!(json.contains("application/json"));
    }

    #[test]
    fn test_point_tuple() {
        let p = Point(1.0, 2.0, 3.0);
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, "[1.0,2.0,3.0]");

        let back: Point = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn test_marker_unit() {
        let m = Marker;
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(json, "null");

        let back: Marker = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn test_user_id_newtype() {
        let id = UserId(42);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "42");

        let back: UserId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }
}
