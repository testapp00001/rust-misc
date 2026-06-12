//! # Solution 08: Batch Stock Check
//!
//! Complete implementation of batch stock checking via Lua.

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
const BATCH_STOCK_SCRIPT: &str = r#"
-- Returns a flat array: [flag1, count1, flag2, count2, ...]
-- flag = 1 if in stock (count > 0), flag = 0 otherwise

local results = {}
for i = 1, #KEYS do
    local stock = redis.call('GET', KEYS[i])
    if stock and tonumber(stock) > 0 then
        table.insert(results, 1)
        table.insert(results, tonumber(stock))
    else
        table.insert(results, 0)
        table.insert(results, 0)
    end
end
return results
"#;

/// Check the stock of multiple products in a single atomic call.
///
/// Returns a `Vec<StockStatus>` with one entry per product, in the same
/// order as the input product IDs.
pub async fn batch_check_stock(
    conn: &mut RedisConnection,
    product_ids: &[String],
) -> Result<Vec<StockStatus>> {
    if product_ids.is_empty() {
        return Ok(vec![]);
    }

    let script = Script::new(BATCH_STOCK_SCRIPT);
    let mut prepared = script.prepare_invoke();
    for pid in product_ids {
        let key = format!("stock:{pid}");
        prepared.key(key);
    }

    let result: Vec<i64> = prepared.invoke_async(conn).await?;

    // The result is a flat array: [flag1, count1, flag2, count2, ...]
    let mut statuses = Vec::with_capacity(product_ids.len());
    for (i, pid) in product_ids.iter().enumerate() {
        let flag_idx = i * 2;
        let count_idx = i * 2 + 1;
        let in_stock = result.get(flag_idx).copied().unwrap_or(0) == 1;
        let count = result.get(count_idx).copied().unwrap_or(0);
        statuses.push(StockStatus {
            product_id: pid.clone(),
            in_stock,
            count,
        });
    }

    Ok(statuses)
}

/// Set stock for multiple products (test helper).
pub async fn setup_batch_stock(
    conn: &mut RedisConnection,
    stock_map: &[(&str, i64)],
) -> Result<()> {
    for (pid, qty) in stock_map {
        let key = format!("stock:{pid}");
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg(qty)
            .query_async(&mut *conn)
            .await?;
    }
    Ok(())
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
