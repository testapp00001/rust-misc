//! # Fallback Strategies
//!
//! Provides degraded-mode responses when external services are unavailable.

use crate::models::stock::StockInfo;

/// Fallback result when the primary path is unavailable.
#[derive(Debug, Clone)]
pub enum FallbackResult<T> {
    /// The primary path succeeded.
    Ok(T),
    /// A degraded response was served from cache or defaults.
    Fallback(T),
    /// Both the primary path and fallback failed.
    Failed(String),
}

/// Fallback stock data served when Redis is unavailable.
///
/// Returns "sold out" conservatively -- it is safer to reject a request
/// than to oversell when we cannot verify stock.
pub fn stock_fallback(product_id: &str) -> StockInfo {
    tracing::warn!(product_id = product_id, "Serving stock fallback (sold out)");
    StockInfo {
        product_id: product_id.to_string(),
        stock_remaining: 0,
        total_vouchers: 0,
        sale_active: false,
    }
}

/// Execute an async operation with a fallback.
///
/// If `primary` succeeds, its value is returned. Otherwise, `fallback_fn`
/// is called to produce a degraded response.
pub async fn with_fallback<T, F, FF>(
    primary: F,
    fallback_fn: FF,
) -> FallbackResult<T>
where
    F: std::future::Future<Output = Result<T, String>>,
    FF: FnOnce() -> T,
{
    match primary.await {
        Ok(value) => FallbackResult::Ok(value),
        Err(err) => {
            tracing::error!(error = %err, "Primary path failed, using fallback");
            FallbackResult::Fallback(fallback_fn())
        }
    }
}

/// Execute an async operation, returning a default if it fails.
pub async fn with_default<T: Default, F>(primary: F) -> T
where
    F: std::future::Future<Output = Result<T, String>>,
{
    match primary.await {
        Ok(value) => value,
        Err(err) => {
            tracing::error!(error = %err, "Operation failed, using default");
            T::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_fallback_is_sold_out() {
        let info = stock_fallback("prod-1");
        assert_eq!(info.stock_remaining, 0);
        assert_eq!(info.product_id, "prod-1");
        assert!(!info.sale_active);
    }

    #[tokio::test]
    async fn test_with_fallback_ok() {
        let future = async { Ok::<i32, String>(42) };
        let result: FallbackResult<i32> = with_fallback(future, || 0).await;
        match result {
            FallbackResult::Ok(v) => assert_eq!(v, 42),
            _ => panic!("Expected Ok"),
        }
    }

    #[tokio::test]
    async fn test_with_fallback_error() {
        let future = async { Err::<i32, String>("connection refused".to_string()) };
        let result: FallbackResult<i32> = with_fallback(future, || 99).await;
        match result {
            FallbackResult::Fallback(v) => assert_eq!(v, 99),
            _ => panic!("Expected Fallback"),
        }
    }

    #[tokio::test]
    async fn test_with_default_ok() {
        let future = async { Ok::<i32, String>(7) };
        let val: i32 = with_default(future).await;
        assert_eq!(val, 7);
    }

    #[tokio::test]
    async fn test_with_default_error() {
        let future = async { Err::<i32, String>("fail".to_string()) };
        let val: i32 = with_default(future).await;
        assert_eq!(val, 0); // i32::default()
    }
}
