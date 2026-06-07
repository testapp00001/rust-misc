//! # Schema Evolution
//!
//! Schema evolution is the practice of changing data structures over time
//! while maintaining backward and forward compatibility. Serde provides
//! tools for managing this gracefully.
//!
//! Key concepts:
//! - **Forward compatibility**: Old code can read new data
//! - **Backward compatibility**: New code can read old data
//! - **serde(default)**: Missing fields use defaults
//! - **serde(alias)**: Accept multiple names for the same field
//! - **Version tags**: Embed version info in serialized data
//! - **Flatten**: Add new fields without breaking existing consumers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Demonstrates a versioned configuration that evolved over time.
/// V1 had only host and port.
/// V2 added timeout and max_connections.
/// V3 added features as a map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,

    /// Added in V2 - defaults for backward compatibility
    #[serde(default = "default_timeout")]
    pub timeout_secs: u32,

    /// Added in V2 - defaults for backward compatibility
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    /// Added in V3 - empty map for backward compatibility
    #[serde(default)]
    pub features: HashMap<String, bool>,

    /// Added in V3 - optional for backward compatibility
    #[serde(default, alias = "description", alias = "desc")]
    pub description: Option<String>,
}

fn default_timeout() -> u32 {
    30
}

fn default_max_connections() -> usize {
    100
}

/// Demonstrates a type that uses serde's `from` and `into` for evolution.
/// Old format: string name
/// New format: structured enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(from = "StorageFormat", into = "StorageFormat")]
pub enum StorageBackend {
    Memory,
    File { path: String },
    Redis { url: String },
}

/// The wire format that supports both old and new representations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
enum StorageFormat {
    Legacy(String),
    Modern(StorageBackendModern),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct StorageBackendModern {
    #[serde(rename = "type")]
    backend_type: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

impl From<StorageFormat> for StorageBackend {
    fn from(format: StorageFormat) -> Self {
        match format {
            StorageFormat::Legacy(name) => match name.as_str() {
                "memory" => StorageBackend::Memory,
                other => StorageBackend::File {
                    path: other.to_string(),
                },
            },
            StorageFormat::Modern(modern) => match modern.backend_type.as_str() {
                "memory" => StorageBackend::Memory,
                "file" => StorageBackend::File {
                    path: modern.path.unwrap_or_default(),
                },
                "redis" => StorageBackend::Redis {
                    url: modern.url.unwrap_or_default(),
                },
                _ => StorageBackend::Memory,
            },
        }
    }
}

impl From<StorageBackend> for StorageFormat {
    fn from(backend: StorageBackend) -> Self {
        match backend {
            StorageBackend::Memory => StorageFormat::Modern(StorageBackendModern {
                backend_type: "memory".to_string(),
                path: None,
                url: None,
            }),
            StorageBackend::File { path } => StorageFormat::Modern(StorageBackendModern {
                backend_type: "file".to_string(),
                path: Some(path),
                url: None,
            }),
            StorageBackend::Redis { url } => StorageFormat::Modern(StorageBackendModern {
                backend_type: "redis".to_string(),
                path: None,
                url: Some(url),
            }),
        }
    }
}

/// Demonstrates a tagged enum with version field for explicit versioning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionedMessage {
    pub version: u32,
    #[serde(flatten)]
    pub payload: MessagePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum MessagePayload {
    #[serde(rename = "text")]
    Text { content: String },
    #[serde(rename = "image")]
    Image { url: String, width: u32, height: u32 },
    #[serde(rename = "video")]
    Video {
        url: String,
        duration_secs: u32,
        #[serde(default)]
        codec: Option<String>,
    },
}

/// Demonstrates a pattern for migrating old data formats.
pub trait Migrate {
    type Old;
    fn migrate(old: Self::Old) -> Self;
}

/// V1 of a user profile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserProfileV1 {
    pub name: String,
    pub email: String,
}

/// V2 adds more fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserProfileV2 {
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub preferences: HashMap<String, String>,
}

impl Migrate for UserProfileV2 {
    type Old = UserProfileV1;

    fn migrate(old: UserProfileV1) -> Self {
        UserProfileV2 {
            name: old.name,
            email: old.email,
            phone: None,
            avatar_url: None,
            preferences: HashMap::new(),
        }
    }
}

/// Demonstrates a container format with a version header.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Container<T> {
    pub format_version: u16,
    pub created_at: String,
    pub data: T,
}

impl<T> Container<T> {
    pub fn wrap(data: T) -> Self {
        Container {
            format_version: 1,
            created_at: chrono_free_timestamp(),
            data,
        }
    }
}

fn chrono_free_timestamp() -> String {
    "2024-01-01T00:00:00Z".to_string()
}

/// Demonstrates a union type that evolved over time.
/// Old format: just a string value
/// New format: typed value with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FlexibleField {
    Simple(String),
    Detailed {
        value: String,
        #[serde(default)]
        unit: Option<String>,
        #[serde(default)]
        source: Option<String>,
    },
}

/// Demonstrates using serde aliases for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiEndpoint {
    #[serde(alias = "url", alias = "path")]
    pub endpoint: String,

    #[serde(alias = "method", alias = "http_method", default = "default_method")]
    pub http_method: String,

    #[serde(alias = "desc", alias = "description", default)]
    pub description: Option<String>,
}

fn default_method() -> String {
    "GET".to_string()
}

/// Demonstrates a pattern for optional migration with version detection.
pub fn detect_and_migrate(json: &str) -> Result<UserProfileV2, serde_json::Error> {
    // Try V2 first
    if let Ok(v2) = serde_json::from_str::<UserProfileV2>(json) {
        return Ok(v2);
    }

    // Fall back to V1 and migrate
    let v1: UserProfileV1 = serde_json::from_str(json)?;
    Ok(UserProfileV2::migrate(v1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_v1_data() {
        // Old data (V1) with only host and port
        let json = r#"{"host": "localhost", "port": 8080}"#;
        let config: ServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.timeout_secs, 30); // default
        assert_eq!(config.max_connections, 100); // default
        assert!(config.features.is_empty()); // default
    }

    #[test]
    fn test_server_config_v3_data() {
        let json = r#"{
            "host": "example.com",
            "port": 443,
            "timeout_secs": 60,
            "max_connections": 500,
            "features": {"tls": true, "compression": false},
            "desc": "Production server"
        }"#;
        let config: ServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.description, Some("Production server".to_string()));
        assert_eq!(config.features.get("tls"), Some(&true));
    }

    #[test]
    fn test_storage_backend_legacy() {
        let json = r#""memory""#;
        let backend: StorageBackend = serde_json::from_str(json).unwrap();
        assert_eq!(backend, StorageBackend::Memory);
    }

    #[test]
    fn test_storage_backend_modern() {
        let json = r#"{"type": "redis", "url": "redis://localhost"}"#;
        let backend: StorageBackend = serde_json::from_str(json).unwrap();
        assert_eq!(
            backend,
            StorageBackend::Redis {
                url: "redis://localhost".to_string()
            }
        );
    }

    #[test]
    fn test_versioned_message() {
        let json = r#"{"version": 1, "kind": "text", "content": "hello"}"#;
        let msg: VersionedMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.version, 1);
        assert!(matches!(msg.payload, MessagePayload::Text { .. }));
    }

    #[test]
    fn test_migrate_v1_to_v2() {
        let v1 = UserProfileV1 {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        let v2 = UserProfileV2::migrate(v1);
        assert_eq!(v2.name, "Alice");
        assert_eq!(v2.phone, None);
        assert!(v2.preferences.is_empty());
    }

    #[test]
    fn test_detect_and_migrate_v1() {
        let json = r#"{"name": "Bob", "email": "bob@example.com"}"#;
        let v2 = detect_and_migrate(json).unwrap();
        assert_eq!(v2.name, "Bob");
        assert_eq!(v2.phone, None);
    }

    #[test]
    fn test_detect_and_migrate_v2() {
        let json = r#"{"name": "Carol", "email": "carol@example.com", "phone": "555-1234"}"#;
        let v2 = detect_and_migrate(json).unwrap();
        assert_eq!(v2.phone, Some("555-1234".to_string()));
    }

    #[test]
    fn test_container_format() {
        let data = vec![1, 2, 3];
        let container = Container::wrap(data);
        let json = serde_json::to_string(&container).unwrap();
        assert!(json.contains("format_version"));

        let back: Container<Vec<i32>> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_flexible_field_simple() {
        let json = r#""hello""#;
        let field: FlexibleField = serde_json::from_str(json).unwrap();
        assert_eq!(field, FlexibleField::Simple("hello".to_string()));
    }

    #[test]
    fn test_flexible_field_detailed() {
        let json = r#"{"value": "100", "unit": "kg"}"#;
        let field: FlexibleField = serde_json::from_str(json).unwrap();
        assert!(matches!(field, FlexibleField::Detailed { .. }));
    }

    #[test]
    fn test_api_endpoint_aliases() {
        let json1 = r#"{"url": "/api/users", "method": "POST"}"#;
        let ep1: ApiEndpoint = serde_json::from_str(json1).unwrap();
        assert_eq!(ep1.endpoint, "/api/users");
        assert_eq!(ep1.http_method, "POST");

        let json2 = r#"{"endpoint": "/api/users", "http_method": "GET"}"#;
        let ep2: ApiEndpoint = serde_json::from_str(json2).unwrap();
        assert_eq!(ep2.endpoint, "/api/users");
    }

    #[test]
    fn test_api_endpoint_defaults() {
        let json = r#"{"endpoint": "/health"}"#;
        let ep: ApiEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(ep.http_method, "GET");
        assert_eq!(ep.description, None);
    }
}
