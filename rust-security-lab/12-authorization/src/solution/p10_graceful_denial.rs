//! # Lesson 10: Graceful Denial (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct InternalDenialReason {
    pub user: String,
    pub resource: String,
    pub action: String,
    pub detailed_reason: String,
    pub rule_matched: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SafeErrorResponse {
    pub status_code: u16,
    pub message: String,
    pub request_id: String,
}

pub struct GracefulDenialService {
    internal_log: Vec<InternalDenialReason>,
    request_counter: u64,
    _dev_mode: bool,
    denial_counts: HashMap<String, Vec<u64>>,
    rate_limit_window: u64,
    rate_limit_max: usize,
}

impl GracefulDenialService {
    pub fn new(dev_mode: bool, rate_limit_window: u64, rate_limit_max: usize) -> Self {
        Self {
            internal_log: Vec::new(),
            request_counter: 0,
            _dev_mode: dev_mode,
            denial_counts: HashMap::new(),
            rate_limit_window,
            rate_limit_max,
        }
    }

    fn next_request_id(&mut self) -> String {
        self.request_counter += 1;
        format!("req-{:08x}", self.request_counter)
    }

    /// Always returns a generic "Access denied" to the client.
    /// Detailed reason goes to the internal log only.
    pub fn access_denied(
        &mut self,
        user: &str,
        resource: &str,
        action: &str,
        detailed_reason: &str,
        rule_matched: Option<&str>,
    ) -> SafeErrorResponse {
        // Log detailed reason server-side
        self.internal_log.push(InternalDenialReason {
            user: user.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            detailed_reason: detailed_reason.to_string(),
            rule_matched: rule_matched.map(|s| s.to_string()),
            timestamp: Self::current_timestamp(),
        });

        // Return a safe, generic response to the client
        SafeErrorResponse {
            status_code: 403,
            message: "Access denied".to_string(),
            request_id: self.next_request_id(),
        }
    }

    /// Returns the SAME generic response as access_denied.
    /// This prevents attackers from distinguishing "doesn't exist" from "no permission."
    pub fn not_found(
        &mut self,
        user: &str,
        resource: &str,
    ) -> SafeErrorResponse {
        // Log internally that the resource was not found
        self.internal_log.push(InternalDenialReason {
            user: user.to_string(),
            resource: resource.to_string(),
            action: "access".to_string(),
            detailed_reason: "Resource not found".to_string(),
            rule_matched: None,
            timestamp: Self::current_timestamp(),
        });

        // Same generic response as access_denied
        SafeErrorResponse {
            status_code: 403,
            message: "Access denied".to_string(),
            request_id: self.next_request_id(),
        }
    }

    pub fn should_throttle(&self, client_ip: &str) -> bool {
        let now = Self::current_timestamp();
        self.denial_counts
            .get(client_ip)
            .map(|timestamps| {
                let recent = timestamps
                    .iter()
                    .filter(|&&ts| now.saturating_sub(ts) <= self.rate_limit_window)
                    .count();
                recent >= self.rate_limit_max
            })
            .unwrap_or(false)
    }

    pub fn record_denial(&mut self, client_ip: &str) {
        let now = Self::current_timestamp();
        let timestamps = self
            .denial_counts
            .entry(client_ip.to_string())
            .or_default();
        timestamps.push(now);

        // Clean up old entries outside the window
        timestamps.retain(|&ts| now.saturating_sub(ts) <= self.rate_limit_window);
    }

    pub fn internal_log(&self) -> &[InternalDenialReason] {
        &self.internal_log
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_denied_generic_message() {
        let mut svc = GracefulDenialService::new(false, 60, 10);
        let response = svc.access_denied(
            "bob",
            "/admin/config",
            "write",
            "User 'bob' lacks role 'admin'",
            Some("admin-only-rule"),
        );
        assert_eq!(response.message, "Access denied");
        assert_eq!(response.status_code, 403);
    }

    #[test]
    fn test_not_found_same_as_denied() {
        let mut svc = GracefulDenialService::new(false, 60, 10);
        let denied = svc.access_denied("bob", "/docs/42", "read", "No permission", None);
        let not_found = svc.not_found("bob", "/docs/99");

        assert_eq!(denied.status_code, not_found.status_code);
        assert_eq!(denied.message, not_found.message);
    }

    #[test]
    fn test_internal_log_detailed() {
        let mut svc = GracefulDenialService::new(false, 60, 10);
        svc.access_denied(
            "bob",
            "/admin/config",
            "write",
            "User 'bob' lacks role 'admin'",
            Some("admin-only-rule"),
        );
        assert_eq!(svc.internal_log().len(), 1);
        let log = &svc.internal_log()[0];
        assert_eq!(log.user, "bob");
        assert_eq!(log.detailed_reason, "User 'bob' lacks role 'admin'");
    }

    #[test]
    fn test_request_ids_unique() {
        let mut svc = GracefulDenialService::new(false, 60, 10);
        let r1 = svc.access_denied("alice", "/a", "read", "reason", None);
        let r2 = svc.access_denied("alice", "/b", "read", "reason", None);
        assert_ne!(r1.request_id, r2.request_id);
    }

    #[test]
    fn test_dev_mode_includes_hint() {
        let mut svc = GracefulDenialService::new(true, 60, 10);
        let response = svc.access_denied(
            "bob",
            "/admin/config",
            "write",
            "Insufficient role",
            Some("admin-rule"),
        );
        // Dev mode: message should still be generic (not leak internals)
        assert!(!response.message.contains("Insufficient role"));
    }

    #[test]
    fn test_rate_limiting() {
        let mut svc = GracefulDenialService::new(false, 60, 3);
        svc.record_denial("10.0.0.1");
        svc.record_denial("10.0.0.1");
        assert!(!svc.should_throttle("10.0.0.1"));
        svc.record_denial("10.0.0.1");
        assert!(svc.should_throttle("10.0.0.1"));
    }

    #[test]
    fn test_rate_limiting_per_ip() {
        let mut svc = GracefulDenialService::new(false, 60, 3);
        svc.record_denial("10.0.0.1");
        svc.record_denial("10.0.0.1");
        svc.record_denial("10.0.0.1");
        assert!(svc.should_throttle("10.0.0.1"));
        assert!(!svc.should_throttle("10.0.0.2"));
    }

    #[test]
    fn test_no_information_leakage_in_response() {
        let mut svc = GracefulDenialService::new(false, 60, 10);
        let response = svc.access_denied(
            "bob",
            "/api/users/456",
            "delete",
            "User 'bob' (id=789) lacks 'delete' on resource 'users/456' (owner=alice)",
            Some("ownership-rule"),
        );
        let response_str = format!("{:?}", response);
        assert!(!response_str.contains("bob"));
        assert!(!response_str.contains("456"));
        assert!(!response_str.contains("alice"));
        assert!(!response_str.contains("ownership-rule"));
    }
}
