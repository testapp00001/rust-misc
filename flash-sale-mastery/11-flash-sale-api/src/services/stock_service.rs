//! # Stock Service
//!
//! Manages stock via a Redis Lua script that atomically checks stock, verifies
//! the account has not already claimed, enforces the voucher cap, and decrements
//! stock -- all in a single round trip.
//!
//! ## Exercise
//!
//! Implement the stock service that wraps the Lua script from Module 02.
//!
//! 1. Load the Lua script into Redis using `redis::Script`.
//! 2. Implement `check_and_decrement_stock` that invokes the script.
//! 3. Map the script's integer return codes to `PurchaseResult` variants.
//! 4. Implement `get_stock_info` for the stock query endpoint.
//! 5. Add tests for each outcome (success, sold out, already claimed, limit reached).

use deadpool_redis::redis::Script;
use deadpool_redis::Pool;
use crate::models::purchase::PurchaseResult;
use crate::models::stock::StockInfo;

/// The atomic purchase Lua script (from Module 02).
///
/// KEYS[1] = product:{id}:stock
/// KEYS[2] = product:{id}:claims
/// KEYS[3] = product:{id}:voucher_count
/// KEYS[4] = idempotency:{key}
/// ARGV[1] = account_id
/// ARGV[2] = product_id
/// ARGV[3] = max_vouchers_per_product
/// ARGV[4] = idempotency_result (JSON)
const PURCHASE_LUA_SCRIPT: &str = r#"
-- Step 1: Idempotency check
local existing = redis.call('GET', KEYS[4])
if existing then
    return {2, existing}
end

-- Step 2: Check stock
local stock = tonumber(redis.call('GET', KEYS[1]) or '0')
if stock <= 0 then
    return {0, 'sold_out'}
end

-- Step 3: Check account claim
if redis.call('SISMEMBER', KEYS[2], ARGV[1]) == 1 then
    return {1, 'already_claimed'}
end

-- Step 4: Check product voucher limit
local voucher_count = tonumber(redis.call('GET', KEYS[3]) or '0')
if voucher_count >= tonumber(ARGV[3]) then
    return {3, 'voucher_limit_reached'}
end

-- Step 5: Execute purchase
redis.call('DECR', KEYS[1])
redis.call('SADD', KEYS[2], ARGV[1])
redis.call('INCR', KEYS[3])

-- Step 6: Store idempotency result
redis.call('SETEX', KEYS[4], 3600, ARGV[4])

return {4, 'success'}
"#;

/// Errors from the stock service.
#[derive(Debug, thiserror::Error)]
pub enum StockServiceError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_redis::PoolError),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Service that manages product stock through Redis.
#[derive(Clone)]
pub struct StockService {
    pool: Pool,
    max_vouchers_per_product: u64,
}

impl StockService {
    /// Create a new stock service.
    pub fn new(pool: Pool, max_vouchers_per_product: u64) -> Self {
        Self {
            pool,
            max_vouchers_per_product,
        }
    }

    /// Atomically check stock, verify eligibility, and decrement.
    ///
    /// Returns the `PurchaseResult` mapped from the Lua script's return code.
    #[cfg(feature = "solution")]
    pub async fn check_and_decrement_stock(
        &self,
        product_id: &str,
        account_id: &str,
        idempotency_key: &str,
    ) -> Result<PurchaseResult, StockServiceError> {
        let mut conn = self.pool.get().await?;

        let stock_key = format!("product:{}:stock", product_id);
        let claims_key = format!("product:{}:claims", product_id);
        let voucher_key = format!("product:{}:voucher_count", product_id);
        let idempotency_key_redis = format!("idempotency:{}", idempotency_key);

        let idempotency_result = serde_json::json!({
            "status": "success",
            "product_id": product_id,
            "account_id": account_id,
        })
        .to_string();

        let result: Vec<deadpool_redis::redis::Value> = Script::new(PURCHASE_LUA_SCRIPT)
            .key(&stock_key)
            .key(&claims_key)
            .key(&voucher_key)
            .key(&idempotency_key_redis)
            .arg(account_id)
            .arg(product_id)
            .arg(self.max_vouchers_per_product.to_string())
            .arg(&idempotency_result)
            .invoke_async(&mut conn)
            .await?;

        let code = match result.first() {
            Some(deadpool_redis::redis::Value::Int(n)) => *n as i64,
            _ => return Err(StockServiceError::Internal("Unexpected script return type".to_string())),
        };

        match code {
            0 => Ok(PurchaseResult::SoldOut),
            1 => Ok(PurchaseResult::AlreadyClaimed),
            2 => {
                // Idempotent replay -- treat as success since the original succeeded.
                Ok(PurchaseResult::Success {
                    product_id: product_id.to_string(),
                    account_id: account_id.to_string(),
                })
            }
            3 => Ok(PurchaseResult::VoucherLimitReached),
            4 => Ok(PurchaseResult::Success {
                product_id: product_id.to_string(),
                account_id: account_id.to_string(),
            }),
            other => Err(StockServiceError::Internal(format!(
                "Unknown Lua return code: {other}"
            ))),
        }
    }

    #[cfg(not(feature = "solution"))]
    pub async fn check_and_decrement_stock(
        &self,
        product_id: &str,
        account_id: &str,
        idempotency_key: &str,
    ) -> Result<PurchaseResult, StockServiceError> {
        todo!("Implement the atomic purchase Lua script invocation")
    }

    /// Read current stock information for a product.
    #[cfg(feature = "solution")]
    pub async fn get_stock_info(&self, product_id: &str) -> Result<StockInfo, StockServiceError> {
        let mut conn = self.pool.get().await?;

        let stock_key = format!("product:{}:stock", product_id);
        let voucher_key = format!("product:{}:voucher_count", product_id);
        let config_key = "sale:config";

        let stock: i64 = deadpool_redis::redis::cmd("GET")
            .arg(&stock_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        let vouchers: i64 = deadpool_redis::redis::cmd("GET")
            .arg(&voucher_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        let sale_active_str: String = deadpool_redis::redis::cmd("HGET")
            .arg(config_key)
            .arg("status")
            .query_async(&mut conn)
            .await
            .unwrap_or_else(|_: deadpool_redis::redis::RedisError| "active".to_string());
        let sale_active = sale_active_str == "active";

        Ok(StockInfo {
            product_id: product_id.to_string(),
            stock_remaining: stock.max(0) as u64,
            total_vouchers: vouchers.max(0) as u64,
            sale_active,
        })
    }

    #[cfg(not(feature = "solution"))]
    pub async fn get_stock_info(&self, product_id: &str) -> Result<StockInfo, StockServiceError> {
        todo!("Implement stock info retrieval from Redis")
    }

    /// Initialize stock for a product (used in tests and pre-sale setup).
    #[cfg(feature = "solution")]
    pub async fn initialize_stock(
        &self,
        product_id: &str,
        stock_count: u64,
    ) -> Result<(), StockServiceError> {
        let mut conn = self.pool.get().await?;
        let stock_key = format!("product:{}:stock", product_id);

        let _: () = deadpool_redis::redis::cmd("SET")
            .arg(&stock_key)
            .arg(stock_count)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    #[cfg(not(feature = "solution"))]
    pub async fn initialize_stock(
        &self,
        product_id: &str,
        stock_count: u64,
    ) -> Result<(), StockServiceError> {
        todo!("Implement stock initialization")
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

    fn unique_id() -> String {
        format!("{}_{}", std::process::id(), uuid::Uuid::new_v4())
    }

    #[tokio::test]
    async fn test_stock_decrement_success() {
        let pool = match create_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let product_id = unique_id();
        let account_id = unique_id();
        let idem_key = unique_id();

        let svc = StockService::new(pool.clone(), 100);
        svc.initialize_stock(&product_id, 10).await.unwrap();

        let result = svc
            .check_and_decrement_stock(&product_id, &account_id, &idem_key)
            .await
            .unwrap();

        assert!(
            matches!(result, PurchaseResult::Success { .. }),
            "Expected Success, got {:?}",
            result
        );

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = deadpool_redis::redis::cmd("DEL")
            .arg(format!("product:{}:stock", product_id))
            .arg(format!("product:{}:claims", product_id))
            .arg(format!("product:{}:voucher_count", product_id))
            .arg(format!("idempotency:{}", idem_key))
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_sold_out() {
        let pool = match create_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let product_id = unique_id();
        let account_id = unique_id();
        let idem_key = unique_id();

        let svc = StockService::new(pool.clone(), 100);
        svc.initialize_stock(&product_id, 0).await.unwrap();

        let result = svc
            .check_and_decrement_stock(&product_id, &account_id, &idem_key)
            .await
            .unwrap();

        assert!(matches!(result, PurchaseResult::SoldOut));

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = deadpool_redis::redis::cmd("DEL")
            .arg(format!("product:{}:stock", product_id))
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_already_claimed() {
        let pool = match create_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let product_id = unique_id();
        let account_id = unique_id();

        let svc = StockService::new(pool.clone(), 100);
        svc.initialize_stock(&product_id, 10).await.unwrap();

        // First claim
        let idem1 = unique_id();
        let r1 = svc
            .check_and_decrement_stock(&product_id, &account_id, &idem1)
            .await
            .unwrap();
        assert!(matches!(r1, PurchaseResult::Success { .. }));

        // Second claim with different idempotency key but same account
        let idem2 = unique_id();
        let r2 = svc
            .check_and_decrement_stock(&product_id, &account_id, &idem2)
            .await
            .unwrap();
        assert!(matches!(r2, PurchaseResult::AlreadyClaimed));

        // Clean up
        let mut conn = pool.get().await.unwrap();
        let _: () = deadpool_redis::redis::cmd("DEL")
            .arg(format!("product:{}:stock", product_id))
            .arg(format!("product:{}:claims", product_id))
            .arg(format!("product:{}:voucher_count", product_id))
            .arg(format!("idempotency:{}", idem1))
            .query_async(&mut conn)
            .await
            .unwrap();
    }
}
