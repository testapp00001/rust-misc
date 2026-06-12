//! # Solution 05: Fallback Strategies
//!
//! Complete implementation of fallback strategies for graceful degradation
//! when primary operations fail.

use std::pin::Pin;
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
/// If the primary fails, the fallback is tried. If both fail, the fallback's
/// error is returned.
pub async fn with_fallback<PF, FF, T, E>(primary: PF, fallback: FF) -> Result<T, FallbackError>
where
    PF: std::future::Future<Output = Result<T, E>>,
    FF: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    match primary.await {
        Ok(value) => Ok(value),
        Err(e) => {
            tracing::warn!("primary operation failed: {e}, trying fallback");
            fallback
                .await
                .map_err(|e| FallbackError::OperationFailed(e.to_string()))
        }
    }
}

/// Execute an operation with a default fallback value.
///
/// If the operation fails, returns the provided default value instead of an error.
pub async fn with_fallback_value<F, T, E>(op: F, default: T) -> Result<T, FallbackError>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    T: Clone,
{
    match op.await {
        Ok(value) => Ok(value),
        Err(e) => {
            tracing::warn!("operation failed: {e}, using default value");
            Ok(default)
        }
    }
}

/// A chain of fallback strategies tried in sequence.
pub struct FallbackChain<T> {
    strategies:
        Vec<Box<dyn Fn() -> Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>> + Send + Sync>>,
}

impl<T: 'static> FallbackChain<T> {
    /// Create a new empty fallback chain.
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    /// Add a fallback strategy to the chain.
    pub fn add_strategy<F, Fut>(&mut self, strategy: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<T, String>> + Send + 'static,
    {
        self.strategies.push(Box::new(move || Box::pin(strategy())));
    }

    /// Execute the fallback chain, trying each strategy in order.
    pub async fn execute(&self) -> Result<T, FallbackError> {
        let mut last_error = String::new();
        for strategy in &self.strategies {
            match strategy().await {
                Ok(value) => return Ok(value),
                Err(e) => {
                    tracing::warn!("fallback strategy failed: {e}");
                    last_error = e;
                }
            }
        }
        Err(FallbackError::AllExhausted { last_error })
    }
}

impl<T: 'static> Default for FallbackChain<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Example: Stock check with fallback to "temporarily unavailable".
pub async fn stock_check_with_fallback(
    check_fn: impl std::future::Future<Output = Result<bool, String>>,
) -> Result<String, FallbackError> {
    with_fallback(
        async {
            let in_stock = check_fn.await?;
            if in_stock {
                Ok("in stock".to_string())
            } else {
                Ok("out of stock".to_string())
            }
        },
        async {
            Ok::<_, String>("temporarily unavailable".to_string())
        },
    )
    .await
}

/// Example: Order write with fallback to queue for retry.
pub async fn order_write_with_fallback(
    write_fn: impl std::future::Future<Output = Result<String, String>>,
    order_data: &str,
) -> Result<String, FallbackError> {
    with_fallback(
        async {
            write_fn
                .await
                .map_err(|e| FallbackError::OperationFailed(e))
        },
        async {
            // In production, this would enqueue to a local buffer or message queue
            let queued_id = format!("queued-{}", &order_data[..std::cmp::min(8, order_data.len())]);
            tracing::warn!("order write failed, queued for retry: {queued_id}");
            Ok(queued_id)
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_primary_succeeds_fallback_not_called() {
        let result = with_fallback(
            async { Ok::<_, String>("primary result") },
            async { Ok::<_, String>("fallback result") },
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
        assert_eq!(result.unwrap(), "in stock");

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
        let msg = result.unwrap();
        assert!(msg.contains("queued") || msg.contains("order"));
    }
}
