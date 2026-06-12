//! # Exercise 02: Span Propagation
//!
//! ## Learning Objective
//! Learn how to propagate distributed trace context across service boundaries
//! using the W3C Trace Context standard. This enables end-to-end tracing of a
//! single request as it flows through multiple microservices.
//!
//! ## Flash Sale Context
//! A flash sale purchase request hits the API gateway, which calls the inventory
//! service, which calls Redis. Each hop must carry the trace context so that
//! when you look at a trace in your dashboard, you see the full request tree.
//!
//! ## Instructions
//! 1. Implement `create_trace_context` to generate a new trace context
//! 2. Implement `propagate_context` to inject trace context into HTTP headers
//! 3. Implement `extract_context` to extract trace context from HTTP headers
//!
//! ## Hints
//! - W3C Trace Context format: `traceparent: {version}-{trace_id}-{parent_id}-{flags}`
//! - version = "00", trace_id = 32 hex chars, parent_id = 16 hex chars, flags = "01"
//! - Use `uuid::Uuid` for generating unique IDs
//! - Headers are case-insensitive; normalize to lowercase

use std::collections::HashMap;

/// A distributed trace context following W3C Trace Context.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceContext {
    /// 128-bit trace identifier (32 hex characters).
    pub trace_id: String,
    /// 64-bit parent span identifier (16 hex characters).
    pub parent_id: String,
    /// Trace flags (e.g., "01" = sampled).
    pub flags: String,
}

/// Custom error type for context propagation.
#[derive(Debug, thiserror::Error)]
pub enum PropagationError {
    #[error("Missing traceparent header")]
    MissingHeader,

    #[error("Invalid traceparent format: {0}")]
    InvalidFormat(String),

    #[error("Invalid trace ID: {0}")]
    InvalidTraceId(String),

    #[error("Invalid parent ID: {0}")]
    InvalidParentId(String),
}

/// Create a new trace context with random trace and parent IDs.
///
/// The generated context should:
/// - Have a 32-character hex trace_id
/// - Have a 16-character hex parent_id
/// - Have flags set to "01" (sampled)
///
/// # Returns
/// A new `TraceContext` with randomly generated identifiers.
pub fn create_trace_context() -> TraceContext {
    // TODO: Generate a 128-bit trace_id (32 hex chars)
    // TODO: Generate a 64-bit parent_id (16 hex chars)
    // TODO: Set flags to "01"
    todo!("Implement trace context creation")
}

/// Inject trace context into HTTP headers (W3C Trace Context format).
///
/// The `traceparent` header format:
/// `traceparent: 00-{trace_id}-{parent_id}-{flags}`
///
/// # Arguments
/// * `ctx` - The trace context to propagate
///
/// # Returns
/// A HashMap containing the `traceparent` header.
pub fn propagate_context(ctx: &TraceContext) -> HashMap<String, String> {
    // TODO: Format the traceparent header value
    // TODO: Return a HashMap with the "traceparent" key
    todo!("Implement context propagation")
}

/// Extract trace context from HTTP headers.
///
/// Parses the `traceparent` header in W3C Trace Context format.
/// Header lookup should be case-insensitive.
///
/// # Arguments
/// * `headers` - HTTP headers as a HashMap
///
/// # Returns
/// The parsed `TraceContext`, or an error if the header is missing/invalid.
pub fn extract_context(
    headers: &HashMap<String, String>,
) -> Result<TraceContext, PropagationError> {
    // TODO: Find the "traceparent" header (case-insensitive)
    // TODO: Parse the format: "00-{trace_id}-{parent_id}-{flags}"
    // TODO: Validate each component
    // TODO: Return the TraceContext
    todo!("Implement context extraction")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_trace_context_format() {
        let ctx = create_trace_context();
        assert_eq!(ctx.trace_id.len(), 32, "trace_id should be 32 hex chars");
        assert_eq!(ctx.parent_id.len(), 16, "parent_id should be 16 hex chars");
        assert_eq!(ctx.flags, "01", "flags should indicate sampled");
        // Verify hex characters
        assert!(
            ctx.trace_id.chars().all(|c| c.is_ascii_hexdigit()),
            "trace_id should be hex"
        );
        assert!(
            ctx.parent_id.chars().all(|c| c.is_ascii_hexdigit()),
            "parent_id should be hex"
        );
    }

    #[test]
    fn test_propagate_context_format() {
        let ctx = TraceContext {
            trace_id: "0af7651916cd43dd8448eb211c80319c".to_string(),
            parent_id: "b7ad6b7169203331".to_string(),
            flags: "01".to_string(),
        };
        let headers = propagate_context(&ctx);
        let traceparent = headers.get("traceparent").expect("should have traceparent");
        assert_eq!(
            traceparent, "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
        );
    }

    #[test]
    fn test_extract_context_valid() {
        let mut headers = HashMap::new();
        headers.insert(
            "traceparent".to_string(),
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".to_string(),
        );
        let ctx = extract_context(&headers).expect("should parse valid header");
        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
        assert_eq!(ctx.parent_id, "b7ad6b7169203331");
        assert_eq!(ctx.flags, "01");
    }

    #[test]
    fn test_extract_context_case_insensitive() {
        let mut headers = HashMap::new();
        headers.insert(
            "Traceparent".to_string(),
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".to_string(),
        );
        let ctx = extract_context(&headers).expect("should handle case-insensitive headers");
        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
    }

    #[test]
    fn test_extract_context_missing_header() {
        let headers = HashMap::new();
        let result = extract_context(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip() {
        let original = create_trace_context();
        let headers = propagate_context(&original);
        let extracted = extract_context(&headers).expect("roundtrip should work");
        assert_eq!(original, extracted);
    }
}
