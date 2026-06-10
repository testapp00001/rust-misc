//! # Lesson 08: Correlation IDs for Request Tracing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn generate_correlation_id() -> String {
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("req-{:016x}", count)
}

pub fn correlation_id_from_seed(seed: u64) -> String {
    format!("req-{:016x}", seed)
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub correlation_id: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub method: String,
    pub path: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

pub fn new_request_context(method: &str, path: &str) -> RequestContext {
    let now = chrono::Utc::now();
    let seed = now.timestamp_nanos_opt().unwrap_or(0) as u64;
    RequestContext {
        correlation_id: correlation_id_from_seed(seed),
        user_id: None,
        ip_address: None,
        method: method.to_string(),
        path: path.to_string(),
        start_time: now,
    }
}

pub fn format_with_correlation(ctx: &RequestContext, message: &str) -> String {
    match &ctx.user_id {
        Some(uid) => format!("[{}] [user:{}] {}", ctx.correlation_id, uid, message),
        None => format!("[{}] {}", ctx.correlation_id, message),
    }
}

pub fn extract_correlation_id(headers: &HashMap<String, String>) -> Option<String> {
    for (key, value) in headers {
        if key.to_lowercase() == "x-correlation-id" {
            return Some(value.clone());
        }
    }
    None
}

pub fn inject_correlation_id(headers: &mut HashMap<String, String>, id: &str) {
    headers.insert("X-Correlation-ID".to_string(), id.to_string());
}

#[derive(Debug, Clone)]
pub struct CorrelatedLogEntry {
    pub correlation_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub service: String,
    pub message: String,
}

pub struct TraceCollector {
    pub entries: Vec<CorrelatedLogEntry>,
}

impl TraceCollector {
    pub fn new() -> Self {
        TraceCollector { entries: Vec::new() }
    }

    pub fn record(&mut self, entry: CorrelatedLogEntry) {
        self.entries.push(entry);
    }

    pub fn trace_for(&self, correlation_id: &str) -> Vec<&CorrelatedLogEntry> {
        let mut trace: Vec<&CorrelatedLogEntry> = self
            .entries
            .iter()
            .filter(|e| e.correlation_id == correlation_id)
            .collect();
        trace.sort_by_key(|e| e.timestamp);
        trace
    }

    pub fn all_correlation_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .entries
            .iter()
            .map(|e| e.correlation_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        ids.sort();
        ids
    }

    pub fn format_trace(&self, correlation_id: &str) -> String {
        let entries = self.trace_for(correlation_id);
        let mut result = format!("Trace: {}\n", correlation_id);
        for entry in &entries {
            result.push_str(&format!("  [{}] {}\n", entry.service, entry.message));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_format() {
        let id = correlation_id_from_seed(42);
        assert!(id.starts_with("req-"));
        assert_eq!(id.len(), 20);
    }

    #[test]
    fn test_correlation_id_deterministic() {
        let id1 = correlation_id_from_seed(42);
        let id2 = correlation_id_from_seed(42);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_correlation_id_unique() {
        let id1 = correlation_id_from_seed(1);
        let id2 = correlation_id_from_seed(2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_request_context_creation() {
        let ctx = new_request_context("GET", "/api/users");
        assert_eq!(ctx.method, "GET");
        assert_eq!(ctx.path, "/api/users");
        assert!(ctx.correlation_id.starts_with("req-"));
    }

    #[test]
    fn test_format_with_correlation() {
        let ctx = RequestContext {
            correlation_id: "req-test-123".to_string(),
            user_id: Some("user-1".to_string()),
            ip_address: None,
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            start_time: chrono::Utc::now(),
        };
        let formatted = format_with_correlation(&ctx, "test message");
        assert!(formatted.contains("req-test-123"));
        assert!(formatted.contains("user:user-1"));
        assert!(formatted.contains("test message"));
    }

    #[test]
    fn test_format_without_user() {
        let ctx = RequestContext {
            correlation_id: "req-test-456".to_string(),
            user_id: None,
            ip_address: None,
            method: "GET".to_string(),
            path: "/health".to_string(),
            start_time: chrono::Utc::now(),
        };
        let formatted = format_with_correlation(&ctx, "health check");
        assert!(formatted.contains("req-test-456"));
        assert!(!formatted.contains("user:"));
    }

    #[test]
    fn test_extract_correlation_id() {
        let mut headers = HashMap::new();
        headers.insert("X-Correlation-ID".to_string(), "req-abc".to_string());
        assert_eq!(extract_correlation_id(&headers), Some("req-abc".to_string()));
    }

    #[test]
    fn test_extract_missing_correlation_id() {
        let headers = HashMap::new();
        assert_eq!(extract_correlation_id(&headers), None);
    }

    #[test]
    fn test_inject_correlation_id() {
        let mut headers = HashMap::new();
        inject_correlation_id(&mut headers, "req-xyz");
        assert_eq!(headers.get("X-Correlation-ID"), Some(&"req-xyz".to_string()));
    }

    #[test]
    fn test_trace_collector_grouping() {
        let mut collector = TraceCollector::new();
        let now = chrono::Utc::now();

        collector.record(CorrelatedLogEntry {
            correlation_id: "req-1".to_string(),
            timestamp: now,
            service: "service-a".to_string(),
            message: "start".to_string(),
        });
        collector.record(CorrelatedLogEntry {
            correlation_id: "req-2".to_string(),
            timestamp: now,
            service: "service-b".to_string(),
            message: "other request".to_string(),
        });
        collector.record(CorrelatedLogEntry {
            correlation_id: "req-1".to_string(),
            timestamp: now + chrono::Duration::milliseconds(10),
            service: "service-a".to_string(),
            message: "end".to_string(),
        });

        let trace = collector.trace_for("req-1");
        assert_eq!(trace.len(), 2);
        assert_eq!(collector.all_correlation_ids().len(), 2);
    }

    #[test]
    fn test_format_trace() {
        let mut collector = TraceCollector::new();
        let now = chrono::Utc::now();
        collector.record(CorrelatedLogEntry {
            correlation_id: "req-abc".to_string(),
            timestamp: now,
            service: "api".to_string(),
            message: "request received".to_string(),
        });
        collector.record(CorrelatedLogEntry {
            correlation_id: "req-abc".to_string(),
            timestamp: now,
            service: "db".to_string(),
            message: "query executed".to_string(),
        });

        let formatted = collector.format_trace("req-abc");
        assert!(formatted.contains("req-abc"));
        assert!(formatted.contains("[api]"));
        assert!(formatted.contains("[db]"));
    }
}
