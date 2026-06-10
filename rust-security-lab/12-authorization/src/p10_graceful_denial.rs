//! # Lesson 10: Graceful Denial
//!
//! ## What is Graceful Denial?
//!
//! When denying access, HOW you deny matters as much as WHETHER you deny.
//! A poorly crafted error response can leak information to attackers.
//!
//! ## Information Leakage in Error Messages
//!
//! ```
//! BAD:  "Access denied: user 'bob' is not in role 'admin' for resource '/api/users/456'"
//!       → Leaks: username format, role model, resource path structure, internal ID scheme
//!
//! BAD:  "Access denied: resource not found" (when resource exists but user lacks access)
//!       → Leaks: tells attacker the resource EXISTS (they just can't see it)
//!
//! GOOD: "Access denied"  (for both "not found" and "no permission")
//!       → Attacker can't distinguish between "doesn't exist" and "exists but hidden"
//!
//! GOOD: "Access denied" with detailed INTERNAL log (server-side only)
//!       → Attacker gets nothing useful; admins get full debugging info
//! ```
//!
//! ## 🔴 Attack: Enumeration via Error Differentiation
//!
//! If "resource not found" returns 404 but "access denied" returns 403, an attacker
//! can enumerate which resources exist. Always return the SAME error for both cases.
//!
//! ## Best Practices
//!
//! 1. **Same error for same category**: 403 for all access denied, regardless of reason
//! 2. **Generic messages**: "Access denied" not "You need role X"
//! 3. **Detailed server logs**: Log the real reason internally
//! 4. **Rate limiting**: Throttle repeated denied requests
//! 5. **No stack traces**: Never expose internal errors to clients

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Internal (server-side) details about why access was denied.
/// NEVER expose this to the client.
#[derive(Debug, Clone)]
pub struct InternalDenialReason {
    pub user: String,
    pub resource: String,
    pub action: String,
    pub detailed_reason: String,
    pub rule_matched: Option<String>,
    pub timestamp: u64,
}

/// The safe, sanitized error response sent to the client.
#[derive(Debug, Clone, PartialEq)]
pub struct SafeErrorResponse {
    /// HTTP status code (e.g., 403 or 404).
    pub status_code: u16,
    /// A generic, non-revealing error message.
    pub message: String,
    /// A request ID for correlation (client can reference this in support tickets).
    pub request_id: String,
}

/// The graceful denial service that produces safe error responses
/// while logging detailed reasons server-side.
pub struct GracefulDenialService {
    /// Internal audit log of denial reasons (server-side only).
    internal_log: Vec<InternalDenialReason>,
    /// Request counter for generating unique request IDs.
    request_counter: u64,
    /// Whether the system is in development mode (may expose more info).
    dev_mode: bool,
    /// Rate limiting: tracks denied requests per IP.
    denial_counts: HashMap<String, Vec<u64>>,
    /// Window in seconds for rate limiting.
    rate_limit_window: u64,
    /// Max denials within the window before throttling.
    rate_limit_max: usize,
}

impl GracefulDenialService {
    /// Create a new graceful denial service.
    pub fn new(dev_mode: bool, rate_limit_window: u64, rate_limit_max: usize) -> Self {
        todo!("Initialize the service")
    }

    /// Generate a unique request ID.
    fn next_request_id(&mut self) -> String {
        self.request_counter += 1;
        format!("req-{:08x}", self.request_counter)
    }

    /// Create a safe error response for access denied.
    ///
    /// In production mode, always returns the same generic message regardless
    /// of the internal reason. In dev mode, includes a sanitized hint.
    ///
    /// The detailed reason is logged server-side (internal_log) but NEVER
    /// in the client response.
    pub fn access_denied(
        &mut self,
        user: &str,
        resource: &str,
        action: &str,
        detailed_reason: &str,
        rule_matched: Option<&str>,
    ) -> SafeErrorResponse {
        todo!("Log internally and return a safe response")
    }

    /// Create a safe error response for "resource not found."
    ///
    /// This returns the SAME status code and message as access_denied
    /// to prevent resource enumeration.
    pub fn not_found(
        &mut self,
        user: &str,
        resource: &str,
    ) -> SafeErrorResponse {
        todo!("Return same error as access_denied to prevent enumeration")
    }

    /// Check if a client IP should be rate-limited.
    ///
    /// Returns true if the IP has exceeded the denial threshold
    /// within the rate limit window.
    pub fn should_throttle(&self, client_ip: &str) -> bool {
        todo!("Count recent denials for this IP and compare to threshold")
    }

    /// Record a denial for rate limiting purposes.
    pub fn record_denial(&mut self, client_ip: &str) {
        todo!("Add the current timestamp to the IP's denial list")
    }

    /// Get the internal audit log (server-side only).
    pub fn internal_log(&self) -> &[InternalDenialReason] {
        &self.internal_log
    }

    /// Get the current Unix timestamp.
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
        // Message should be generic
        assert_eq!(response.message, "Access denied");
        assert_eq!(response.status_code, 403);
    }

    #[test]
    fn test_not_found_same_as_denied() {
        let mut svc = GracefulDenialService::new(false, 60, 10);
        let denied = svc.access_denied("bob", "/docs/42", "read", "No permission", None);
        let not_found = svc.not_found("bob", "/docs/99");

        // Both should return the same status code (prevent enumeration)
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
        // Dev mode may include a sanitized hint (but still no internals)
        // The message should NOT contain the detailed reason
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
        // IP 10.0.0.1 is throttled
        assert!(svc.should_throttle("10.0.0.1"));
        // IP 10.0.0.2 is NOT throttled
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
        // Response must NOT contain any internal details
        let response_str = format!("{:?}", response);
        assert!(!response_str.contains("bob"));
        assert!(!response_str.contains("456"));
        assert!(!response_str.contains("alice"));
        assert!(!response_str.contains("ownership-rule"));
    }
}
