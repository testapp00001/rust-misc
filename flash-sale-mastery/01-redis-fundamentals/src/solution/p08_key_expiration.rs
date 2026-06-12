//! # Solution 08: Redis Key Expiration
//!
//! Complete implementation of Redis key expiration management.

use deadpool_redis::redis::RedisError;
use deadpool_redis::Connection as RedisConnection;

/// Error type for expiration operations.
#[derive(Debug, thiserror::Error)]
pub enum ExpiryError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Key does not exist: {key}")]
    KeyNotFound { key: String },

    #[error("Invalid TTL value: {0}")]
    InvalidTtl(String),
}

/// Set a key with an expiration in seconds.
pub async fn set_with_expiry(
    conn: &mut RedisConnection,
    key: &str,
    value: &str,
    ttl_secs: u64,
) -> Result<(), ExpiryError> {
    deadpool_redis::redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(ttl_secs)
        .query_async::<()>(&mut *conn)
        .await?;
    Ok(())
}

/// Get the remaining TTL for a key in seconds.
pub async fn get_remaining_ttl(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<i64, ExpiryError> {
    let ttl: i64 = deadpool_redis::redis::cmd("TTL")
        .arg(key)
        .query_async(&mut *conn)
        .await?;
    Ok(ttl)
}

/// Refresh a key's TTL only if the key already exists.
pub async fn refresh_if_exists(
    conn: &mut RedisConnection,
    key: &str,
    new_ttl_secs: u64,
) -> Result<bool, ExpiryError> {
    // Check existence first
    let exists: bool = deadpool_redis::redis::cmd("EXISTS")
        .arg(key)
        .query_async(&mut *conn)
        .await?;
    if !exists {
        return Ok(false);
    }
    let result: bool = deadpool_redis::redis::cmd("EXPIRE")
        .arg(key)
        .arg(new_ttl_secs)
        .query_async(&mut *conn)
        .await?;
    Ok(result)
}

/// Set a key with a millisecond-precision expiration.
pub async fn set_with_ms_expiry(
    conn: &mut RedisConnection,
    key: &str,
    value: &str,
    ttl_ms: u64,
) -> Result<(), ExpiryError> {
    deadpool_redis::redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("PX")
        .arg(ttl_ms)
        .query_async::<()>(&mut *conn)
        .await?;
    Ok(())
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
        format!("test:expiry:{}:{}", suffix, std::process::id())
    }

    #[tokio::test]
    async fn test_set_with_expiry() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("ttl");
        set_with_expiry(&mut conn, &key, "flash_sale_token", 60)
            .await
            .expect("set failed");
        let ttl = get_remaining_ttl(&mut conn, &key).await.unwrap();
        assert!(ttl > 0 && ttl <= 60, "TTL should be between 0 and 60, got: {ttl}");
    }

    #[tokio::test]
    async fn test_ttl_nonexistent_key() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("nonexistent_ttl");
        let ttl = get_remaining_ttl(&mut conn, &key).await.unwrap();
        assert_eq!(ttl, -2);
    }

    #[tokio::test]
    async fn test_refresh_if_exists() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("refresh");
        set_with_expiry(&mut conn, &key, "value", 10).await.unwrap();
        assert!(refresh_if_exists(&mut conn, &key, 300).await.unwrap());
        let ttl = get_remaining_ttl(&mut conn, &key).await.unwrap();
        assert!(ttl > 10, "TTL should be extended beyond original 10s");
        let key2 = test_key("refresh_missing");
        assert!(!refresh_if_exists(&mut conn, &key2, 300).await.unwrap());
    }

    #[tokio::test]
    async fn test_set_with_ms_expiry() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("ms_ttl");
        set_with_ms_expiry(&mut conn, &key, "ephemeral", 60000).await.unwrap();
        let ttl = get_remaining_ttl(&mut conn, &key).await.unwrap();
        assert!(ttl > 0 && ttl <= 61);
    }
}
