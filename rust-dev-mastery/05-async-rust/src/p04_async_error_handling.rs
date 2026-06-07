//! # Async Error Handling
//!
//! Error handling in async Rust has unique challenges: spawned tasks panic separately,
//! errors must be propagated across `await` points, and `JoinError` wraps both panics
//! and cancellations. This module covers production patterns for robust error handling.
//!
//! ## Key Concepts
//! - **Result propagation**: The `?` operator works across `.await`
//! - **JoinError**: Wraps task panics and cancellations from `tokio::spawn`
//! - **panic handling**: `catch_unwind` is not available in async; use `AssertUnwindSafe`
//! - **Error context**: Adding context to errors across async boundaries

use std::time::Duration;

/// A structured error type for async service operations.
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceError {
    ConnectionFailed { host: String, reason: String },
    Timeout { operation: String, duration_ms: u64 },
    Serialization(String),
    RateLimited { retry_after_ms: u64 },
    Internal(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::ConnectionFailed { host, reason } => {
                write!(f, "Failed to connect to {host}: {reason}")
            }
            ServiceError::Timeout { operation, duration_ms } => {
                write!(f, "Operation '{operation}' timed out after {duration_ms}ms")
            }
            ServiceError::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            ServiceError::RateLimited { retry_after_ms } => {
                write!(f, "Rate limited, retry after {retry_after_ms}ms")
            }
            ServiceError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for ServiceError {}

/// Demonstrates safe error handling when spawning tasks.
/// Returns a JoinError-aware wrapper that distinguishes panics from cancellations.
pub async fn safe_spawn_and_await<F, T>(task: F) -> Result<T, SpawnError>
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let handle = tokio::spawn(task);

    match handle.await {
        Ok(value) => Ok(value),
        Err(join_err) => {
            if join_err.is_cancelled() {
                Err(SpawnError::Cancelled)
            } else if join_err.is_panic() {
                // Extract the panic message if possible
                let panic_msg = join_err
                    .try_into_panic()
                    .map(|payload| {
                        if let Some(s) = payload.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = payload.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "unknown panic".to_string()
                        }
                    })
                    .unwrap_or_else(|_| "panic payload unavailable".to_string());
                Err(SpawnError::Panicked(panic_msg))
            } else {
                Err(SpawnError::Unknown(format!("{join_err}")))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpawnError {
    Cancelled,
    Panicked(String),
    Unknown(String),
}

/// Demonstrates using `tokio::task::spawn_blocking` with proper error handling.
/// This is the pattern for wrapping synchronous, blocking code in async.
pub async fn blocking_with_timeout<F, T>(
    timeout: Duration,
    task: F,
) -> Result<T, BlockingError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let result = tokio::time::timeout(timeout, tokio::task::spawn_blocking(task)).await;

    match result {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(join_err)) => Err(BlockingError::TaskFailed(format!("{join_err}"))),
        Err(_elapsed) => Err(BlockingError::TimedOut),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockingError {
    TimedOut,
    TaskFailed(String),
}

/// A resilient fetcher that demonstrates error recovery with fallbacks.
pub struct ResilientFetcher {
    primary: String,
    fallback: String,
}

impl ResilientFetcher {
    pub fn new(primary: impl Into<String>, fallback: impl Into<String>) -> Self {
        ResilientFetcher {
            primary: primary.into(),
            fallback: fallback.into(),
        }
    }

    /// Tries the primary endpoint, falls back to the secondary on failure.
    pub async fn fetch(&self, path: &str) -> Result<String, ServiceError> {
        match self.try_fetch(&self.primary, path).await {
            Ok(data) => Ok(data),
            Err(primary_err) => {
                eprintln!("Primary failed: {primary_err}, trying fallback");
                self.try_fetch(&self.fallback, path).await.map_err(|fallback_err| {
                    ServiceError::Internal(format!(
                        "Both endpoints failed. Primary: {primary_err}. Fallback: {fallback_err}"
                    ))
                })
            }
        }
    }

    async fn try_fetch(&self, endpoint: &str, path: &str) -> Result<String, ServiceError> {
        // Simulate fetch with possible failure
        tokio::time::sleep(Duration::from_millis(5)).await;
        if endpoint == "fail" {
            Err(ServiceError::ConnectionFailed {
                host: endpoint.to_string(),
                reason: "connection refused".to_string(),
            })
        } else {
            Ok(format!("{endpoint}/{path}:data"))
        }
    }
}

/// Aggregates errors from multiple concurrent operations.
/// Unlike try_join! which short-circuits, this collects all errors.
pub async fn collect_results<I, F, Fut, T, E>(items: I, f: F) -> (Vec<T>, Vec<(usize, E)>)
where
    I: IntoIterator,
    F: Fn(I::Item) -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut handles = Vec::new();
    for item in items {
        handles.push(f(item));
    }

    let mut successes = Vec::new();
    let mut errors = Vec::new();

    for (i, result) in futures::future::join_all(handles).await.into_iter().enumerate() {
        match result {
            Ok(value) => successes.push(value),
            Err(e) => errors.push((i, e)),
        }
    }

    (successes, errors)
}

/// A typed error context wrapper for async operations.
/// Adds descriptive context to errors without losing the original type.
#[derive(Debug)]
pub struct ErrorContext<E> {
    pub error: E,
    pub context: Vec<String>,
}

impl<E: std::fmt::Display> std::fmt::Display for ErrorContext<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ctx in &self.context {
            writeln!(f, "  Context: {ctx}")?;
        }
        write!(f, "  Error: {}", self.error)
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for ErrorContext<E> {}

/// Extension trait for adding context to Results in async code.
pub trait ResultExt<T, E> {
    fn with_context(self, context: impl Into<String>) -> Result<T, ErrorContext<E>>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn with_context(self, context: impl Into<String>) -> Result<T, ErrorContext<E>> {
        self.map_err(|error| ErrorContext {
            error,
            context: vec![context.into()],
        })
    }
}

/// Pipeline that processes items through multiple fallible async stages.
/// Each stage can fail, and errors include stage information.
pub struct AsyncPipeline<I> {
    stages: Vec<Box<dyn Fn(I) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<I, ServiceError>> + Send>> + Send + Sync>>,
}

impl<I: Send + 'static> AsyncPipeline<I> {
    pub fn new() -> Self {
        AsyncPipeline { stages: Vec::new() }
    }

    pub fn add_stage<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<I, ServiceError>> + Send + 'static,
    {
        self.stages.push(Box::new(move |input| Box::pin(f(input))));
        self
    }

    pub async fn execute(&self, input: I) -> Result<I, ServiceError> {
        let mut current = input;
        for (stage_idx, stage) in self.stages.iter().enumerate() {
            current = stage(current).await.map_err(|e| {
                ServiceError::Internal(format!("Stage {stage_idx} failed: {e}"))
            })?;
        }
        Ok(current)
    }
}

/// Demonstrates graceful error handling with a fallback chain.
pub async fn with_fallbacks<T, E, F, Fut>(
    operations: Vec<(String, F)>,
) -> Result<T, Vec<(String, E)>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut errors = Vec::new();

    for (name, op) in operations {
        match op().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                errors.push((name, e));
            }
        }
    }

    Err(errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_safe_spawn_success() {
        let result = safe_spawn_and_await(async { 42 }).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_safe_spawn_panic() {
        let result = safe_spawn_and_await(async {
            panic!("test panic message");
        })
        .await;

        match result {
            Err(SpawnError::Panicked(msg)) => assert!(msg.contains("test panic message")),
            other => panic!("Expected Panicked, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_safe_spawn_cancelled() {
        let handle = tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(100)).await;
            42
        });
        handle.abort();

        let result = handle.await;
        assert!(result.is_err());
        assert!(result.unwrap_err().is_cancelled());
    }

    #[tokio::test]
    async fn test_blocking_with_timeout_success() {
        let result = blocking_with_timeout(Duration::from_secs(5), || 42).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_blocking_with_timeout_expires() {
        let result = blocking_with_timeout(Duration::from_millis(10), || {
            std::thread::sleep(Duration::from_secs(10));
            42
        })
        .await;
        assert_eq!(result, Err(BlockingError::TimedOut));
    }

    #[tokio::test]
    async fn test_resilient_fetcher_primary_ok() {
        let fetcher = ResilientFetcher::new("primary", "fallback");
        let result = fetcher.fetch("/data").await;
        assert!(result.unwrap().contains("primary"));
    }

    #[tokio::test]
    async fn test_resilient_fetcher_fallback() {
        let fetcher = ResilientFetcher::new("fail", "fallback");
        let result = fetcher.fetch("/data").await;
        assert!(result.unwrap().contains("fallback"));
    }

    #[tokio::test]
    async fn test_resilient_fetcher_both_fail() {
        let fetcher = ResilientFetcher::new("fail", "fail");
        let result = fetcher.fetch("/data").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Both endpoints failed"));
    }

    #[tokio::test]
    async fn test_collect_results_all_ok() {
        let items = vec![1, 2, 3];
        let (successes, errors) =
            collect_results(items, |n| async move { Ok::<_, String>(n * 2) }).await;
        assert_eq!(successes, vec![2, 4, 6]);
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_collect_results_mixed() {
        let items: Vec<i32> = vec![1, 2, 3, 4, 5];
        let (successes, errors) = collect_results(items, |n| async move {
            if n % 2 == 0 {
                Err(format!("even: {n}"))
            } else {
                Ok(n)
            }
        })
        .await;

        assert_eq!(successes, vec![1, 3, 5]);
        assert_eq!(errors.len(), 2);
    }

    #[tokio::test]
    async fn test_error_context() {
        let result: Result<i32, &str> = Err("something broke");
        let with_ctx = result.with_context("during database query");
        assert!(with_ctx.is_err());
        let ctx = with_ctx.unwrap_err();
        assert!(ctx.context.contains(&"during database query".to_string()));
    }

    #[tokio::test]
    async fn test_async_pipeline() {
        let pipeline = AsyncPipeline::new()
            .add_stage(|n: i32| async move { Ok(n + 1) })
            .add_stage(|n: i32| async move { Ok(n * 2) })
            .add_stage(|n: i32| async move { Ok(n + 10) });

        let result = pipeline.execute(5).await.unwrap();
        // (5+1) * 2 + 10 = 22
        assert_eq!(result, 22);
    }

    #[tokio::test]
    async fn test_async_pipeline_error() {
        let pipeline = AsyncPipeline::new()
            .add_stage(|n: i32| async move { Ok(n + 1) })
            .add_stage(|_n: i32| async move {
                Err(ServiceError::Internal("stage failed".into()))
            });

        let result = pipeline.execute(5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_fallbacks_first_succeeds() {
        // Use a helper function to create uniform closures
        fn make_op(val: i32, fail: bool) -> impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<i32, String>> + Send>> {
            move || {
                if fail {
                    Box::pin(async { Err("fail".to_string()) })
                } else {
                    Box::pin(async move { Ok(val) })
                }
            }
        }

        let result = with_fallbacks(vec![
            ("primary".into(), make_op(42, false)),
            ("fallback".into(), make_op(99, false)),
        ])
        .await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_with_fallbacks_uses_second() {
        fn make_op(val: i32, fail: bool) -> impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<i32, String>> + Send>> {
            move || {
                if fail {
                    Box::pin(async { Err("fail".to_string()) })
                } else {
                    Box::pin(async move { Ok(val) })
                }
            }
        }

        let result = with_fallbacks(vec![
            ("primary".into(), make_op(0, true)),
            ("fallback".into(), make_op(99, false)),
        ])
        .await;
        assert_eq!(result, Ok(99));
    }

    #[tokio::test]
    async fn test_with_fallbacks_all_fail() {
        fn make_op(val: i32, fail: bool) -> impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<i32, String>> + Send>> {
            move || {
                if fail {
                    Box::pin(async move { Err(format!("fail_{val}")) })
                } else {
                    Box::pin(async move { Ok(val) })
                }
            }
        }

        let result: Result<i32, Vec<(String, String)>> = with_fallbacks(vec![
            ("a".into(), make_op(1, true)),
            ("b".into(), make_op(2, true)),
        ])
        .await;
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_service_error_display() {
        let err = ServiceError::ConnectionFailed {
            host: "db.local".into(),
            reason: "timeout".into(),
        };
        assert!(err.to_string().contains("db.local"));
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn test_spawn_error_variants() {
        assert_eq!(
            SpawnError::Cancelled,
            SpawnError::Cancelled
        );
        assert_ne!(
            SpawnError::Panicked("a".into()),
            SpawnError::Panicked("b".into())
        );
    }
}
