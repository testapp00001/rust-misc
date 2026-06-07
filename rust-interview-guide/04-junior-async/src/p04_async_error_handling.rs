/// Problem: Async Error Handling
///
/// Master async error handling.
///
/// Key Concepts:
/// - Result in async
/// - ? operator in async
/// - Error propagation
/// - Custom error types
/// - Error recovery

use tokio::time::{sleep, Duration};

/// Problem 1: Basic async Result
/// Return Result from async
pub async fn basic_result() -> Result<i32, String> {
    Ok(42)
}

/// Problem 2: Async with error
/// Return error from async
pub async fn async_error() -> Result<i32, String> {
    Err("Something went wrong".to_string())
}

/// Problem 3: Async with ? operator
/// Use ? in async
pub async fn async_question_mark() -> Result<i32, String> {
    let value = basic_result().await?;
    Ok(value * 2)
}

/// Problem 4: Async with map_err
/// Map errors in async
pub async fn async_map_err() -> Result<i32, String> {
    async_error().await.map_err(|e| format!("Mapped: {}", e))
}

/// Problem 5: Async with and_then
/// Chain async operations
pub async fn async_and_then() -> Result<i32, String> {
    basic_result().await.and_then(|value| {
        Ok(value * 2)
    })
}

/// Problem 6: Async with or_else
/// Recover from errors
pub async fn async_or_else() -> Result<i32, String> {
    async_error().await.or_else(|_| {
        Ok(0)
    })
}

/// Problem 7: Async with unwrap_or
/// Provide default on error
pub async fn async_unwrap_or() -> i32 {
    async_error().await.unwrap_or(0)
}

/// Problem 8: Async with unwrap_or_else
/// Compute default on error
pub async fn async_unwrap_or_else() -> i32 {
    async_error().await.unwrap_or_else(|_| 42)
}

/// Problem 9: Custom error type
/// Define custom error type
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized,
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

/// Problem 10: Async with custom error
/// Use custom error type
pub async fn async_custom_error() -> Result<i32, AppError> {
    Err(AppError::NotFound("Resource not found".to_string()))
}

/// Problem 11: Async error recovery
/// Recover from errors
pub async fn async_error_recovery() -> i32 {
    match async_error().await {
        Ok(value) => value,
        Err(_) => 42,
    }
}

/// Problem 12: Async with timeout error
/// Handle timeout errors
pub async fn async_timeout_error() -> Result<i32, String> {
    tokio::time::timeout(Duration::from_millis(10), async {
        sleep(Duration::from_millis(100)).await;
        42
    })
    .await
    .map_err(|_| "Timeout".to_string())
}

/// Problem 13: Async with multiple errors
/// Handle multiple error types
pub async fn async_multiple_errors() -> Result<i32, String> {
    let value1 = basic_result().await?;
    let value2 = async_error().await.unwrap_or(0);
    Ok(value1 + value2)
}

/// Problem 14: Async with error context
/// Add context to errors
pub async fn async_error_context() -> Result<i32, String> {
    async_error().await.map_err(|e| format!("Context: {}", e))
}

/// Problem 15: Async with retry
/// Retry on error
pub async fn async_retry() -> Result<i32, String> {
    for _ in 0..3 {
        match basic_result().await {
            Ok(value) => return Ok(value),
            Err(_) => sleep(Duration::from_millis(10)).await,
        }
    }
    Err("All retries failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_result() {
        assert_eq!(basic_result().await, Ok(42));
    }

    #[tokio::test]
    async fn test_async_error() {
        assert!(async_error().await.is_err());
    }

    #[tokio::test]
    async fn test_async_question_mark() {
        assert_eq!(async_question_mark().await, Ok(84));
    }

    #[tokio::test]
    async fn test_async_map_err() {
        assert!(async_map_err().await.is_err());
    }

    #[tokio::test]
    async fn test_async_and_then() {
        assert_eq!(async_and_then().await, Ok(84));
    }

    #[tokio::test]
    async fn test_async_or_else() {
        assert_eq!(async_or_else().await, Ok(0));
    }

    #[tokio::test]
    async fn test_async_unwrap_or() {
        assert_eq!(async_unwrap_or().await, 0);
    }

    #[tokio::test]
    async fn test_async_unwrap_or_else() {
        assert_eq!(async_unwrap_or_else().await, 42);
    }

    #[tokio::test]
    async fn test_async_custom_error() {
        assert!(async_custom_error().await.is_err());
    }

    #[tokio::test]
    async fn test_async_error_recovery() {
        assert_eq!(async_error_recovery().await, 42);
    }

    #[tokio::test]
    async fn test_async_timeout_error() {
        assert!(async_timeout_error().await.is_err());
    }

    #[tokio::test]
    async fn test_async_multiple_errors() {
        assert_eq!(async_multiple_errors().await, Ok(42));
    }

    #[tokio::test]
    async fn test_async_error_context() {
        assert!(async_error_context().await.is_err());
    }

    #[tokio::test]
    async fn test_async_retry() {
        assert_eq!(async_retry().await, Ok(42));
    }
}
