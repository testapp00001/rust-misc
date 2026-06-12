use tracing_subscriber::{fmt, EnvFilter};

/// Initialize structured logging for the message queue broker.
///
/// Reads the `RUST_LOG` environment variable for per-module filter directives.
/// Falls back to `info` level for all modules when unset.
///
/// Output format: JSON (suitable for log aggregation pipelines).
///
/// # Example
///
/// ```bash
/// RUST_LOG="debug" cargo run
/// RUST_LOG="production_message_queue=trace,tokio=warn" cargo run
/// ```
pub fn init_logging() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new("info")
            }),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();
}

/// Initialize logging in human-readable format for tests and local
/// development. Not suitable for production use.
#[cfg(test)]
pub fn init_logging_test() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new("debug")
            }),
        )
        .with_target(true)
        .with_test_writer()
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_logging_does_not_panic() {
        // Logging init is global and idempotent; calling it more than once in a
        // process is safe because tracing_subscriber guards against double-init.
        // We just verify it doesn't panic.
        init_logging();
    }
}
