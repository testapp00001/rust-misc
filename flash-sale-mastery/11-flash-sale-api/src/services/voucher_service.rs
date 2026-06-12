//! # Voucher Service
//!
//! Generates unique voucher codes and enforces per-product voucher caps.
//!
//! ## Exercise
//!
//! 1. Implement `generate_voucher` that creates a unique code using UUID.
//! 2. Implement `check_voucher_limit` that reads the current count from Redis.
//! 3. Store generated vouchers in Redis for later retrieval.
//! 4. Write tests for generation and limit enforcement.

use deadpool_redis::Pool;
use crate::models::voucher::VoucherCode;

/// Errors from the voucher service.
#[derive(Debug, thiserror::Error)]
pub enum VoucherServiceError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_redis::PoolError),

    #[error("Voucher limit reached for product {product_id}")]
    LimitReached { product_id: String },
}

/// Service responsible for voucher generation and limit tracking.
#[derive(Clone)]
pub struct VoucherService {
    pool: Pool,
    max_vouchers_per_product: u64,
}

impl VoucherService {
    /// Create a new voucher service.
    pub fn new(pool: Pool, max_vouchers_per_product: u64) -> Self {
        Self {
            pool,
            max_vouchers_per_product,
        }
    }

    /// Generate a unique voucher code for a product/account pair.
    #[cfg(feature = "solution")]
    pub fn generate_voucher(&self, product_id: &str, account_id: &str) -> VoucherCode {
        let code = format!("VS-{}-{}", &product_id[..std::cmp::min(8, product_id.len())], uuid::Uuid::new_v4());
        VoucherCode {
            code,
            product_id: product_id.to_string(),
            account_id: account_id.to_string(),
            created_at: chrono::Utc::now(),
        }
    }

    #[cfg(not(feature = "solution"))]
    pub fn generate_voucher(&self, product_id: &str, account_id: &str) -> VoucherCode {
        todo!("Generate a unique voucher code using UUID")
    }

    /// Check whether the product has reached its voucher cap.
    #[cfg(feature = "solution")]
    pub async fn check_voucher_limit(&self, product_id: &str) -> Result<bool, VoucherServiceError> {
        let mut conn = self.pool.get().await.map_err(VoucherServiceError::Pool)?;
        let key = format!("product:{}:voucher_count", product_id);

        let count: u64 = deadpool_redis::redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        Ok(count < self.max_vouchers_per_product)
    }

    #[cfg(not(feature = "solution"))]
    pub async fn check_voucher_limit(&self, product_id: &str) -> Result<bool, VoucherServiceError> {
        todo!("Check the voucher count in Redis against the product cap")
    }

    /// Store a voucher in Redis for later retrieval / verification.
    #[cfg(feature = "solution")]
    pub async fn store_voucher(&self, voucher: &VoucherCode) -> Result<(), VoucherServiceError> {
        let mut conn = self.pool.get().await.map_err(VoucherServiceError::Pool)?;
        let key = format!("voucher:{}", voucher.code);
        let value = serde_json::to_string(voucher).unwrap_or_default();

        let _: () = deadpool_redis::redis::cmd("SETEX")
            .arg(&key)
            .arg(86400) // 24h TTL
            .arg(&value)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    #[cfg(not(feature = "solution"))]
    pub async fn store_voucher(&self, voucher: &VoucherCode) -> Result<(), VoucherServiceError> {
        todo!("Store the voucher JSON in Redis with a TTL")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::Config;

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn create_pool() -> Result<Pool, String> {
        let cfg = Config::from_url(REDIS_URL);
        let pool = cfg
            .builder()
            .map_err(|e| format!("Pool builder failed: {e}"))?
            .max_size(8)
            .build()
            .map_err(|e| format!("Pool creation failed: {e}"))?;
        // Verify connectivity.
        let mut conn = pool.get().await.map_err(|e| format!("Redis not available: {e}"))?;
        let _: String = deadpool_redis::redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("Redis PING failed: {e}"))?;
        Ok(pool)
    }

    #[test]
    fn test_voucher_generation() {
        // Voucher generation is synchronous and does not need Redis.
        let cfg = deadpool_redis::Config::from_url(REDIS_URL);
        let pool = cfg
            .builder()
            .unwrap()
            .max_size(1)
            .build()
            .unwrap();
        let svc = VoucherService::new(pool, 100);

        let v1 = svc.generate_voucher("prod-1", "acct-1");
        let v2 = svc.generate_voucher("prod-1", "acct-2");

        assert_ne!(v1.code, v2.code, "Voucher codes must be unique");
        assert_eq!(v1.product_id, "prod-1");
        assert_eq!(v1.account_id, "acct-1");
        assert!(v1.code.starts_with("VS-"));
    }

    #[tokio::test]
    async fn test_voucher_limit_enforcement() {
        let pool = match create_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };

        let product_id = format!("limit_test_{}", uuid::Uuid::new_v4());
        let svc = VoucherService::new(pool.clone(), 2);

        // Set voucher count to the max
        let mut conn = pool.get().await.unwrap();
        let key = format!("product:{}:voucher_count", product_id);
        let _: () = deadpool_redis::redis::cmd("SET")
            .arg(&key)
            .arg(2u64)
            .query_async(&mut conn)
            .await
            .unwrap();

        let has_capacity = svc.check_voucher_limit(&product_id).await.unwrap();
        assert!(!has_capacity, "Should be at limit");

        // Clean up
        let _: () = deadpool_redis::redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_store_and_retrieve_voucher() {
        let pool = match create_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };

        let svc = VoucherService::new(pool.clone(), 100);
        let voucher = svc.generate_voucher("prod-1", "acct-1");
        let code = voucher.code.clone();

        svc.store_voucher(&voucher).await.unwrap();

        // Verify it was stored
        let mut conn = pool.get().await.unwrap();
        let key = format!("voucher:{}", code);
        let stored: Option<String> = deadpool_redis::redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap();

        assert!(stored.is_some(), "Voucher should be stored in Redis");
        let parsed: VoucherCode = serde_json::from_str(&stored.unwrap()).unwrap();
        assert_eq!(parsed.code, code);
        assert_eq!(parsed.product_id, "prod-1");

        // Clean up
        let _: () = deadpool_redis::redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap();
    }
}
