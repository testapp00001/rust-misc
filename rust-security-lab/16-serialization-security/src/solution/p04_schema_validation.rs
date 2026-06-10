//! # Lesson 04: Schema Validation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::Deserialize;

/// A server configuration that must pass strict validation.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub hostname: String,
    pub port: u16,
    pub max_connections: u32,
    pub log_level: String,
}

/// Validate a ServerConfig.
pub fn validate_server_config(config: &ServerConfig) -> Result<(), String> {
    if config.hostname.is_empty() {
        return Err("hostname must not be empty".to_string());
    }
    if config.hostname.len() > 253 {
        return Err("hostname must be at most 253 characters".to_string());
    }
    if config.port == 0 {
        return Err("port must not be 0".to_string());
    }
    if config.port <= 1023 {
        return Err("port must be above 1023 (non-privileged)".to_string());
    }
    if config.max_connections == 0 || config.max_connections > 100_000 {
        return Err("max_connections must be between 1 and 100,000".to_string());
    }
    let allowed_levels = ["trace", "debug", "info", "warn", "error"];
    if !allowed_levels.contains(&config.log_level.as_str()) {
        return Err(format!(
            "log_level must be one of {:?}",
            allowed_levels
        ));
    }
    Ok(())
}

/// Parse and validate a ServerConfig from JSON.
pub fn parse_server_config(json: &str) -> Result<ServerConfig, String> {
    let config: ServerConfig =
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))?;
    validate_server_config(&config)?;
    Ok(config)
}

/// An API request with validation.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CreatePostRequest {
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

/// Validate a CreatePostRequest.
pub fn validate_create_post(request: &CreatePostRequest) -> Result<(), String> {
    if request.title.is_empty() || request.title.len() > 200 {
        return Err("title must be 1-200 characters".to_string());
    }
    if request.body.is_empty() || request.body.len() > 10_000 {
        return Err("body must be 1-10,000 characters".to_string());
    }
    if request.tags.len() > 10 {
        return Err("at most 10 tags allowed".to_string());
    }
    for tag in &request.tags {
        if tag.is_empty() || tag.len() > 50 {
            return Err("each tag must be 1-50 characters".to_string());
        }
        if !tag.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
            return Err("tags must be lowercase alphanumeric".to_string());
        }
    }
    Ok(())
}

/// Parse and validate a CreatePostRequest from JSON.
pub fn parse_create_post(json: &str) -> Result<CreatePostRequest, String> {
    let request: CreatePostRequest =
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))?;
    validate_create_post(&request)?;
    Ok(request)
}

/// Generic value range validator.
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    field_name: &str,
) -> Result<(), String> {
    if value < min || value > max {
        return Err(format!(
            "{}: value {} is out of range [{}, {}]",
            field_name, value, min, max
        ));
    }
    Ok(())
}

/// Validate a string against an allowlist.
pub fn validate_enum(
    value: &str,
    allowed: &[&str],
    field_name: &str,
) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "{}: '{}' is not in allowed values {:?}",
            field_name, value, allowed
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let config = ServerConfig {
            hostname: "api.example.com".to_string(),
            port: 8080,
            max_connections: 1000,
            log_level: "info".to_string(),
        };
        assert!(validate_server_config(&config).is_ok());
    }

    #[test]
    fn test_empty_hostname() {
        let config = ServerConfig {
            hostname: "".to_string(),
            port: 8080,
            max_connections: 1000,
            log_level: "info".to_string(),
        };
        assert!(validate_server_config(&config).is_err());
    }

    #[test]
    fn test_reserved_port() {
        let config = ServerConfig {
            hostname: "api.example.com".to_string(),
            port: 0,
            max_connections: 1000,
            log_level: "info".to_string(),
        };
        assert!(validate_server_config(&config).is_err());
    }

    #[test]
    fn test_privileged_port() {
        let config = ServerConfig {
            hostname: "api.example.com".to_string(),
            port: 80,
            max_connections: 1000,
            log_level: "info".to_string(),
        };
        assert!(validate_server_config(&config).is_err());
    }

    #[test]
    fn test_invalid_log_level() {
        let config = ServerConfig {
            hostname: "api.example.com".to_string(),
            port: 8080,
            max_connections: 1000,
            log_level: "verbose".to_string(),
        };
        assert!(validate_server_config(&config).is_err());
    }

    #[test]
    fn test_parse_and_validate_config() {
        let json = r#"{
            "hostname": "api.example.com",
            "port": 8080,
            "max_connections": 1000,
            "log_level": "info"
        }"#;
        assert!(parse_server_config(json).is_ok());
    }

    #[test]
    fn test_valid_post_request() {
        let req = CreatePostRequest {
            title: "Hello World".to_string(),
            body: "This is a valid post body.".to_string(),
            tags: vec!["rust".to_string(), "security".to_string()],
        };
        assert!(validate_create_post(&req).is_ok());
    }

    #[test]
    fn test_too_many_tags() {
        let req = CreatePostRequest {
            title: "Post".to_string(),
            body: "Body text here.".to_string(),
            tags: (0..11).map(|i| format!("tag{}", i)).collect(),
        };
        assert!(validate_create_post(&req).is_err());
    }

    #[test]
    fn test_range_validator() {
        assert!(validate_range(50, 0, 100, "score").is_ok());
        assert!(validate_range(101, 0, 100, "score").is_err());
        assert!(validate_range(-1, 0, 100, "score").is_err());
    }

    #[test]
    fn test_enum_validator() {
        let allowed = &["trace", "debug", "info", "warn", "error"];
        assert!(validate_enum("info", allowed, "log_level").is_ok());
        assert!(validate_enum("verbose", allowed, "log_level").is_err());
    }
}
