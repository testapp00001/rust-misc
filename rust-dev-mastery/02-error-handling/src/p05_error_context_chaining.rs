//! # Lesson 5: Error Context Chaining
//!
//! Adding context to errors as they propagate makes debugging much easier.
//! This lesson covers error chains, preserving root causes, and the
//! wrap/with_context patterns from anyhow and custom implementations.

use std::fmt;

// ---------------------------------------------------------------------------
// Custom context wrapper
// ---------------------------------------------------------------------------

/// An error with a chain of context messages.
#[derive(Debug)]
pub struct ChainedError {
    /// Messages added at each layer, from outermost to innermost.
    pub context_chain: Vec<String>,
    /// The original root cause.
    pub root_cause: Box<dyn std::error::Error + Send + Sync>,
}

impl fmt::Display for ChainedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, ctx) in self.context_chain.iter().enumerate() {
            if i > 0 {
                write!(f, "\n  caused by: ")?;
            }
            write!(f, "{}", ctx)?;
        }
        write!(f, "\n  caused by: {}", self.root_cause)
    }
}

impl std::error::Error for ChainedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.root_cause.as_ref())
    }
}

/// Trait for adding context to Results.
pub trait WithContext<T> {
    fn context(self, msg: impl Into<String>) -> Result<T, ChainedError>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> WithContext<T> for Result<T, E> {
    fn context(self, msg: impl Into<String>) -> Result<T, ChainedError> {
        self.map_err(|e| ChainedError {
            context_chain: vec![msg.into()],
            root_cause: Box::new(e),
        })
    }
}

/// Extension trait for adding context to already-chained errors.
pub trait WithChainedContext<T> {
    fn context(self, msg: impl Into<String>) -> Result<T, ChainedError>;
}

impl<T> WithChainedContext<T> for Result<T, ChainedError> {
    fn context(self, msg: impl Into<String>) -> Result<T, ChainedError> {
        self.map_err(|mut e| {
            e.context_chain.insert(0, msg.into());
            e
        })
    }
}

// ---------------------------------------------------------------------------
// Domain-specific error with context
// ---------------------------------------------------------------------------

/// A database operation error with layered context.
#[derive(Debug)]
pub enum DbError {
    ConnectionFailed(std::io::Error),
    QueryFailed(String),
    PoolError(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::ConnectionFailed(e) => write!(f, "connection failed: {}", e),
            DbError::QueryFailed(msg) => write!(f, "query failed: {}", msg),
            DbError::PoolError(msg) => write!(f, "pool error: {}", msg),
        }
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DbError::ConnectionFailed(e) => Some(e),
            _ => None,
        }
    }
}

/// A service-level operation that adds context at each layer.
pub fn get_user_profile(user_id: u64) -> Result<UserProfile, ChainedError> {
    let conn = establish_connection()
        .map_err(|e| ChainedError {
            context_chain: vec![format!("establishing connection for user {}", user_id)],
            root_cause: Box::new(e),
        })?;

    let row = execute_query(&conn, user_id)
        .map_err(|e| ChainedError {
            context_chain: vec![format!("fetching user {} profile", user_id)],
            root_cause: Box::new(e),
        })?;

    parse_profile(row)
        .map_err(|mut e: ChainedError| {
            e.context_chain.push(format!("parsing user {} data", user_id));
            e
        })
}

#[derive(Debug, PartialEq)]
pub struct UserProfile {
    pub id: u64,
    pub name: String,
    pub email: String,
}

fn establish_connection() -> Result<String, DbError> {
    // Simulate connection
    Err(DbError::ConnectionFailed(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "connection refused on port 5432",
    )))
}

fn execute_query(_conn: &str, _user_id: u64) -> Result<String, DbError> {
    Err(DbError::QueryFailed("table 'users' does not exist".into()))
}

fn parse_profile(_row: String) -> Result<UserProfile, ChainedError> {
    Err(ChainedError {
        context_chain: vec!["invalid data format".into()],
        root_cause: Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected JSON, got binary",
        )),
    })
}

// ---------------------------------------------------------------------------
// Error chain inspection utilities
// ---------------------------------------------------------------------------

/// Walk an error chain and collect all messages.
pub fn collect_chain(err: &dyn std::error::Error) -> Vec<String> {
    let mut messages = vec![err.to_string()];
    let mut current = err.source();
    while let Some(source) = current {
        messages.push(source.to_string());
        current = source.source();
    }
    messages
}

/// Check if an error chain contains an error of a specific type.
pub fn chain_contains_io_error(err: &(dyn std::error::Error + 'static)) -> bool {
    if err.downcast_ref::<std::io::Error>().is_some() {
        return true;
    }
    let mut current = err.source();
    while let Some(source) = current {
        if source.downcast_ref::<std::io::Error>().is_some() {
            return true;
        }
        current = source.source();
    }
    false
}

/// Count the depth of an error chain.
pub fn chain_depth(err: &dyn std::error::Error) -> usize {
    let mut depth = 1;
    let mut current = err.source();
    while let Some(source) = current {
        depth += 1;
        current = source.source();
    }
    depth
}

// ---------------------------------------------------------------------------
// Scoped context (like tracing spans for errors)
// ---------------------------------------------------------------------------

/// A scoped context that automatically prepends to errors.
pub struct ErrorScope {
    name: String,
}

impl ErrorScope {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Execute a closure within this error scope.
    pub fn run<T, E, F>(&self, f: F) -> Result<T, ChainedError>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        f().context(format!("[{}]", self.name))
    }

    /// Execute a closure that may return a ChainedError.
    pub fn run_chained<T, F>(&self, f: F) -> Result<T, ChainedError>
    where
        F: FnOnce() -> Result<T, ChainedError>,
    {
        f().map_err(|mut e: ChainedError| {
            e.context_chain.push(format!("[{}]", self.name));
            e
        })
    }
}

// ---------------------------------------------------------------------------
// Builder for multi-step operations with context
// ---------------------------------------------------------------------------

/// A pipeline builder that accumulates context at each step.
pub struct Pipeline<T> {
    result: Result<T, ChainedError>,
}

impl<T> Pipeline<T> {
    pub fn start(value: T) -> Self {
        Self { result: Ok(value) }
    }

    pub fn then<U, E, F>(self, name: &str, f: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> Result<U, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        Pipeline {
            result: self
                .result
                .and_then(|v| f(v).context(format!("step '{}'", name))),
        }
    }

    pub fn finish(self) -> Result<T, ChainedError> {
        self.result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_chained_error_display() {
        let err = ChainedError {
            context_chain: vec!["step 3".into(), "step 2".into(), "step 1".into()],
            root_cause: Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file missing",
            )),
        };
        let msg = err.to_string();
        assert!(msg.contains("step 3"));
        assert!(msg.contains("step 2"));
        assert!(msg.contains("step 1"));
        assert!(msg.contains("file missing"));
    }

    #[test]
    fn test_chained_error_source() {
        let err = ChainedError {
            context_chain: vec!["context".into()],
            root_cause: Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "root",
            )),
        };
        assert!(err.source().is_some());
        assert!(err.source().unwrap().to_string().contains("root"));
    }

    #[test]
    fn test_with_context() {
        let result: Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"));
        let err = result.context("step 1").unwrap_err();
        assert_eq!(err.context_chain.len(), 1);
        assert_eq!(err.context_chain[0], "step 1");
    }

    #[test]
    fn test_chained_context() {
        let result: Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::Other, "root"));
        // The first .context() uses WithContext (Result<T, io::Error>).
        // Subsequent .context() calls use WithChainedContext (Result<T, ChainedError>),
        // disambiguated via UFCS to avoid ambiguity with WithContext.
        let step1: Result<(), ChainedError> = result.context("step 1");
        let step2 = WithChainedContext::context(step1, "step 2");
        let step3 = WithChainedContext::context(step2, "step 3");
        let err = step3.unwrap_err();

        assert_eq!(err.context_chain.len(), 3);
        assert_eq!(err.context_chain[0], "step 3");
        assert_eq!(err.context_chain[1], "step 2");
        assert_eq!(err.context_chain[2], "step 1");
    }

    #[test]
    fn test_collect_chain() {
        let err = ChainedError {
            context_chain: vec!["ctx".into()],
            root_cause: Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "root",
            )),
        };
        let chain = collect_chain(&err);
        assert_eq!(chain.len(), 2);
        assert!(chain[0].contains("ctx"));
        assert!(chain[1].contains("root"));
    }

    #[test]
    fn test_chain_contains_io_error() {
        let err = ChainedError {
            context_chain: vec!["ctx".into()],
            root_cause: Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "root",
            )),
        };
        assert!(chain_contains_io_error(&err));
    }

    #[test]
    fn test_chain_does_not_contain_io_error() {
        let err = DbError::QueryFailed("bad query".into());
        assert!(!chain_contains_io_error(&err));
    }

    #[test]
    fn test_chain_depth() {
        let err = ChainedError {
            context_chain: vec!["ctx".into()],
            root_cause: Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "root",
            )),
        };
        assert_eq!(chain_depth(&err), 2);
    }

    #[test]
    fn test_error_scope() {
        let scope = ErrorScope::new("database");
        let result: Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"));
        let scoped = scope.run(|| result);
        let err = scoped.unwrap_err();
        assert!(err.context_chain.iter().any(|c| c.contains("database")));
    }

    #[test]
    fn test_error_scope_chained() {
        let scope = ErrorScope::new("outer");
        let inner = ErrorScope::new("inner");
        let result: Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::Other, "root"));

        let scoped = scope.run_chained(|| inner.run(|| result));
        let err = scoped.unwrap_err();
        assert!(err.context_chain.iter().any(|c| c.contains("outer")));
        assert!(err.context_chain.iter().any(|c| c.contains("inner")));
    }

    #[test]
    fn test_pipeline_success() {
        let result = Pipeline::start(10)
            .then("double", |v| Ok::<_, std::io::Error>(v * 2))
            .then("add_one", |v| Ok::<_, std::io::Error>(v + 1))
            .finish()
            .unwrap();

        assert_eq!(result, 21);
    }

    #[test]
    fn test_pipeline_failure() {
        let result = Pipeline::start(10)
            .then("step_ok", |v| Ok::<_, std::io::Error>(v * 2))
            .then("step_fail", |_v| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "boom",
                ))
            })
            .then("step_never", |v: i32| Ok::<_, std::io::Error>(v + 1))
            .finish();

        let err = result.unwrap_err();
        assert!(err.context_chain.iter().any(|c| c.contains("step_fail")));
        // step_never should not appear
        assert!(!err.context_chain.iter().any(|c| c.contains("step_never")));
    }

    #[test]
    fn test_get_user_profile_context_chain() {
        let result = get_user_profile(42);
        let err = result.unwrap_err();
        let chain = &err.context_chain;
        // Should have context from multiple layers
        assert!(chain.iter().any(|c| c.contains("establishing connection")));
    }
}
