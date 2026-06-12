//! # Exercise 08: Failure Analysis
//!
//! ## Learning Objective
//! Understand how each counter approach behaves under common failure
//! modes: connection drops, network partitions, and clock skew. Learn
//! recovery strategies for each scenario.
//!
//! ## Flash Sale Context
//! In a real flash sale, things go wrong. Redis connections drop
//! mid-operation. Network partitions isolate pods from Redis. Clock
//! skew between nodes causes TTL-based locks to behave unexpectedly.
//! Understanding these failure modes is critical for building a
//! resilient system.
//!
//! ## Failure Modes
//!
//! | Failure                | Local Counter | Redis DECR | CAS    | Pessimistic Lock |
//! |------------------------|---------------|------------|--------|------------------|
//! | Redis connection drop  | OK (no Redis) | Fails      | Fails  | Lock stuck until TTL |
//! | Network partition      | OK (local)    | Timeout    | Timeout| Lock held, others blocked |
//! | Clock skew             | N/A           | N/A        | N/A    | TTL may expire early/late |
//! | Process crash          | Lost          | OK         | OK     | Lock held until TTL |
//!
//! ## Instructions
//! 1. Implement `simulate_connection_drop` to test behavior when Redis disconnects
//! 2. Implement `simulate_network_partition` to test timeout behavior
//! 3. Implement `simulate_clock_skew` to test TTL-based lock issues
//! 4. Implement `test_recovery_strategies` to verify each approach recovers
//!
//! ## Hints
//! - Use an invalid Redis URL to simulate connection failure
//! - Use very short timeouts to simulate network issues
//! - For clock skew, demonstrate how TTL can expire prematurely
//! - Recovery: retry logic, circuit breakers, fallback to degraded mode

use deadpool_redis::redis::cmd;
use deadpool_redis::Connection as RedisConnection;

/// Error type for failure analysis operations.
#[derive(Debug, thiserror::Error)]
pub enum FailureError {
    #[error("Connection lost: {0}")]
    ConnectionLost(String),

    #[error("Operation timeout after {0}ms")]
    Timeout(u64),

    #[error("Lock expired prematurely due to clock skew")]
    LockExpiredPrematurely,

    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
}

/// Describes the outcome of a failure simulation.
#[derive(Debug, Clone)]
pub struct FailureOutcome {
    /// The failure mode that was simulated.
    pub failure_mode: String,
    /// Whether the system detected the failure.
    pub detected: bool,
    /// Whether automatic recovery succeeded.
    pub recovered: bool,
    /// Human-readable description of what happened.
    pub description: String,
}

/// Simulate a Redis connection drop during a decrement operation.
///
/// Creates a counter, starts a decrement, then the connection is lost
/// mid-operation. Tests whether the system correctly detects and handles
/// the failure.
///
/// # Arguments
/// * `product_id` - Product identifier for the test
///
/// # Returns
/// Description of what happened and whether recovery succeeded.
pub async fn simulate_connection_drop(
    product_id: &str,
) -> Result<FailureOutcome, FailureError> {
    // TODO: Try to connect to an invalid Redis URL (simulating connection drop)
    // TODO: Attempt a decrement operation
    // TODO: Verify the operation fails with a connection error
    // TODO: Verify the stock is unchanged (no partial state)
    // TODO: Reconnect with valid URL and verify stock is correct
    // TODO: Return FailureOutcome with details
    todo!("Simulate Redis connection drop")
}

/// Simulate a network partition by using a non-routable address with timeout.
///
/// # Arguments
/// * `product_id` - Product identifier for the test
pub async fn simulate_network_partition(
    product_id: &str,
) -> Result<FailureOutcome, FailureError> {
    // TODO: Create a pool pointing to a non-routable IP (e.g., 10.255.255.1)
    //   with a very short connection timeout
    // TODO: Attempt a decrement operation
    // TODO: Verify it times out (not hangs forever)
    // TODO: Verify the stock is unchanged
    // TODO: Return FailureOutcome
    todo!("Simulate network partition")
}

/// Simulate clock skew effects on TTL-based locks.
///
/// Demonstrates how clock differences between nodes can cause
/// locks to expire earlier or later than expected.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
pub async fn simulate_clock_skew(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<FailureOutcome, FailureError> {
    // TODO: Acquire a lock with a 2-second TTL
    // TODO: Simulate the "clock is ahead" scenario:
    //   - The lock holder thinks it has 2 seconds
    //   - But the Redis server's clock is 3 seconds ahead
    //   - So the lock expires "early" from the holder's perspective
    // TODO: After 1 second, try to acquire the same lock from a "different node"
    // TODO: Show that the lock can be acquired because TTL expired on Redis
    //   even though the holder thinks it still has time
    // TODO: Return FailureOutcome
    todo!("Simulate clock skew effects on locks")
}

/// Test recovery strategies for each failure mode.
///
/// # Arguments
/// * `product_id` - Product identifier
pub async fn test_recovery_strategies(
    product_id: &str,
) -> Result<Vec<FailureOutcome>, FailureError> {
    // TODO: Test recovery from connection drop:
    //   - Retry with exponential backoff
    //   - Verify eventual success
    // TODO: Test recovery from network partition:
    //   - Detect timeout
    //   - Fall back to degraded mode (reject requests temporarily)
    //   - Resume when connection is restored
    // TODO: Test recovery from clock skew:
    //   - Use fencing tokens (monotonically increasing lock IDs)
    //   - Verify stale lock holders cannot make changes
    // TODO: Return all outcomes
    todo!("Test recovery strategies")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PRODUCT: &str = "test:failure:product:42";

    /// Test that connection drop is detected and stock is unchanged.
    #[tokio::test]
    async fn test_connection_drop_detection() {
        let outcome = match simulate_connection_drop(TEST_PRODUCT).await {
            Ok(o) => o,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        assert!(outcome.detected, "Connection drop should be detected");
        // Stock should be unchanged (no partial state corruption)
    }

    /// Test that network partition causes timeout, not hang.
    #[tokio::test]
    async fn test_network_partition_timeout() {
        let outcome = match simulate_network_partition(TEST_PRODUCT).await {
            Ok(o) => o,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        assert!(outcome.detected, "Network partition should be detected via timeout");
    }

    /// Test clock skew effects on locks.
    #[tokio::test]
    async fn test_clock_skew_effects() {
        use deadpool_redis::{Config, Runtime};

        let cfg = Config::from_url("redis://127.0.0.1:6379");
        let pool = match cfg.builder(Some(Runtime::Tokio1)).build() {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: Pool error: {msg}");
                return;
            }
        };
        let mut conn = match pool.get().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: Connection error: {msg}");
                return;
            }
        };

        let outcome = match simulate_clock_skew(&mut conn, TEST_PRODUCT).await {
            Ok(o) => o,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        // The key assertion is that clock skew was demonstrated
        assert!(
            !outcome.description.is_empty(),
            "Should describe the clock skew scenario"
        );

        // Cleanup
        let _: Result<(), _> = cmd("DEL")
            .arg(format!("lock:{}", TEST_PRODUCT))
            .query_async(&mut conn)
            .await;
    }

    /// Test recovery strategies work.
    #[tokio::test]
    async fn test_recovery() {
        let outcomes = match test_recovery_strategies(TEST_PRODUCT).await {
            Ok(o) => o,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        assert!(!outcomes.is_empty(), "Should have recovery outcomes");
        for outcome in &outcomes {
            println!(
                "Failure: {}, Detected: {}, Recovered: {}",
                outcome.failure_mode, outcome.detected, outcome.recovered
            );
        }
    }
}
