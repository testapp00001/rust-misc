//! # Configuration
//!
//! Application configuration loaded from environment variables with sensible defaults.

use serde::{Deserialize, Serialize};

/// Per-account rate limit settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests allowed within the window.
    pub max_requests: u64,
    /// Sliding window duration in seconds.
    pub window_seconds: u64,
}

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Redis connection URL.
    pub redis_url: String,
    /// Database connection URL.
    pub db_url: String,
    /// Identifier for the current flash sale.
    pub sale_id: String,
    /// Maximum stock units per product.
    pub max_stock_per_product: u64,
    /// Maximum vouchers that can be issued per product.
    pub max_vouchers_per_product: u64,
    /// Rate limiting parameters.
    pub rate_limit_config: RateLimitConfig,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Falls back to sensible defaults when variables are not set.
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            db_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite::memory:".to_string()),
            sale_id: std::env::var("SALE_ID")
                .unwrap_or_else(|_| "flash-sale-2024".to_string()),
            max_stock_per_product: std::env::var("MAX_STOCK_PER_PRODUCT")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()?,
            max_vouchers_per_product: std::env::var("MAX_VOUCHERS_PER_PRODUCT")
                .unwrap_or_else(|_| "500".to_string())
                .parse()?,
            rate_limit_config: RateLimitConfig {
                max_requests: std::env::var("RATE_LIMIT_MAX_REQUESTS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                window_seconds: std::env::var("RATE_LIMIT_WINDOW_SECONDS")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()?,
            },
        })
    }

    /// Create a default configuration suitable for tests.
    pub fn default_test() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            db_url: "sqlite::memory:".to_string(),
            sale_id: "test-sale".to_string(),
            max_stock_per_product: 100,
            max_vouchers_per_product: 50,
            rate_limit_config: RateLimitConfig {
                max_requests: 10,
                window_seconds: 60,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_test_config() {
        let config = Config::default_test();
        assert_eq!(config.sale_id, "test-sale");
        assert_eq!(config.max_stock_per_product, 100);
        assert_eq!(config.max_vouchers_per_product, 50);
        assert_eq!(config.rate_limit_config.max_requests, 10);
        assert_eq!(config.rate_limit_config.window_seconds, 60);
    }

    #[test]
    fn test_from_env_defaults() {
        // SAFETY: Tests run single-threaded and we control the env vars.
        unsafe {
            std::env::remove_var("REDIS_URL");
            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("SALE_ID");
            std::env::remove_var("MAX_STOCK_PER_PRODUCT");
            std::env::remove_var("MAX_VOUCHERS_PER_PRODUCT");
            std::env::remove_var("RATE_LIMIT_MAX_REQUESTS");
            std::env::remove_var("RATE_LIMIT_WINDOW_SECONDS");
        }

        let config = Config::from_env().expect("config from env with defaults");
        assert_eq!(config.redis_url, "redis://127.0.0.1:6379");
        assert_eq!(config.sale_id, "flash-sale-2024");
        assert_eq!(config.max_stock_per_product, 1000);
    }

    #[test]
    fn test_from_env_custom() {
        // SAFETY: Tests run single-threaded and we control the env vars.
        unsafe {
            std::env::set_var("SALE_ID", "black-friday-2025");
            std::env::set_var("MAX_STOCK_PER_PRODUCT", "5000");
        }

        let config = Config::from_env().expect("config from env with custom vars");
        assert_eq!(config.sale_id, "black-friday-2025");
        assert_eq!(config.max_stock_per_product, 5000);

        // Clean up
        // SAFETY: Tests run single-threaded and we control the env vars.
        unsafe {
            std::env::remove_var("SALE_ID");
            std::env::remove_var("MAX_STOCK_PER_PRODUCT");
        }
    }
}
