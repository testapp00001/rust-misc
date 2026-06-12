//! # Solution 03: Atomic Account Claim Check
//!
//! Complete implementation of atomic SISMEMBER + SADD for claim deduplication.

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::Script;

/// Result of an atomic claim check.
#[derive(Debug, Clone, PartialEq)]
pub enum ClaimResult {
    /// This is a new claim; the account was added to the claims set.
    NewClaim,
    /// The account had already claimed this item.
    AlreadyClaimed,
}

/// Atomic claim check Lua script.
///
/// Checks whether ARGV[1] (account_id) is a member of the set at KEYS[1].
/// If already present returns 0; otherwise adds the member and returns 1.
const CLAIM_CHECK_SCRIPT: &str = r#"
-- KEYS[1]: claims set key (e.g. "claims:product:42")
-- ARGV[1]: account_id
-- Returns: 1 = new claim, 0 = already claimed

local is_member = redis.call('SISMEMBER', KEYS[1], ARGV[1])
if is_member == 1 then
    return 0  -- already claimed
end
redis.call('SADD', KEYS[1], ARGV[1])
return 1  -- new claim
"#;

/// Atomically check whether `account_id` has already claimed `product_id`,
/// and if not, record the claim.
pub async fn atomic_claim_check(
    conn: &mut RedisConnection,
    product_id: &str,
    account_id: &str,
) -> Result<ClaimResult> {
    let key = format!("claims:{product_id}");
    let result: i64 = Script::new(CLAIM_CHECK_SCRIPT)
        .key(key.as_str())
        .arg(account_id)
        .invoke_async(conn)
        .await?;

    match result {
        1 => Ok(ClaimResult::NewClaim),
        _ => Ok(ClaimResult::AlreadyClaimed),
    }
}

/// Get all accounts that have claimed a product (test helper).
pub async fn get_claimed_accounts(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<Vec<String>> {
    let key = format!("claims:{product_id}");
    let members: Vec<String> = redis::cmd("SMEMBERS")
        .arg(&key)
        .query_async(conn)
        .await?;
    Ok(members)
}

/// Check whether a specific account has claimed (test helper).
pub async fn has_claimed(
    conn: &mut RedisConnection,
    product_id: &str,
    account_id: &str,
) -> Result<bool> {
    let key = format!("claims:{product_id}");
    let result: bool = redis::cmd("SISMEMBER")
        .arg(&key)
        .arg(account_id)
        .query_async(conn)
        .await?;
    Ok(result)
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

    async fn cleanup(conn: &mut RedisConnection, product_id: &str) {
        let _: () = redis::cmd("DEL")
            .arg(format!("claims:{product_id}"))
            .query_async(&mut *conn)
            .await
            .unwrap_or(());
    }

    /// First claim by an account should return NewClaim.
    #[tokio::test]
    async fn test_new_claim() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_new_claim";

        cleanup(&mut conn, pid).await;

        let result = atomic_claim_check(&mut conn, pid, "acct_1").await.unwrap();
        assert_eq!(result, ClaimResult::NewClaim);

        assert!(has_claimed(&mut conn, pid, "acct_1").await.unwrap());

        cleanup(&mut conn, pid).await;
    }

    /// Duplicate claim should return AlreadyClaimed.
    #[tokio::test]
    async fn test_duplicate_claim() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_dup_claim";

        cleanup(&mut conn, pid).await;

        let r1 = atomic_claim_check(&mut conn, pid, "acct_1").await.unwrap();
        assert_eq!(r1, ClaimResult::NewClaim);

        let r2 = atomic_claim_check(&mut conn, pid, "acct_1").await.unwrap();
        assert_eq!(r2, ClaimResult::AlreadyClaimed);

        // Set should contain exactly one member
        let members = get_claimed_accounts(&mut conn, pid).await.unwrap();
        assert_eq!(members.len(), 1);

        cleanup(&mut conn, pid).await;
    }

    /// Different accounts should each get NewClaim.
    #[tokio::test]
    async fn test_different_accounts() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_diff_accts";

        cleanup(&mut conn, pid).await;

        let r1 = atomic_claim_check(&mut conn, pid, "acct_1").await.unwrap();
        let r2 = atomic_claim_check(&mut conn, pid, "acct_2").await.unwrap();
        assert_eq!(r1, ClaimResult::NewClaim);
        assert_eq!(r2, ClaimResult::NewClaim);

        let members = get_claimed_accounts(&mut conn, pid).await.unwrap();
        assert_eq!(members.len(), 2);

        cleanup(&mut conn, pid).await;
    }

    /// CRITICAL: 1000 concurrent requests from the same account.
    /// Exactly 1 should produce NewClaim; the rest must be AlreadyClaimed.
    #[test]
    fn test_concurrent_duplicate_claims() {
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
        let pid = "concurrent_claim";

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            cleanup(&mut conn, pid).await;
        });

        let mut handles = vec![];
        for _ in 0..1000 {
            let p = pool.clone();
            let id = pid.to_string();
            let handle = std::thread::Builder::new()
                .spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut conn = match p.get().await {
                            Ok(c) => c,
                            Err(_) => return Ok::<_, anyhow::Error>(ClaimResult::AlreadyClaimed),
                        };
                        match atomic_claim_check(&mut conn, &id, "same_account").await {
                            Ok(result) => Ok(result),
                            Err(_) => Ok(ClaimResult::AlreadyClaimed),
                        }
                    })
                });
            match handle {
                Ok(h) => handles.push(h),
                Err(_) => {} // OS thread limit reached, skip
            }
        }

        let mut new_claims = 0i64;
        let mut already_claimed = 0i64;
        for handle in handles {
            match handle.join().unwrap() {
                Ok(ClaimResult::NewClaim) => new_claims += 1,
                Ok(ClaimResult::AlreadyClaimed) => already_claimed += 1,
                Err(_) => {} // connection errors treated as non-claim
            }
        }

        assert_eq!(new_claims, 1, "Exactly 1 new claim should succeed");

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            let members = get_claimed_accounts(&mut conn, pid).await.unwrap();
            assert_eq!(members.len(), 1);
            cleanup(&mut conn, pid).await;
        });
    }
}
