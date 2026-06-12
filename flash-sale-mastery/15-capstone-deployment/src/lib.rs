//! Capstone Deployment Module
//!
//! Provides deployment configuration and orchestration for the flash sale system.
//! This module ties together all preceding modules into a deployable unit.

use serde::{Deserialize, Serialize};

/// Configuration for the flash sale deployment, loaded from environment variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// Redis connection URL (e.g., redis://localhost:6379)
    pub redis_url: String,
    /// PostgreSQL connection URL
    pub database_url: String,
    /// Log level (trace, debug, info, warn, error)
    pub rust_log: String,
    /// HTTP listen address
    pub listen_addr: String,
}

impl DeploymentConfig {
    /// Load configuration from environment variables with sensible defaults.
    pub fn from_env() -> Self {
        Self {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://flash_sale:flash_sale_dev@localhost/flash_sale".to_string()),
            rust_log: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
            listen_addr: std::env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
        }
    }
}

/// The mode in which the binary should operate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Run as the HTTP API server.
    Api,
    /// Run as the background order worker.
    Worker,
    /// Run a health check probe (exit 0 on success, non-zero on failure).
    HealthCheck,
}

impl RunMode {
    /// Parse a run mode from CLI argument string.
    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg {
            "api" => Some(Self::Api),
            "worker" => Some(Self::Worker),
            "--health-check" => Some(Self::HealthCheck),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_mode_from_arg() {
        assert_eq!(RunMode::from_arg("api"), Some(RunMode::Api));
        assert_eq!(RunMode::from_arg("worker"), Some(RunMode::Worker));
        assert_eq!(RunMode::from_arg("--health-check"), Some(RunMode::HealthCheck));
        assert_eq!(RunMode::from_arg("unknown"), None);
    }

    #[test]
    fn test_default_config() {
        // Clear any env vars that might be set
        // SAFETY: tests run single-threaded, no concurrent env access
        unsafe {
            std::env::remove_var("REDIS_URL");
            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("RUST_LOG");
            std::env::remove_var("LISTEN_ADDR");
        }

        let config = DeploymentConfig::from_env();
        assert_eq!(config.redis_url, "redis://localhost:6379");
        assert_eq!(config.listen_addr, "0.0.0.0:3000");
    }
}
