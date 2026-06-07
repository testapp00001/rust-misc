//! # Lesson 3: anyhow Patterns
//!
//! anyhow is the standard crate for application-level error handling.
//! This lesson covers anyhow::Result, context, error chains, downcasting,
//! and when to use anyhow vs thiserror.

use anyhow::{anyhow, bail, Context, Result};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Basic anyhow usage
// ---------------------------------------------------------------------------

/// A function that returns anyhow::Result for flexible error handling.
/// Use anyhow in application code where you don't need callers to
/// match on specific error variants.
pub fn read_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config from '{}'", path))?;

    let config: Config = serde_json::from_str(&content)
        .context("failed to parse config JSON")?;

    Ok(config)
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub name: String,
    pub port: u16,
    pub database_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Using bail! for early returns
// ---------------------------------------------------------------------------

/// Validate a user registration request.
pub fn validate_registration(request: &RegistrationRequest) -> Result<()> {
    if request.username.is_empty() {
        bail!("username cannot be empty");
    }

    if request.username.len() < 3 {
        bail!("username must be at least 3 characters, got {}", request.username.len());
    }

    if !request.email.contains('@') {
        bail!("invalid email address: '{}'", request.email);
    }

    if request.password.len() < 8 {
        bail!("password must be at least 8 characters");
    }

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

// ---------------------------------------------------------------------------
// anyhow! macro for creating ad-hoc errors
// ---------------------------------------------------------------------------

/// Fetch data from a URL with descriptive error messages.
pub fn fetch_url(url: &str) -> Result<String> {
    if url.is_empty() {
        return Err(anyhow!("URL cannot be empty"));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(anyhow!("URL must start with http:// or https://, got '{}'", url));
    }

    // In real code, this would use reqwest or similar
    Ok(format!("Response from {}", url))
}

// ---------------------------------------------------------------------------
// Context chaining - adding context to errors
// ---------------------------------------------------------------------------

/// A multi-step pipeline that demonstrates context chaining.
pub fn process_file(path: &str) -> Result<ProcessedData> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("step 1: reading file '{}'", path))?;

    let parsed: Vec<DataPoint> = serde_json::from_str(&raw)
        .context("step 2: parsing JSON data")?;

    let validated = validate_data(&parsed)
        .context("step 3: validating data points")?;

    Ok(ProcessedData {
        points: validated,
        source: PathBuf::from(path),
    })
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
}

pub struct ProcessedData {
    pub points: Vec<DataPoint>,
    pub source: PathBuf,
}

fn validate_data(points: &[DataPoint]) -> Result<Vec<DataPoint>> {
    for (i, point) in points.iter().enumerate() {
        if point.value.is_nan() {
            bail!("data point {} has NaN value", i);
        }
        if point.label.is_empty() {
            bail!("data point {} has empty label", i);
        }
    }
    Ok(points.to_vec())
}

// ---------------------------------------------------------------------------
// Downcasting anyhow errors to concrete types
// ---------------------------------------------------------------------------

/// Process an error and extract specific information.
pub fn handle_error(err: &anyhow::Error) -> ErrorResponse {
    // Try to downcast to specific error types
    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
        return ErrorResponse {
            code: 500,
            message: format!("I/O error: {}", io_err.kind()),
            retryable: matches!(
                io_err.kind(),
                std::io::ErrorKind::TimedOut | std::io::ErrorKind::ConnectionRefused
            ),
        };
    }

    if let Some(json_err) = err.downcast_ref::<serde_json::Error>() {
        return ErrorResponse {
            code: 400,
            message: format!("JSON parse error: {}", json_err),
            retryable: false,
        };
    }

    // Default response for unknown errors
    ErrorResponse {
        code: 500,
        message: format!("internal error: {}", err),
        retryable: false,
    }
}

#[derive(Debug, PartialEq)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    pub retryable: bool,
}

// ---------------------------------------------------------------------------
// Error chain inspection
// ---------------------------------------------------------------------------

/// Collect all error messages in the chain.
pub fn error_chain(err: &anyhow::Error) -> Vec<String> {
    let mut messages = vec![format!("{}", err)];
    for cause in err.chain().skip(1) {
        messages.push(format!("{}", cause));
    }
    messages
}

/// Get the root cause of an error chain.
pub fn root_cause(err: &anyhow::Error) -> String {
    err.chain()
        .last()
        .map(|e| format!("{}", e))
        .unwrap_or_else(|| "unknown".to_string())
}

// ---------------------------------------------------------------------------
// Combining multiple errors
// ---------------------------------------------------------------------------

/// Validate multiple fields and collect all errors.
pub fn validate_config(config: &Config) -> Result<()> {
    let mut errors = Vec::new();

    if config.name.is_empty() {
        errors.push("name cannot be empty".to_string());
    }

    if config.port == 0 {
        errors.push("port cannot be 0".to_string());
    }

    if config.port > 65535 {
        errors.push(format!("port {} exceeds maximum 65535", config.port));
    }

    if !errors.is_empty() {
        let combined = errors.join("; ");
        bail!("validation failed: {}", combined);
    }

    Ok(())
}

/// A pipeline that demonstrates when to use anyhow vs thiserror.
///
/// **Use thiserror** when:
/// - You're writing a library
/// - Callers need to match on specific error variants
/// - You want a stable public API for errors
///
/// **Use anyhow** when:
/// - You're writing application code
/// - You don't need callers to match on errors
/// - You want quick, flexible error propagation
pub fn demonstrate_when_to_use_anyhow() -> &'static str {
    "Use thiserror for libraries, anyhow for applications"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"name": "test", "port": 8080, "database_url": null}"#,
        )
        .unwrap();

        let config = read_config(path.to_str().unwrap()).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_read_config_not_found() {
        let result = read_config("/nonexistent/config.json");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let chain = error_chain(&err);
        assert!(chain.iter().any(|m| m.contains("failed to read config")));
    }

    #[test]
    fn test_read_config_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "not json").unwrap();

        let result = read_config(path.to_str().unwrap());
        assert!(result.is_err());
        let err = result.unwrap_err();
        let chain = error_chain(&err);
        assert!(chain.iter().any(|m| m.contains("parse config")));
    }

    #[test]
    fn test_validate_registration_valid() {
        let req = RegistrationRequest {
            username: "alice".into(),
            email: "alice@example.com".into(),
            password: "securepassword".into(),
        };
        assert!(validate_registration(&req).is_ok());
    }

    #[test]
    fn test_validate_registration_empty_username() {
        let req = RegistrationRequest {
            username: "".into(),
            email: "a@b.com".into(),
            password: "securepassword".into(),
        };
        let err = validate_registration(&req).unwrap_err();
        assert!(err.to_string().contains("username cannot be empty"));
    }

    #[test]
    fn test_validate_registration_short_username() {
        let req = RegistrationRequest {
            username: "ab".into(),
            email: "a@b.com".into(),
            password: "securepassword".into(),
        };
        let err = validate_registration(&req).unwrap_err();
        assert!(err.to_string().contains("at least 3 characters"));
    }

    #[test]
    fn test_validate_registration_invalid_email() {
        let req = RegistrationRequest {
            username: "alice".into(),
            email: "invalid".into(),
            password: "securepassword".into(),
        };
        let err = validate_registration(&req).unwrap_err();
        assert!(err.to_string().contains("invalid email"));
    }

    #[test]
    fn test_validate_registration_short_password() {
        let req = RegistrationRequest {
            username: "alice".into(),
            email: "a@b.com".into(),
            password: "short".into(),
        };
        let err = validate_registration(&req).unwrap_err();
        assert!(err.to_string().contains("at least 8 characters"));
    }

    #[test]
    fn test_fetch_url_empty() {
        let result = fetch_url("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_fetch_url_invalid_scheme() {
        let result = fetch_url("ftp://example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("http:// or https://"));
    }

    #[test]
    fn test_fetch_url_success() {
        let result = fetch_url("https://example.com").unwrap();
        assert!(result.contains("example.com"));
    }

    #[test]
    fn test_handle_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "timed out");
        let anyhow_err: anyhow::Error = io_err.into();
        let resp = handle_error(&anyhow_err);
        assert_eq!(resp.code, 500);
        assert!(resp.retryable);
    }

    #[test]
    fn test_handle_error_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("bad").unwrap_err();
        let anyhow_err: anyhow::Error = json_err.into();
        let resp = handle_error(&anyhow_err);
        assert_eq!(resp.code, 400);
        assert!(!resp.retryable);
    }

    #[test]
    fn test_error_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let anyhow_err = anyhow::Error::from(io_err)
            .context("loading configuration")
            .context("application startup");

        let chain = error_chain(&anyhow_err);
        assert_eq!(chain.len(), 3);
        assert!(chain[0].contains("application startup"));
        assert!(chain[1].contains("loading configuration"));
        assert!(chain[2].contains("file missing"));
    }

    #[test]
    fn test_root_cause() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "deep problem");
        let anyhow_err = anyhow::Error::from(io_err).context("wrapper");
        assert_eq!(root_cause(&anyhow_err), "deep problem");
    }

    #[test]
    fn test_validate_config_valid() {
        let config = Config {
            name: "test".into(),
            port: 8080,
            database_url: None,
        };
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_empty_name() {
        let config = Config {
            name: "".into(),
            port: 8080,
            database_url: None,
        };
        let err = validate_config(&config).unwrap_err();
        assert!(err.to_string().contains("name cannot be empty"));
    }

    #[test]
    fn test_validate_config_zero_port() {
        let config = Config {
            name: "test".into(),
            port: 0,
            database_url: None,
        };
        let err = validate_config(&config).unwrap_err();
        assert!(err.to_string().contains("port cannot be 0"));
    }

    #[test]
    fn test_validate_config_multiple_errors() {
        let config = Config {
            name: "".into(),
            port: 0,
            database_url: None,
        };
        let err = validate_config(&config).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("name cannot be empty"));
        assert!(msg.contains("port cannot be 0"));
    }

    #[test]
    fn test_demonstrate_when_to_use() {
        let msg = demonstrate_when_to_use_anyhow();
        assert!(msg.contains("thiserror"));
        assert!(msg.contains("anyhow"));
    }
}
