//! # Exercise 09: Redis Error Handling
//!
//! ## Learning Objective
//! Build robust error handling for Redis operations using custom error types,
//! error classification, and recovery strategies.
//!
//! ## Flash Sale Context
//! In a flash sale, Redis errors can cascade into user-facing failures. A
//! connection timeout should trigger a retry, not a 500 error. A type error
//! (wrong data in a key) should be logged and alerted, not silently swallowed.
//! Proper error classification lets the system respond differently to transient
//! vs. permanent failures.
//!
//! ## Instructions
//! 1. Define a `RedisOperationError` enum with `thiserror` covering common failure modes
//! 2. Implement `classify_error` to categorize errors by severity/retryability
//! 3. Implement `safe_get` that wraps a GET with proper error handling
//! 4. Implement `safe_get_or_default` that never fails (uses fallback)
//!
//! ## Hints
//! - `deadpool_redis::redis::RedisError` has `.kind()` for classification
//! - `RedisErrorKind` includes `IoError`, `TypeError`, `Timeout`, etc.
//! - Connection pool errors (`deadpool_redis::PoolError`) are separate from Redis errors

use deadpool_redis::Connection as RedisConnection;

/// Classification of Redis errors for recovery decisions.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// Transient error that may succeed on retry (connection drop, timeout).
    Transient,
    /// Permanent error that won't succeed on retry (type mismatch, wrong key type).
    Permanent,
    /// Configuration or setup error (bad URL, pool creation failed).
    Configuration,
    /// Unknown error, treat as permanent.
    Unknown,
}

/// Custom error type for Redis operations with classification.
#[derive(Debug, thiserror::Error)]
pub enum RedisOperationError {
    #[error("Redis protocol error: {0}")]
    Protocol(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Type error: expected {expected}, got {actual}")]
    TypeError { expected: String, actual: String },

    #[error("Key not found: {key}")]
    KeyNotFound { key: String },

    #[error("Pool error: {0}")]
    Pool(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl RedisOperationError {
    /// Classify the error severity for recovery decisions.
    pub fn severity(&self) -> ErrorSeverity {
        // TODO: Classify each variant
        // Connection, Timeout -> Transient
        // TypeError, KeyNotFound -> Permanent
        // Pool -> Transient or Configuration depending on context
        // Protocol -> Permanent
        // Serialization -> Permanent
        todo!("Implement severity classification")
    }

    /// Whether this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self.severity(), ErrorSeverity::Transient)
    }
}

/// Classify a raw Redis error into our custom error type.
pub fn classify_error(err: &deadpool_redis::redis::RedisError) -> RedisOperationError {
    // TODO: Match on err.kind() to create appropriate variant
    // RedisErrorKind::IoError -> Connection
    // RedisErrorKind::TypeError -> TypeError
    // RedisErrorKind::Timeout -> Timeout (if available)
    // _ -> Protocol
    todo!("Implement error classification")
}

/// Safely get a value from Redis, converting errors to our custom type.
pub async fn safe_get(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<Option<String>, RedisOperationError> {
    // TODO: Execute GET and map errors using classify_error
    todo!("Implement safe_get")
}

/// Get a value from Redis with a default fallback. Never returns an error.
pub async fn safe_get_or_default(
    conn: &mut RedisConnection,
    key: &str,
    default: &str,
) -> String {
    // TODO: Call safe_get, return default on any error
    todo!("Implement safe_get_or_default")
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn get_conn() -> Option<RedisConnection> {
        let cfg = Config::from_url(REDIS_URL);
        let pool = cfg.builder().ok()?.build().ok()?;
        pool.get().await.ok()
    }

    fn test_key(suffix: &str) -> String {
        format!("test:error:{}:{}", suffix, std::process::id())
    }

    #[test]
    fn test_error_severity_classification() {
        let conn_err = RedisOperationError::Connection("refused".to_string());
        assert_eq!(conn_err.severity(), ErrorSeverity::Transient);
        assert!(conn_err.is_retryable());

        let type_err = RedisOperationError::TypeError {
            expected: "string".to_string(),
            actual: "list".to_string(),
        };
        assert_eq!(type_err.severity(), ErrorSeverity::Permanent);
        assert!(!type_err.is_retryable());

        let timeout_err = RedisOperationError::Timeout("3s".to_string());
        assert_eq!(timeout_err.severity(), ErrorSeverity::Transient);
        assert!(timeout_err.is_retryable());
    }

    #[tokio::test]
    async fn test_safe_get_existing_key() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("safe_get");
        deadpool_redis::redis::cmd("SET")
            .arg(&key)
            .arg("hello")
            .query_async::<()>(&mut *conn)
            .await
            .unwrap();
        let val = safe_get(&mut conn, &key)
            .await
            .expect("safe_get should succeed");
        assert_eq!(val.as_deref(), Some("hello"));
    }

    #[tokio::test]
    async fn test_safe_get_missing_key() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("safe_get_missing");
        let val = safe_get(&mut conn, &key)
            .await
            .expect("safe_get should succeed for missing key");
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_safe_get_or_default() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("default");
        let val = safe_get_or_default(&mut conn, &key, "fallback").await;
        assert_eq!(val, "fallback", "Should return default for missing key");

        deadpool_redis::redis::cmd("SET")
            .arg(&key)
            .arg("real")
            .query_async::<()>(&mut *conn)
            .await
            .unwrap();
        let val = safe_get_or_default(&mut conn, &key, "fallback").await;
        assert_eq!(val, "real", "Should return actual value when key exists");
    }
}
