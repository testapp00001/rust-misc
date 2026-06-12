//! # Solution 04: Redis Set Operations
//!
//! Complete implementation of Redis SET operations for flash sale claim tracking.

use deadpool_redis::redis::RedisError;
use deadpool_redis::Connection as RedisConnection;

/// Error type for set operations.
#[derive(Debug, thiserror::Error)]
pub enum ClaimError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Claim already exists for account {account_id} on product {product_id}")]
    DuplicateClaim {
        product_id: String,
        account_id: String,
    },
}

fn claims_key(product_id: &str) -> String {
    format!("claims:{product_id}")
}

/// Record that an account has claimed a product. Returns true if new claim.
pub async fn add_claim(
    conn: &mut RedisConnection,
    product_id: &str,
    account_id: &str,
) -> Result<bool, ClaimError> {
    let key = claims_key(product_id);
    let added: i64 = deadpool_redis::redis::cmd("SADD")
        .arg(&key)
        .arg(account_id)
        .query_async(&mut *conn)
        .await?;
    Ok(added == 1)
}

/// Check if an account has already claimed a product.
pub async fn has_claimed(
    conn: &mut RedisConnection,
    product_id: &str,
    account_id: &str,
) -> Result<bool, ClaimError> {
    let key = claims_key(product_id);
    let is_member: i64 = deadpool_redis::redis::cmd("SISMEMBER")
        .arg(&key)
        .arg(account_id)
        .query_async(&mut *conn)
        .await?;
    Ok(is_member == 1)
}

/// Remove a claim. Returns true if removed, false if wasn't in set.
pub async fn remove_claim(
    conn: &mut RedisConnection,
    product_id: &str,
    account_id: &str,
) -> Result<bool, ClaimError> {
    let key = claims_key(product_id);
    let removed: i64 = deadpool_redis::redis::cmd("SREM")
        .arg(&key)
        .arg(account_id)
        .query_async(&mut *conn)
        .await?;
    Ok(removed == 1)
}

/// Get the total number of unique claims for a product.
pub async fn get_claim_count(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<usize, ClaimError> {
    let key = claims_key(product_id);
    let count: usize = deadpool_redis::redis::cmd("SCARD")
        .arg(&key)
        .query_async(&mut *conn)
        .await?;
    Ok(count)
}

/// Get all account IDs that have claimed a product.
pub async fn get_all_claims(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<Vec<String>, ClaimError> {
    let key = claims_key(product_id);
    let members: Vec<String> = deadpool_redis::redis::cmd("SMEMBERS")
        .arg(&key)
        .query_async(&mut *conn)
        .await?;
    Ok(members)
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
        format!("test:claims:{}:{}", suffix, std::process::id())
    }

    #[tokio::test]
    async fn test_add_and_check_claim() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let product = test_key("basic");
        let is_new = add_claim(&mut conn, &product, "user:100").await.unwrap();
        assert!(is_new);
        assert!(has_claimed(&mut conn, &product, "user:100").await.unwrap());
        assert!(!has_claimed(&mut conn, &product, "user:200").await.unwrap());
    }

    #[tokio::test]
    async fn test_duplicate_claim_returns_false() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let product = test_key("dup");
        add_claim(&mut conn, &product, "user:100").await.unwrap();
        let is_new = add_claim(&mut conn, &product, "user:100").await.unwrap();
        assert!(!is_new);
    }

    #[tokio::test]
    async fn test_claim_count_and_members() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let product = test_key("count");
        for i in 0..5 {
            add_claim(&mut conn, &product, &format!("user:{i}")).await.unwrap();
        }
        assert_eq!(get_claim_count(&mut conn, &product).await.unwrap(), 5);
        assert_eq!(get_all_claims(&mut conn, &product).await.unwrap().len(), 5);
    }

    #[tokio::test]
    async fn test_remove_claim() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let product = test_key("remove");
        add_claim(&mut conn, &product, "user:100").await.unwrap();
        assert!(remove_claim(&mut conn, &product, "user:100").await.unwrap());
        assert!(!has_claimed(&mut conn, &product, "user:100").await.unwrap());
        assert!(!remove_claim(&mut conn, &product, "user:999").await.unwrap());
    }
}
