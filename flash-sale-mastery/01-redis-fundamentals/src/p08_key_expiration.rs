//! # Exercise 08: Redis Key Expiration
//!
//! ## Learning Objective
//! Learn to manage key lifetimes using EXPIRE, TTL, PTTL, and SET with EX/PX
//! options for ephemeral data in a flash sale system.
//!
//! ## Flash Sale Context
//! Flash sale sessions are time-bound: a sale starts at a specific time and
//! ends after a duration. Keys for temporary data (session tokens, captcha
//! challenges, rate limit windows) need automatic cleanup. Redis key expiration
//! handles this without requiring a separate cleanup process.
//!
//! ## Instructions
//! 1. Implement `set_with_expiry` to store a value with a TTL in seconds
//! 2. Implement `get_remaining_ttl` to check how long a key has left
//! 3. Implement `refresh_if_exists` to extend TTL only if the key exists
//! 4. Implement `set_with_ms_expiry` for sub-second TTLs (e.g., idempotency tokens)
//!
//! ## Hints
//! - SET key value EX seconds -- set with expiry in one command
//! - TTL returns -1 (no expiry), -2 (key missing), or seconds remaining
//! - PTTL returns milliseconds remaining
//! - EXPIRE returns 1 if set, 0 if key doesn't exist

use deadpool_redis::Connection as RedisConnection;

/// Error type for expiration operations.
#[derive(Debug, thiserror::Error)]
pub enum ExpiryError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

    #[error("Key does not exist: {key}")]
    KeyNotFound { key: String },

    #[error("Invalid TTL value: {0}")]
    InvalidTtl(String),
}

/// Set a key with an expiration in seconds.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Key to set
/// * `value` - Value to store
/// * `ttl_secs` - Time-to-live in seconds
pub async fn set_with_expiry(
    conn: &mut RedisConnection,
    key: &str,
    value: &str,
    ttl_secs: u64,
) -> Result<(), ExpiryError> {
    // TODO: Use SET with EX option to set key with expiry
    todo!("Implement set_with_expiry")
}

/// Get the remaining TTL for a key in seconds.
/// Returns -2 if key doesn't exist, -1 if no expiry set.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Key to check
pub async fn get_remaining_ttl(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<i64, ExpiryError> {
    // TODO: Use TTL command
    todo!("Implement get_remaining_ttl")
}

/// Refresh a key's TTL only if the key already exists.
/// Returns true if TTL was refreshed, false if key doesn't exist.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Key to refresh
/// * `new_ttl_secs` - New TTL in seconds
pub async fn refresh_if_exists(
    conn: &mut RedisConnection,
    key: &str,
    new_ttl_secs: u64,
) -> Result<bool, ExpiryError> {
    // TODO: Use EXPIRE with NX option (only set if no expiry exists)
    // Or check existence first, then EXPIRE
    todo!("Implement refresh_if_exists")
}

/// Set a key with a millisecond-precision expiration.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Key to set
/// * `value` - Value to store
/// * `ttl_ms` - Time-to-live in milliseconds
pub async fn set_with_ms_expiry(
    conn: &mut RedisConnection,
    key: &str,
    value: &str,
    ttl_ms: u64,
) -> Result<(), ExpiryError> {
    // TODO: Use SET with PX option for millisecond expiry
    todo!("Implement set_with_ms_expiry")
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
        let ttl = get_remaining_ttl(&mut conn, &key)
            .await
            .expect("ttl failed");
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
        let ttl = get_remaining_ttl(&mut conn, &key)
            .await
            .expect("ttl failed");
        assert_eq!(ttl, -2, "Non-existent key should return -2");
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
        set_with_expiry(&mut conn, &key, "value", 10)
            .await
            .expect("set failed");
        let refreshed = refresh_if_exists(&mut conn, &key, 300)
            .await
            .expect("refresh failed");
        assert!(refreshed, "Should refresh existing key");
        let ttl = get_remaining_ttl(&mut conn, &key)
            .await
            .expect("ttl failed");
        assert!(ttl > 10, "TTL should be extended beyond original 10s");

        // Try refreshing non-existent key
        let key2 = test_key("refresh_missing");
        let refreshed = refresh_if_exists(&mut conn, &key2, 300)
            .await
            .expect("refresh failed");
        assert!(!refreshed, "Should not refresh non-existent key");
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
        set_with_ms_expiry(&mut conn, &key, "ephemeral", 60000)
            .await
            .expect("set failed");
        let ttl = get_remaining_ttl(&mut conn, &key)
            .await
            .expect("ttl failed");
        assert!(ttl > 0 && ttl <= 61, "TTL should reflect ~60 seconds");
    }
}
