//! # Lesson 3: Structured Logging
//!
//! Structured logging attaches typed fields to log messages instead of
//! embedding data in format strings. This lesson covers structured fields,
//! Debug/Display formatting, and custom field types.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Structured log entry model
// ---------------------------------------------------------------------------

/// A structured log entry with typed fields.
/// In tracing, fields are attached via macros like `info!(key = value)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub fields: BTreeMap<String, FieldValue>,
    pub span: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldValue {
    Str(String),
    I64(i64),
    U64(u64),
    F64(f64),
    Bool(bool),
    Array(Vec<FieldValue>),
    Object(BTreeMap<String, FieldValue>),
}

impl FieldValue {
    /// Get the type name for this field value.
    pub fn type_name(&self) -> &'static str {
        match self {
            FieldValue::Str(_) => "string",
            FieldValue::I64(_) => "integer",
            FieldValue::U64(_) => "unsigned",
            FieldValue::F64(_) => "float",
            FieldValue::Bool(_) => "boolean",
            FieldValue::Array(_) => "array",
            FieldValue::Object(_) => "object",
        }
    }

    /// Format for display.
    pub fn display(&self) -> String {
        match self {
            FieldValue::Str(s) => format!("\"{}\"", s),
            FieldValue::I64(v) => v.to_string(),
            FieldValue::U64(v) => v.to_string(),
            FieldValue::F64(v) => format!("{:.2}", v),
            FieldValue::Bool(v) => v.to_string(),
            FieldValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.display()).collect();
                format!("[{}]", items.join(", "))
            }
            FieldValue::Object(map) => {
                let items: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.display()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
        }
    }
}

impl std::fmt::Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl From<&str> for FieldValue {
    fn from(s: &str) -> Self {
        FieldValue::Str(s.to_string())
    }
}

impl From<String> for FieldValue {
    fn from(s: String) -> Self {
        FieldValue::Str(s)
    }
}

impl From<i64> for FieldValue {
    fn from(v: i64) -> Self {
        FieldValue::I64(v)
    }
}

impl From<u64> for FieldValue {
    fn from(v: u64) -> Self {
        FieldValue::U64(v)
    }
}

impl From<f64> for FieldValue {
    fn from(v: f64) -> Self {
        FieldValue::F64(v)
    }
}

impl From<bool> for FieldValue {
    fn from(v: bool) -> Self {
        FieldValue::Bool(v)
    }
}

// ---------------------------------------------------------------------------
// Structured log builder
// ---------------------------------------------------------------------------

/// Builder for creating structured log entries.
pub struct LogEntryBuilder {
    entry: StructuredLogEntry,
}

impl LogEntryBuilder {
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            entry: StructuredLogEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                level,
                message: message.into(),
                fields: BTreeMap::new(),
                span: None,
            },
        }
    }

    pub fn field(mut self, key: impl Into<String>, value: impl Into<FieldValue>) -> Self {
        self.entry.fields.insert(key.into(), value.into());
        self
    }

    pub fn span(mut self, name: impl Into<String>) -> Self {
        self.entry.span = Some(name.into());
        self
    }

    pub fn build(self) -> StructuredLogEntry {
        self.entry
    }
}

// ---------------------------------------------------------------------------
// Custom field types (Display and Debug for logging)
// ---------------------------------------------------------------------------

/// A user ID that implements Display for clean log output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserId(pub u64);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user:{}", self.0)
    }
}

/// A request ID that implements Display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestId(pub String);

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "req:{}", self.0)
    }
}

/// A duration that formats nicely in logs.
#[derive(Debug, Clone)]
pub struct LogDuration(pub Duration);

impl std::fmt::Display for LogDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ms = self.0.as_millis();
        if ms < 1000 {
            write!(f, "{}ms", ms)
        } else {
            write!(f, "{:.2}s", self.0.as_secs_f64())
        }
    }
}

/// A size value that formats with units.
#[derive(Debug, Clone)]
pub struct ByteSize(pub u64);

impl std::fmt::Display for ByteSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 < 1024 {
            write!(f, "{}B", self.0)
        } else if self.0 < 1024 * 1024 {
            write!(f, "{:.1}KB", self.0 as f64 / 1024.0)
        } else if self.0 < 1024 * 1024 * 1024 {
            write!(f, "{:.1}MB", self.0 as f64 / (1024.0 * 1024.0))
        } else {
            write!(f, "{:.1}GB", self.0 as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

// ---------------------------------------------------------------------------
// JSON formatter for structured logs
// ---------------------------------------------------------------------------

/// Format a structured log entry as JSON.
pub fn format_json(entry: &StructuredLogEntry) -> String {
    serde_json::to_string(entry).unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.into())
}

/// Format a structured log entry as a human-readable line.
pub fn format_pretty(entry: &StructuredLogEntry) -> String {
    let level = match entry.level {
        LogLevel::Error => "ERROR",
        LogLevel::Warn => " WARN",
        LogLevel::Info => " INFO",
        LogLevel::Debug => "DEBUG",
        LogLevel::Trace => "TRACE",
    };

    let mut line = format!("{} {} {}", entry.timestamp, level, entry.message);

    for (key, value) in &entry.fields {
        line.push_str(&format!(" {}={}", key, value));
    }

    if let Some(ref span) = entry.span {
        line.push_str(&format!(" span={}", span));
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_builder() {
        let entry = LogEntryBuilder::new(LogLevel::Info, "user created")
            .field("user_id", 42i64)
            .field("email", "alice@example.com")
            .span("auth")
            .build();

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "user created");
        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.span, Some("auth".to_string()));
    }

    #[test]
    fn test_field_value_types() {
        let str_val: FieldValue = "hello".into();
        let i64_val: FieldValue = 42i64.into();
        let u64_val: FieldValue = 42u64.into();
        let f64_val: FieldValue = std::f64::consts::PI.into();
        let bool_val: FieldValue = true.into();

        assert_eq!(str_val.type_name(), "string");
        assert_eq!(i64_val.type_name(), "integer");
        assert_eq!(u64_val.type_name(), "unsigned");
        assert_eq!(f64_val.type_name(), "float");
        assert_eq!(bool_val.type_name(), "boolean");
    }

    #[test]
    fn test_field_value_display() {
        assert_eq!(FieldValue::Str("test".into()).display(), "\"test\"");
        assert_eq!(FieldValue::I64(-42).display(), "-42");
        assert_eq!(FieldValue::Bool(true).display(), "true");
        assert_eq!(FieldValue::F64(3.14).display(), "3.14");
    }

    #[test]
    fn test_field_value_array() {
        let arr = FieldValue::Array(vec![
            FieldValue::I64(1),
            FieldValue::I64(2),
            FieldValue::I64(3),
        ]);
        assert_eq!(arr.display(), "[1, 2, 3]");
        assert_eq!(arr.type_name(), "array");
    }

    #[test]
    fn test_field_value_object() {
        let mut map = BTreeMap::new();
        map.insert("key".to_string(), FieldValue::Str("value".into()));
        let obj = FieldValue::Object(map);
        assert!(obj.display().contains("key: \"value\""));
        assert_eq!(obj.type_name(), "object");
    }

    #[test]
    fn test_user_id_display() {
        let uid = UserId(42);
        assert_eq!(uid.to_string(), "user:42");
    }

    #[test]
    fn test_request_id_display() {
        let rid = RequestId("abc-123".into());
        assert_eq!(rid.to_string(), "req:abc-123");
    }

    #[test]
    fn test_log_duration_millis() {
        let d = LogDuration(Duration::from_millis(150));
        assert_eq!(d.to_string(), "150ms");
    }

    #[test]
    fn test_log_duration_seconds() {
        let d = LogDuration(Duration::from_millis(2500));
        assert_eq!(d.to_string(), "2.50s");
    }

    #[test]
    fn test_byte_size() {
        assert_eq!(ByteSize(100).to_string(), "100B");
        assert_eq!(ByteSize(1024).to_string(), "1.0KB");
        assert_eq!(ByteSize(1024 * 1024).to_string(), "1.0MB");
        assert_eq!(ByteSize(1024 * 1024 * 1024).to_string(), "1.0GB");
    }

    #[test]
    fn test_format_json() {
        let entry = LogEntryBuilder::new(LogLevel::Info, "test")
            .field("key", "value")
            .build();

        let json = format_json(&entry);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["level"], "Info");
        assert_eq!(parsed["message"], "test");
    }

    #[test]
    fn test_format_pretty() {
        let entry = LogEntryBuilder::new(LogLevel::Info, "request completed")
            .field("duration_ms", 42i64)
            .span("handler")
            .build();

        let pretty = format_pretty(&entry);
        assert!(pretty.contains(" INFO"));
        assert!(pretty.contains("request completed"));
        assert!(pretty.contains("duration_ms=42"));
        assert!(pretty.contains("span=handler"));
    }

    #[test]
    fn test_format_pretty_error() {
        let entry = LogEntryBuilder::new(LogLevel::Error, "something failed").build();
        let pretty = format_pretty(&entry);
        assert!(pretty.contains("ERROR"));
    }

    #[test]
    fn test_format_pretty_warn() {
        let entry = LogEntryBuilder::new(LogLevel::Warn, "warning").build();
        let pretty = format_pretty(&entry);
        assert!(pretty.contains(" WARN"));
    }

    #[test]
    fn test_format_pretty_debug() {
        let entry = LogEntryBuilder::new(LogLevel::Debug, "debug info").build();
        let pretty = format_pretty(&entry);
        assert!(pretty.contains("DEBUG"));
    }

    #[test]
    fn test_log_entry_no_span() {
        let entry = LogEntryBuilder::new(LogLevel::Info, "test").build();
        assert!(entry.span.is_none());
        let pretty = format_pretty(&entry);
        assert!(!pretty.contains("span="));
    }

    #[test]
    fn test_log_entry_no_fields() {
        let entry = LogEntryBuilder::new(LogLevel::Info, "test").build();
        assert!(entry.fields.is_empty());
    }

    #[test]
    fn test_field_value_from_string() {
        let val: FieldValue = String::from("test").into();
        assert_eq!(val.type_name(), "string");
    }
}
