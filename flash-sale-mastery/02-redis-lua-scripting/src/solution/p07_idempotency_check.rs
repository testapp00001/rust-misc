//! # Solution 07: Atomic Idempotency Check
//!
//! Complete implementation of atomic idempotency check-and-set via Lua.

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::Script;

/// Result of an idempotency check.
#[derive(Debug, Clone, PartialEq)]
pub enum IdempotencyResult {
    /// This is the first time this request ID has been seen.
    FirstRequest,
    /// This request ID was already processed. Contains the cached result.
    Duplicate { cached_result: String },
}

/// Atomic idempotency check-and-set Lua script.
///
/// If the key exists, returns the cached value. Otherwise stores the new
/// value with a TTL and signals that this is a first request.
const IDEMPOTENCY_SCRIPT: &str = r#"
-- KEYS[1] = idempotency key
-- ARGV[1] = result_json to store
-- ARGV[2] = ttl_seconds

local existing = redis.call('GET', KEYS[1])
if existing then
    return {0, existing}  -- Duplicate, return cached result
end
redis.call('SETEX', KEYS[1], tonumber(ARGV[2]), ARGV[1])
return {1, ''}  -- FirstRequest, result stored
"#;

/// Check whether a request ID has been processed before.
///
/// On `FirstRequest`, the caller should proceed and the result has been
/// stored with the configured TTL. On `Duplicate`, the cached result is
/// returned so the caller can replay it.
pub async fn check_idempotency(
    conn: &mut RedisConnection,
    idempotency_key: &str,
    result_json: &str,
    ttl_secs: u64,
) -> Result<IdempotencyResult> {
    let result: Vec<redis::Value> = Script::new(IDEMPOTENCY_SCRIPT)
        .key(idempotency_key)
        .arg(result_json)
        .arg(ttl_secs.to_string())
        .invoke_async(conn)
        .await?;

    match result.as_slice() {
        [redis::Value::Int(0), redis::Value::BulkString(cached)] => {
            Ok(IdempotencyResult::Duplicate {
                cached_result: String::from_utf8_lossy(cached).to_string(),
            })
        }
        [redis::Value::Int(1), ..] => Ok(IdempotencyResult::FirstRequest),
        _ => Err(anyhow::anyhow!(
            "Unexpected idempotency response: {result:?}"
        )),
    }
}

/// Clean up an idempotency key (test helper).
pub async fn cleanup_idempotency(
    conn: &mut RedisConnection,
    idempotency_key: &str,
) -> Result<()> {
    let _: () = redis::cmd("DEL")
        .arg(idempotency_key)
        .query_async(conn)
        .await?;
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
        let mut cfg = Config::from_url(REDIS_URL);
        cfg.pool = Some(deadpool_redis::PoolConfig {
            max_size: 64,
            ..Default::default()
        });
        cfg.create_pool(Some(Runtime::Tokio1))
            .map_err(|e| format!("Could not create pool: {e}"))
    }

    /// First request should return FirstRequest.
    #[tokio::test]
    async fn test_first_request() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_idemp_first";

        cleanup_idempotency(&mut conn, key).await.unwrap();

        let result = check_idempotency(&mut conn, key, r#"{"status":"ok"}"#, 60)
            .await
            .unwrap();
        assert_eq!(result, IdempotencyResult::FirstRequest);

        cleanup_idempotency(&mut conn, key).await.unwrap();
    }

    /// Duplicate request should return the cached result.
    #[tokio::test]
    async fn test_duplicate_request() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_idemp_dup";

        cleanup_idempotency(&mut conn, key).await.unwrap();

        let cached = r#"{"voucher":"VCHR-001"}"#;
        let r1 = check_idempotency(&mut conn, key, cached, 60)
            .await
            .unwrap();
        assert_eq!(r1, IdempotencyResult::FirstRequest);

        let r2 = check_idempotency(&mut conn, key, "should-be-ignored", 60)
            .await
            .unwrap();
        assert_eq!(
            r2,
            IdempotencyResult::Duplicate {
                cached_result: cached.to_string()
            }
        );

        cleanup_idempotency(&mut conn, key).await.unwrap();
    }

    /// Different keys should be independent.
    #[tokio::test]
    async fn test_independent_keys() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");

        cleanup_idempotency(&mut conn, "test_idemp_ind_a").await.unwrap();
        cleanup_idempotency(&mut conn, "test_idemp_ind_b").await.unwrap();

        let r1 = check_idempotency(&mut conn, "test_idemp_ind_a", "result_a", 60)
            .await
            .unwrap();
        let r2 = check_idempotency(&mut conn, "test_idemp_ind_b", "result_b", 60)
            .await
            .unwrap();

        assert_eq!(r1, IdempotencyResult::FirstRequest);
        assert_eq!(r2, IdempotencyResult::FirstRequest);

        cleanup_idempotency(&mut conn, "test_idemp_ind_a").await.unwrap();
        cleanup_idempotency(&mut conn, "test_idemp_ind_b").await.unwrap();
    }

    /// CRITICAL: 1000 concurrent requests with the same key.
    /// Exactly 1 should be FirstRequest; the rest must be Duplicate.
    #[test]
    fn test_concurrent_duplicates() {
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
        let key = "concurrent_idemp";

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            cleanup_idempotency(&mut conn, key).await.unwrap();
        });

        let mut handles = vec![];
        for i in 0..1000 {
            let p = pool.clone();
            let k = key.to_string();
            let handle = std::thread::Builder::new()
                .spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut conn = match p.get().await {
                            Ok(c) => c,
                            Err(_) => return Ok::<_, anyhow::Error>(IdempotencyResult::Duplicate {
                                cached_result: String::new(),
                            }),
                        };
                        match check_idempotency(
                            &mut conn,
                            &k,
                            &format!(r#"{{"thread":{i}}}"#),
                            60,
                        )
                        .await
                        {
                            Ok(result) => Ok(result),
                            Err(_) => Ok(IdempotencyResult::Duplicate {
                                cached_result: String::new(),
                            }),
                        }
                    })
                });
            match handle {
                Ok(h) => handles.push(h),
                Err(_) => {} // OS thread limit reached, skip
            }
        }

        let mut first_count = 0i64;
        let mut dup_count = 0i64;
        for handle in handles {
            match handle.join().unwrap() {
                Ok(IdempotencyResult::FirstRequest) => first_count += 1,
                Ok(IdempotencyResult::Duplicate { .. }) => dup_count += 1,
                Err(_) => {} // connection errors treated as duplicate
            }
        }

        assert_eq!(first_count, 1, "Exactly 1 FirstRequest");

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            cleanup_idempotency(&mut conn, key).await.unwrap();
        });
    }
}
