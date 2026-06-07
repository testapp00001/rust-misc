//! # Lesson 1: Tracing Fundamentals
//!
//! The `tracing` crate provides structured, context-aware diagnostics for Rust.
//! This lesson covers the core concepts: the Subscriber trait, global dispatcher,
//! and the fundamental difference between events and spans.

use std::sync::{Arc, Mutex};
use tracing::{span, Event, Level, Metadata, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

// ---------------------------------------------------------------------------
// A minimal custom subscriber for demonstration
// ---------------------------------------------------------------------------

/// A simple in-memory subscriber that captures events for testing.
/// In production, you'd use tracing-subscriber's fmt or a log aggregator.
#[derive(Debug, Clone)]
pub struct InMemorySubscriber {
    events: Arc<Mutex<Vec<CapturedEvent>>>,
}

#[derive(Debug, Clone)]
pub struct CapturedEvent {
    pub level: Level,
    pub message: String,
    pub fields: Vec<(String, String)>,
}

impl InMemorySubscriber {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn events(&self) -> Vec<CapturedEvent> {
        self.events.lock().unwrap().clone()
    }

    pub fn event_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    pub fn events_at_level(&self, level: Level) -> Vec<CapturedEvent> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.level == level)
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

impl Default for InMemorySubscriber {
    fn default() -> Self {
        Self::new()
    }
}

/// A tracing layer that captures events into our InMemorySubscriber.
pub struct CaptureLayer {
    events: Arc<Mutex<Vec<CapturedEvent>>>,
}

impl CaptureLayer {
    pub fn new(events: Arc<Mutex<Vec<CapturedEvent>>>) -> Self {
        Self { events }
    }
}

impl<S: Subscriber> Layer<S> for CaptureLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        let captured = CapturedEvent {
            level: *metadata.level(),
            message: visitor.message.unwrap_or_default(),
            fields: visitor.fields,
        };

        self.events.lock().unwrap().push(captured);
    }
}

struct FieldVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: Vec::new(),
        }
    }
}

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let value_str = format!("{:?}", value);
        if field.name() == "message" {
            self.message = Some(value_str);
        } else {
            self.fields
                .push((field.name().to_string(), value_str));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields
                .push((field.name().to_string(), value.to_string()));
        }
    }
}

// ---------------------------------------------------------------------------
// Core tracing concepts
// ---------------------------------------------------------------------------

/// Describes the key components of the tracing system.
pub struct TracingArchitecture {
    pub components: Vec<Component>,
}

pub struct Component {
    pub name: &'static str,
    pub description: &'static str,
    pub purpose: &'static str,
}

impl TracingArchitecture {
    pub fn describe() -> Self {
        Self {
            components: vec![
                Component {
                    name: "Span",
                    description: "A period of time during which a program was executing",
                    purpose: "Provides context for events; represents operations",
                },
                Component {
                    name: "Event",
                    description: "A single point-in-time occurrence",
                    purpose: "Records discrete data points like log messages",
                },
                Component {
                    name: "Subscriber",
                    description: "Collects and processes spans and events",
                    purpose: "Receives all tracing data and decides what to do with it",
                },
                Component {
                    name: "Layer",
                    description: "A composable processing step for a subscriber",
                    purpose: "Filters, formats, or forwards tracing data",
                },
                Component {
                    name: "Dispatcher",
                    description: "The global subscriber that routes data to layers",
                    purpose: "Thread-local and global subscriber management",
                },
            ],
        }
    }
}

/// Log levels and when to use them.
pub fn log_level_guide() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("ERROR", "Something failed and needs attention", "Alerting, immediate response"),
        ("WARN", "Something unexpected but recoverable", "Monitoring, trend analysis"),
        ("INFO", "Significant operational events", "Audit trail, business events"),
        ("DEBUG", "Detailed diagnostic information", "Development, troubleshooting"),
        ("TRACE", "Very detailed, high-volume data", "Deep debugging, performance analysis"),
    ]
}

// ---------------------------------------------------------------------------
// Functions that produce tracing output
// ---------------------------------------------------------------------------

/// A simple function that emits tracing events at different levels.
pub fn emit_sample_events() {
    tracing::error!("something went wrong");
    tracing::warn!("potential issue detected");
    tracing::info!("operation completed");
    tracing::debug!("intermediate state: {}", 42);
    tracing::trace!("detailed trace data");
}

/// A function that demonstrates span creation.
pub fn demonstrate_span_lifecycle() {
    let span = tracing::info_span!("processing", request_id = "req-123");
    let _guard = span.enter();

    tracing::info!("started processing");
    tracing::debug!("step 1 complete");

    // Span exits when _guard is dropped
}

/// Version information for the tracing ecosystem.
pub fn tracing_versions() -> Vec<(&'static str, &'static str)> {
    vec![
        ("tracing", "0.1.x"),
        ("tracing-subscriber", "0.3.x"),
        ("tracing-core", "0.1.x"),
        ("tracing-log", "0.2.x"),
        ("tracing-opentelemetry", "0.22+"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    fn setup_subscriber() -> (InMemorySubscriber, tracing::subscriber::DefaultGuard) {
        let subscriber = InMemorySubscriber::new();
        let layer = CaptureLayer::new(Arc::clone(&subscriber.events));
        let registry = Registry::default().with(layer);
        let guard = tracing::subscriber::set_default(registry);
        (subscriber, guard)
    }

    #[test]
    fn test_capture_error_event() {
        let (sub, _guard) = setup_subscriber();
        tracing::error!("test error");
        let events = sub.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, Level::ERROR);
    }

    #[test]
    fn test_capture_info_event() {
        let (sub, _guard) = setup_subscriber();
        tracing::info!("test info");
        let events = sub.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, Level::INFO);
    }

    #[test]
    fn test_capture_debug_event() {
        let (sub, _guard) = setup_subscriber();
        tracing::debug!("test debug");
        let events = sub.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, Level::DEBUG);
    }

    #[test]
    fn test_capture_multiple_events() {
        let (sub, _guard) = setup_subscriber();
        tracing::error!("error 1");
        tracing::warn!("warn 1");
        tracing::info!("info 1");
        assert_eq!(sub.event_count(), 3);
    }

    #[test]
    fn test_events_at_level() {
        let (sub, _guard) = setup_subscriber();
        tracing::error!("error");
        tracing::info!("info 1");
        tracing::info!("info 2");
        tracing::debug!("debug");

        assert_eq!(sub.events_at_level(Level::ERROR).len(), 1);
        assert_eq!(sub.events_at_level(Level::INFO).len(), 2);
        assert_eq!(sub.events_at_level(Level::WARN).len(), 0);
    }

    #[test]
    fn test_clear_events() {
        let (sub, _guard) = setup_subscriber();
        tracing::info!("event");
        assert_eq!(sub.event_count(), 1);
        sub.clear();
        assert_eq!(sub.event_count(), 0);
    }

    #[test]
    fn test_capture_with_fields() {
        let (sub, _guard) = setup_subscriber();
        tracing::info!(key = "value", count = 42, "test");
        let events = sub.events();
        assert_eq!(events.len(), 1);
        assert!(!events[0].fields.is_empty());
    }

    #[test]
    fn test_tracing_architecture() {
        let arch = TracingArchitecture::describe();
        assert_eq!(arch.components.len(), 5);
        assert!(arch.components.iter().any(|c| c.name == "Span"));
        assert!(arch.components.iter().any(|c| c.name == "Event"));
        assert!(arch.components.iter().any(|c| c.name == "Subscriber"));
    }

    #[test]
    fn test_log_level_guide() {
        let guide = log_level_guide();
        assert_eq!(guide.len(), 5);
        assert!(guide.iter().any(|(name, _, _)| *name == "ERROR"));
        assert!(guide.iter().any(|(name, _, _)| *name == "TRACE"));
    }

    #[test]
    fn test_emit_sample_events() {
        let (sub, _guard) = setup_subscriber();
        emit_sample_events();
        assert_eq!(sub.event_count(), 5);
    }

    #[test]
    fn test_span_lifecycle() {
        let (sub, _guard) = setup_subscriber();
        demonstrate_span_lifecycle();
        // Events emitted inside the span should be captured
        let events = sub.events();
        assert!(events.iter().any(|e| e.message.contains("started processing")));
    }

    #[test]
    fn test_tracing_versions() {
        let versions = tracing_versions();
        assert!(!versions.is_empty());
        assert!(versions.iter().any(|(name, _)| *name == "tracing"));
    }

    #[test]
    fn test_in_memory_subscriber_clone() {
        let sub = InMemorySubscriber::new();
        let cloned = sub.clone();
        // Both should share the same event storage
        let _guard = setup_subscriber_for(&sub);
        tracing::info!("test");
        assert_eq!(sub.event_count(), cloned.event_count());
    }

    fn setup_subscriber_for(
        sub: &InMemorySubscriber,
    ) -> tracing::subscriber::DefaultGuard {
        let layer = CaptureLayer::new(Arc::clone(&sub.events));
        let registry = Registry::default().with(layer);
        tracing::subscriber::set_default(registry)
    }
}
