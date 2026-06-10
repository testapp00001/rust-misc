//! # Lesson 08: Correlation IDs for Request Tracing
//!
//! ## The Problem
//!
//! In a microservices architecture, a single user request may traverse dozens of
//! services. When something goes wrong, you need to correlate all the log entries
//! from that request across all services. Without a correlation ID, you're left
//! grepping logs by timestamp and hoping for the best.
//!
//! ## How Correlation IDs Work
//!
//! 1. The edge service (API gateway) generates a unique ID when a request arrives
//! 2. The ID is passed to every downstream service (via HTTP header, message queue, etc.)
//! 3. Every service includes the ID in all log entries
//! 4. To investigate an issue, search logs for the correlation ID
//!
//! ```
//! Request arrives -> Generate correlation ID: "req-abc-123"
//!   Service A: [req-abc-123] Processing request
//!   Service B: [req-abc-123] Calling database
//!   Service C: [req-abc-123] Sending email
//!   Service A: [req-abc-123] Request complete
//! ```
//!
//! ## Attack Scenario
//!
//! An attacker performs a complex multi-step attack spanning several services.
//! Without correlation IDs, you can't reconstruct the attack chain. With them,
//! you can pull all related log entries and see the full picture:
//! "Show me everything that happened during request req-abc-123"
//!
//! ## What You'll Implement
//!
//! 1. Correlation ID generation (UUID v4 format)
//! 2. A request context that carries the ID through the call chain
//! 3. Log formatting with correlation ID injection
//! 4. Correlation ID propagation (extract from headers, inject into requests)
//! 5. A trace collector that groups logs by correlation ID
//! 6. Trace visualization (show the call chain)

use std::collections::HashMap;

/// Exercise 1: Generate a correlation ID.
///
/// Format: `req-` followed by 16 hex characters.
/// Example: `req-a1b2c3d4e5f67890`
///
/// For this exercise, use a simple counter-based approach
/// (production code would use UUID v4 or similar).
///
/// Hints:
/// - Use a static counter with `std::sync::atomic`
/// - Format: `format!("req-{:016x}", counter)`
pub fn generate_correlation_id() -> String {
    todo!("Implement correlation ID generation")
}

/// Exercise 2: Generate a correlation ID from a seed (deterministic).
///
/// This is useful for testing. Given the same seed, produce the same ID.
///
/// Hints:
/// - Format: `format!("req-{:016x}", seed)`
pub fn correlation_id_from_seed(seed: u64) -> String {
    todo!("Implement deterministic correlation ID")
}

/// A request context that carries metadata through the call chain.
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub correlation_id: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub method: String,
    pub path: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

/// Exercise 3: Create a new RequestContext.
///
/// Generate a correlation_id using `correlation_id_from_seed` with
/// the current timestamp nanos as the seed.
pub fn new_request_context(method: &str, path: &str) -> RequestContext {
    todo!("Implement request context creation")
}

/// Exercise 4: Format a log message with the correlation ID.
///
/// Format: `[<correlation_id>] <message>`
///
/// If the context has a user_id, include it:
/// `[<correlation_id>] [user:<user_id>] <message>`
///
/// Hints:
/// - Check if user_id is Some
/// - Use format! to build the string
pub fn format_with_correlation(ctx: &RequestContext, message: &str) -> String {
    todo!("Implement correlation ID formatting")
}

/// Exercise 5: Extract a correlation ID from a header map.
///
/// Look for the key "X-Correlation-ID" (case-insensitive).
/// If not found, return None.
///
/// Hints:
/// - Iterate headers, compare lowercase keys
/// - Return the value if found
pub fn extract_correlation_id(headers: &HashMap<String, String>) -> Option<String> {
    todo!("Implement correlation ID extraction")
}

/// Exercise 6: Inject a correlation ID into a header map.
///
/// Add or replace "X-Correlation-ID" with the given value.
///
/// Hints:
/// - `headers.insert("X-Correlation-ID".to_string(), id.to_string())`
pub fn inject_correlation_id(headers: &mut HashMap<String, String>, id: &str) {
    todo!("Implement correlation ID injection")
}

/// A log entry with its correlation ID for trace grouping.
#[derive(Debug, Clone)]
pub struct CorrelatedLogEntry {
    pub correlation_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub service: String,
    pub message: String,
}

/// A trace collector that groups log entries by correlation ID.
pub struct TraceCollector {
    pub entries: Vec<CorrelatedLogEntry>,
}

impl TraceCollector {
    pub fn new() -> Self {
        TraceCollector { entries: Vec::new() }
    }

    /// Exercise 7: Record a correlated log entry.
    pub fn record(&mut self, entry: CorrelatedLogEntry) {
        todo!("Implement entry recording")
    }

    /// Exercise 8: Get all entries for a specific correlation ID, sorted by time.
    pub fn trace_for(&self, correlation_id: &str) -> Vec<&CorrelatedLogEntry> {
        todo!("Implement trace retrieval")
    }

    /// Exercise 9: Get all unique correlation IDs.
    pub fn all_correlation_ids(&self) -> Vec<String> {
        todo!("Implement correlation ID listing")
    }

    /// Exercise 10: Format a trace as a readable call chain.
    ///
    /// Format:
    /// ```
    /// Trace: req-abc-123
    ///   [service-a] Processing request
    ///   [service-b] Calling database
    ///   [service-a] Request complete
    /// ```
    pub fn format_trace(&self, correlation_id: &str) -> String {
        todo!("Implement trace formatting")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_format() {
        let id = correlation_id_from_seed(42);
        assert!(id.starts_with("req-"));
        assert_eq!(id.len(), 20); // "req-" + 16 hex chars
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
