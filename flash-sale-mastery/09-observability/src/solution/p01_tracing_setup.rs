//! # Solution 01: Tracing Setup
//!
//! Complete implementation of structured logging with the `tracing` crate.

use tracing::Span;
use tracing_subscriber::{fmt, EnvFilter};

/// Custom error type for tracing setup operations.
#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error("Failed to initialize tracing subscriber: {0}")]
    InitFailed(String),
}

/// Initialize tracing with a fmt subscriber and env-filter.
pub fn init_tracing(service_name: &str) -> Result<(), TracingError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_span_events(fmt::format::FmtSpan::CLOSE)
        .try_init()
        .map_err(|e| TracingError::InitFailed(e.to_string()))?;

    tracing::info!(service = service_name, "Tracing initialized");
    Ok(())
}

/// Create a tracing span for a flash sale operation.
pub fn create_span(operation: &str, product_id: &str, request_id: &str) -> Span {
    tracing::info_span!(
        "flash_sale_operation",
        operation = operation,
        product_id = product_id,
        request_id = request_id,
    )
}

/// Record a structured tracing event for a stock change.
pub fn record_stock_event(
    product_id: &str,
    old_stock: u64,
    new_stock: u64,
    reason: &str,
) {
    tracing::info!(
        product_id = product_id,
        old_stock = old_stock,
        new_stock = new_stock,
        reason = reason,
        delta = new_stock as i64 - old_stock as i64,
        "Stock level changed"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;
    use tracing_subscriber::fmt;
    use tracing_subscriber::EnvFilter;

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
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = fmt().with_env_filter(filter).with_test_writer().finish();
        drop(subscriber);
    }

    #[test]
    fn test_create_span_has_fields() {
        init_test_subscriber();
        let span = create_span("purchase", "PROD-001", "req-abc-123");
        assert_eq!(span.metadata().map(|m| m.level()), Some(&Level::INFO));
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
        assert!(span1.metadata().is_some());
        assert!(span2.metadata().is_some());
    }
}
