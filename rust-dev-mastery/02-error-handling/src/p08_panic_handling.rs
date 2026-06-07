//! # Lesson 8: Panic Handling
//!
//! Panics are Rust's mechanism for unrecoverable errors. This lesson covers
//! catch_unwind, panic hooks, abort vs unwind strategies, and panic safety.

use std::panic;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Catching panics with catch_unwind
// ---------------------------------------------------------------------------

/// Execute a closure and catch any panic, returning a Result.
/// This is useful for isolating panicking code (e.g., FFI boundaries).
pub fn catch_panic<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> T + panic::UnwindSafe,
{
    match panic::catch_unwind(f) {
        Ok(value) => Ok(value),
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            Err(msg)
        }
    }
}

/// A safe wrapper that catches panics in closures and converts them to errors.
pub fn safe_execute<F, T, E>(f: F) -> Result<T, ExecutionError<E>>
where
    F: FnOnce() -> Result<T, E> + panic::UnwindSafe,
{
    match panic::catch_unwind(f) {
        Ok(result) => result.map_err(ExecutionError::Error),
        Err(payload) => {
            let msg = extract_panic_message(&payload);
            Err(ExecutionError::Panic(msg))
        }
    }
}

#[derive(Debug)]
pub enum ExecutionError<E> {
    Error(E),
    Panic(String),
}

impl<E: std::fmt::Display> std::fmt::Display for ExecutionError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::Error(e) => write!(f, "operation error: {}", e),
            ExecutionError::Panic(msg) => write!(f, "operation panicked: {}", msg),
        }
    }
}

fn extract_panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

// ---------------------------------------------------------------------------
// Custom panic hooks
// ---------------------------------------------------------------------------

/// A panic hook that captures panic information for logging.
#[derive(Debug, Clone)]
pub struct PanicInfo {
    pub message: String,
    pub location: Option<String>,
    pub thread: String,
}

/// Install a custom panic hook that captures panic info.
/// Returns a shared vector that collects all panic info.
pub fn install_capture_hook() -> Arc<Mutex<Vec<PanicInfo>>> {
    let panics: Arc<Mutex<Vec<PanicInfo>>> = Arc::new(Mutex::new(Vec::new()));
    let panics_clone = panics.clone();

    panic::set_hook(Box::new(move |info| {
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };

        let location = info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()));

        let thread = std::thread::current()
            .name()
            .unwrap_or("<unnamed>")
            .to_string();

        let panic_info = PanicInfo {
            message,
            location,
            thread,
        };

        if let Ok(mut panics) = panics_clone.lock() {
            panics.push(panic_info);
        }
    }));

    panics
}

// ---------------------------------------------------------------------------
// Panic safety: ensuring invariants after panic
// ---------------------------------------------------------------------------

/// A struct that demonstrates panic safety.
/// If a panic occurs during an operation, the struct maintains its
/// previous valid state through RAII guards.
pub struct SafeCounter {
    value: Arc<Mutex<u64>>,
}

impl SafeCounter {
    pub fn new(initial: u64) -> Self {
        Self {
            value: Arc::new(Mutex::new(initial)),
        }
    }

    /// Increment the counter. If `should_panic` is true, panic after
    /// acquiring the lock but before releasing it. The mutex will
    /// be poisoned, but the value is still consistent.
    pub fn increment(&self, should_panic: bool) -> Result<u64, String> {
        let mut value = self.value.lock().map_err(|e| format!("lock: {}", e))?;
        *value += 1;
        if should_panic {
            panic!("intentional panic in increment");
        }
        Ok(*value)
    }

    /// Try to get the value, handling poisoned mutex.
    pub fn get(&self) -> Result<u64, String> {
        self.value
            .lock()
            .map(|v| *v)
            .map_err(|e| format!("mutex poisoned: {}", e))
    }

    /// Recover from a poisoned mutex by clearing the poison and resetting.
    pub fn recover(&self) -> u64 {
        self.value.clear_poison();
        let mut value = self.value.lock().unwrap();
        *value = 0;
        *value
    }
}

// ---------------------------------------------------------------------------
// Abort vs Unwind strategy
// ---------------------------------------------------------------------------

/// Demonstrates the tradeoffs between abort and unwind panic strategies.
pub struct PanicStrategyInfo {
    pub strategy: &'static str,
    pub binary_size: &'static str,
    pub compile_time: &'static str,
    pub use_case: &'static str,
}

pub fn panic_strategy_comparison() -> Vec<PanicStrategyInfo> {
    vec![
        PanicStrategyInfo {
            strategy: "unwind",
            binary_size: "Larger (unwinding tables)",
            compile_time: "Slightly slower",
            use_case: "Default; allows catch_unwind, graceful cleanup",
        },
        PanicStrategyInfo {
            strategy: "abort",
            binary_size: "Smaller (no unwinding tables)",
            compile_time: "Faster",
            use_case: "Production binaries, WASM, embedded; immediate termination",
        },
    ]
}

// ---------------------------------------------------------------------------
// Assert macros and debug assertions
// ---------------------------------------------------------------------------

/// Custom assertion macro that provides better error messages.
macro_rules! assert_valid {
    ($expr:expr, $($arg:tt)*) => {
        if !$expr {
            panic!("assertion failed: {}", format!($($arg)*));
        }
    };
}

/// Validate a configuration, panicking with descriptive messages.
pub fn validate_config_strict(name: &str, port: u16, max_conn: usize) {
    assert_valid!(!name.is_empty(), "config name cannot be empty");
    assert_valid!(port > 0, "port must be > 0, got {}", port);
    assert_valid!(
        max_conn > 0,
        "max_connections must be > 0, got {}",
        max_conn
    );
    assert_valid!(
        max_conn <= 10000,
        "max_connections must be <= 10000, got {}",
        max_conn
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catch_panic_success() {
        let result = catch_panic(|| 42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_catch_panic_with_str() {
        let result: Result<(), _> = catch_panic(|| panic!("oh no"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "oh no");
    }

    #[test]
    fn test_catch_panic_with_string() {
        let result: Result<(), _> = catch_panic(|| panic!("{}", format!("error {}", 42)));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "error 42");
    }

    #[test]
    fn test_catch_panic_unknown() {
        let result: Result<(), _> = catch_panic(|| panic!());
        assert!(result.is_err());
        // A bare panic!() produces "explicit panic" in Rust.
        // "unknown panic" is returned only for non-string panic payloads.
        assert_eq!(result.unwrap_err(), "explicit panic");
    }

    #[test]
    fn test_safe_execute_success() {
        let result = safe_execute(|| Ok::<_, String>(42));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_safe_execute_error() {
        let result = safe_execute(|| Err::<i32, _>("business error".to_string()));
        match result.unwrap_err() {
            ExecutionError::Error(e) => assert_eq!(e, "business error"),
            _ => panic!("expected Error, not Panic"),
        }
    }

    #[test]
    fn test_safe_execute_panic() {
        let result = safe_execute(|| -> Result<i32, String> {
            panic!("unexpected panic");
        });
        match result.unwrap_err() {
            ExecutionError::Panic(msg) => assert_eq!(msg, "unexpected panic"),
            _ => panic!("expected Panic"),
        }
    }

    #[test]
    fn test_execution_error_display() {
        let err: ExecutionError<String> = ExecutionError::Error("test".into());
        assert!(err.to_string().contains("operation error"));

        let err: ExecutionError<String> = ExecutionError::Panic("boom".into());
        assert!(err.to_string().contains("panicked"));
    }

    #[test]
    fn test_panic_hook_captures() {
        let panics = install_capture_hook();

        let _ = catch_panic(|| panic!("hook test 1"));
        let _ = catch_panic(|| panic!("hook test 2"));

        let captured = panics.lock().unwrap();
        assert!(captured.len() >= 2);
        // The hook runs even though catch_unwind catches the panic
    }

    #[test]
    fn test_safe_counter_normal() {
        let counter = SafeCounter::new(0);
        assert_eq!(counter.get().unwrap(), 0);

        counter.increment(false).unwrap();
        assert_eq!(counter.get().unwrap(), 1);

        counter.increment(false).unwrap();
        assert_eq!(counter.get().unwrap(), 2);
    }

    #[test]
    fn test_safe_counter_panic_poisons_mutex() {
        let counter = SafeCounter::new(0);
        let _ = catch_panic(|| counter.increment(true));
        // Mutex is now poisoned
        assert!(counter.get().is_err());
    }

    #[test]
    fn test_safe_counter_recover() {
        let counter = SafeCounter::new(0);
        counter.increment(false).unwrap();
        let _ = catch_panic(|| counter.increment(true));

        // Recover from poisoned mutex
        counter.recover();
        assert_eq!(counter.get().unwrap(), 0);
    }

    #[test]
    fn test_panic_strategy_comparison() {
        let strategies = panic_strategy_comparison();
        assert_eq!(strategies.len(), 2);
        assert_eq!(strategies[0].strategy, "unwind");
        assert_eq!(strategies[1].strategy, "abort");
    }

    #[test]
    fn test_validate_config_strict_valid() {
        validate_config_strict("myapp", 8080, 100);
    }

    #[test]
    #[should_panic(expected = "config name cannot be empty")]
    fn test_validate_config_empty_name() {
        validate_config_strict("", 8080, 100);
    }

    #[test]
    #[should_panic(expected = "port must be > 0")]
    fn test_validate_config_zero_port() {
        validate_config_strict("myapp", 0, 100);
    }

    #[test]
    #[should_panic(expected = "max_connections must be > 0")]
    fn test_validate_config_zero_connections() {
        validate_config_strict("myapp", 8080, 0);
    }

    #[test]
    #[should_panic(expected = "max_connections must be <= 10000")]
    fn test_validate_config_too_many_connections() {
        validate_config_strict("myapp", 8080, 10001);
    }

    #[test]
    fn test_catch_and_inspect_panic_info() {
        let panics = install_capture_hook();
        let _ = catch_panic(|| panic!("test panic info"));

        let captured = panics.lock().unwrap();
        assert!(!captured.is_empty());
        let info = captured.last().unwrap();
        assert_eq!(info.message, "test panic info");
        // Location should be set
        assert!(info.location.is_some());
    }
}
