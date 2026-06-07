//! # Lesson 2: Spans and Events
//!
//! Spans represent periods of execution; events are discrete occurrences.
//! This lesson covers creating spans, entering spans, span fields,
//! and the span lifecycle.

use std::sync::{Arc, Mutex};
use tracing::{span, Level};

// ---------------------------------------------------------------------------
// Span data model for teaching concepts
// ---------------------------------------------------------------------------

/// Represents a span in the tracing model.
#[derive(Debug, Clone)]
pub struct SpanInfo {
    pub name: String,
    pub level: Level,
    pub fields: Vec<(String, String)>,
    pub parent: Option<String>,
    pub children: Vec<String>,
}

impl SpanInfo {
    pub fn new(name: impl Into<String>, level: Level) -> Self {
        Self {
            name: name.into(),
            level,
            fields: Vec::new(),
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }
}

/// A span tree that models parent-child relationships.
#[derive(Debug)]
pub struct SpanTree {
    pub spans: Vec<SpanInfo>,
}

impl SpanTree {
    pub fn new() -> Self {
        Self { spans: Vec::new() }
    }

    pub fn add_span(&mut self, span: SpanInfo) {
        self.spans.push(span);
    }

    pub fn root_spans(&self) -> Vec<&SpanInfo> {
        self.spans.iter().filter(|s| s.parent.is_none()).collect()
    }

    pub fn children_of(&self, parent_name: &str) -> Vec<&SpanInfo> {
        self.spans
            .iter()
            .filter(|s| s.parent.as_deref() == Some(parent_name))
            .collect()
    }

    pub fn max_depth(&self) -> usize {
        self.root_spans()
            .iter()
            .map(|r| self.depth_of(&r.name, 1))
            .max()
            .unwrap_or(0)
    }

    fn depth_of(&self, name: &str, current: usize) -> usize {
        let children = self.children_of(name);
        if children.is_empty() {
            current
        } else {
            children
                .iter()
                .map(|c| self.depth_of(&c.name, current + 1))
                .max()
                .unwrap_or(current)
        }
    }
}

impl Default for SpanTree {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Event model
// ---------------------------------------------------------------------------

/// Represents a tracing event.
#[derive(Debug, Clone)]
pub struct EventInfo {
    pub level: Level,
    pub message: String,
    pub fields: Vec<(String, String)>,
    pub span_context: Option<String>,
}

impl EventInfo {
    pub fn new(level: Level, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            fields: Vec::new(),
            span_context: None,
        }
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }

    pub fn with_span(mut self, span_name: impl Into<String>) -> Self {
        self.span_context = Some(span_name.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Span field types
// ---------------------------------------------------------------------------

/// The different types of fields that can be attached to spans.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Str(String),
    I64(i64),
    U64(u64),
    F64(f64),
    Bool(bool),
    Debug(String),
}

impl FieldValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            FieldValue::Str(_) => "string",
            FieldValue::I64(_) => "i64",
            FieldValue::U64(_) => "u64",
            FieldValue::F64(_) => "f64",
            FieldValue::Bool(_) => "bool",
            FieldValue::Debug(_) => "debug",
        }
    }

    pub fn display_value(&self) -> String {
        match self {
            FieldValue::Str(s) => s.clone(),
            FieldValue::I64(v) => v.to_string(),
            FieldValue::U64(v) => v.to_string(),
            FieldValue::F64(v) => v.to_string(),
            FieldValue::Bool(v) => v.to_string(),
            FieldValue::Debug(s) => s.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Span lifecycle stages
// ---------------------------------------------------------------------------

/// The stages a span goes through during its lifetime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpanLifecycle {
    /// Span was created but not yet entered.
    Created,
    /// Span is currently active (entered).
    Active,
    /// Span was exited but may still be referenced.
    Exited,
    /// Span has been fully closed.
    Closed,
}

/// A span with tracked lifecycle state.
pub struct TrackedSpan {
    pub name: String,
    pub lifecycle: SpanLifecycle,
    pub enter_count: u32,
    pub events_within: u32,
}

impl TrackedSpan {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            lifecycle: SpanLifecycle::Created,
            enter_count: 0,
            events_within: 0,
        }
    }

    pub fn enter(&mut self) {
        self.lifecycle = SpanLifecycle::Active;
        self.enter_count += 1;
    }

    pub fn exit(&mut self) {
        self.lifecycle = SpanLifecycle::Exited;
    }

    pub fn close(&mut self) {
        self.lifecycle = SpanLifecycle::Closed;
    }

    pub fn record_event(&mut self) {
        self.events_within += 1;
    }

    pub fn is_active(&self) -> bool {
        self.lifecycle == SpanLifecycle::Active
    }
}

// ---------------------------------------------------------------------------
// Practical span patterns
// ---------------------------------------------------------------------------

/// Models a request handler that creates nested spans.
pub struct RequestHandler {
    pub spans: Vec<TrackedSpan>,
}

impl RequestHandler {
    pub fn new() -> Self {
        Self { spans: Vec::new() }
    }

    /// Simulate handling a request with nested spans.
    pub fn handle_request(&mut self, request_id: &str) -> String {
        let mut req_span = TrackedSpan::new(format!("request:{}", request_id));
        req_span.enter();
        self.spans.push(req_span);

        // Simulate middleware
        let mut auth_span = TrackedSpan::new("authenticate");
        auth_span.enter();
        auth_span.record_event();
        auth_span.exit();
        self.spans.push(auth_span);

        // Simulate handler
        let mut handler_span = TrackedSpan::new("handler");
        handler_span.enter();
        handler_span.record_event();
        handler_span.record_event();
        handler_span.exit();
        self.spans.push(handler_span);

        // Close request span
        if let Some(last) = self.spans.first_mut() {
            last.exit();
            last.close();
        }

        format!("handled request {}", request_id)
    }

    pub fn active_spans(&self) -> Vec<&TrackedSpan> {
        self.spans.iter().filter(|s| s.is_active()).collect()
    }

    pub fn total_events(&self) -> u32 {
        self.spans.iter().map(|s| s.events_within).sum()
    }
}

impl Default for RequestHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_info_creation() {
        let span = SpanInfo::new("test", Level::INFO);
        assert_eq!(span.name, "test");
        assert_eq!(span.level, Level::INFO);
        assert!(span.fields.is_empty());
        assert!(span.parent.is_none());
    }

    #[test]
    fn test_span_info_with_fields() {
        let span = SpanInfo::new("req", Level::DEBUG)
            .with_field("method", "GET")
            .with_field("path", "/api/users");

        assert_eq!(span.fields.len(), 2);
        assert_eq!(span.fields[0].0, "method");
    }

    #[test]
    fn test_span_info_with_parent() {
        let span = SpanInfo::new("child", Level::INFO).with_parent("parent");
        assert_eq!(span.parent, Some("parent".to_string()));
    }

    #[test]
    fn test_span_tree() {
        let mut tree = SpanTree::new();
        tree.add_span(SpanInfo::new("root", Level::INFO));
        tree.add_span(SpanInfo::new("child1", Level::DEBUG).with_parent("root"));
        tree.add_span(SpanInfo::new("child2", Level::DEBUG).with_parent("root"));
        tree.add_span(SpanInfo::new("grandchild", Level::TRACE).with_parent("child1"));

        assert_eq!(tree.root_spans().len(), 1);
        assert_eq!(tree.children_of("root").len(), 2);
        assert_eq!(tree.children_of("child1").len(), 1);
        assert_eq!(tree.max_depth(), 3);
    }

    #[test]
    fn test_span_tree_empty() {
        let tree = SpanTree::new();
        assert!(tree.root_spans().is_empty());
        assert_eq!(tree.max_depth(), 0);
    }

    #[test]
    fn test_event_info() {
        let event = EventInfo::new(Level::INFO, "user logged in")
            .with_field("user_id", "42")
            .with_span("auth");

        assert_eq!(event.level, Level::INFO);
        assert_eq!(event.message, "user logged in");
        assert_eq!(event.fields.len(), 1);
        assert_eq!(event.span_context, Some("auth".to_string()));
    }

    #[test]
    fn test_field_value_types() {
        assert_eq!(FieldValue::Str("test".into()).type_name(), "string");
        assert_eq!(FieldValue::I64(-1).type_name(), "i64");
        assert_eq!(FieldValue::U64(1).type_name(), "u64");
        assert_eq!(FieldValue::F64(1.5).type_name(), "f64");
        assert_eq!(FieldValue::Bool(true).type_name(), "bool");
    }

    #[test]
    fn test_field_value_display() {
        assert_eq!(FieldValue::Str("hello".into()).display_value(), "hello");
        assert_eq!(FieldValue::I64(-42).display_value(), "-42");
        assert_eq!(FieldValue::Bool(true).display_value(), "true");
    }

    #[test]
    fn test_tracked_span_lifecycle() {
        let mut span = TrackedSpan::new("test");
        assert_eq!(span.lifecycle, SpanLifecycle::Created);
        assert!(!span.is_active());

        span.enter();
        assert_eq!(span.lifecycle, SpanLifecycle::Active);
        assert!(span.is_active());
        assert_eq!(span.enter_count, 1);

        span.exit();
        assert_eq!(span.lifecycle, SpanLifecycle::Exited);
        assert!(!span.is_active());

        span.close();
        assert_eq!(span.lifecycle, SpanLifecycle::Closed);
    }

    #[test]
    fn test_tracked_span_events() {
        let mut span = TrackedSpan::new("test");
        assert_eq!(span.events_within, 0);

        span.record_event();
        span.record_event();
        span.record_event();
        assert_eq!(span.events_within, 3);
    }

    #[test]
    fn test_tracked_span_reentry() {
        let mut span = TrackedSpan::new("test");
        span.enter();
        span.exit();
        span.enter();
        assert_eq!(span.enter_count, 2);
    }

    #[test]
    fn test_request_handler() {
        let mut handler = RequestHandler::new();
        let result = handler.handle_request("req-1");

        assert!(result.contains("req-1"));
        assert_eq!(handler.spans.len(), 3); // request, authenticate, handler
        assert_eq!(handler.total_events(), 3); // 1 + 2 events
    }

    #[test]
    fn test_request_handler_active_spans() {
        let mut handler = RequestHandler::new();
        handler.handle_request("req-1");
        // After handle_request completes, no spans should be active
        let active = handler.active_spans();
        assert!(active.is_empty());
    }

    #[test]
    fn test_span_levels() {
        let error_span = SpanInfo::new("error_span", Level::ERROR);
        let warn_span = SpanInfo::new("warn_span", Level::WARN);
        let info_span = SpanInfo::new("info_span", Level::INFO);
        let debug_span = SpanInfo::new("debug_span", Level::DEBUG);
        let trace_span = SpanInfo::new("trace_span", Level::TRACE);

        assert_eq!(error_span.level, Level::ERROR);
        assert_eq!(warn_span.level, Level::WARN);
        assert_eq!(info_span.level, Level::INFO);
        assert_eq!(debug_span.level, Level::DEBUG);
        assert_eq!(trace_span.level, Level::TRACE);
    }

    #[test]
    fn test_span_tree_multiple_roots() {
        let mut tree = SpanTree::new();
        tree.add_span(SpanInfo::new("root1", Level::INFO));
        tree.add_span(SpanInfo::new("root2", Level::INFO));
        tree.add_span(SpanInfo::new("child", Level::DEBUG).with_parent("root1"));

        assert_eq!(tree.root_spans().len(), 2);
        assert_eq!(tree.children_of("root1").len(), 1);
        assert_eq!(tree.children_of("root2").len(), 0);
    }
}
