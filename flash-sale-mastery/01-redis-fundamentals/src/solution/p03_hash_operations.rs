//! # Solution 03: Redis Hash Operations
//!
//! Complete implementation of Redis HASH operations for product details management.

use deadpool_redis::redis::RedisError;
use deadpool_redis::Connection as RedisConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error type for hash operations.
#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

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
pub async fn set_product_details(
    conn: &mut RedisConnection,
    product_id: &str,
    details: &ProductDetails,
) -> Result<(), HashError> {
    deadpool_redis::redis::cmd("HSET")
        .arg(product_id)
        .arg("name")
        .arg(&details.name)
        .arg("price_cents")
        .arg(details.price_cents)
        .arg("discount_percent")
        .arg(details.discount_percent)
        .arg("description")
        .arg(&details.description)
        .query_async::<()>(&mut *conn)
        .await?;
    Ok(())
}

/// Retrieve all fields of a product hash.
pub async fn get_product_details(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<ProductDetails, HashError> {
    let map: HashMap<String, String> = deadpool_redis::redis::cmd("HGETALL")
        .arg(product_id)
        .query_async(&mut *conn)
        .await?;

    if map.is_empty() {
        return Err(HashError::FieldNotFound {
            field: product_id.to_string(),
        });
    }

    let name = map
        .get("name")
        .cloned()
        .unwrap_or_default();
    let price_cents = map
        .get("price_cents")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let discount_percent = map
        .get("discount_percent")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let description = map
        .get("description")
        .cloned()
        .unwrap_or_default();

    Ok(ProductDetails {
        name,
        price_cents,
        discount_percent,
        description,
    })
}

/// Retrieve a single field from a product hash.
pub async fn get_product_field(
    conn: &mut RedisConnection,
    product_id: &str,
    field: &str,
) -> Result<String, HashError> {
    let val: Option<String> = deadpool_redis::redis::cmd("HGET")
        .arg(product_id)
        .arg(field)
        .query_async(&mut *conn)
        .await?;
    val.ok_or_else(|| HashError::FieldNotFound {
        field: field.to_string(),
    })
}

/// Atomically increment a voucher count for a product.
pub async fn increment_voucher_count(
    conn: &mut RedisConnection,
    product_id: &str,
    voucher_type: &str,
    increment: i64,
) -> Result<i64, HashError> {
    let val: i64 = deadpool_redis::redis::cmd("HINCRBY")
        .arg(product_id)
        .arg(voucher_type)
        .arg(increment)
        .query_async(&mut *conn)
        .await?;
    Ok(val)
}

/// Delete a specific field from a product hash.
pub async fn delete_product_field(
    conn: &mut RedisConnection,
    product_id: &str,
    field: &str,
) -> Result<bool, HashError> {
    let count: i64 = deadpool_redis::redis::cmd("HDEL")
        .arg(product_id)
        .arg(field)
        .query_async(&mut *conn)
        .await?;
    Ok(count > 0)
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
        assert!(delete_product_field(&mut conn, &key, "description").await.unwrap());
        assert!(!delete_product_field(&mut conn, &key, "nonexistent").await.unwrap());
    }
}
