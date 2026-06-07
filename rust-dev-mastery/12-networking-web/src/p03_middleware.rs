//! # Middleware with Tower
//!
//! Tower is the backbone of the Rust async networking ecosystem. It defines the
//! `Service` trait and composable middleware via `Layer`. Axum, Tonic, and Hyper
//! all build on Tower.
//!
//! ## Key Concepts
//! - The `Service` trait: `poll_ready` + `call`
//! - `Layer` trait for wrapping services with additional behavior
//! - Request/response transformation patterns
//! - Logging, timing, and tracing middleware
//! - Conditionally applying middleware
//! - Composing multiple layers

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 1. Core Service Abstraction
// ---------------------------------------------------------------------------

/// A simplified version of Tower's Service trait for learning purposes.
/// The real Tower Service trait is generic over the request type.
pub trait SimpleService {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: SimpleRequest) -> Self::Future;
}

/// A simplified HTTP request for demonstration.
#[derive(Debug, Clone)]
pub struct SimpleRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl SimpleRequest {
    pub fn get(path: &str) -> Self {
        Self {
            method: "GET".into(),
            path: path.into(),
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn post(path: &str, body: &str) -> Self {
        Self {
            method: "POST".into(),
            path: path.into(),
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// A simplified HTTP response.
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub duration: Duration,
}

impl SimpleRequest {
    /// Create a response builder from this request.
    pub fn respond(&self, status: u16) -> ResponseBuilder {
        ResponseBuilder {
            status,
            headers: HashMap::new(),
            body: String::new(),
            start: Instant::now(),
        }
    }
}

pub struct ResponseBuilder {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
    start: Instant,
}

impl ResponseBuilder {
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn body(mut self, body: &str) -> SimpleResponse {
        self.body = body.into();
        SimpleResponse {
            status: self.status,
            headers: self.headers,
            body: self.body,
            duration: self.start.elapsed(),
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Basic Handler (Leaf Service)
// ---------------------------------------------------------------------------

/// A handler function type that processes requests.
pub type HandlerFn = fn(&SimpleRequest) -> SimpleResponse;

/// A simple service that delegates to a handler function.
pub struct HandlerService {
    pub handler: HandlerFn,
    pub path: String,
}

impl HandlerService {
    pub fn new(path: impl Into<String>, handler: HandlerFn) -> Self {
        Self {
            handler,
            path: path.into(),
        }
    }

    pub fn handle(&self, req: &SimpleRequest) -> SimpleResponse {
        if req.path == self.path {
            (self.handler)(req)
        } else {
            SimpleResponse {
                status: 404,
                headers: HashMap::new(),
                body: format!("no route for {}", req.path),
                duration: Duration::ZERO,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Logging Middleware
// ---------------------------------------------------------------------------

/// A record of a completed request for logging.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration: Duration,
    pub timestamp: Instant,
}

/// Collects log entries from the logging middleware.
#[derive(Clone, Default)]
pub struct LogCollector {
    entries: Arc<RwLock<Vec<LogEntry>>>,
}

impl LogCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, entry: LogEntry) {
        if let Ok(mut entries) = self.entries.write() {
            entries.push(entry);
        }
    }

    pub fn entries(&self) -> Vec<LogEntry> {
        self.entries.read().map(|e| e.clone()).unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Middleware that logs request method, path, status, and duration.
pub struct LoggingMiddleware {
    inner: HandlerService,
    collector: LogCollector,
}

impl LoggingMiddleware {
    pub fn new(inner: HandlerService, collector: LogCollector) -> Self {
        Self { inner, collector }
    }

    pub fn handle(&self, req: &SimpleRequest) -> SimpleResponse {
        let start = Instant::now();
        let mut response = self.inner.handle(req);
        let duration = start.elapsed();
        response.duration = duration;

        self.collector.push(LogEntry {
            method: req.method.clone(),
            path: req.path.clone(),
            status: response.status,
            duration,
            timestamp: start,
        });

        response
    }
}

// ---------------------------------------------------------------------------
// 4. Timing Middleware
// ---------------------------------------------------------------------------

/// Middleware that enforces a request timeout and adds timing headers.
pub struct TimeoutMiddleware {
    inner: LoggingMiddleware,
    timeout: Duration,
}

impl TimeoutMiddleware {
    pub fn new(inner: LoggingMiddleware, timeout: Duration) -> Self {
        Self { inner, timeout }
    }

    pub fn handle(&self, req: &SimpleRequest) -> SimpleResponse {
        let start = Instant::now();
        let mut response = self.inner.handle(req);

        if start.elapsed() > self.timeout {
            return SimpleResponse {
                status: 503,
                headers: {
                    let mut h = HashMap::new();
                    h.insert("x-timeout".into(), "true".into());
                    h
                },
                body: "request timed out".into(),
                duration: start.elapsed(),
            };
        }

        response
            .headers
            .insert("x-response-time".into(), format!("{:?}", response.duration));
        response
    }
}

// ---------------------------------------------------------------------------
// 5. CORS Middleware
// ---------------------------------------------------------------------------

/// CORS (Cross-Origin Resource Sharing) middleware.
#[derive(Debug, Clone)]
pub struct CorsMiddleware {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
    max_age: Duration,
}

impl CorsMiddleware {
    pub fn new() -> Self {
        Self {
            allowed_origins: vec!["*".into()],
            allowed_methods: vec!["GET".into(), "POST".into(), "PUT".into(), "DELETE".into()],
            allowed_headers: vec!["content-type".into(), "authorization".into()],
            max_age: Duration::from_secs(3600),
        }
    }

    pub fn allowed_origins(mut self, origins: Vec<&str>) -> Self {
        self.allowed_origins = origins.into_iter().map(String::from).collect();
        self
    }

    pub fn allowed_methods(mut self, methods: Vec<&str>) -> Self {
        self.allowed_methods = methods.into_iter().map(String::from).collect();
        self
    }

    /// Apply CORS headers to a response.
    pub fn apply(&self, req: &SimpleRequest, mut resp: SimpleResponse) -> SimpleResponse {
        let origin = req
            .headers
            .get("origin")
            .cloned()
            .unwrap_or_else(|| "*".into());

        if self.is_origin_allowed(&origin) {
            resp.headers
                .insert("access-control-allow-origin".into(), origin);
            resp.headers.insert(
                "access-control-allow-methods".into(),
                self.allowed_methods.join(", "),
            );
            resp.headers.insert(
                "access-control-allow-headers".into(),
                self.allowed_headers.join(", "),
            );
            resp.headers.insert(
                "access-control-max-age".into(),
                self.max_age.as_secs().to_string(),
            );
        }

        resp
    }

    /// Handle CORS preflight (OPTIONS) requests.
    pub fn handle_preflight(&self, req: &SimpleRequest) -> Option<SimpleResponse> {
        if req.method != "OPTIONS" {
            return None;
        }

        let origin = req
            .headers
            .get("origin")
            .cloned()
            .unwrap_or_else(|| "*".into());

        if !self.is_origin_allowed(&origin) {
            return Some(SimpleResponse {
                status: 403,
                headers: HashMap::new(),
                body: "origin not allowed".into(),
                duration: Duration::ZERO,
            });
        }

        let mut headers = HashMap::new();
        headers.insert("access-control-allow-origin".into(), origin);
        headers.insert(
            "access-control-allow-methods".into(),
            self.allowed_methods.join(", "),
        );
        headers.insert(
            "access-control-allow-headers".into(),
            self.allowed_headers.join(", "),
        );
        headers.insert(
            "access-control-max-age".into(),
            self.max_age.as_secs().to_string(),
        );

        Some(SimpleResponse {
            status: 204,
            headers,
            body: String::new(),
            duration: Duration::ZERO,
        })
    }

    fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.iter().any(|o| o == "*" || o == origin)
    }
}

// ---------------------------------------------------------------------------
// 6. Authentication Middleware
// ---------------------------------------------------------------------------

/// Extracts and validates a bearer token from the Authorization header.
pub struct AuthMiddleware {
    valid_tokens: Vec<String>,
}

impl AuthMiddleware {
    pub fn new(valid_tokens: Vec<&str>) -> Self {
        Self {
            valid_tokens: valid_tokens.into_iter().map(String::from).collect(),
        }
    }

    /// Validate the request's auth token. Returns the token if valid.
    pub fn validate(&self, req: &SimpleRequest) -> Result<String, SimpleResponse> {
        let auth_header = req.headers.get("authorization").ok_or_else(|| SimpleResponse {
            status: 401,
            headers: HashMap::new(),
            body: "missing authorization header".into(),
            duration: Duration::ZERO,
        })?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| SimpleResponse {
                status: 401,
                headers: HashMap::new(),
                body: "invalid authorization format, expected 'Bearer <token>'".into(),
                duration: Duration::ZERO,
            })?;

        if self.valid_tokens.iter().any(|t| t == token) {
            Ok(token.to_string())
        } else {
            Err(SimpleResponse {
                status: 401,
                headers: HashMap::new(),
                body: "invalid token".into(),
                duration: Duration::ZERO,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Middleware Pipeline
// ---------------------------------------------------------------------------

/// Composes multiple middleware layers into a single processing pipeline.
pub struct MiddlewarePipeline {
    cors: CorsMiddleware,
    auth: AuthMiddleware,
    timeout: Duration,
}

impl MiddlewarePipeline {
    pub fn new(cors: CorsMiddleware, auth: AuthMiddleware, timeout: Duration) -> Self {
        Self {
            cors,
            auth,
            timeout,
        }
    }

    /// Process a request through the full middleware chain.
    pub fn process(
        &self,
        req: &SimpleRequest,
        handler: &HandlerService,
        collector: &LogCollector,
    ) -> SimpleResponse {
        // 1. Handle CORS preflight first
        if let Some(preflight) = self.cors.handle_preflight(req) {
            return preflight;
        }

        // 2. Authenticate
        let _token = match self.auth.validate(req) {
            Ok(token) => token,
            Err(resp) => return resp,
        };

        // 3. Handle the request through logging + timeout
        let logging = LoggingMiddleware::new(
            HandlerService {
                handler: handler.handler,
                path: handler.path.clone(),
            },
            collector.clone(),
        );
        let timeout_mw = TimeoutMiddleware::new(logging, self.timeout);
        let resp = timeout_mw.handle(req);

        // 4. Apply CORS headers
        self.cors.apply(req, resp)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn hello_handler(req: &SimpleRequest) -> SimpleResponse {
        req.respond(200).body("hello world")
    }

    fn slow_handler(_req: &SimpleRequest) -> SimpleResponse {
        std::thread::sleep(Duration::from_millis(10));
        SimpleResponse {
            status: 200,
            headers: HashMap::new(),
            body: "slow response".into(),
            duration: Duration::from_millis(10),
        }
    }

    #[test]
    fn test_handler_service_basic() {
        let svc = HandlerService::new("/hello", hello_handler);
        let req = SimpleRequest::get("/hello");
        let resp = svc.handle(&req);
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, "hello world");
    }

    #[test]
    fn test_handler_service_not_found() {
        let svc = HandlerService::new("/hello", hello_handler);
        let req = SimpleRequest::get("/other");
        let resp = svc.handle(&req);
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn test_logging_middleware() {
        let handler = HandlerService::new("/test", hello_handler);
        let collector = LogCollector::new();
        let mw = LoggingMiddleware::new(handler, collector.clone());

        let req = SimpleRequest::get("/test");
        let resp = mw.handle(&req);
        assert_eq!(resp.status, 200);

        let entries = collector.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].method, "GET");
        assert_eq!(entries[0].path, "/test");
        assert_eq!(entries[0].status, 200);
    }

    #[test]
    fn test_logging_middleware_multiple_requests() {
        let handler = HandlerService::new("/test", hello_handler);
        let collector = LogCollector::new();
        let mw = LoggingMiddleware::new(handler, collector.clone());

        mw.handle(&SimpleRequest::get("/test"));
        mw.handle(&SimpleRequest::post("/test", "data"));
        mw.handle(&SimpleRequest::get("/other"));

        let entries = collector.entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[1].method, "POST");
        assert_eq!(entries[2].status, 404);
    }

    #[test]
    fn test_timeout_middleware_within_limit() {
        let handler = HandlerService::new("/fast", |_req| {
            SimpleResponse {
                status: 200,
                headers: HashMap::new(),
                body: "fast".into(),
                duration: Duration::from_millis(1),
            }
        });
        let collector = LogCollector::new();
        let logging = LoggingMiddleware::new(handler, collector);
        let timeout_mw = TimeoutMiddleware::new(logging, Duration::from_secs(5));

        let req = SimpleRequest::get("/fast");
        let resp = timeout_mw.handle(&req);
        assert_eq!(resp.status, 200);
        assert!(resp.headers.contains_key("x-response-time"));
    }

    #[test]
    fn test_cors_middleware_allows_origin() {
        let cors = CorsMiddleware::new();
        let req = SimpleRequest::get("/api").header("origin", "https://example.com");
        let resp = SimpleResponse {
            status: 200,
            headers: HashMap::new(),
            body: "ok".into(),
            duration: Duration::ZERO,
        };

        let resp = cors.apply(&req, resp);
        assert_eq!(
            resp.headers.get("access-control-allow-origin").unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn test_cors_preflight() {
        let cors = CorsMiddleware::new();
        let req = SimpleRequest {
            method: "OPTIONS".into(),
            path: "/api/users".into(),
            headers: {
                let mut h = HashMap::new();
                h.insert("origin".into(), "https://example.com".into());
                h
            },
            body: String::new(),
        };

        let resp = cors.handle_preflight(&req).unwrap();
        assert_eq!(resp.status, 204);
        assert!(resp
            .headers
            .contains_key("access-control-allow-methods"));
    }

    #[test]
    fn test_cors_preflight_restricted_origin() {
        let cors = CorsMiddleware::new().allowed_origins(vec!["https://trusted.com"]);
        let req = SimpleRequest {
            method: "OPTIONS".into(),
            path: "/api".into(),
            headers: {
                let mut h = HashMap::new();
                h.insert("origin".into(), "https://evil.com".into());
                h
            },
            body: String::new(),
        };

        let resp = cors.handle_preflight(&req).unwrap();
        assert_eq!(resp.status, 403);
    }

    #[test]
    fn test_cors_not_options() {
        let cors = CorsMiddleware::new();
        let req = SimpleRequest::get("/api");
        assert!(cors.handle_preflight(&req).is_none());
    }

    #[test]
    fn test_auth_middleware_valid_token() {
        let auth = AuthMiddleware::new(vec!["secret-token-123"]);
        let req = SimpleRequest::get("/api").header("authorization", "Bearer secret-token-123");

        let result = auth.validate(&req);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "secret-token-123");
    }

    #[test]
    fn test_auth_middleware_missing_header() {
        let auth = AuthMiddleware::new(vec!["token"]);
        let req = SimpleRequest::get("/api");

        let result = auth.validate(&req);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 401);
    }

    #[test]
    fn test_auth_middleware_invalid_format() {
        let auth = AuthMiddleware::new(vec!["token"]);
        let req = SimpleRequest::get("/api").header("authorization", "Basic abc123");

        let result = auth.validate(&req);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status, 401);
        assert!(err.body.contains("expected 'Bearer"));
    }

    #[test]
    fn test_auth_middleware_wrong_token() {
        let auth = AuthMiddleware::new(vec!["correct-token"]);
        let req = SimpleRequest::get("/api").header("authorization", "Bearer wrong-token");

        let result = auth.validate(&req);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 401);
    }

    #[test]
    fn test_middleware_pipeline_full_flow() {
        let cors = CorsMiddleware::new();
        let auth = AuthMiddleware::new(vec!["valid-token"]);
        let pipeline = MiddlewarePipeline::new(cors, auth, Duration::from_secs(5));

        let handler = HandlerService::new("/api/data", hello_handler);
        let collector = LogCollector::new();

        let req = SimpleRequest::get("/api/data")
            .header("authorization", "Bearer valid-token")
            .header("origin", "https://example.com");

        let resp = pipeline.process(&req, &handler, &collector);
        assert_eq!(resp.status, 200);
        assert!(resp
            .headers
            .contains_key("access-control-allow-origin"));
        assert_eq!(collector.len(), 1);
    }

    #[test]
    fn test_middleware_pipeline_auth_failure() {
        let cors = CorsMiddleware::new();
        let auth = AuthMiddleware::new(vec!["valid-token"]);
        let pipeline = MiddlewarePipeline::new(cors, auth, Duration::from_secs(5));

        let handler = HandlerService::new("/api/data", hello_handler);
        let collector = LogCollector::new();

        let req = SimpleRequest::get("/api/data")
            .header("authorization", "Bearer bad-token");

        let resp = pipeline.process(&req, &handler, &collector);
        assert_eq!(resp.status, 401);
        // Auth failure means handler was never called
        assert!(collector.is_empty());
    }

    #[test]
    fn test_log_collector_default() {
        let collector = LogCollector::default();
        assert!(collector.is_empty());
        assert_eq!(collector.len(), 0);
    }

    #[test]
    fn test_simple_request_builders() {
        let req = SimpleRequest::get("/test");
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/test");
        assert!(req.body.is_empty());

        let req = SimpleRequest::post("/submit", "data=1");
        assert_eq!(req.method, "POST");
        assert_eq!(req.body, "data=1");

        let req = SimpleRequest::get("/test").header("x-custom", "value");
        assert_eq!(req.headers.get("x-custom").unwrap(), "value");
    }

    #[test]
    fn test_cors_custom_methods() {
        let cors = CorsMiddleware::new().allowed_methods(vec!["GET", "POST"]);
        let req = SimpleRequest {
            method: "OPTIONS".into(),
            path: "/".into(),
            headers: {
                let mut h = HashMap::new();
                h.insert("origin".into(), "https://example.com".into());
                h
            },
            body: String::new(),
        };

        let resp = cors.handle_preflight(&req).unwrap();
        let methods = resp.headers.get("access-control-allow-methods").unwrap();
        assert_eq!(methods, "GET, POST");
    }
}
