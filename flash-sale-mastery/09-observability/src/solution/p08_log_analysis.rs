//! # Solution 08: Log Analysis
//!
//! Complete implementation of structured log parsing, filtering, and aggregation.

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

    /// Return a numeric severity value for comparison (higher = more severe).
    fn severity(&self) -> u8 {
        match self {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        }
    }
}

/// A parsed structured log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub service: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default)]
    pub fields: HashMap<String, serde_json::Value>,
}

/// Summary of error patterns.
#[derive(Debug, Clone)]
pub struct ErrorSummary {
    pub total_errors: usize,
    pub error_counts: HashMap<String, usize>,
    pub top_error: Option<String>,
    pub affected_services: Vec<String>,
}

/// Parse a JSON-formatted structured log line.
pub fn parse_structured_log(line: &str) -> Option<LogEntry> {
    serde_json::from_str(line).ok()
}

/// Filter log entries by severity level.
/// Returns entries at the specified level or more severe.
pub fn filter_by_level<'a>(
    entries: &'a [LogEntry],
    level: LogLevel,
) -> Vec<&'a LogEntry> {
    let min_severity = level.severity();
    entries
        .iter()
        .filter(|e| e.level.severity() >= min_severity)
        .collect()
}

/// Filter log entries by time range [start, end).
pub fn filter_by_time_range<'a>(
    entries: &'a [LogEntry],
    start: &NaiveDateTime,
    end: &NaiveDateTime,
) -> Vec<&'a LogEntry> {
    entries
        .iter()
        .filter(|e| {
            if let Ok(ts) = NaiveDateTime::parse_from_str(
                &e.timestamp.replace('T', " ").trim_end_matches('Z'),
                "%Y-%m-%d %H:%M:%S%.3f",
            )
            .or_else(|_| {
                NaiveDateTime::parse_from_str(
                    &e.timestamp.replace('T', " ").trim_end_matches('Z'),
                    "%Y-%m-%d %H:%M:%S",
                )
            }) {
                ts >= *start && ts < *end
            } else {
                false
            }
        })
        .collect()
}

/// Aggregate error entries into a summary.
pub fn aggregate_errors(entries: &[LogEntry]) -> ErrorSummary {
    let errors: Vec<&LogEntry> = entries
        .iter()
        .filter(|e| e.level == LogLevel::Error)
        .collect();

    let mut error_counts: HashMap<String, usize> = HashMap::new();
    let mut affected_services_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    for entry in &errors {
        *error_counts.entry(entry.message.clone()).or_insert(0) += 1;
        affected_services_set.insert(entry.service.clone());
    }

    let top_error = error_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(msg, _)| msg.clone());

    let mut affected_services: Vec<String> = affected_services_set.into_iter().collect();
    affected_services.sort();

    ErrorSummary {
        total_errors: errors.len(),
        error_counts,
        top_error,
        affected_services,
    }
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
        assert_eq!(summary.error_counts.get("Redis connection timeout"), Some(&2));
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
