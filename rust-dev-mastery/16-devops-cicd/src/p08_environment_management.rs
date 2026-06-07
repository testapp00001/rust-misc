//! # Environment Management for Rust Applications
//!
//! Managing configurations across development, staging, and production
//! environments is critical for reliable deployments. This module covers
//! configuration loading, validation, secret injection, and environment
//! promotion patterns.
//!
//! ## Configuration Layers (lowest to highest priority):
//!
//! 1. Default values (compiled into binary)
//! 2. Configuration file (config.toml)
//! 3. Environment variables
//! 4. Command-line arguments
//! 5. Remote configuration (feature flags, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an application environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

impl Environment {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Ok(Self::Development),
            "test" | "testing" => Ok(Self::Testing),
            "staging" | "stage" => Ok(Self::Staging),
            "prod" | "production" => Ok(Self::Production),
            _ => Err(format!("Unknown environment: {}", s)),
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    /// Return the RUST_LOG level appropriate for this environment.
    pub fn default_log_level(&self) -> &str {
        match self {
            Self::Development => "debug",
            Self::Testing => "debug",
            Self::Staging => "info",
            Self::Production => "warn",
        }
    }
}

/// Multi-layered application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub environment: Environment,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub features: FeatureConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub request_timeout_secs: u64,
    pub keep_alive_secs: u64,
    pub max_connections: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 8080,
            workers: None, // Use number of CPUs
            request_timeout_secs: 30,
            keep_alive_secs: 75,
            max_connections: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
    pub ssl_mode: SslMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost:5432/app".into(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            max_lifetime_secs: 1800,
            ssl_mode: SslMode::Prefer,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
    pub enable_request_logging: bool,
    pub enable_correlation_ids: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Text,
    Json,
    Pretty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    File(String),
    Both(String),
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".into(),
            format: LogFormat::Text,
            output: LogOutput::Stdout,
            enable_request_logging: true,
            enable_correlation_ids: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub enable_profiling: bool,
    pub experimental_features: Vec<String>,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            enable_tracing: false,
            enable_profiling: false,
            experimental_features: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub tls_enabled: bool,
    pub cors_origins: Vec<String>,
    pub rate_limit_requests_per_second: u32,
    pub api_key_required: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tls_enabled: false,
            cors_origins: vec!["*".into()],
            rate_limit_requests_per_second: 100,
            api_key_required: false,
        }
    }
}

impl AppConfig {
    /// Load configuration for a specific environment with sensible defaults.
    pub fn for_environment(env: &Environment) -> Self {
        match env {
            Environment::Development => Self::development(),
            Environment::Testing => Self::testing(),
            Environment::Staging => Self::staging(),
            Environment::Production => Self::production(),
        }
    }

    pub fn development() -> Self {
        Self {
            environment: Environment::Development,
            server: ServerConfig {
                port: 3000,
                ..Default::default()
            },
            database: DatabaseConfig {
                url: "postgres://localhost:5432/app_dev".into(),
                max_connections: 5,
                ..Default::default()
            },
            logging: LoggingConfig {
                level: "debug".into(),
                format: LogFormat::Pretty,
                ..Default::default()
            },
            features: FeatureConfig {
                enable_tracing: true,
                enable_profiling: true,
                experimental_features: vec!["new-ui".into()],
                ..Default::default()
            },
            security: SecurityConfig {
                cors_origins: vec!["http://localhost:3000".into()],
                rate_limit_requests_per_second: 1000,
                ..Default::default()
            },
        }
    }

    pub fn testing() -> Self {
        Self {
            environment: Environment::Testing,
            server: ServerConfig {
                port: 3001,
                ..Default::default()
            },
            database: DatabaseConfig {
                url: "postgres://localhost:5432/app_test".into(),
                max_connections: 5,
                ..Default::default()
            },
            logging: LoggingConfig {
                level: "debug".into(),
                format: LogFormat::Text,
                ..Default::default()
            },
            features: FeatureConfig::default(),
            security: SecurityConfig {
                rate_limit_requests_per_second: 10000,
                ..Default::default()
            },
        }
    }

    pub fn staging() -> Self {
        Self {
            environment: Environment::Staging,
            server: ServerConfig {
                port: 8080,
                workers: Some(2),
                ..Default::default()
            },
            database: DatabaseConfig {
                url: "postgres://staging-db:5432/app".into(),
                max_connections: 20,
                ssl_mode: SslMode::Require,
                ..Default::default()
            },
            logging: LoggingConfig {
                level: "info".into(),
                format: LogFormat::Json,
                ..Default::default()
            },
            features: FeatureConfig {
                enable_metrics: true,
                enable_tracing: true,
                ..Default::default()
            },
            security: SecurityConfig {
                tls_enabled: true,
                cors_origins: vec!["https://staging.example.com".into()],
                rate_limit_requests_per_second: 500,
                api_key_required: true,
                ..Default::default()
            },
        }
    }

    pub fn production() -> Self {
        Self {
            environment: Environment::Production,
            server: ServerConfig {
                port: 8080,
                workers: Some(4),
                max_connections: 5000,
                ..Default::default()
            },
            database: DatabaseConfig {
                url: "postgres://prod-db:5432/app".into(),
                max_connections: 50,
                min_connections: 10,
                ssl_mode: SslMode::Require,
                ..Default::default()
            },
            logging: LoggingConfig {
                level: "warn".into(),
                format: LogFormat::Json,
                ..Default::default()
            },
            features: FeatureConfig {
                enable_metrics: true,
                enable_tracing: true,
                enable_profiling: false,
                experimental_features: vec![],
            },
            security: SecurityConfig {
                tls_enabled: true,
                cors_origins: vec!["https://example.com".into()],
                rate_limit_requests_per_second: 200,
                api_key_required: true,
            },
        }
    }

    /// Override config values from environment variables.
    pub fn apply_env_overrides(&mut self) {
        if let Ok(port) = std::env::var("APP_PORT") {
            if let Ok(port) = port.parse::<u16>() {
                self.server.port = port;
            }
        }
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            self.database.url = db_url;
        }
        if let Ok(log_level) = std::env::var("RUST_LOG") {
            self.logging.level = log_level;
        }
        if let Ok(workers) = std::env::var("APP_WORKERS") {
            if let Ok(w) = workers.parse::<usize>() {
                self.server.workers = Some(w);
            }
        }
    }

    /// Validate the configuration for the target environment.
    pub fn validate(&self) -> Result<(), Vec<ConfigValidationError>> {
        let mut errors = Vec::new();

        if self.environment.is_production() {
            if !self.security.tls_enabled {
                errors.push(ConfigValidationError {
                    field: "security.tls_enabled".into(),
                    message: "TLS must be enabled in production".into(),
                    severity: ValidationSeverity::Error,
                });
            }
            if self.security.cors_origins.contains(&"*".to_string()) {
                errors.push(ConfigValidationError {
                    field: "security.cors_origins".into(),
                    message: "Wildcard CORS origins not allowed in production".into(),
                    severity: ValidationSeverity::Error,
                });
            }
            if self.database.max_connections < 10 {
                errors.push(ConfigValidationError {
                    field: "database.max_connections".into(),
                    message: "Production should have at least 10 DB connections".into(),
                    severity: ValidationSeverity::Warning,
                });
            }
        }

        if self.server.port == 0 {
            errors.push(ConfigValidationError {
                field: "server.port".into(),
                message: "Port cannot be 0".into(),
                severity: ValidationSeverity::Error,
            });
        }

        if self.database.url.is_empty() {
            errors.push(ConfigValidationError {
                field: "database.url".into(),
                message: "Database URL cannot be empty".into(),
                severity: ValidationSeverity::Error,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

/// Secret management for sensitive configuration values.
/// In production, these would come from a secrets manager (Vault, AWS Secrets Manager, etc.)
#[derive(Debug, Clone)]
pub struct SecretManager {
    secrets: HashMap<String, String>,
}

impl SecretManager {
    pub fn new() -> Self {
        Self {
            secrets: HashMap::new(),
        }
    }

    /// Load secrets from environment variables.
    pub fn from_env(prefix: &str) -> Self {
        let mut manager = Self::new();
        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let secret_name = key
                    .strip_prefix(prefix)
                    .unwrap_or(&key)
                    .to_lowercase();
                manager.secrets.insert(secret_name, value);
            }
        }
        manager
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.secrets.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.secrets.get(key).map(|s| s.as_str())
    }

    /// Mask a secret value for logging (show only first and last character).
    pub fn mask(value: &str) -> String {
        if value.len() <= 4 {
            return "*".repeat(value.len());
        }
        format!(
            "{}{}{}",
            &value[..1],
            "*".repeat(value.len() - 2),
            &value[value.len() - 1..]
        )
    }
}

/// Environment promotion workflow tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionRecord {
    pub version: String,
    pub from_env: Environment,
    pub to_env: Environment,
    pub promoted_by: String,
    pub promoted_at: String,
    pub checks_passed: Vec<String>,
    pub approval_required: bool,
    pub approved_by: Option<String>,
}

impl PromotionRecord {
    pub fn new(
        version: &str,
        from: Environment,
        to: Environment,
        promoted_by: &str,
    ) -> Self {
        let approval_required = matches!(to, Environment::Production);
        Self {
            version: version.into(),
            from_env: from,
            to_env: to,
            promoted_by: promoted_by.into(),
            promoted_at: "2024-06-01T12:00:00Z".into(),
            checks_passed: Vec::new(),
            approval_required,
            approved_by: None,
        }
    }

    pub fn is_approved(&self) -> bool {
        !self.approval_required || self.approved_by.is_some()
    }

    pub fn approve(&mut self, approver: &str) {
        self.approved_by = Some(approver.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_from_str() {
        assert_eq!(Environment::from_str("dev").unwrap(), Environment::Development);
        assert_eq!(Environment::from_str("testing").unwrap(), Environment::Testing);
        assert_eq!(Environment::from_str("staging").unwrap(), Environment::Staging);
        assert_eq!(Environment::from_str("prod").unwrap(), Environment::Production);
        assert!(Environment::from_str("unknown").is_err());
    }

    #[test]
    fn test_environment_is_production() {
        assert!(!Environment::Development.is_production());
        assert!(Environment::Production.is_production());
    }

    #[test]
    fn test_environment_log_levels() {
        assert_eq!(Environment::Development.default_log_level(), "debug");
        assert_eq!(Environment::Production.default_log_level(), "warn");
    }

    #[test]
    fn test_app_config_for_environment() {
        let dev = AppConfig::for_environment(&Environment::Development);
        assert_eq!(dev.server.port, 3000);
        assert!(dev.features.enable_profiling);

        let prod = AppConfig::for_environment(&Environment::Production);
        assert_eq!(prod.server.port, 8080);
        assert!(!prod.features.enable_profiling);
        assert!(prod.security.tls_enabled);
    }

    #[test]
    fn test_app_config_development() {
        let config = AppConfig::development();
        assert_eq!(config.environment, Environment::Development);
        assert!(config.database.url.contains("app_dev"));
        assert!(matches!(config.logging.format, LogFormat::Pretty));
    }

    #[test]
    fn test_app_config_production() {
        let config = AppConfig::production();
        assert_eq!(config.environment, Environment::Production);
        assert!(config.security.tls_enabled);
        assert!(config.security.api_key_required);
        assert!(!config.security.cors_origins.contains(&"*".to_string()));
    }

    #[test]
    fn test_config_validation_production_requires_tls() {
        let mut config = AppConfig::production();
        config.security.tls_enabled = false;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field.contains("tls")));
    }

    #[test]
    fn test_config_validation_production_no_wildcard_cors() {
        let mut config = AppConfig::production();
        config.security.cors_origins = vec!["*".into()];
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field.contains("cors")));
    }

    #[test]
    fn test_config_validation_production_happy_path() {
        let config = AppConfig::production();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_port() {
        let mut config = AppConfig::development();
        config.server.port = 0;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field.contains("port")));
    }

    #[test]
    fn test_config_validation_empty_db_url() {
        let mut config = AppConfig::development();
        config.database.url = String::new();
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_secret_manager_basic() {
        let mut manager = SecretManager::new();
        manager.set("db_password", "supersecret");
        assert_eq!(manager.get("db_password"), Some("supersecret"));
        assert_eq!(manager.get("nonexistent"), None);
    }

    #[test]
    fn test_secret_manager_mask() {
        assert_eq!(SecretManager::mask("abc"), "***");
        assert_eq!(SecretManager::mask("abcdef"), "a****f");
        assert_eq!(SecretManager::mask("supersecret"), "s*********t"); // 11 chars: 1 + 9 + 1
        assert_eq!(SecretManager::mask("ab"), "**");
    }

    #[test]
    fn test_promotion_record() {
        let mut record = PromotionRecord::new(
            "1.2.3",
            Environment::Staging,
            Environment::Production,
            "deploy-bot",
        );

        assert!(record.approval_required);
        assert!(!record.is_approved());

        record.approve("senior-engineer");
        assert!(record.is_approved());
        assert_eq!(record.approved_by, Some("senior-engineer".into()));
    }

    #[test]
    fn test_promotion_no_approval_needed() {
        let record = PromotionRecord::new(
            "1.2.3",
            Environment::Development,
            Environment::Staging,
            "dev-team",
        );

        assert!(!record.approval_required);
        assert!(record.is_approved());
    }

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(config.workers.is_none());
    }

    #[test]
    fn test_database_config_defaults() {
        let config = DatabaseConfig::default();
        assert!(config.url.contains("localhost"));
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::development();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.environment, Environment::Development);
        assert_eq!(deserialized.server.port, 3000);
    }
}
