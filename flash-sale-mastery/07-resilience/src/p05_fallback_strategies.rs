//! # Exercise 05: Fallback Strategies
//!
//! ## Learning Objective
//!
//! Implement fallback strategies that provide graceful degradation when primary
//! operations fail. Instead of returning an error to the user, a fallback provides
//! an alternative response -- perhaps a cached result, a default value, or a
//! queued write for later processing.
//!
//! ## Flash Sale Context
//!
//! During a flash sale, the stock check service might become unavailable. Rather
//! than showing users an error page, a fallback can return "temporarily unavailable"
//! or serve stale cached data. Similarly, if the database is overloaded for writing
//! order records, the system can queue the write to a local buffer and acknowledge
//! the user immediately, processing the write when the database recovers.
//!
//! ## Instructions
//!
//! 1. Implement `with_fallback` that tries a primary operation, then a fallback on failure
//! 2. Implement `with_fallback_value` that returns a default value on failure
//! 3. Implement `FallbackChain` that tries multiple strategies in sequence
//! 4. Create example fallback strategies for stock checks and order writes
//!
//! ## Hints
//!
//! - The primary and fallback can have different internal implementations but same return type
//! - Use `tracing::warn!` to log when fallback is triggered
//! - `FallbackChain` can store a Vec of boxed async closures
//! - For the fallback value variant, the value is cloned on each call

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during fallback operations.
#[derive(Debug, Error)]
pub enum FallbackError {
    #[error("all fallback strategies exhausted: {last_error}")]
    AllExhausted { last_error: String },
    #[error("operation failed: {0}")]
    OperationFailed(String),
}

/// Execute a primary operation with a fallback.
///
/// If the primary operation fails, the fallback is called with the error.
/// If both fail, the fallback's error is returned.
///
/// # Arguments
/// * `primary` - The primary async operation to try first
/// * `fallback` - The fallback async operation to try if primary fails
///
/// # Returns
/// `Ok(T)` from whichever operation succeeds first, or the fallback's error
pub async fn with_fallback<PF, FF, T, E>(primary: PF, fallback: FF) -> Result<T, FallbackError>
where
    PF: std::future::Future<Output = Result<T, E>>,
    FF: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    // TODO: Implement fallback logic:
    //   1. Try the primary operation
    //   2. If it succeeds, return the result
    //   3. If it fails, log a warning and try the fallback
    //   4. Map errors from both operations to FallbackError
    todo!("Implement with_fallback")
}

/// Execute an operation with a default fallback value.
///
/// If the operation fails, returns the provided default value instead of an error.
/// Use this for non-critical operations where a reasonable default exists.
///
/// # Arguments
/// * `op` - The async operation to try
/// * `default` - The default value to return on failure (must implement Clone)
///
/// # Returns
/// `Ok(T)` from the operation, or `Ok(default)` if it fails
pub async fn with_fallback_value<F, T, E>(op: F, default: T) -> Result<T, FallbackError>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    T: Clone,
{
    // TODO: Implement fallback value logic:
    //   1. Try the operation
    //   2. If it succeeds, return the result
    //   3. If it fails, log a warning and return the cloned default
    todo!("Implement with_fallback_value")
}

/// A chain of fallback strategies tried in sequence.
///
/// Stores multiple async operations and tries them in order until one succeeds
/// or all are exhausted.
pub struct FallbackChain<T> {
    // TODO: Add a field to store fallback strategies
    // Hint: Vec<Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<T, String>>>>>>
    _phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> FallbackChain<T> {
    /// Create a new empty fallback chain.
    pub fn new() -> Self {
        // TODO: Initialize the chain
        todo!("Implement FallbackChain::new")
    }

    /// Add a fallback strategy to the chain.
    ///
    /// Strategies are tried in the order they are added.
    pub fn add_strategy<F, Fut>(&mut self, _strategy: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<T, String>> + Send + 'static,
    {
        // TODO: Box and store the strategy
        todo!("Implement FallbackChain::add_strategy")
    }

    /// Execute the fallback chain, trying each strategy in order.
    ///
    /// # Returns
    /// `Ok(T)` from the first successful strategy, or `FallbackError::AllExhausted`
    /// if all strategies fail
    pub async fn execute(&self) -> Result<T, FallbackError> {
        // TODO: Iterate through strategies, return first success
        todo!("Implement FallbackChain::execute")
    }
}

/// Example: Stock check with fallback to "temporarily unavailable".
///
/// Tries to check stock from the primary source. If that fails, returns a
/// fallback response indicating the service is temporarily unavailable.
pub async fn stock_check_with_fallback(
    check_fn: impl std::future::Future<Output = Result<bool, String>>,
) -> Result<String, FallbackError> {
    // TODO: Try the stock check, fall back to "temporarily unavailable" message
    todo!("Implement stock_check_with_fallback")
}

/// Example: Order write with fallback to queue for retry.
///
/// Tries to write the order to the database. If that fails, queues it in a
/// local buffer for later processing.
pub async fn order_write_with_fallback(
    write_fn: impl std::future::Future<Output = Result<String, String>>,
    order_data: &str,
) -> Result<String, FallbackError> {
    // TODO: Try the write, fall back to queuing the order data
    // Return a message like "order queued for processing: {order_id}"
    todo!("Implement order_write_with_fallback")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_primary_succeeds_fallback_not_called() {
        let result = with_fallback(
            async { Ok::<_, &str>("primary result") },
            async { Ok::<_, &str>("fallback result") },
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "primary result");
    }

    #[tokio::test]
    async fn test_primary_fails_fallback_used() {
        let result = with_fallback(
            async { Err::<&str, _>("primary failed") },
            async { Ok::<_, &str>("fallback result") },
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fallback result");
    }

    #[tokio::test]
    async fn test_both_fail_returns_fallback_error() {
        let result: Result<&str, FallbackError> = with_fallback(
            async { Err::<&str, _>("primary failed") },
            async { Err::<&str, _>("fallback also failed") },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fallback_value_returns_default_on_failure() {
        let result = with_fallback_value(
            async { Err::<&str, _>("operation failed") },
            "default_value",
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "default_value");
    }

    #[tokio::test]
    async fn test_fallback_value_returns_result_on_success() {
        let result = with_fallback_value(
            async { Ok::<_, &str>("actual_value") },
            "default_value",
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "actual_value");
    }

    #[tokio::test]
    async fn test_fallback_chain_first_succeeds() {
        let mut chain = FallbackChain::new();
        chain.add_strategy(|| async { Ok::<_, String>("first".to_string()) });
        chain.add_strategy(|| async { Ok::<_, String>("second".to_string()) });

        let result = chain.execute().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "first");
    }

    #[tokio::test]
    async fn test_fallback_chain_skips_failures() {
        let mut chain = FallbackChain::new();
        chain.add_strategy(|| async { Err::<String, _>("first failed".to_string()) });
        chain.add_strategy(|| async { Ok::<_, String>("second succeeded".to_string()) });

        let result = chain.execute().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "second succeeded");
    }

    #[tokio::test]
    async fn test_fallback_chain_all_exhausted() {
        let mut chain = FallbackChain::new();
        chain.add_strategy(|| async { Err::<String, _>("fail 1".to_string()) });
        chain.add_strategy(|| async { Err::<String, _>("fail 2".to_string()) });

        let result: Result<String, FallbackError> = chain.execute().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stock_check_fallback() {
        let result = stock_check_with_fallback(async { Ok::<_, String>(true) }).await;
        assert!(result.is_ok());
        let result = stock_check_with_fallback(async { Err::<bool, String>("redis down".to_string()) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "temporarily unavailable");
    }

    #[tokio::test]
    async fn test_order_write_fallback() {
        let result = order_write_with_fallback(
            async { Ok::<_, String>("order-123".to_string()) },
            "order_data",
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "order-123");

        let result = order_write_with_fallback(
            async { Err::<String, String>("db overload".to_string()) },
            "order_data",
        )
        .await;
        assert!(result.is_ok());
        // Should indicate the order was queued
        let msg = result.unwrap();
        assert!(msg.contains("queued") || msg.contains("order"));
    }
}
