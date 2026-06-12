//! # Exercise 08: Log Analysis
//!
//! ## Learning Objective
//! Learn how to parse, filter, and aggregate structured log entries. When
//! debugging a flash sale incident, you need to quickly find relevant log
//! entries among millions of lines. Structured log analysis replaces manual
//! grep with programmatic queries.
//!
//! ## Flash Sale Context
//! After a flash sale, you may need to answer: "How many ERROR logs occurred
//! between 14:00 and 14:05?" or "What were the top 5 error messages?" or
//! "Which service had the most failures?" Structured log analysis makes this
//! tractable.
//!
//! ## Instructions
//! 1. Implement `parse_structured_log` to parse JSON log lines
//! 2. Implement `filter_by_level` to filter entries by severity
//! 3. Implement `filter_by_time_range` to filter entries by timestamp
//! 4. Implement `aggregate_errors` to summarize error patterns
//!
//! ## Hints
//! - Use `serde_json` to parse JSON log lines
//! - Use `chrono::NaiveDateTime` for timestamp parsing
//! - Use a HashMap to count error message occurrences
//! - Handle malformed log lines gracefully (return None)

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Log severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

/// A parsed structured log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// ISO 8601 timestamp (e.g., "2024-01-15T14:00:01.123Z").
    pub timestamp: String,
    /// Log severity level.
    pub level: LogLevel,
    /// Log message.
    pub message: String,
    /// Service that produced the log.
    pub service: String,
    /// Optional request ID for correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Additional structured fields.
    #[serde(default)]
    pub fields: HashMap<String, serde_json::Value>,
}

/// Summary of error patterns.
#[derive(Debug, Clone)]
pub struct ErrorSummary {
    /// Total number of error entries.
    pub total_errors: usize,
    /// Count of each unique error message.
    pub error_counts: HashMap<String, usize>,
    /// The most frequent error message (if any errors exist).
    pub top_error: Option<String>,
    /// Services that produced errors.
    pub affected_services: Vec<String>,
}

/// Parse a JSON-formatted structured log line.
///
/// # Arguments
/// * `line` - A single line of JSON-encoded log data
///
/// # Returns
/// `Some(LogEntry)` if the line is valid JSON with required fields,
/// `None` if the line is malformed or missing required fields.
pub fn parse_structured_log(line: &str) -> Option<LogEntry> {
    // TODO: Try to deserialize the line as a LogEntry
    // TODO: Return None if deserialization fails
    todo!("Implement log line parsing")
}

/// Filter log entries by severity level.
///
/// Returns entries at the specified level or more severe.
/// Severity order: Debug < Info < Warn < Error.
///
/// # Arguments
/// * `entries` - Log entries to filter
/// * `level` - Minimum severity level to include
///
/// # Returns
/// A vector of references to entries at or above the specified level.
pub fn filter_by_level<'a>(
    entries: &'a [LogEntry],
    level: LogLevel,
) -> Vec<&'a LogEntry> {
    // TODO: Filter entries where the level is >= the specified level
    todo!("Implement level filtering")
}

/// Filter log entries by time range.
///
/// Returns entries whose parsed timestamp falls within [start, end).
///
/// # Arguments
/// * `entries` - Log entries to filter
/// * `start` - Start of the time range (inclusive)
/// * `end` - End of the time range (exclusive)
///
/// # Returns
/// A vector of references to entries within the time range.
pub fn filter_by_time_range<'a>(
    entries: &'a [LogEntry],
    start: &NaiveDateTime,
    end: &NaiveDateTime,
) -> Vec<&'a LogEntry> {
    // TODO: Parse each entry's timestamp and compare to [start, end)
    // TODO: Skip entries with unparseable timestamps
    todo!("Implement time range filtering")
}

/// Aggregate error entries into a summary.
///
/// # Arguments
/// * `entries` - Log entries to analyze
///
/// # Returns
/// An `ErrorSummary` with error counts, top error, and affected services.
pub fn aggregate_errors(entries: &[LogEntry]) -> ErrorSummary {
    // TODO: Filter to only Error level entries
    // TODO: Count occurrences of each unique message
    // TODO: Find the most frequent error message
    // TODO: Collect unique service names
    todo!("Implement error aggregation")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<LogEntry> {
        vec![
            LogEntry {
                timestamp: "2024-01-15T14:00:01.000Z".to_string(),
                level: LogLevel::Info,
                message: "Purchase completed".to_string(),
                service: "api-gateway".to_string(),
                request_id: Some("req-1".to_string()),
                fields: HashMap::new(),
            },
            LogEntry {
                timestamp: "2024-01-15T14:00:02.000Z".to_string(),
                level: LogLevel::Error,
                message: "Redis connection timeout".to_string(),
                service: "inventory".to_string(),
                request_id: Some("req-2".to_string()),
                fields: HashMap::new(),
            },
            LogEntry {
                timestamp: "2024-01-15T14:00:03.000Z".to_string(),
                level: LogLevel::Error,
                message: "Redis connection timeout".to_string(),
                service: "inventory".to_string(),
                request_id: Some("req-3".to_string()),
                fields: HashMap::new(),
            },
            LogEntry {
                timestamp: "2024-01-15T14:00:04.000Z".to_string(),
                level: LogLevel::Warn,
                message: "Stock running low".to_string(),
                service: "inventory".to_string(),
                request_id: None,
                fields: HashMap::new(),
            },
            LogEntry {
                timestamp: "2024-01-15T14:00:05.000Z".to_string(),
                level: LogLevel::Error,
                message: "Payment processing failed".to_string(),
                service: "payment".to_string(),
                request_id: Some("req-4".to_string()),
                fields: HashMap::new(),
            },
        ]
    }

    #[test]
    fn test_parse_valid_log_line() {
        let line = r#"{"timestamp":"2024-01-15T14:00:01.000Z","level":"info","message":"test","service":"api"}"#;
        let entry = parse_structured_log(line);
        assert!(entry.is_some(), "Should parse valid JSON");
        let entry = entry.unwrap();
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "test");
        assert_eq!(entry.service, "api");
    }

    #[test]
    fn test_parse_with_optional_fields() {
        let line = r#"{"timestamp":"2024-01-15T14:00:01.000Z","level":"error","message":"fail","service":"api","request_id":"req-1","fields":{"code":500}}"#;
        let entry = parse_structured_log(line).expect("should parse");
        assert_eq!(entry.request_id, Some("req-1".to_string()));
        assert!(entry.fields.contains_key("code"));
    }

    #[test]
    fn test_parse_malformed_json() {
        let entry = parse_structured_log("not json at all");
        assert!(entry.is_none(), "Should return None for malformed input");
    }

    #[test]
    fn test_parse_empty_string() {
        let entry = parse_structured_log("");
        assert!(entry.is_none());
    }

    #[test]
    fn test_filter_by_level_error_only() {
        let entries = sample_entries();
        let errors = filter_by_level(&entries, LogLevel::Error);
        assert_eq!(errors.len(), 3, "Should have 3 error entries");
    }

    #[test]
    fn test_filter_by_level_warn_and_above() {
        let entries = sample_entries();
        let warn_and_above = filter_by_level(&entries, LogLevel::Warn);
        assert_eq!(warn_and_above.len(), 4, "Should have 4 entries (3 errors + 1 warn)");
    }

    #[test]
    fn test_filter_by_level_all() {
        let entries = sample_entries();
        let all = filter_by_level(&entries, LogLevel::Debug);
        assert_eq!(all.len(), 5, "Should have all 5 entries");
    }

    #[test]
    fn test_filter_by_time_range() {
        let entries = sample_entries();
        let start = NaiveDateTime::parse_from_str("2024-01-15 14:00:02", "%Y-%m-%d %H:%M:%S")
            .unwrap();
        let end = NaiveDateTime::parse_from_str("2024-01-15 14:00:05", "%Y-%m-%d %H:%M:%S")
            .unwrap();
        let filtered = filter_by_time_range(&entries, &start, &end);
        assert_eq!(filtered.len(), 3, "Should have 3 entries in [02, 05)");
    }

    #[test]
    fn test_aggregate_errors() {
        let entries = sample_entries();
        let summary = aggregate_errors(&entries);
        assert_eq!(summary.total_errors, 3);
        assert_eq!(
            summary.top_error,
            Some("Redis connection timeout".to_string()),
            "Most frequent error should be Redis timeout"
        );
        assert_eq!(
            summary.error_counts.get("Redis connection timeout"),
            Some(&2)
        );
        assert!(
            summary.affected_services.contains(&"inventory".to_string()),
            "inventory should be an affected service"
        );
        assert!(
            summary.affected_services.contains(&"payment".to_string()),
            "payment should be an affected service"
        );
    }

    #[test]
    fn test_aggregate_no_errors() {
        let entries: Vec<LogEntry> = vec![LogEntry {
            timestamp: "2024-01-15T14:00:01.000Z".to_string(),
            level: LogLevel::Info,
            message: "All good".to_string(),
            service: "api".to_string(),
            request_id: None,
            fields: HashMap::new(),
        }];
        let summary = aggregate_errors(&entries);
        assert_eq!(summary.total_errors, 0);
        assert!(summary.top_error.is_none());
    }
}
