//! # Exercise 04: Redis Set Operations
//!
//! ## Learning Objective
//! Master Redis SET commands (SADD, SREM, SISMEMBER, SMEMBERS, SCARD) for
//! tracking unique claims in a flash sale system.
//!
//! ## Flash Sale Context
//! In a flash sale, each account can only claim one unit per product. A Redis
//! SET keyed by `claims:{product_id}` stores the set of account IDs that have
//! already claimed. Before allowing a purchase, we check SISMEMBER. After
//! purchase, we SADD the account ID. The SET guarantees uniqueness with O(1)
//! lookups.
//!
//! ## Instructions
//! 1. Implement `add_claim` to record that an account claimed a product
//! 2. Implement `has_claimed` to check if an account already claimed
//! 3. Implement `remove_claim` to revoke a claim (e.g., order cancelled)
//! 4. Implement `get_claim_count` to get total unique claims (SCARD)
//! 5. Implement `get_all_claims` to retrieve all claiming account IDs
//!
//! ## Hints
//! - SADD returns 1 if added, 0 if already present
//! - SISMEMBER returns 1 if member exists, 0 otherwise
//! - SMEMBERS returns all members as a list

use deadpool_redis::Connection as RedisConnection;

/// Error type for set operations.
#[derive(Debug, thiserror::Error)]
pub enum ClaimError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

    #[error("Claim already exists for account {account_id} on product {product_id}")]
    DuplicateClaim {
        product_id: String,
        account_id: String,
    },
}

/// Record that an account has claimed a product.
/// Returns true if this is a new claim, false if already claimed.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
/// * `account_id` - Account identifier
pub async fn add_claim(
    conn: &mut RedisConnection,
    product_id: &str,
    account_id: &str,
) -> Result<bool, ClaimError> {
    // TODO: Use SADD to add account_id to the set "claims:{product_id}"
    // TODO: Return true if newly added (1), false if already existed (0)
    todo!("Implement add_claim")
}

/// Check if an account has already claimed a product.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
/// * `account_id` - Account identifier
pub async fn has_claimed(
    conn: &mut RedisConnection,
    product_id: &str,
    account_id: &str,
) -> Result<bool, ClaimError> {
    // TODO: Use SISMEMBER to check membership
    todo!("Implement has_claimed")
}

/// Remove a claim (e.g., when an order is cancelled).
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
/// * `account_id` - Account to remove
pub async fn remove_claim(
    conn: &mut RedisConnection,
    product_id: &str,
    account_id: &str,
) -> Result<bool, ClaimError> {
    // TODO: Use SREM to remove the member
    // TODO: Return true if removed, false if wasn't in set
    todo!("Implement remove_claim")
}

/// Get the total number of unique claims for a product.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
pub async fn get_claim_count(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<usize, ClaimError> {
    // TODO: Use SCARD to get set cardinality
    todo!("Implement get_claim_count")
}

/// Get all account IDs that have claimed a product.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
pub async fn get_all_claims(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<Vec<String>, ClaimError> {
    // TODO: Use SMEMBERS to get all set members
    todo!("Implement get_all_claims")
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
        let is_new = add_claim(&mut conn, &product, "user:100")
            .await
            .expect("add failed");
        assert!(is_new, "First claim should be new");
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
        let is_new = add_claim(&mut conn, &product, "user:100")
            .await
            .expect("add failed");
        assert!(!is_new, "Duplicate claim should return false");
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
            add_claim(&mut conn, &product, &format!("user:{i}"))
                .await
                .unwrap();
        }
        let count = get_claim_count(&mut conn, &product).await.unwrap();
        assert_eq!(count, 5);
        let members = get_all_claims(&mut conn, &product).await.unwrap();
        assert_eq!(members.len(), 5);
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
        let removed = remove_claim(&mut conn, &product, "user:100")
            .await
            .expect("remove failed");
        assert!(removed, "Should have removed existing claim");
        assert!(!has_claimed(&mut conn, &product, "user:100").await.unwrap());
        let removed = remove_claim(&mut conn, &product, "user:999")
            .await
            .expect("remove failed");
        assert!(!removed, "Removing non-existent should return false");
    }
}
