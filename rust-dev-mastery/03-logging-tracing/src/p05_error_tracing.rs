//! # Lesson 5: Error Tracing
//!
//! Combining error handling with tracing provides rich diagnostic context.
//! This lesson covers tracing errors, span context for errors, error events,
//! and the tracing-error bridge.

use std::fmt;

// ---------------------------------------------------------------------------
// Error type with tracing integration
// ---------------------------------------------------------------------------

/// An error that carries tracing context.
#[derive(Debug)]
pub struct TracedError {
    pub message: String,
    pub span_context: Vec<String>,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorSeverity {
    /// Error that can be retried.
    Transient,
    /// Error that requires user action.
    ClientError,
    /// Error that indicates a bug.
    Internal,
    /// Critical error requiring immediate attention.
    Critical,
}

impl fmt::Display for TracedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.severity, self.message)?;
        if !self.span_context.is_empty() {
            write!(f, " (context: {})", self.span_context.join(" > "))?;
        }
        Ok(())
    }
}

impl std::error::Error for TracedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|s| s.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl TracedError {
    pub fn new(message: impl Into<String>, severity: ErrorSeverity) -> Self {
        Self {
            message: message.into(),
            span_context: Vec::new(),
            source: None,
            severity,
        }
    }

    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    pub fn with_span_context(mut self, context: Vec<String>) -> Self {
        self.span_context = context;
        self
    }
}

// ---------------------------------------------------------------------------
// Error event model (what gets emitted to the subscriber)
// ---------------------------------------------------------------------------

/// An error event that would be emitted to a tracing subscriber.
#[derive(Debug, Clone)]
pub struct ErrorEvent {
    pub level: ErrorLevel,
    pub message: String,
    pub error_type: String,
    pub span_name: Option<String>,
    pub fields: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorLevel {
    Error,
    Warn,
}

impl ErrorEvent {
    pub fn error(message: &str, error_type: &str) -> Self {
        Self {
            level: ErrorLevel::Error,
            message: message.to_string(),
            error_type: error_type.to_string(),
            span_name: None,
            fields: Vec::new(),
        }
    }

    pub fn warn(message: &str, error_type: &str) -> Self {
        Self {
            level: ErrorLevel::Warn,
            message: message.to_string(),
            error_type: error_type.to_string(),
            span_name: None,
            fields: Vec::new(),
        }
    }

    pub fn with_span(mut self, name: &str) -> Self {
        self.span_name = Some(name.to_string());
        self
    }

    pub fn with_field(mut self, key: &str, value: &str) -> Self {
        self.fields.push((key.to_string(), value.to_string()));
        self
    }
}

// ---------------------------------------------------------------------------
// Error tracing patterns
// ---------------------------------------------------------------------------

/// A request processor that demonstrates error tracing.
pub struct RequestProcessor {
    pub events: Vec<ErrorEvent>,
}

impl RequestProcessor {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Process a request, emitting error events as appropriate.
    pub fn process(&mut self, request_id: &str) -> Result<String, TracedError> {
        // Simulate authentication
        if request_id.starts_with("anon-") {
            let event = ErrorEvent::error("authentication failed", "AuthError")
                .with_span("authenticate")
                .with_field("request_id", request_id)
                .with_field("reason", "anonymous access not allowed");
            self.events.push(event);

            return Err(TracedError::new(
                "authentication failed",
                ErrorSeverity::ClientError,
            ));
        }

        // Simulate processing
        if request_id.contains("fail") {
            let event = ErrorEvent::error("processing failed", "ProcessingError")
                .with_span("process_request")
                .with_field("request_id", request_id);
            self.events.push(event);

            return Err(TracedError::new(
                "processing failed",
                ErrorSeverity::Internal,
            )
            .with_source(std::io::Error::new(
                std::io::ErrorKind::Other,
                "simulated failure",
            )));
        }

        // Simulate warning
        if request_id.contains("slow") {
            let event = ErrorEvent::warn("slow request detected", "PerformanceWarning")
                .with_span("process_request")
                .with_field("request_id", request_id)
                .with_field("duration_ms", "5000");
            self.events.push(event);
        }

        Ok(format!("processed {}", request_id))
    }

    /// Get all error events.
    pub fn error_events(&self) -> Vec<&ErrorEvent> {
        self.events
            .iter()
            .filter(|e| e.level == ErrorLevel::Error)
            .collect()
    }

    /// Get all warning events.
    pub fn warn_events(&self) -> Vec<&ErrorEvent> {
        self.events
            .iter()
            .filter(|e| e.level == ErrorLevel::Warn)
            .collect()
    }
}

impl Default for RequestProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Format an error event as a structured log line.
pub fn format_error_event(event: &ErrorEvent) -> String {
    let level = match event.level {
        ErrorLevel::Error => "ERROR",
        ErrorLevel::Warn => " WARN",
    };

    let mut line = format!("{} {}", level, event.message);
    line.push_str(&format!(" error_type={}", event.error_type));

    if let Some(ref span) = event.span_name {
        line.push_str(&format!(" span={}", span));
    }

    for (key, value) in &event.fields {
        line.push_str(&format!(" {}={}", key, value));
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_traced_error_display() {
        let err = TracedError::new("something failed", ErrorSeverity::Internal);
        assert!(err.to_string().contains("something failed"));
        assert!(err.to_string().contains("Internal"));
    }

    #[test]
    fn test_traced_error_with_context() {
        let err = TracedError::new("failed", ErrorSeverity::Critical)
            .with_span_context(vec![
                "request".into(),
                "database".into(),
                "query".into(),
            ]);
        let msg = err.to_string();
        assert!(msg.contains("request > database > query"));
    }

    #[test]
    fn test_traced_error_with_source() {
        let inner = std::io::Error::new(std::io::ErrorKind::Other, "root cause");
        let err = TracedError::new("wrapper", ErrorSeverity::Internal).with_source(inner);
        assert!(err.source().is_some());
        assert!(err.source().unwrap().to_string().contains("root cause"));
    }

    #[test]
    fn test_error_event_creation() {
        let event = ErrorEvent::error("test error", "TestError");
        assert_eq!(event.level, ErrorLevel::Error);
        assert_eq!(event.message, "test error");
        assert_eq!(event.error_type, "TestError");
    }

    #[test]
    fn test_error_event_with_span() {
        let event = ErrorEvent::warn("slow query", "PerfWarning")
            .with_span("database")
            .with_field("duration_ms", "5000");

        assert_eq!(event.span_name, Some("database".to_string()));
        assert_eq!(event.fields.len(), 1);
    }

    #[test]
    fn test_request_processor_success() {
        let mut proc = RequestProcessor::new();
        let result = proc.process("req-123");
        assert!(result.is_ok());
        assert!(proc.error_events().is_empty());
    }

    #[test]
    fn test_request_processor_auth_failure() {
        let mut proc = RequestProcessor::new();
        let result = proc.process("anon-123");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.severity, ErrorSeverity::ClientError);
        assert_eq!(proc.error_events().len(), 1);
    }

    #[test]
    fn test_request_processor_processing_failure() {
        let mut proc = RequestProcessor::new();
        let result = proc.process("req-fail-123");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.severity, ErrorSeverity::Internal);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_request_processor_slow_warning() {
        let mut proc = RequestProcessor::new();
        let result = proc.process("req-slow-123");
        assert!(result.is_ok());
        assert_eq!(proc.warn_events().len(), 1);
        assert!(proc.error_events().is_empty());
    }

    #[test]
    fn test_format_error_event() {
        let event = ErrorEvent::error("query failed", "DbError")
            .with_span("database")
            .with_field("query", "SELECT * FROM users");

        let formatted = format_error_event(&event);
        assert!(formatted.contains("ERROR"));
        assert!(formatted.contains("query failed"));
        assert!(formatted.contains("error_type=DbError"));
        assert!(formatted.contains("span=database"));
        assert!(formatted.contains("query=SELECT * FROM users"));
    }

    #[test]
    fn test_format_warn_event() {
        let event = ErrorEvent::warn("deprecated endpoint", "DeprecationWarning");
        let formatted = format_error_event(&event);
        assert!(formatted.contains(" WARN"));
    }

    #[test]
    fn test_error_severity_variants() {
        assert_ne!(ErrorSeverity::Transient, ErrorSeverity::Critical);
        assert_ne!(ErrorSeverity::ClientError, ErrorSeverity::Internal);
    }

    #[test]
    fn test_error_event_without_span() {
        let event = ErrorEvent::error("test", "Test");
        assert!(event.span_name.is_none());
        let formatted = format_error_event(&event);
        assert!(!formatted.contains("span="));
    }

    #[test]
    fn test_traced_error_no_source() {
        let err = TracedError::new("test", ErrorSeverity::Transient);
        assert!(err.source().is_none());
    }
}
