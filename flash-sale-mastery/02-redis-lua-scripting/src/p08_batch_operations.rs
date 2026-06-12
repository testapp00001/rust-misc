//! # Exercise 08: Batch Stock Check
//!
//! ## Learning Objective
//! Build a Lua script that checks the stock of multiple products in a
//! single atomic call. Batch operations reduce round trips and ensure a
//! consistent snapshot across all products.
//!
//! ## Flash Sale Context
//! A product listing page needs to display stock status for 20+ items.
//! Issuing 20 separate GET commands adds unnecessary latency. A single
//! Lua script reads all keys atomically, guaranteeing a consistent view.
//!
//! ## Instructions
//! 1. Define a Lua script that iterates over all KEYS and reads their stock
//! 2. Implement `batch_check_stock` returning a `Vec<StockStatus>`
//! 3. Implement `setup_batch_stock` for test setup
//!
//! ## Hints
//! - Lua `#KEYS` gives the number of keys
//! - Return a flat array: pairs of (flag, count) for each product
//! - Redis Lua tables with only integer keys become arrays in the protocol

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::Script;

/// Stock status for a single product in a batch check.
#[derive(Debug, Clone, PartialEq)]
pub struct StockStatus {
    pub product_id: String,
    pub in_stock: bool,
    pub count: i64,
}

/// Batch stock check Lua script.
///
/// For each key in KEYS, reads the stock value and returns a flat array
/// of (flag, count) pairs. Flag is 1 if in stock, 0 if not.
///
/// Example return for 3 products: {1, 10, 0, 0, 1, 5}
const BATCH_STOCK_SCRIPT: &str = r#"
-- Returns a flat array: [flag1, count1, flag2, count2, ...]
-- flag = 1 if in stock (count > 0), flag = 0 otherwise

-- TODO: Implement batch stock check
-- 1. Loop over i = 1 to #KEYS
-- 2. For each key, GET the stock value
-- 3. If value exists and > 0, append {1, value} to results
-- 4. Otherwise append {0, 0}
"#;

/// Check the stock of multiple products in a single atomic call.
///
/// Returns a `Vec<StockStatus>` with one entry per product, in the same
/// order as the input product IDs.
pub async fn batch_check_stock(
    conn: &mut RedisConnection,
    product_ids: &[String],
) -> Result<Vec<StockStatus>> {
    // TODO: Build keys, invoke script, parse flat array into StockStatus vec
    todo!("Implement batch stock check using BATCH_STOCK_SCRIPT")
}

/// Set stock for multiple products (test helper).
pub async fn setup_batch_stock(
    conn: &mut RedisConnection,
    stock_map: &[(&str, i64)],
) -> Result<()> {
    // TODO: SET each stock key
    todo!("Implement: SET each stock:<product_id> to its quantity")
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
        Config::from_url(REDIS_URL)
            .builder(Some(Runtime::Tokio1))
            .max_size(16)
            .build()
            .map_err(|e| format!("Could not create pool: {e}"))
    }

    async fn cleanup_batch(conn: &mut RedisConnection, product_ids: &[&str]) {
        for pid in product_ids {
            let _: () = redis::cmd("DEL")
                .arg(format!("stock:{pid}"))
                .query_async(&mut *conn)
                .await
                .unwrap_or(());
        }
    }

    /// All products in stock should report in_stock=true.
    #[tokio::test]
    async fn test_all_in_stock() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pids = ["batch_all_a", "batch_all_b", "batch_all_c"];

        cleanup_batch(&mut conn, &pids).await;
        setup_batch_stock(&mut conn, &[("batch_all_a", 5), ("batch_all_b", 3), ("batch_all_c", 8)])
            .await
            .unwrap();

        let ids: Vec<String> = pids.iter().map(|s| s.to_string()).collect();
        let result = batch_check_stock(&mut conn, &ids).await.unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|s| s.in_stock));
        assert_eq!(result[0].count, 5);
        assert_eq!(result[1].count, 3);
        assert_eq!(result[2].count, 8);

        cleanup_batch(&mut conn, &pids).await;
    }

    /// Some in stock, some out of stock.
    #[tokio::test]
    async fn test_mixed_stock() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pids = ["batch_mix_a", "batch_mix_b", "batch_mix_c"];

        cleanup_batch(&mut conn, &pids).await;
        setup_batch_stock(&mut conn, &[("batch_mix_a", 5), ("batch_mix_b", 0)])
            .await
            .unwrap();
        // batch_mix_c not set => missing

        let ids: Vec<String> = pids.iter().map(|s| s.to_string()).collect();
        let result = batch_check_stock(&mut conn, &ids).await.unwrap();

        assert_eq!(result.len(), 3);
        assert!(result[0].in_stock);
        assert!(!result[1].in_stock);
        assert!(!result[2].in_stock);

        cleanup_batch(&mut conn, &pids).await;
    }

    /// None in stock.
    #[tokio::test]
    async fn test_none_in_stock() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pids = ["batch_none_a", "batch_none_b"];

        cleanup_batch(&mut conn, &pids).await;

        let ids: Vec<String> = pids.iter().map(|s| s.to_string()).collect();
        let result = batch_check_stock(&mut conn, &ids).await.unwrap();

        assert_eq!(result.len(), 2);
        assert!(!result[0].in_stock);
        assert!(!result[1].in_stock);

        cleanup_batch(&mut conn, &pids).await;
    }

    /// Empty product list should return an empty vec.
    #[tokio::test]
    async fn test_empty_list() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");

        let result = batch_check_stock(&mut conn, &[]).await.unwrap();
        assert!(result.is_empty());
    }
}
