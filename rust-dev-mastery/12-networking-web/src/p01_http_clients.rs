//! # HTTP Clients with reqwest
//!
//! The `reqwest` crate is the de facto standard HTTP client in Rust. This lesson
//! covers building robust HTTP clients with proper timeout handling, retry logic,
//! connection pooling, and request/response manipulation.
//!
//! ## Key Concepts
//! - Building clients with `ClientBuilder` for connection reuse
//! - Configuring timeouts at the client and request level
//! - Working with headers, cookies, and authentication
//! - Implementing retry logic with exponential backoff
//! - Streaming large responses
//! - Connection pooling and DNS resolution

use std::collections::HashMap;
use std::time::Duration;

// ---------------------------------------------------------------------------
// 1. Client Configuration
// ---------------------------------------------------------------------------

/// Represents the configuration for building an HTTP client.
/// In production, you'd typically load these from environment variables or config files.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub max_idle_per_host: usize,
    pub pool_idle_timeout: Duration,
    pub user_agent: String,
    pub default_headers: HashMap<String, String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_idle_per_host: 10,
            pool_idle_timeout: Duration::from_secs(90),
            user_agent: format!("rust-app/{}", env!("CARGO_PKG_VERSION")),
            default_headers: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Retry Policy
// ---------------------------------------------------------------------------

/// Defines when and how to retry failed HTTP requests.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub retry_on_status: Vec<u16>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            retry_on_status: vec![429, 500, 502, 503, 504],
        }
    }
}

impl RetryPolicy {
    /// Calculate the delay for a given attempt using exponential backoff with jitter.
    /// Returns `None` if the attempt exceeds max retries.
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_retries {
            return None;
        }
        let exp = 2u64.saturating_pow(attempt);
        let delay_ms = self.base_delay.as_millis() as u64 * exp;
        let capped = Duration::from_millis(delay_ms).min(self.max_delay);
        Some(capped)
    }

    /// Check whether a given status code should trigger a retry.
    pub fn should_retry_status(&self, status: u16) -> bool {
        self.retry_on_status.contains(&status)
    }
}

// ---------------------------------------------------------------------------
// 3. Request Builder Pattern
// ---------------------------------------------------------------------------

/// A convenient wrapper for building HTTP requests with common patterns.
/// This demonstrates the builder pattern used extensively in production HTTP clients.
#[derive(Debug, Clone)]
pub struct ApiRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: Vec<(String, String)>,
    pub body: Option<String>,
    pub timeout_override: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl ApiRequest {
    pub fn get(path: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Get,
            path: path.into(),
            headers: HashMap::new(),
            query_params: Vec::new(),
            body: None,
            timeout_override: None,
        }
    }

    pub fn post(path: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Post,
            path: path.into(),
            headers: HashMap::new(),
            query_params: Vec::new(),
            body: None,
            timeout_override: None,
        }
    }

    pub fn put(path: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Put,
            path: path.into(),
            headers: HashMap::new(),
            query_params: Vec::new(),
            body: None,
            timeout_override: None,
        }
    }

    pub fn delete(path: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Delete,
            path: path.into(),
            headers: HashMap::new(),
            query_params: Vec::new(),
            body: None,
            timeout_override: None,
        }
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((key.into(), value.into()));
        self
    }

    pub fn json_body(mut self, body: impl serde::Serialize) -> Self {
        self.body = Some(serde_json::to_string(&body).unwrap_or_default());
        self.headers
            .entry("content-type".to_string())
            .or_insert_with(|| "application/json".to_string());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout_override = Some(timeout);
        self
    }

    /// Build the full URL from base URL and path.
    pub fn build_url(&self, base_url: &str) -> String {
        let base = base_url.trim_end_matches('/');
        let path = self.path.trim_start_matches('/');
        let mut url = format!("{base}/{path}");

        if !self.query_params.is_empty() {
            url.push('?');
            let qs: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            url.push_str(&qs.join("&"));
        }
        url
    }
}

// ---------------------------------------------------------------------------
// 4. Response Wrapper
// ---------------------------------------------------------------------------

/// A structured wrapper around HTTP responses for easier consumption.
#[derive(Debug, Clone)]
pub struct ApiResponse<T> {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: T,
    pub latency: Duration,
}

impl<T> ApiResponse<T> {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }
}

// ---------------------------------------------------------------------------
// 5. HTTP Error Types
// ---------------------------------------------------------------------------

/// Rich error type for HTTP operations, covering all failure modes a
/// production client should handle.
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("request timeout after {0:?}")]
    Timeout(Duration),

    #[error("HTTP {status}: {body}")]
    Status { status: u16, body: String },

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("URL parse error: {0}")]
    UrlParse(String),
}

// ---------------------------------------------------------------------------
// 6. URL Building Utilities
// ---------------------------------------------------------------------------

/// Utility for constructing URLs with proper encoding and validation.
pub struct UrlBuilder {
    base: String,
    segments: Vec<String>,
    query: Vec<(String, String)>,
}

impl UrlBuilder {
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            segments: Vec::new(),
            query: Vec::new(),
        }
    }

    pub fn push_segment(mut self, segment: impl Into<String>) -> Self {
        self.segments.push(segment.into());
        self
    }

    pub fn push_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((key.into(), value.into()));
        self
    }

    pub fn build(self) -> String {
        let mut url = self.base.trim_end_matches('/').to_string();
        for seg in &self.segments {
            url.push('/');
            url.push_str(seg.trim_start_matches('/'));
        }
        if !self.query.is_empty() {
            url.push('?');
            let parts: Vec<String> = self
                .query
                .iter()
                .map(|(k, v)| {
                    // Simple percent encoding for demonstration
                    let ek = k
                        .replace('%', "%25")
                        .replace('&', "%26")
                        .replace('=', "%3D");
                    let ev = v
                        .replace('%', "%25")
                        .replace('&', "%26")
                        .replace('=', "%3D");
                    format!("{ek}={ev}")
                })
                .collect();
            url.push_str(&parts.join("&"));
        }
        url
    }
}

// ---------------------------------------------------------------------------
// 7. Header Map Abstraction
// ---------------------------------------------------------------------------

/// A typed header map that enforces common header patterns in production clients.
#[derive(Debug, Clone, Default)]
pub struct HeaderMap {
    inner: HashMap<String, String>,
}

impl HeaderMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bearer_token(token: &str) -> Self {
        let mut map = Self::new();
        map.set_auth_bearer(token);
        map
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(key)
    }

    pub fn set_auth_bearer(&mut self, token: &str) {
        self.inner
            .insert("authorization".into(), format!("Bearer {token}"));
    }

    pub fn set_json_content_type(&mut self) {
        self.inner
            .insert("content-type".into(), "application/json".into());
    }

    pub fn set_accept_json(&mut self) {
        self.inner
            .insert("accept".into(), "application/json".into());
    }

    pub fn into_iter(self) -> impl Iterator<Item = (String, String)> {
        self.inner.into_iter()
    }
}

// ---------------------------------------------------------------------------
// 8. Cookie Jar Abstraction
// ---------------------------------------------------------------------------

/// A simple in-memory cookie jar for session-based HTTP clients.
#[derive(Debug, Clone, Default)]
pub struct CookieJar {
    cookies: HashMap<String, String>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a Set-Cookie header value and store the cookie.
    pub fn set_from_header(&mut self, header_value: &str) {
        if let Some(name_value) = header_value.split(';').next() {
            if let Some((name, value)) = name_value.split_once('=') {
                self.cookies
                    .insert(name.trim().to_string(), value.trim().to_string());
            }
        }
    }

    /// Get a cookie value by name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.cookies.get(name).map(|s| s.as_str())
    }

    /// Serialize all cookies into a Cookie header value.
    pub fn to_header_value(&self) -> String {
        self.cookies
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    pub fn len(&self) -> usize {
        self.cookies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }
}

// ---------------------------------------------------------------------------
// 9. Rate Limiter for Client-Side Throttling
// ---------------------------------------------------------------------------

/// A simple token-bucket rate limiter to avoid overwhelming servers.
#[derive(Debug)]
pub struct ClientRateLimiter {
    max_tokens: u32,
    tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: std::time::Instant,
}

impl ClientRateLimiter {
    pub fn new(max_tokens: u32, refill_rate: f64) -> Self {
        Self {
            max_tokens,
            tokens: max_tokens as f64,
            refill_rate,
            last_refill: std::time::Instant::now(),
        }
    }

    /// Try to consume one token. Returns true if allowed.
    pub fn try_acquire(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Calculate how long until the next token is available.
    pub fn time_until_available(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let deficit = 1.0 - self.tokens;
            Duration::from_secs_f64(deficit / self.refill_rate)
        }
    }

    fn refill(&mut self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens as f64);
        self.last_refill = now;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_defaults() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.max_idle_per_host, 10);
        assert!(config.base_url.is_empty());
        assert!(config.default_headers.is_empty());
    }

    #[test]
    fn test_retry_policy_exponential_backoff() {
        let policy = RetryPolicy {
            max_retries: 4,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(2),
            retry_on_status: vec![500, 503],
        };

        // Attempt 0: 100ms
        let d0 = policy.delay_for_attempt(0).unwrap();
        assert_eq!(d0, Duration::from_millis(100));

        // Attempt 1: 200ms
        let d1 = policy.delay_for_attempt(1).unwrap();
        assert_eq!(d1, Duration::from_millis(200));

        // Attempt 2: 400ms
        let d2 = policy.delay_for_attempt(2).unwrap();
        assert_eq!(d2, Duration::from_millis(400));

        // Attempt 3: 800ms (still under max_delay)
        let d3 = policy.delay_for_attempt(3).unwrap();
        assert_eq!(d3, Duration::from_millis(800));

        // Attempt 4: exceeds max_retries
        assert!(policy.delay_for_attempt(4).is_none());
    }

    #[test]
    fn test_retry_policy_max_delay_cap() {
        let policy = RetryPolicy {
            max_retries: 10,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(2),
            retry_on_status: vec![],
        };

        // Attempt 2 would be 4000ms, but capped at 2000ms
        let d = policy.delay_for_attempt(2).unwrap();
        assert_eq!(d, Duration::from_millis(2000));
    }

    #[test]
    fn test_retry_policy_status_matching() {
        let policy = RetryPolicy::default();
        assert!(policy.should_retry_status(429));
        assert!(policy.should_retry_status(503));
        assert!(!policy.should_retry_status(404));
        assert!(!policy.should_retry_status(401));
    }

    #[test]
    fn test_api_request_builder() {
        let req = ApiRequest::get("/users")
            .header("x-request-id", "abc-123")
            .query("page", "1")
            .query("per_page", "20");

        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(req.path, "/users");
        assert_eq!(req.headers.get("x-request-id").unwrap(), "abc-123");
        assert_eq!(req.query_params.len(), 2);
        assert!(req.body.is_none());
    }

    #[test]
    fn test_api_request_json_body() {
        #[derive(serde::Serialize)]
        struct CreateUser {
            name: String,
            email: String,
        }

        let req = ApiRequest::post("/users").json_body(CreateUser {
            name: "Alice".into(),
            email: "alice@example.com".into(),
        });

        assert_eq!(req.method, HttpMethod::Post);
        assert!(req.body.is_some());
        let body: serde_json::Value = serde_json::from_str(req.body.as_ref().unwrap()).unwrap();
        assert_eq!(body["name"], "Alice");
        assert_eq!(
            req.headers.get("content-type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_api_request_build_url() {
        let req = ApiRequest::get("/users/42")
            .query("format", "json");

        let url = req.build_url("https://api.example.com");
        assert_eq!(url, "https://api.example.com/users/42?format=json");
    }

    #[test]
    fn test_api_request_build_url_trailing_slash() {
        let req = ApiRequest::get("/health");
        let url = req.build_url("https://api.example.com/v1/");
        assert_eq!(url, "https://api.example.com/v1/health");
    }

    #[test]
    fn test_api_response_is_success() {
        let resp = ApiResponse {
            status: 200,
            headers: HashMap::new(),
            body: "ok",
            latency: Duration::from_millis(50),
        };
        assert!(resp.is_success());
        assert!(!resp.is_client_error());
        assert!(!resp.is_server_error());
    }

    #[test]
    fn test_api_response_is_client_error() {
        let resp: ApiResponse<()> = ApiResponse {
            status: 404,
            headers: HashMap::new(),
            body: (),
            latency: Duration::from_millis(10),
        };
        assert!(!resp.is_success());
        assert!(resp.is_client_error());
        assert!(!resp.is_server_error());
    }

    #[test]
    fn test_api_response_is_server_error() {
        let resp: ApiResponse<()> = ApiResponse {
            status: 503,
            headers: HashMap::new(),
            body: (),
            latency: Duration::from_millis(100),
        };
        assert!(!resp.is_success());
        assert!(!resp.is_client_error());
        assert!(resp.is_server_error());
    }

    #[test]
    fn test_url_builder() {
        let url = UrlBuilder::new("https://api.example.com")
            .push_segment("v1")
            .push_segment("users")
            .push_query("page", "1")
            .push_query("sort", "name")
            .build();

        assert_eq!(url, "https://api.example.com/v1/users?page=1&sort=name");
    }

    #[test]
    fn test_url_builder_no_query() {
        let url = UrlBuilder::new("https://api.example.com")
            .push_segment("health")
            .build();

        assert_eq!(url, "https://api.example.com/health");
    }

    #[test]
    fn test_header_map_bearer_token() {
        let headers = HeaderMap::with_bearer_token("my-secret-token");
        assert_eq!(
            headers.get("authorization").unwrap(),
            "Bearer my-secret-token"
        );
    }

    #[test]
    fn test_header_map_json_content_type() {
        let mut headers = HeaderMap::new();
        headers.set_json_content_type();
        headers.set_accept_json();
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
        assert_eq!(headers.get("accept").unwrap(), "application/json");
    }

    #[test]
    fn test_cookie_jar_parse_set_cookie() {
        let mut jar = CookieJar::new();
        jar.set_from_header("session_id=abc123; Path=/; HttpOnly");
        assert_eq!(jar.get("session_id"), Some("abc123"));
    }

    #[test]
    fn test_cookie_jar_multiple_cookies() {
        let mut jar = CookieJar::new();
        jar.set_from_header("session_id=abc123; Path=/");
        jar.set_from_header("csrf_token=xyz789; Path=/");
        assert_eq!(jar.len(), 2);

        let header = jar.to_header_value();
        assert!(header.contains("session_id=abc123"));
        assert!(header.contains("csrf_token=xyz789"));
    }

    #[test]
    fn test_cookie_jar_clear() {
        let mut jar = CookieJar::new();
        jar.set_from_header("a=1");
        jar.set_from_header("b=2");
        assert!(!jar.is_empty());
        jar.clear();
        assert!(jar.is_empty());
    }

    #[test]
    fn test_rate_limiter_allows_within_budget() {
        let mut limiter = ClientRateLimiter::new(5, 10.0);
        for _ in 0..5 {
            assert!(limiter.try_acquire());
        }
        // Sixth request should be rejected (no time has passed for refill)
        assert!(!limiter.try_acquire());
    }

    #[test]
    fn test_rate_limiter_refill() {
        let mut limiter = ClientRateLimiter::new(1, 100.0); // 100 tokens/sec
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire());

        // After sleeping, tokens should refill
        std::thread::sleep(Duration::from_millis(50)); // ~5 tokens
        assert!(limiter.try_acquire());
    }

    #[test]
    fn test_rate_limiter_time_until_available() {
        let mut limiter = ClientRateLimiter::new(2, 10.0);
        limiter.try_acquire();
        limiter.try_acquire();
        let wait = limiter.time_until_available();
        assert!(wait > Duration::ZERO);
        assert!(wait <= Duration::from_millis(200)); // 1 token / 10 tokens/sec = 100ms
    }

    #[test]
    fn test_http_error_display() {
        let err = HttpError::Timeout(Duration::from_secs(5));
        assert_eq!(err.to_string(), "request timeout after 5s");

        let err = HttpError::Status {
            status: 404,
            body: "not found".into(),
        };
        assert_eq!(err.to_string(), "HTTP 404: not found");

        let err = HttpError::Connection("refused".into());
        assert_eq!(err.to_string(), "connection failed: refused");
    }

    #[test]
    fn test_api_request_timeout_override() {
        let req = ApiRequest::get("/slow").timeout(Duration::from_secs(60));
        assert_eq!(req.timeout_override, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_api_request_put_and_delete() {
        let put = ApiRequest::put("/users/1");
        assert_eq!(put.method, HttpMethod::Put);
        assert_eq!(put.path, "/users/1");

        let del = ApiRequest::delete("/users/1");
        assert_eq!(del.method, HttpMethod::Delete);
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.base_delay, Duration::from_millis(100));
        assert!(policy.retry_on_status.contains(&429));
        assert!(policy.retry_on_status.contains(&503));
    }
}
