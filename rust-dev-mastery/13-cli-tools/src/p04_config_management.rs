//! # Configuration Management
//!
//! Real CLI tools need layered configuration: defaults, config files, environment
//! variables, and command-line arguments. This lesson covers building a robust
//! configuration system with TOML config files, environment variable overrides,
//! and layered merging.
//!
//! ## Key Concepts
//! - Configuration layers (defaults < file < env < CLI)
//! - TOML parsing and serialization
//! - Environment variable mapping
//! - Config file discovery
//! - Config validation
//! - Secrets handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// 1. Configuration Structure
// ---------------------------------------------------------------------------

/// Application configuration with all settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub database: DatabaseConfig,

    #[serde(default)]
    pub logging: LoggingConfig,

    #[serde(default)]
    pub features: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralConfig {
    #[serde(default = "default_app_name")]
    pub app_name: String,

    #[serde(default = "default_environment")]
    pub environment: String,

    #[serde(default)]
    pub debug: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_workers")]
    pub workers: usize,

    #[serde(default)]
    pub tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_url")]
    pub url: String,

    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    #[serde(default)]
    pub run_migrations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default)]
    pub json_format: bool,

    #[serde(default)]
    pub file: Option<String>,
}

fn default_app_name() -> String {
    "my-app".into()
}
fn default_environment() -> String {
    "development".into()
}
fn default_host() -> String {
    "127.0.0.1".into()
}
fn default_port() -> u16 {
    8080
}
fn default_workers() -> usize {
    4
}
fn default_db_url() -> String {
    "sqlite://app.db".into()
}
fn default_pool_size() -> u32 {
    10
}
fn default_log_level() -> String {
    "info".into()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
            features: HashMap::new(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            app_name: default_app_name(),
            environment: default_environment(),
            debug: false,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            workers: default_workers(),
            tls: false,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: default_db_url(),
            pool_size: default_pool_size(),
            run_migrations: false,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            json_format: false,
            file: None,
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Configuration Builder
// ---------------------------------------------------------------------------

/// Builds configuration from multiple sources with proper layering.
pub struct ConfigBuilder {
    defaults: AppConfig,
    file_path: Option<PathBuf>,
    env_prefix: String,
    overrides: HashMap<String, String>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            defaults: AppConfig::default(),
            file_path: None,
            env_prefix: "APP".into(),
            overrides: HashMap::new(),
        }
    }

    pub fn with_defaults(mut self, defaults: AppConfig) -> Self {
        self.defaults = defaults;
        self
    }

    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    pub fn set_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides.insert(key.into(), value.into());
        self
    }

    /// Build the final configuration.
    pub fn build(self) -> Result<AppConfig, ConfigError> {
        let mut config = self.defaults;

        // Layer 1: File config
        if let Some(ref path) = self.file_path {
            if path.exists() {
                let file_config = load_toml_file(path)?;
                merge_config(&mut config, &file_config);
            }
        }

        // Layer 2: Environment variables
        apply_env_overrides(&mut config, &self.env_prefix);

        // Layer 3: Explicit overrides
        for (key, value) in &self.overrides {
            apply_override(&mut config, key, value)?;
        }

        // Validate
        validate_config(&config)?;

        Ok(config)
    }
}

// ---------------------------------------------------------------------------
// 3. TOML File Loading
// ---------------------------------------------------------------------------

/// Load configuration from a TOML file.
pub fn load_toml_file(path: &Path) -> Result<AppConfig, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::IoError {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;

    toml::from_str(&content).map_err(|e| ConfigError::ParseError {
        path: path.display().to_string(),
        message: e.to_string(),
    })
}

/// Serialize configuration to TOML string.
pub fn to_toml_string(config: &AppConfig) -> Result<String, ConfigError> {
    toml::to_string_pretty(config).map_err(|e| ConfigError::SerializationError(e.to_string()))
}

// ---------------------------------------------------------------------------
// 4. Configuration Merging
// ---------------------------------------------------------------------------

/// Merge file config into the base config (non-default values override).
fn merge_config(base: &mut AppConfig, overlay: &AppConfig) {
    // Only override non-default values
    if overlay.general.app_name != default_app_name() {
        base.general.app_name = overlay.general.app_name.clone();
    }
    if overlay.general.environment != default_environment() {
        base.general.environment = overlay.general.environment.clone();
    }
    if overlay.general.debug {
        base.general.debug = true;
    }
    if overlay.server.host != default_host() {
        base.server.host = overlay.server.host.clone();
    }
    if overlay.server.port != default_port() {
        base.server.port = overlay.server.port;
    }
    if overlay.server.workers != default_workers() {
        base.server.workers = overlay.server.workers;
    }
    if overlay.database.url != default_db_url() {
        base.database.url = overlay.database.url.clone();
    }
    if overlay.database.pool_size != default_pool_size() {
        base.database.pool_size = overlay.database.pool_size;
    }
    if overlay.logging.level != default_log_level() {
        base.logging.level = overlay.logging.level.clone();
    }
    if overlay.logging.json_format {
        base.logging.json_format = true;
    }
    if overlay.logging.file.is_some() {
        base.logging.file = overlay.logging.file.clone();
    }
    for (k, v) in &overlay.features {
        base.features.insert(k.clone(), *v);
    }
}

// ---------------------------------------------------------------------------
// 5. Environment Variable Mapping
// ---------------------------------------------------------------------------

/// Map environment variables to config values.
/// Convention: `APP_SERVER_PORT=8080` maps to `server.port`.
fn apply_env_overrides(config: &mut AppConfig, prefix: &str) {
    macro_rules! apply_env {
        ($config:expr, $prefix:expr, $suffix:expr, $setter:expr) => {
            let env_key = format!("{}_{}", $prefix, $suffix);
            if let Ok(value) = std::env::var(&env_key) {
                $setter($config, &value);
            }
        };
    }

    apply_env!(config, prefix, "GENERAL_APP_NAME", |c: &mut AppConfig, v: &str| {
        c.general.app_name = v.into()
    });
    apply_env!(config, prefix, "GENERAL_ENVIRONMENT", |c: &mut AppConfig, v: &str| {
        c.general.environment = v.into()
    });
    apply_env!(config, prefix, "GENERAL_DEBUG", |c: &mut AppConfig, v: &str| {
        c.general.debug = parse_bool(v)
    });
    apply_env!(config, prefix, "SERVER_HOST", |c: &mut AppConfig, v: &str| {
        c.server.host = v.into()
    });
    apply_env!(config, prefix, "SERVER_PORT", |c: &mut AppConfig, v: &str| {
        if let Ok(port) = v.parse() {
            c.server.port = port
        }
    });
    apply_env!(config, prefix, "SERVER_WORKERS", |c: &mut AppConfig, v: &str| {
        if let Ok(w) = v.parse() {
            c.server.workers = w
        }
    });
    apply_env!(config, prefix, "DATABASE_URL", |c: &mut AppConfig, v: &str| {
        c.database.url = v.into()
    });
    apply_env!(config, prefix, "DATABASE_POOL_SIZE", |c: &mut AppConfig, v: &str| {
        if let Ok(s) = v.parse() {
            c.database.pool_size = s
        }
    });
    apply_env!(config, prefix, "LOGGING_LEVEL", |c: &mut AppConfig, v: &str| {
        c.logging.level = v.into()
    });
    apply_env!(config, prefix, "LOGGING_JSON_FORMAT", |c: &mut AppConfig, v: &str| {
        c.logging.json_format = parse_bool(v)
    });
}

fn parse_bool(s: &str) -> bool {
    matches!(
        s.to_lowercase().as_str(),
        "true" | "1" | "yes" | "on"
    )
}

// ---------------------------------------------------------------------------
// 6. Key-Path Override
// ---------------------------------------------------------------------------

/// Apply a dot-separated key override like "server.port=9090".
fn apply_override(config: &mut AppConfig, key: &str, value: &str) -> Result<(), ConfigError> {
    match key {
        "general.app_name" => config.general.app_name = value.into(),
        "general.environment" => config.general.environment = value.into(),
        "general.debug" => config.general.debug = parse_bool(value),
        "server.host" => config.server.host = value.into(),
        "server.port" => {
            config.server.port = value
                .parse()
                .map_err(|_| ConfigError::InvalidValue(key.into(), value.into()))?
        }
        "server.workers" => {
            config.server.workers = value
                .parse()
                .map_err(|_| ConfigError::InvalidValue(key.into(), value.into()))?
        }
        "database.url" => config.database.url = value.into(),
        "database.pool_size" => {
            config.database.pool_size = value
                .parse()
                .map_err(|_| ConfigError::InvalidValue(key.into(), value.into()))?
        }
        "logging.level" => config.logging.level = value.into(),
        "logging.json_format" => config.logging.json_format = parse_bool(value),
        _ => {
            // Treat as feature flag if starts with "features."
            if let Some(feature_name) = key.strip_prefix("features.") {
                config.features.insert(feature_name.into(), parse_bool(value));
            } else {
                return Err(ConfigError::UnknownKey(key.into()));
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 7. Configuration Validation
// ---------------------------------------------------------------------------

/// Validate the configuration for consistency and correctness.
pub fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    // Validate port range
    if config.server.port == 0 {
        return Err(ConfigError::Validation(
            "server.port cannot be 0".into(),
        ));
    }

    // Validate workers
    if config.server.workers == 0 {
        return Err(ConfigError::Validation(
            "server.workers must be at least 1".into(),
        ));
    }

    // Validate pool size
    if config.database.pool_size == 0 {
        return Err(ConfigError::Validation(
            "database.pool_size must be at least 1".into(),
        ));
    }

    // Validate log level
    let valid_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_levels.contains(&config.logging.level.as_str()) {
        return Err(ConfigError::Validation(format!(
            "invalid log level '{}', must be one of: {:?}",
            config.logging.level, valid_levels
        )));
    }

    // Validate environment
    let valid_envs = ["development", "staging", "production", "test"];
    if !valid_envs.contains(&config.general.environment.as_str()) {
        return Err(ConfigError::Validation(format!(
            "invalid environment '{}', must be one of: {:?}",
            config.general.environment, valid_envs
        )));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 8. Config File Discovery
// ---------------------------------------------------------------------------

/// Search for configuration files in standard locations.
pub fn find_config_file(app_name: &str) -> Option<PathBuf> {
    let candidates = vec![
        PathBuf::from(format!("./{app_name}.toml")),
        PathBuf::from(format!("./{app_name}.conf")),
        PathBuf::from(format!("./config/{app_name}.toml")),
        PathBuf::from(format!("./.config/{app_name}/config.toml")),
    ];

    candidates.into_iter().find(|p| p.exists())
}

/// Get the default config file path for writing.
pub fn default_config_path(app_name: &str) -> PathBuf {
    PathBuf::from(format!("./config/{app_name}.toml"))
}

// ---------------------------------------------------------------------------
// 9. Configuration Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error reading {path}: {message}")]
    IoError { path: String, message: String },

    #[error("parse error in {path}: {message}")]
    ParseError { path: String, message: String },

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("invalid value for {0}: '{1}'")]
    InvalidValue(String, String),

    #[error("unknown config key: {0}")]
    UnknownKey(String),

    #[error("validation error: {0}")]
    Validation(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.general.app_name, "my-app");
        assert_eq!(config.general.environment, "development");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.workers, 4);
        assert_eq!(config.database.url, "sqlite://app.db");
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = AppConfig::default();
        let toml_str = to_toml_string(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_toml_partial() {
        let toml_str = r#"
[server]
port = 3000
host = "0.0.0.0"

[logging]
level = "debug"
"#;
        let parsed: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(parsed.server.port, 3000);
        assert_eq!(parsed.server.host, "0.0.0.0");
        assert_eq!(parsed.logging.level, "debug");
        // Defaults for unspecified fields
        assert_eq!(parsed.general.app_name, "my-app");
        assert_eq!(parsed.server.workers, 4);
    }

    #[test]
    fn test_config_builder_with_overrides() {
        let config = ConfigBuilder::new()
            .set_override("server.port", "9090")
            .set_override("logging.level", "debug")
            .set_override("general.debug", "true")
            .build()
            .unwrap();

        assert_eq!(config.server.port, 9090);
        assert_eq!(config.logging.level, "debug");
        assert!(config.general.debug);
    }

    #[test]
    fn test_config_builder_feature_flags() {
        let config = ConfigBuilder::new()
            .set_override("features.dark_mode", "true")
            .set_override("features.beta_ui", "false")
            .build()
            .unwrap();

        assert_eq!(config.features.get("dark_mode"), Some(&true));
        assert_eq!(config.features.get("beta_ui"), Some(&false));
    }

    #[test]
    fn test_apply_override_all_fields() {
        let mut config = AppConfig::default();

        apply_override(&mut config, "general.app_name", "new-app").unwrap();
        assert_eq!(config.general.app_name, "new-app");

        apply_override(&mut config, "general.environment", "production").unwrap();
        assert_eq!(config.general.environment, "production");

        apply_override(&mut config, "server.host", "0.0.0.0").unwrap();
        assert_eq!(config.server.host, "0.0.0.0");

        apply_override(&mut config, "server.port", "3000").unwrap();
        assert_eq!(config.server.port, 3000);

        apply_override(&mut config, "server.workers", "8").unwrap();
        assert_eq!(config.server.workers, 8);

        apply_override(&mut config, "database.url", "postgres://localhost/db").unwrap();
        assert_eq!(config.database.url, "postgres://localhost/db");

        apply_override(&mut config, "database.pool_size", "20").unwrap();
        assert_eq!(config.database.pool_size, 20);
    }

    #[test]
    fn test_apply_override_invalid_port() {
        let mut config = AppConfig::default();
        let result = apply_override(&mut config, "server.port", "not-a-number");
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_override_unknown_key() {
        let mut config = AppConfig::default();
        let result = apply_override(&mut config, "unknown.key", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_valid() {
        let config = AppConfig::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_zero_port() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_zero_workers() {
        let mut config = AppConfig::default();
        config.server.workers = 0;
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_invalid_log_level() {
        let mut config = AppConfig::default();
        config.logging.level = "verbose".into();
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_invalid_environment() {
        let mut config = AppConfig::default();
        config.general.environment = "dev".into();
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse_bool("true"));
        assert!(parse_bool("TRUE"));
        assert!(parse_bool("1"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("on"));
        assert!(!parse_bool("false"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool(""));
    }

    #[test]
    fn test_merge_config() {
        let mut base = AppConfig::default();
        let mut overlay = AppConfig::default();
        overlay.server.port = 9090;
        overlay.logging.level = "debug".into();

        merge_config(&mut base, &overlay);
        assert_eq!(base.server.port, 9090);
        assert_eq!(base.logging.level, "debug");
        // Other defaults remain
        assert_eq!(base.general.app_name, "my-app");
    }

    #[test]
    fn test_config_file_toml_parsing() {
        let toml = r#"
[general]
app_name = "test-app"
environment = "production"
debug = true

[server]
host = "0.0.0.0"
port = 443
workers = 16
tls = true

[database]
url = "postgres://user:pass@localhost/mydb"
pool_size = 50
run_migrations = true

[logging]
level = "warn"
json_format = true
file = "/var/log/app.log"

[features]
dark_mode = true
beta_ui = false
"#;

        let config: AppConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.general.app_name, "test-app");
        assert_eq!(config.general.environment, "production");
        assert!(config.general.debug);
        assert_eq!(config.server.port, 443);
        assert!(config.server.tls);
        assert_eq!(config.database.pool_size, 50);
        assert!(config.database.run_migrations);
        assert!(config.logging.json_format);
        assert_eq!(config.logging.file.as_deref(), Some("/var/log/app.log"));
        assert_eq!(config.features.get("dark_mode"), Some(&true));
    }

    #[test]
    fn test_default_config_path() {
        let path = default_config_path("my-app");
        assert_eq!(path, PathBuf::from("./config/my-app.toml"));
    }
}
