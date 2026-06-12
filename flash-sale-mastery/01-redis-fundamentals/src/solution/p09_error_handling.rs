//! # Solution 09: Redis Error Handling
//!
//! Complete implementation of Redis error classification and recovery patterns.

use deadpool_redis::redis::RedisError;
use deadpool_redis::Connection as RedisConnection;

/// Classification of Redis errors for recovery decisions.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Transient,
    Permanent,
    Configuration,
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
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            RedisOperationError::Connection(_) => ErrorSeverity::Transient,
            RedisOperationError::Timeout(_) => ErrorSeverity::Transient,
            RedisOperationError::Pool(_) => ErrorSeverity::Transient,
            RedisOperationError::TypeError { .. } => ErrorSeverity::Permanent,
            RedisOperationError::KeyNotFound { .. } => ErrorSeverity::Permanent,
            RedisOperationError::Protocol(_) => ErrorSeverity::Permanent,
            RedisOperationError::Serialization(_) => ErrorSeverity::Permanent,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self.severity(), ErrorSeverity::Transient)
    }
}

/// Classify a raw Redis error into our custom error type.
pub fn classify_error(err: &RedisError) -> RedisOperationError {
    let kind = err.kind();
    match kind {
        deadpool_redis::redis::ErrorKind::IoError => {
            RedisOperationError::Connection(err.to_string())
        }
        deadpool_redis::redis::ErrorKind::TypeError => {
            RedisOperationError::TypeError {
                expected: "unknown".to_string(),
                actual: err.to_string(),
            }
        }
        deadpool_redis::redis::ErrorKind::ExecAbortError
        | deadpool_redis::redis::ErrorKind::NoScriptError => {
            RedisOperationError::Protocol(err.to_string())
        }
        deadpool_redis::redis::ErrorKind::BusyLoadingError => {
            RedisOperationError::Connection(err.to_string())
        }
        deadpool_redis::redis::ErrorKind::InvalidClientConfig => {
            RedisOperationError::Pool(err.to_string())
        }
        deadpool_redis::redis::ErrorKind::Moved
        | deadpool_redis::redis::ErrorKind::Ask => {
            RedisOperationError::Connection(err.to_string())
        }
        _ => RedisOperationError::Protocol(err.to_string()),
    }
}

/// Safely get a value from Redis.
pub async fn safe_get(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<Option<String>, RedisOperationError> {
    let result: Result<Option<String>, RedisError> = deadpool_redis::redis::cmd("GET")
        .arg(key)
        .query_async(&mut *conn)
        .await;
    result.map_err(|e| classify_error(&e))
}

/// Get a value from Redis with a default fallback. Never returns an error.
pub async fn safe_get_or_default(
    conn: &mut RedisConnection,
    key: &str,
    default: &str,
) -> String {
    safe_get(conn, key)
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::Config;

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
        let val = safe_get(&mut conn, &key).await.unwrap();
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
        let val = safe_get(&mut conn, &key).await.unwrap();
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
        assert_eq!(val, "fallback");
        deadpool_redis::redis::cmd("SET")
            .arg(&key)
            .arg("real")
            .query_async::<()>(&mut *conn)
            .await
            .unwrap();
        let val = safe_get_or_default(&mut conn, &key, "fallback").await;
        assert_eq!(val, "real");
    }
}
