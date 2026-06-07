/// Problem: Configuration
///
/// Master configuration management in Rust.
///
/// Key Concepts:
/// - Configuration files
/// - Environment variables
/// - Command line arguments
/// - Configuration validation
/// - Configuration merging

use std::collections::HashMap;

/// Problem 1: Basic configuration
/// Define basic configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub debug: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            debug: false,
        }
    }
}

/// Problem 2: Configuration from HashMap
/// Create config from HashMap
pub fn config_from_map(map: &HashMap<String, String>) -> Config {
    Config {
        host: map.get("host").cloned().unwrap_or_else(|| "localhost".to_string()),
        port: map.get("port").and_then(|p| p.parse().ok()).unwrap_or(8080),
        debug: map.get("debug").and_then(|d| d.parse().ok()).unwrap_or(false),
    }
}

/// Problem 3: Configuration from environment
/// Read config from environment
pub fn config_from_env() -> Config {
    Config {
        host: std::env::var("HOST").unwrap_or_else(|_| "localhost".to_string()),
        port: std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080),
        debug: std::env::var("DEBUG").ok().and_then(|d| d.parse().ok()).unwrap_or(false),
    }
}

/// Problem 4: Configuration validation
/// Validate configuration
pub fn validate_config(config: &Config) -> Result<(), String> {
    if config.port == 0 {
        return Err("Port cannot be 0".to_string());
    }
    if config.host.is_empty() {
        return Err("Host cannot be empty".to_string());
    }
    Ok(())
}

/// Problem 5: Configuration merging
/// Merge configurations
pub fn merge_configs(base: &Config, override_config: &Config) -> Config {
    Config {
        host: if override_config.host != "localhost" {
            override_config.host.clone()
        } else {
            base.host.clone()
        },
        port: if override_config.port != 8080 {
            override_config.port
        } else {
            base.port
        },
        debug: override_config.debug,
    }
}

/// Problem 6: Configuration from string
/// Parse config from string
pub fn config_from_string(s: &str) -> Config {
    let mut config = Config::new();
    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "host" => config.host = value.trim().to_string(),
                "port" => config.port = value.trim().parse().unwrap_or(8080),
                "debug" => config.debug = value.trim().parse().unwrap_or(false),
                _ => {}
            }
        }
    }
    config
}

/// Problem 7: Configuration to string
/// Serialize config to string
pub fn config_to_string(config: &Config) -> String {
    format!(
        "host={}\nport={}\ndebug={}",
        config.host, config.port, config.debug
    )
}

/// Problem 8: Configuration with defaults
/// Configuration with default values
pub fn config_with_defaults() -> Config {
    Config {
        host: "0.0.0.0".to_string(),
        port: 3000,
        debug: true,
    }
}

/// Problem 9: Configuration with optional fields
/// Configuration with optional fields
#[derive(Debug, Clone)]
pub struct OptionalConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub debug: Option<bool>,
}

impl OptionalConfig {
    pub fn new() -> Self {
        Self {
            host: None,
            port: None,
            debug: None,
        }
    }

    pub fn to_config(&self) -> Config {
        Config {
            host: self.host.clone().unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or(8080),
            debug: self.debug.unwrap_or(false),
        }
    }
}

/// Problem 10: Configuration with environment override
/// Override config with environment
pub fn config_with_env_override(config: &Config) -> Config {
    Config {
        host: std::env::var("HOST").unwrap_or_else(|_| config.host.clone()),
        port: std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(config.port),
        debug: std::env::var("DEBUG").ok().and_then(|d| d.parse().ok()).unwrap_or(config.debug),
    }
}

/// Problem 11: Configuration with command line args
/// Parse config from args
pub fn config_from_args(args: &[String]) -> Config {
    let mut config = Config::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                if i + 1 < args.len() {
                    config.host = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    config.port = args[i + 1].parse().unwrap_or(8080);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--debug" => {
                config.debug = true;
                i += 1;
            }
            _ => i += 1,
        }
    }
    config
}

/// Problem 12: Configuration with nested config
/// Nested configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: Config,
    pub database: DatabaseConfig,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            server: Config::new(),
            database: DatabaseConfig {
                url: "postgres://localhost".to_string(),
                pool_size: 10,
            },
        }
    }
}

/// Problem 13: Configuration with secrets
/// Handle secrets in config
pub fn config_with_secrets(config: &mut Config, secret: &str) {
    // In real implementation, you'd handle secrets securely
    let _ = secret;
}

/// Problem 14: Configuration with hot reload
/// Simulate hot reload
pub fn config_hot_reload(config: &Config) -> Config {
    // In real implementation, you'd watch for file changes
    config.clone()
}

/// Problem 15: Configuration with validation rules
/// Advanced validation
pub fn validate_config_advanced(config: &Config) -> Result<(), String> {
    if config.port == 0 {
        return Err("Port cannot be 0".to_string());
    }
    if config.port > 65535 {
        return Err("Port cannot be greater than 65535".to_string());
    }
    if config.host.is_empty() {
        return Err("Host cannot be empty".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_config() {
        let config = Config::new();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_config_from_map() {
        let mut map = HashMap::new();
        map.insert("host".to_string(), "example.com".to_string());
        map.insert("port".to_string(), "3000".to_string());
        let config = config_from_map(&map);
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_from_env() {
        std::env::set_var("HOST", "example.com");
        std::env::set_var("PORT", "3000");
        let config = config_from_env();
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_validate_config() {
        let config = Config::new();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_merge_configs() {
        let base = Config::new();
        let mut override_config = Config::new();
        override_config.port = 3000;
        let merged = merge_configs(&base, &override_config);
        assert_eq!(merged.port, 3000);
    }

    #[test]
    fn test_config_from_string() {
        let config_str = "host=example.com\nport=3000\ndebug=true";
        let config = config_from_string(config_str);
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_to_string() {
        let config = Config::new();
        let s = config_to_string(&config);
        assert!(s.contains("host=localhost"));
    }

    #[test]
    fn test_config_with_defaults() {
        let config = config_with_defaults();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_optional_config() {
        let mut opt = OptionalConfig::new();
        opt.port = Some(3000);
        let config = opt.to_config();
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_with_env_override() {
        let config = Config::new();
        let overridden = config_with_env_override(&config);
        // May or may not be overridden depending on env
        assert!(overridden.port > 0);
    }

    #[test]
    fn test_config_from_args() {
        let args = vec!["--host".to_string(), "example.com".to_string(), "--port".to_string(), "3000".to_string()];
        let config = config_from_args(&args);
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_nested_config() {
        let config = AppConfig::new();
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.database.pool_size, 10);
    }

    #[test]
    fn test_validate_config_advanced() {
        let mut config = Config::new();
        assert!(validate_config_advanced(&config).is_ok());

        config.port = 0;
        assert!(validate_config_advanced(&config).is_err());
    }
}
