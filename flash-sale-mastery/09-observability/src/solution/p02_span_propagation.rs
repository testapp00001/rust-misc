//! # Solution 02: Span Propagation
//!
//! Complete implementation of W3C Trace Context propagation.

use std::collections::HashMap;
use uuid::Uuid;

/// A distributed trace context following W3C Trace Context.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceContext {
    pub trace_id: String,
    pub parent_id: String,
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
pub fn create_trace_context() -> TraceContext {
    let trace_id = Uuid::new_v4().to_string().replace('-', "");
    let parent_id = Uuid::new_v4().to_string().replace('-', "")[..16].to_string();

    TraceContext {
        trace_id,
        parent_id,
        flags: "01".to_string(),
    }
}

/// Inject trace context into HTTP headers (W3C Trace Context format).
pub fn propagate_context(ctx: &TraceContext) -> HashMap<String, String> {
    let traceparent = format!("00-{}-{}-{}", ctx.trace_id, ctx.parent_id, ctx.flags);
    let mut headers = HashMap::new();
    headers.insert("traceparent".to_string(), traceparent);
    headers
}

/// Extract trace context from HTTP headers.
pub fn extract_context(
    headers: &HashMap<String, String>,
) -> Result<TraceContext, PropagationError> {
    let traceparent = headers
        .iter()
        .find(|(k, _)| k.to_lowercase() == "traceparent")
        .map(|(_, v)| v.as_str())
        .ok_or(PropagationError::MissingHeader)?;

    let parts: Vec<&str> = traceparent.split('-').collect();
    if parts.len() != 4 {
        return Err(PropagationError::InvalidFormat(format!(
            "expected 4 parts, got {}",
            parts.len()
        )));
    }

    let version = parts[0];
    let trace_id = parts[1];
    let parent_id = parts[2];
    let flags = parts[3];

    if version != "00" {
        return Err(PropagationError::InvalidFormat(format!(
            "unsupported version: {version}"
        )));
    }

    if trace_id.len() != 32 || !trace_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(PropagationError::InvalidTraceId(trace_id.to_string()));
    }

    if parent_id.len() != 16 || !parent_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(PropagationError::InvalidParentId(parent_id.to_string()));
    }

    Ok(TraceContext {
        trace_id: trace_id.to_string(),
        parent_id: parent_id.to_string(),
        flags: flags.to_string(),
    })
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
