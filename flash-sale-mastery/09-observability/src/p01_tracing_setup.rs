//! # Exercise 01: Tracing Setup
//!
//! ## Learning Objective
//! Learn how to configure structured logging with the `tracing` crate.
//! Structured logging produces machine-parseable log entries with typed fields,
//! enabling powerful filtering and analysis that plain `println!` cannot match.
//!
//! ## Flash Sale Context
//! During a flash sale, you need to correlate log entries across services.
//! A single purchase request generates logs in the API gateway, inventory
//! service, payment processor, and order worker. Without structured tracing,
//! grepping through millions of log lines is hopeless.
//!
//! ## Instructions
//! 1. Implement `init_tracing` to set up a fmt subscriber with env-filter
//! 2. Implement `create_span` to create a tracing span with structured fields
//! 3. Implement `record_event` to emit a structured tracing event
//!
//! ## Hints
//! - Use `tracing_subscriber::fmt()` to build a subscriber
//! - Use `tracing_subscriber::EnvFilter` for log level control
//! - `tracing::info_span!()` creates a span with fields
//! - `tracing::info!()` emits an event inside the current span

use tracing::Span;

/// Custom error type for tracing setup operations.
#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error("Failed to initialize tracing subscriber: {0}")]
    InitFailed(String),
}

/// Initialize tracing with a fmt subscriber and env-filter.
///
/// The subscriber should:
/// - Use `EnvFilter` reading from the `RUST_LOG` env var (defaulting to "info")
/// - Output structured fields in the log output
/// - Include the span context in log lines
///
/// # Arguments
/// * `service_name` - Name of the service (used as a target prefix)
///
/// # Returns
/// `Ok(())` on success, or a `TracingError` if initialization fails.
pub fn init_tracing(service_name: &str) -> Result<(), TracingError> {
    // TODO: Create an EnvFilter that reads from RUST_LOG, defaulting to "info"
    // TODO: Build a fmt subscriber with the filter
    // TODO: Set it as the global default subscriber
    todo!("Implement tracing initialization")
}

/// Create a tracing span for a flash sale operation.
///
/// The span should include:
/// - `operation` field: the name of the operation
/// - `product_id` field: the product being operated on
/// - `request_id` field: unique request identifier
///
/// # Arguments
/// * `operation` - Name of the operation (e.g., "purchase", "stock_check")
/// * `product_id` - Product identifier
/// * `request_id` - Unique request identifier
///
/// # Returns
/// A `tracing::Span` with the specified fields.
pub fn create_span(operation: &str, product_id: &str, request_id: &str) -> Span {
    // TODO: Use tracing::info_span! to create a span with the three fields
    todo!("Implement span creation")
}

/// Record a structured tracing event for a stock change.
///
/// # Arguments
/// * `product_id` - Product whose stock changed
/// * `old_stock` - Previous stock level
/// * `new_stock` - New stock level
/// * `reason` - Why the stock changed (e.g., "purchase", "restock")
pub fn record_stock_event(
    product_id: &str,
    old_stock: u64,
    new_stock: u64,
    reason: &str,
) {
    // TODO: Use tracing::info! with structured fields to record the event
    todo!("Implement stock event recording")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;
    use tracing_subscriber::fmt;
    use tracing_subscriber::EnvFilter;

    /// Helper to set up a test subscriber (non-global, so tests don't conflict).
    fn init_test_subscriber() {
        let _ = fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn test_tracing_init_does_not_panic() {
        // We can't easily test global init multiple times, but we can verify
        // the function signature and that a subscriber can be built.
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = fmt().with_env_filter(filter).with_test_writer().finish();
        // If we got here, subscriber construction succeeded
        drop(subscriber);
    }

    #[test]
    fn test_create_span_has_fields() {
        init_test_subscriber();
        let span = create_span("purchase", "PROD-001", "req-abc-123");
        assert_eq!(span.metadata().map(|m| m.level()), Some(&Level::INFO));
        // Enter the span to verify it's valid
        let _guard = span.enter();
    }

    #[test]
    fn test_record_stock_event_does_not_panic() {
        init_test_subscriber();
        record_stock_event("PROD-001", 100, 99, "purchase");
    }

    #[test]
    fn test_create_span_different_operations() {
        init_test_subscriber();
        let span1 = create_span("purchase", "PROD-001", "req-1");
        let span2 = create_span("stock_check", "PROD-002", "req-2");
        // Both spans should be valid
        assert!(span1.metadata().is_some());
        assert!(span2.metadata().is_some());
    }
}
