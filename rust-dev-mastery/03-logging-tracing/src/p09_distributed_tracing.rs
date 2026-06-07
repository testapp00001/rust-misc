//! # Lesson 9: Distributed Tracing
//!
//! Distributed tracing follows requests across services. This lesson covers
//! trace context propagation, span context, B3/W3C headers, and
//! OpenTelemetry concepts.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Trace context model (W3C Trace Context)
// ---------------------------------------------------------------------------

/// A trace context following the W3C Trace Context specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraceContext {
    /// A unique identifier for the trace (32 hex chars).
    pub trace_id: String,
    /// A unique identifier for this span (16 hex chars).
    pub span_id: String,
    /// The ID of the parent span (None for root spans).
    pub parent_span_id: Option<String>,
    /// Trace flags (e.g., sampled).
    pub trace_flags: u8,
    /// Trace state for vendor-specific data.
    pub trace_state: BTreeMap<String, String>,
}

impl TraceContext {
    /// Create a new root trace context.
    pub fn new_root() -> Self {
        Self {
            trace_id: generate_trace_id(),
            span_id: generate_span_id(),
            parent_span_id: None,
            trace_flags: 1, // sampled
            trace_state: BTreeMap::new(),
        }
    }

    /// Create a child span context.
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: generate_span_id(),
            parent_span_id: Some(self.span_id.clone()),
            trace_flags: self.trace_flags,
            trace_state: self.trace_state.clone(),
        }
    }

    /// Serialize to W3C traceparent header format.
    /// Format: {version}-{trace_id}-{parent_id}-{trace_flags}
    pub fn to_traceparent(&self) -> String {
        format!("00-{}-{:016x}-{:02x}", self.trace_id, 0u64, self.trace_flags)
    }

    /// Parse a W3C traceparent header.
    pub fn from_traceparent(header: &str) -> Result<Self, TraceError> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 {
            return Err(TraceError::InvalidFormat);
        }

        let version = u8::from_str_radix(parts[0], 16)
            .map_err(|_| TraceError::InvalidFormat)?;
        if version != 0 {
            return Err(TraceError::UnsupportedVersion(version));
        }

        let trace_id = parts[1].to_string();
        if trace_id.len() != 32 {
            return Err(TraceError::InvalidTraceId);
        }

        let span_id = parts[2].to_string();
        if span_id.len() != 16 {
            return Err(TraceError::InvalidSpanId);
        }

        let trace_flags = u8::from_str_radix(parts[3], 16)
            .map_err(|_| TraceError::InvalidFormat)?;

        Ok(Self {
            trace_id,
            span_id,
            parent_span_id: None,
            trace_flags,
            trace_state: BTreeMap::new(),
        })
    }

    /// Check if this trace is sampled.
    pub fn is_sampled(&self) -> bool {
        self.trace_flags & 1 == 1
    }
}

#[derive(Debug, PartialEq)]
pub enum TraceError {
    InvalidFormat,
    UnsupportedVersion(u8),
    InvalidTraceId,
    InvalidSpanId,
}

// Generate a random trace ID (32 hex chars = 128 bits)
fn generate_trace_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:032x}", t)
}

// Generate a random span ID (16 hex chars = 64 bits)
fn generate_span_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:016x}", t as u64)
}

// ---------------------------------------------------------------------------
// B3 propagation format
// ---------------------------------------------------------------------------

/// B3 header propagation (Zipkin format).
pub struct B3Propagator;

impl B3Propagator {
    /// Serialize to B3 single header format.
    /// Format: {trace_id}-{span_id}-{sampling_state}-{parent_span_id}
    pub fn to_single_header(ctx: &TraceContext) -> String {
        let sampling = if ctx.is_sampled() { "1" } else { "0" };
        match &ctx.parent_span_id {
            Some(parent) => format!(
                "{}-{}-{}-{}",
                ctx.trace_id, ctx.span_id, sampling, parent
            ),
            None => format!("{}-{}-{}", ctx.trace_id, ctx.span_id, sampling),
        }
    }

    /// Serialize to B3 multi-header format.
    pub fn to_multi_headers(ctx: &TraceContext) -> BTreeMap<String, String> {
        let mut headers = BTreeMap::new();
        headers.insert("X-B3-TraceId".to_string(), ctx.trace_id.clone());
        headers.insert("X-B3-SpanId".to_string(), ctx.span_id.clone());
        if let Some(ref parent) = ctx.parent_span_id {
            headers.insert("X-B3-ParentSpanId".to_string(), parent.clone());
        }
        headers.insert(
            "X-B3-Sampled".to_string(),
            if ctx.is_sampled() { "1".to_string() } else { "0".to_string() },
        );
        headers
    }
}

// ---------------------------------------------------------------------------
// Span context for service-to-service propagation
// ---------------------------------------------------------------------------

/// A span context that carries baggage and attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    pub trace_context: TraceContext,
    pub service_name: String,
    pub operation_name: String,
    pub baggage: BTreeMap<String, String>,
    pub attributes: BTreeMap<String, String>,
}

impl SpanContext {
    pub fn new(service: &str, operation: &str) -> Self {
        Self {
            trace_context: TraceContext::new_root(),
            service_name: service.to_string(),
            operation_name: operation.to_string(),
            baggage: BTreeMap::new(),
            attributes: BTreeMap::new(),
        }
    }

    pub fn child(&self, service: &str, operation: &str) -> Self {
        Self {
            trace_context: self.trace_context.child(),
            service_name: service.to_string(),
            operation_name: operation.to_string(),
            baggage: self.baggage.clone(),
            attributes: BTreeMap::new(),
        }
    }

    pub fn set_baggage(&mut self, key: &str, value: &str) {
        self.baggage.insert(key.to_string(), value.to_string());
    }

    pub fn set_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }

    /// Inject context into HTTP headers.
    pub fn inject_headers(&self) -> BTreeMap<String, String> {
        let mut headers = BTreeMap::new();
        headers.insert(
            "traceparent".to_string(),
            self.trace_context.to_traceparent(),
        );
        headers.insert("X-Service-Name".to_string(), self.service_name.clone());

        for (key, value) in &self.baggage {
            headers.insert(format!("baggage-{}", key), value.clone());
        }

        headers
    }
}

/// Simulate a distributed trace across services.
pub fn simulate_distributed_trace() -> Vec<SpanContext> {
    let mut trace = Vec::new();

    // Service A: API Gateway
    let gateway = SpanContext::new("api-gateway", "handle_request");
    trace.push(gateway.clone());

    // Service B: Auth Service
    let auth = gateway.child("auth-service", "authenticate");
    trace.push(auth.clone());

    // Service C: User Service
    let user = auth.child("user-service", "get_user");
    trace.push(user);

    trace
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_new_root() {
        let ctx = TraceContext::new_root();
        assert_eq!(ctx.trace_id.len(), 32);
        assert_eq!(ctx.span_id.len(), 16);
        assert!(ctx.parent_span_id.is_none());
        assert!(ctx.is_sampled());
    }

    #[test]
    fn test_trace_context_child() {
        let parent = TraceContext::new_root();
        let child = parent.child();

        assert_eq!(child.trace_id, parent.trace_id);
        assert_ne!(child.span_id, parent.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id.clone()));
    }

    #[test]
    fn test_traceparent_format() {
        let ctx = TraceContext::new_root();
        let header = ctx.to_traceparent();
        let parts: Vec<&str> = header.split('-').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "00");
    }

    #[test]
    fn test_from_traceparent() {
        let header = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let ctx = TraceContext::from_traceparent(header).unwrap();
        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
        assert!(ctx.is_sampled());
    }

    #[test]
    fn test_from_traceparent_invalid() {
        assert!(TraceContext::from_traceparent("invalid").is_err());
        assert!(TraceContext::from_traceparent("01-abc-123-01").is_err());
    }

    #[test]
    fn test_from_traceparent_unsupported_version() {
        let header = "ff-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        assert!(matches!(
            TraceContext::from_traceparent(header),
            Err(TraceError::UnsupportedVersion(255))
        ));
    }

    #[test]
    fn test_is_sampled() {
        let mut ctx = TraceContext::new_root();
        ctx.trace_flags = 1;
        assert!(ctx.is_sampled());

        ctx.trace_flags = 0;
        assert!(!ctx.is_sampled());
    }

    #[test]
    fn test_b3_single_header() {
        let ctx = TraceContext::new_root();
        let header = B3Propagator::to_single_header(&ctx);
        let parts: Vec<&str> = header.split('-').collect();
        assert!(parts.len() >= 3);
    }

    #[test]
    fn test_b3_multi_headers() {
        let ctx = TraceContext::new_root();
        let headers = B3Propagator::to_multi_headers(&ctx);
        assert!(headers.contains_key("X-B3-TraceId"));
        assert!(headers.contains_key("X-B3-SpanId"));
        assert!(headers.contains_key("X-B3-Sampled"));
        assert!(!headers.contains_key("X-B3-ParentSpanId"));
    }

    #[test]
    fn test_b3_multi_headers_with_parent() {
        let parent = TraceContext::new_root();
        let child = parent.child();
        let headers = B3Propagator::to_multi_headers(&child);
        assert!(headers.contains_key("X-B3-ParentSpanId"));
    }

    #[test]
    fn test_span_context() {
        let ctx = SpanContext::new("my-service", "handle_request");
        assert_eq!(ctx.service_name, "my-service");
        assert_eq!(ctx.operation_name, "handle_request");
    }

    #[test]
    fn test_span_context_child() {
        let parent = SpanContext::new("api", "handle");
        let child = parent.child("db", "query");

        assert_eq!(child.service_name, "db");
        assert_eq!(child.operation_name, "query");
        assert_eq!(
            child.trace_context.trace_id,
            parent.trace_context.trace_id
        );
    }

    #[test]
    fn test_span_context_baggage() {
        let mut ctx = SpanContext::new("service", "op");
        ctx.set_baggage("user_id", "42");
        ctx.set_baggage("request_id", "req-123");

        assert_eq!(ctx.baggage.len(), 2);

        // Baggage should propagate to children
        let child = ctx.child("other", "op");
        assert_eq!(child.baggage.len(), 2);
    }

    #[test]
    fn test_span_context_attributes() {
        let mut ctx = SpanContext::new("service", "op");
        ctx.set_attribute("http.method", "GET");
        ctx.set_attribute("http.url", "/api/users");

        assert_eq!(ctx.attributes.len(), 2);
    }

    #[test]
    fn test_inject_headers() {
        let mut ctx = SpanContext::new("api", "handle");
        ctx.set_baggage("session", "abc");

        let headers = ctx.inject_headers();
        assert!(headers.contains_key("traceparent"));
        assert!(headers.contains_key("X-Service-Name"));
        assert!(headers.contains_key("baggage-session"));
    }

    #[test]
    fn test_simulate_distributed_trace() {
        let trace = simulate_distributed_trace();
        assert_eq!(trace.len(), 3);
        assert_eq!(trace[0].service_name, "api-gateway");
        assert_eq!(trace[1].service_name, "auth-service");
        assert_eq!(trace[2].service_name, "user-service");

        // All should share the same trace ID
        assert_eq!(
            trace[0].trace_context.trace_id,
            trace[1].trace_context.trace_id
        );
        assert_eq!(
            trace[1].trace_context.trace_id,
            trace[2].trace_context.trace_id
        );
    }

    #[test]
    fn test_trace_context_serialization() {
        let ctx = TraceContext::new_root();
        let json = serde_json::to_string(&ctx).unwrap();
        let parsed: TraceContext = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.trace_id, ctx.trace_id);
    }
}
