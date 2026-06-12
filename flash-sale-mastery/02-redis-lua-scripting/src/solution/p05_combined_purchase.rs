//! # Solution 05: Combined Atomic Purchase -- THE MASTER SCRIPT
//!
//! Complete implementation of the combined atomic purchase flow.
//! This is the most critical piece of the flash sale system.

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::{Script, Value};
use serde::{Deserialize, Serialize};

/// TTL (seconds) for the idempotency cache entry.
const TTL_SECS: u64 = 3600;

/// Request parameters for a combined purchase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseRequest {
    pub product_id: String,
    pub account_id: String,
    pub request_id: String,
    pub max_vouchers: u64,
}

/// Result of a combined purchase attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum PurchaseResult {
    /// Purchase succeeded. Contains the generated voucher code.
    Success { voucher_code: String },
    /// The product is sold out.
    SoldOut,
    /// This account has already claimed this product.
    AlreadyClaimed,
    /// The product's voucher limit has been reached.
    VoucherLimitReached,
    /// This request_id was already processed. Returns the cached result.
    IdempotentReplay { original_result: String },
}

/// Combined atomic purchase Lua script.
///
/// Performs idempotency check, account claim check, stock check,
/// voucher limit check, and all mutations in a single atomic block.
///
/// KEYS: [stock, claims, vouchers, idempotency]
/// ARGV: [account_id, max_vouchers, request_id, ttl_secs]
const PURCHASE_SCRIPT: &str = r#"
-- KEYS[1] = stock key
-- KEYS[2] = claims set key
-- KEYS[3] = voucher counter key
-- KEYS[4] = idempotency key
-- ARGV[1] = account_id
-- ARGV[2] = max_vouchers (string)
-- ARGV[3] = request_id
-- ARGV[4] = ttl_seconds (string)

-- Step 1: Idempotency check
local existing = redis.call('GET', KEYS[4])
if existing then
    return {5, existing}  -- IdempotentReplay
end

-- Step 2: Account claim check
local already_claimed = redis.call('SISMEMBER', KEYS[2], ARGV[1])
if already_claimed == 1 then
    redis.call('SETEX', KEYS[4], tonumber(ARGV[4]), 'ALREADY_CLAIMED')
    return {3}  -- AlreadyClaimed
end

-- Step 3: Stock check and decrement
local stock = redis.call('GET', KEYS[1])
if not stock or tonumber(stock) <= 0 then
    redis.call('SETEX', KEYS[4], tonumber(ARGV[4]), 'SOLD_OUT')
    return {2}  -- SoldOut
end
redis.call('DECR', KEYS[1])

-- Step 4: Voucher limit check
local voucher_count = tonumber(redis.call('GET', KEYS[3]) or '0')
local max_vouchers = tonumber(ARGV[2])
if voucher_count >= max_vouchers then
    -- Roll back stock decrement
    redis.call('INCR', KEYS[1])
    redis.call('SETEX', KEYS[4], tonumber(ARGV[4]), 'VOUCHER_LIMIT_REACHED')
    return {4}  -- VoucherLimitReached
end
redis.call('INCR', KEYS[3])

-- Step 5: Record the account claim
redis.call('SADD', KEYS[2], ARGV[1])

-- Step 6: Generate voucher code
local new_voucher_count = voucher_count + 1
local voucher_code = 'VCHR-' .. ARGV[1] .. '-' .. tostring(new_voucher_count)

-- Step 7: Store idempotency result
redis.call('SETEX', KEYS[4], tonumber(ARGV[4]), 'SUCCESS:' .. voucher_code)

return {1, voucher_code}  -- Success
"#;

/// Execute the full atomic purchase flow.
pub async fn execute_purchase(
    conn: &mut RedisConnection,
    req: &PurchaseRequest,
) -> Result<PurchaseResult> {
    let stock_key = format!("stock:{}", req.product_id);
    let claims_key = format!("claims:{}", req.product_id);
    let voucher_key = format!("vouchers:{}", req.product_id);
    let idempotency_key = format!("idemp:{}", req.request_id);

    let result: Value = Script::new(PURCHASE_SCRIPT)
        .key(stock_key.as_str())
        .key(claims_key.as_str())
        .key(voucher_key.as_str())
        .key(idempotency_key.as_str())
        .arg(req.account_id.as_str())
        .arg(req.max_vouchers.to_string())
        .arg(req.request_id.as_str())
        .arg(TTL_SECS.to_string())
        .invoke_async(conn)
        .await?;

    parse_purchase_result(&result)
}

/// Parse the Lua return array into a `PurchaseResult`.
fn parse_purchase_result(value: &Value) -> Result<PurchaseResult> {
    match value {
        Value::Array(vals) if !vals.is_empty() => match &vals[0] {
            Value::Int(1) => {
                // Success
                let voucher_code = extract_string(vals.get(1));
                Ok(PurchaseResult::Success { voucher_code })
            }
            Value::Int(2) => Ok(PurchaseResult::SoldOut),
            Value::Int(3) => Ok(PurchaseResult::AlreadyClaimed),
            Value::Int(4) => Ok(PurchaseResult::VoucherLimitReached),
            Value::Int(5) => {
                // IdempotentReplay
                let cached = extract_string(vals.get(1));
                Ok(PurchaseResult::IdempotentReplay {
                    original_result: cached,
                })
            }
            other => Err(anyhow::anyhow!(
                "Unknown status code in Lua response: {other:?}"
            )),
        },
        _ => Err(anyhow::anyhow!(
            "Unexpected Lua response format: {value:?}"
        )),
    }
}

/// Extract a string from an optional `Value::BulkString`.
fn extract_string(value: Option<&Value>) -> String {
    match value {
        Some(Value::BulkString(bytes)) => String::from_utf8_lossy(bytes).to_string(),
        Some(Value::SimpleString(s)) => s.clone(),
        _ => String::new(),
    }
}

/// Set up stock for a product (test helper).
pub async fn setup_stock(
    conn: &mut RedisConnection,
    product_id: &str,
    quantity: i64,
) -> Result<()> {
    let key = format!("stock:{product_id}");
    let _: () = redis::cmd("SET")
        .arg(&key)
        .arg(quantity)
        .query_async(conn)
        .await?;
    Ok(())
}

/// Get current stock (test helper).
pub async fn get_stock(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64> {
    let key = format!("stock:{product_id}");
    let value: Option<i64> = redis::cmd("GET")
        .arg(&key)
        .query_async(conn)
        .await?;
    Ok(value.unwrap_or(0))
}

/// Get current voucher count (test helper).
pub async fn get_voucher_count(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<u64> {
    let key = format!("vouchers:{product_id}");
    let value: Option<u64> = redis::cmd("GET")
        .arg(&key)
        .query_async(conn)
        .await?;
    Ok(value.unwrap_or(0))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Pool, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn try_connect() -> Result<Pool, String> {
        let cfg = Config::from_url(REDIS_URL);
        cfg.create_pool(Some(Runtime::Tokio1))
            .map_err(|e| format!("Could not create pool: {e}"))
    }

    async fn cleanup_all(conn: &mut RedisConnection, product_id: &str) {
        for prefix in &["stock:", "claims:", "vouchers:"] {
            let _: () = redis::cmd("DEL")
                .arg(format!("{prefix}{product_id}"))
                .query_async(&mut *conn)
                .await
                .unwrap_or(());
        }
    }

    async fn cleanup_idemp(conn: &mut RedisConnection, request_id: &str) {
        let _: () = redis::cmd("DEL")
            .arg(format!("idemp:{request_id}"))
            .query_async(&mut *conn)
            .await
            .unwrap_or(());
    }

    /// Happy path: valid request should succeed with a voucher code.
    #[tokio::test]
    async fn test_successful_purchase() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_success";
        let rid = "test_success_req1";

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
        setup_stock(&mut conn, pid, 10).await.unwrap();

        let req = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid.to_string(),
            max_vouchers: 5,
        };

        let result = execute_purchase(&mut conn, &req).await.unwrap();
        match result {
            PurchaseResult::Success { voucher_code } => {
                assert!(!voucher_code.is_empty());
            }
            other => panic!("Expected Success, got {other:?}"),
        }

        let stock = get_stock(&mut conn, pid).await.unwrap();
        assert_eq!(stock, 9);

        let vouchers = get_voucher_count(&mut conn, pid).await.unwrap();
        assert_eq!(vouchers, 1);

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
    }

    /// Sold-out product should return SoldOut.
    #[tokio::test]
    async fn test_sold_out() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_combined_soldout";
        let rid = "test_combined_soldout_req";

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;

        let req = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid.to_string(),
            max_vouchers: 5,
        };

        let result = execute_purchase(&mut conn, &req).await.unwrap();
        assert_eq!(result, PurchaseResult::SoldOut);

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
    }

    /// Duplicate account should return AlreadyClaimed.
    #[tokio::test]
    async fn test_already_claimed() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_combined_claimed";
        let rid1 = "test_combined_claimed_r1";
        let rid2 = "test_combined_claimed_r2";

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid1).await;
        cleanup_idemp(&mut conn, rid2).await;
        setup_stock(&mut conn, pid, 10).await.unwrap();

        let req1 = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid1.to_string(),
            max_vouchers: 5,
        };
        let r1 = execute_purchase(&mut conn, &req1).await.unwrap();
        assert!(matches!(r1, PurchaseResult::Success { .. }));

        // Same account, different request ID
        let req2 = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid2.to_string(),
            max_vouchers: 5,
        };
        let r2 = execute_purchase(&mut conn, &req2).await.unwrap();
        assert_eq!(r2, PurchaseResult::AlreadyClaimed);

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid1).await;
        cleanup_idemp(&mut conn, rid2).await;
    }

    /// Voucher limit exhausted should return VoucherLimitReached.
    #[tokio::test]
    async fn test_voucher_limit() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_combined_vlimit";
        let rid = "test_combined_vlimit_req";

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
        setup_stock(&mut conn, pid, 10).await.unwrap();

        let req = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid.to_string(),
            max_vouchers: 0,
        };

        let result = execute_purchase(&mut conn, &req).await.unwrap();
        assert_eq!(result, PurchaseResult::VoucherLimitReached);

        // Stock should be unchanged (rolled back)
        let stock = get_stock(&mut conn, pid).await.unwrap();
        assert_eq!(stock, 10);

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
    }

    /// Same request_id should return IdempotentReplay.
    #[tokio::test]
    async fn test_idempotent_replay() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_combined_idemp";
        let rid = "test_combined_idemp_req";

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
        setup_stock(&mut conn, pid, 10).await.unwrap();

        let req = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid.to_string(),
            max_vouchers: 5,
        };

        let r1 = execute_purchase(&mut conn, &req).await.unwrap();
        assert!(matches!(r1, PurchaseResult::Success { .. }));

        let r2 = execute_purchase(&mut conn, &req).await.unwrap();
        assert!(matches!(r2, PurchaseResult::IdempotentReplay { .. }));

        // Stock should only have been decremented once
        let stock = get_stock(&mut conn, pid).await.unwrap();
        assert_eq!(stock, 9);

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
    }

    /// CRITICAL: 500 concurrent threads, stock=100, max_vouchers=100.
    /// Exactly 100 should succeed. Stock and voucher count must match.
    #[test]
    fn test_concurrent_purchase() {
        let pool = match tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(try_connect())
        {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let pid = "concurrent_purchase";

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            cleanup_all(&mut conn, pid).await;
            // Clean up idempotency keys from prior runs
            for i in 0..500 {
                cleanup_idemp(&mut conn, &format!("req_{i}")).await;
            }
            setup_stock(&mut conn, pid, 100).await.unwrap();
        });

        let mut handles = vec![];
        for i in 0..500 {
            let p = pool.clone();
            let id = pid.to_string();
            let handle = std::thread::Builder::new()
                .spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut conn = match p.get().await {
                            Ok(c) => c,
                            Err(_) => return Ok::<_, anyhow::Error>(PurchaseResult::SoldOut),
                        };
                        let req = PurchaseRequest {
                            product_id: id.clone(),
                            account_id: format!("acct_{i}"),
                            request_id: format!("req_{i}"),
                            max_vouchers: 100,
                        };
                        match execute_purchase(&mut conn, &req).await {
                            Ok(result) => Ok(result),
                            Err(_) => Ok(PurchaseResult::SoldOut),
                        }
                    })
                });
            match handle {
                Ok(h) => handles.push(h),
                Err(_) => {} // OS thread limit reached, skip
            }
        }

        let mut success_count = 0i64;
        let mut sold_out_count = 0i64;
        let mut _replay_count = 0i64;
        for handle in handles {
            match handle.join().unwrap() {
                Ok(PurchaseResult::Success { .. }) => success_count += 1,
                Ok(PurchaseResult::SoldOut) => sold_out_count += 1,
                Ok(PurchaseResult::IdempotentReplay { .. }) => _replay_count += 1,
                Ok(other) => panic!("Unexpected result: {other:?}"),
                Err(_) => {} // connection errors treated as non-success
            }
        }

        assert!(success_count <= 100, "Success count must not exceed 100, got {success_count}");
        assert!(success_count > 0, "At least some purchases should succeed");

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            let stock = get_stock(&mut conn, pid).await.unwrap();
            assert_eq!(stock, 0, "Stock must be 0");
            let vouchers = get_voucher_count(&mut conn, pid).await.unwrap();
            assert_eq!(vouchers, 100, "Voucher count must be 100");
            cleanup_all(&mut conn, pid).await;
        });
    }
}
