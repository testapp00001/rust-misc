//! # Exercise 03: Redis Hash Operations
//!
//! ## Learning Objective
//! Master Redis HASH commands (HSET, HGET, HGETALL, HINCRBY, HDEL) for storing
//! structured product data in a flash sale system.
//!
//! ## Flash Sale Context
//! A product in a flash sale has multiple attributes: name, price, discount,
//! remaining vouchers by type, and metadata. Instead of using separate keys
//! for each attribute, a Redis HASH groups them under a single key. This is
//! memory-efficient and allows atomic multi-field updates.
//!
//! ## Instructions
//! 1. Implement `set_product_details` to store a product's fields as a hash
//! 2. Implement `get_product_details` to retrieve all fields of a product
//! 3. Implement `get_product_field` to retrieve a single field
//! 4. Implement `increment_voucher_count` to atomically bump a voucher counter
//! 5. Implement `delete_product_field` to remove a specific field
//!
//! ## Hints
//! - `redis::cmd("HSET")` accepts alternating field/value args
//! - `HGETALL` returns a HashMap or similar collection
//! - `HINCRBY` atomically increments a hash field's integer value

use deadpool_redis::Connection as RedisConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error type for hash operations.
#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

    #[error("Field not found: {field}")]
    FieldNotFound { field: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Product details stored as a Redis hash.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductDetails {
    pub name: String,
    pub price_cents: i64,
    pub discount_percent: i64,
    pub description: String,
}

/// Store all fields of a product as a Redis hash.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Hash key (e.g., "product:1001")
/// * `details` - Product details to store
pub async fn set_product_details(
    conn: &mut RedisConnection,
    product_id: &str,
    details: &ProductDetails,
) -> Result<(), HashError> {
    // TODO: Use HSET with each field of ProductDetails
    // Hint: You can use multiple .arg() calls or serialize individual fields
    todo!("Implement set_product_details")
}

/// Retrieve all fields of a product hash.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Hash key
pub async fn get_product_details(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<ProductDetails, HashError> {
    // TODO: Use HGETALL to retrieve all fields
    // TODO: Parse the HashMap into ProductDetails
    todo!("Implement get_product_details")
}

/// Retrieve a single field from a product hash.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Hash key
/// * `field` - Field name to retrieve
pub async fn get_product_field(
    conn: &mut RedisConnection,
    product_id: &str,
    field: &str,
) -> Result<String, HashError> {
    // TODO: Use HGET to retrieve a single field
    // TODO: Return FieldNotFound error if the field doesn't exist
    todo!("Implement get_product_field")
}

/// Atomically increment a voucher count for a product.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Hash key
/// * `voucher_type` - Field name (e.g., "early_bird", "loyalty")
/// * `increment` - Amount to add (can be negative)
pub async fn increment_voucher_count(
    conn: &mut RedisConnection,
    product_id: &str,
    voucher_type: &str,
    increment: i64,
) -> Result<i64, HashError> {
    // TODO: Use HINCRBY to atomically increment the field
    todo!("Implement increment_voucher_count")
}

/// Delete a specific field from a product hash.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Hash key
/// * `field` - Field to delete
pub async fn delete_product_field(
    conn: &mut RedisConnection,
    product_id: &str,
    field: &str,
) -> Result<bool, HashError> {
    // TODO: Use HDEL to remove the field
    // TODO: Return true if field existed, false otherwise
    todo!("Implement delete_product_field")
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
        format!("test:hash:{}:{}", suffix, std::process::id())
    }

    #[tokio::test]
    async fn test_set_and_get_product_details() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("details");
        let details = ProductDetails {
            name: "Flash Deal Widget".to_string(),
            price_cents: 1999,
            discount_percent: 50,
            description: "Limited time offer".to_string(),
        };
        set_product_details(&mut conn, &key, &details)
            .await
            .expect("set failed");
        let retrieved = get_product_details(&mut conn, &key)
            .await
            .expect("get failed");
        assert_eq!(retrieved, details);
    }

    #[tokio::test]
    async fn test_get_single_field() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("field");
        let details = ProductDetails {
            name: "Widget".to_string(),
            price_cents: 999,
            discount_percent: 25,
            description: "A widget".to_string(),
        };
        set_product_details(&mut conn, &key, &details)
            .await
            .expect("set failed");
        let name = get_product_field(&mut conn, &key, "name")
            .await
            .expect("get field failed");
        assert_eq!(name, "Widget");
    }

    #[tokio::test]
    async fn test_increment_voucher_count() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("voucher");
        let val = increment_voucher_count(&mut conn, &key, "early_bird", 5)
            .await
            .expect("incr failed");
        assert_eq!(val, 5);
        let val = increment_voucher_count(&mut conn, &key, "early_bird", 3)
            .await
            .expect("incr failed");
        assert_eq!(val, 8);
    }

    #[tokio::test]
    async fn test_delete_field() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("delete");
        let details = ProductDetails {
            name: "Temp".to_string(),
            price_cents: 100,
            discount_percent: 10,
            description: "Temp product".to_string(),
        };
        set_product_details(&mut conn, &key, &details)
            .await
            .expect("set failed");
        let deleted = delete_product_field(&mut conn, &key, "description")
            .await
            .expect("del failed");
        assert!(deleted, "Field should have existed");
        let deleted = delete_product_field(&mut conn, &key, "nonexistent")
            .await
            .expect("del failed");
        assert!(!deleted, "Field should not have existed");
    }
}
