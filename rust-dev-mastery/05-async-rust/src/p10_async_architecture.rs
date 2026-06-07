//! # Async Architecture
//!
//! Structuring real-world async applications requires patterns for graceful shutdown,
//! connection pooling, middleware layers, and service composition. This module
//! covers the architectural patterns used in production async Rust systems.
//!
//! ## Key Concepts
//! - **Graceful shutdown**: Using broadcast channels or CancellationToken
//! - **Connection pools**: Managed resource pools with health checks
//! - **Middleware**: Layered request processing using tower-like patterns
//! - **Service composition**: Building complex services from simple ones

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// A graceful shutdown coordinator using broadcast channels.
/// Multiple tasks can listen for the shutdown signal and clean up concurrently.
pub struct ShutdownCoordinator {
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
    started: Arc<tokio::sync::Notify>,
    finished: Arc<tokio::sync::Notify>,
}

impl ShutdownCoordinator {
    pub fn new() -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        ShutdownCoordinator {
            shutdown_tx,
            started: Arc::new(tokio::sync::Notify::new()),
            finished: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Returns a receiver that will receive the shutdown signal.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Triggers shutdown. All subscribers will receive the signal.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Runs a shutdown-aware task that cleans up when the signal is received.
    pub async fn run_until_shutdown<F, Fut, T>(&self, task_name: &str, task: F) -> Option<T>
    where
        F: FnOnce(tokio::sync::broadcast::Receiver<()>) -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let rx = self.subscribe();
        self.started.notify_one();

        let result = task(rx).await;

        self.finished.notify_one();
        Some(result)
    }
}

impl Clone for ShutdownCoordinator {
    fn clone(&self) -> Self {
        ShutdownCoordinator {
            shutdown_tx: self.shutdown_tx.clone(),
            started: self.started.clone(),
            finished: self.finished.clone(),
        }
    }
}

/// A generic connection pool with health checking.
pub struct ConnectionPool<C: Connection> {
    connections: Arc<tokio::sync::Mutex<Vec<PooledConnection<C>>>>,
    factory: Arc<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<C, C::Error>> + Send>> + Send + Sync>,
    max_size: usize,
    max_idle: Duration,
}

struct PooledConnection<C> {
    conn: C,
    last_used: tokio::time::Instant,
}

pub trait Connection: Send + 'static {
    type Error: std::fmt::Debug;
    fn is_healthy(&self) -> bool;
    fn close(self);
}

impl<C: Connection> ConnectionPool<C> {
    pub fn new<F, Fut>(factory: F, max_size: usize, max_idle: Duration) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<C, C::Error>> + Send + 'static,
    {
        ConnectionPool {
            connections: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            factory: Arc::new(move || Box::pin(factory())),
            max_size,
            max_idle,
        }
    }

    /// Acquires a connection from the pool. Creates a new one if needed.
    pub async fn acquire(&self) -> Result<C, C::Error> {
        let mut conns = self.connections.lock().await;

        // Try to reuse a healthy connection
        while let Some(pc) = conns.pop() {
            if pc.conn.is_healthy() && (tokio::time::Instant::now() - pc.last_used) < self.max_idle {
                return Ok(pc.conn);
            } else {
                pc.conn.close();
            }
        }

        // Create a new connection
        drop(conns);
        (self.factory)().await
    }

    /// Returns a connection to the pool.
    pub async fn release(&self, conn: C) {
        let mut conns = self.connections.lock().await;
        if conns.len() < self.max_size {
            conns.push(PooledConnection {
                conn,
                last_used: tokio::time::Instant::now(),
            });
        } else {
            conn.close();
        }
    }

    pub async fn available(&self) -> usize {
        self.connections.lock().await.len()
    }

    /// Starts a background task that removes expired connections.
    pub fn start_reaper(&self) -> tokio::task::JoinHandle<()> {
        let conns = self.connections.clone();
        let max_idle = self.max_idle;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(max_idle).await;
                let mut conns = conns.lock().await;
                let now = tokio::time::Instant::now();
                conns.retain(|pc| {
                    let keep = (now - pc.last_used) < max_idle && pc.conn.is_healthy();
                    if !keep {
                        // Connection dropped here, calling close implicitly
                    }
                    keep
                });
            }
        })
    }
}

/// A middleware-style request processor.
/// Each middleware can inspect/modify the request and response.
#[derive(Debug, Clone)]
pub struct Request {
    pub path: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        Response {
            status: 200,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    pub fn not_found() -> Self {
        Response {
            status: 404,
            headers: HashMap::new(),
            body: b"Not Found".to_vec(),
        }
    }

    pub fn internal_error(msg: &str) -> Self {
        Response {
            status: 500,
            headers: HashMap::new(),
            body: msg.as_bytes().to_vec(),
        }
    }
}

/// A middleware trait for async request processing.
pub trait Middleware: Send + Sync {
    fn process<'a>(
        &'a self,
        req: Request,
        next: Next<'a>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'a>>;
}

/// The continuation type for middleware chains.
pub struct Next<'a> {
    middlewares: &'a [Box<dyn Middleware>],
    handler: &'a (dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Sync),
}

impl<'a> Next<'a> {
    pub async fn run(self, req: Request) -> Response {
        if let Some((middleware, rest)) = self.middlewares.split_first() {
            let next = Next {
                middlewares: rest,
                handler: self.handler,
            };
            middleware.process(req, next).await
        } else {
            (self.handler)(req).await
        }
    }
}

/// Logging middleware that records request timing.
pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn process<'a>(
        &'a self,
        req: Request,
        next: Next<'a>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'a>> {
        Box::pin(async move {
            let start = tokio::time::Instant::now();
            let path = req.path.clone();
            let response = next.run(req).await;
            let elapsed = start.elapsed();
            // In production, use tracing::info!
            let _ = (&path, elapsed);
            response
        })
    }
}

/// Rate limiting middleware.
pub struct RateLimitMiddleware {
    requests: tokio::sync::Mutex<Vec<tokio::time::Instant>>,
    max_per_second: usize,
}

impl RateLimitMiddleware {
    pub fn new(max_per_second: usize) -> Self {
        RateLimitMiddleware {
            requests: tokio::sync::Mutex::new(Vec::new()),
            max_per_second,
        }
    }
}

impl Middleware for RateLimitMiddleware {
    fn process<'a>(
        &'a self,
        req: Request,
        next: Next<'a>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'a>> {
        Box::pin(async move {
            let now = tokio::time::Instant::now();
            let mut requests = self.requests.lock().await;
            requests.retain(|t| now - *t < Duration::from_secs(1));

            if requests.len() >= self.max_per_second {
                return Response {
                    status: 429,
                    headers: HashMap::new(),
                    body: b"Too Many Requests".to_vec(),
                };
            }

            requests.push(now);
            drop(requests);

            next.run(req).await
        })
    }
}

/// An application builder that composes middleware and handlers.
pub struct AppBuilder {
    middlewares: Vec<Box<dyn Middleware>>,
    routes: HashMap<String, Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Send + Sync>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        AppBuilder {
            middlewares: Vec::new(),
            routes: HashMap::new(),
        }
    }

    pub fn middleware<M: Middleware + 'static>(mut self, m: M) -> Self {
        self.middlewares.push(Box::new(m));
        self
    }

    pub fn route<F, Fut>(mut self, path: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Response> + Send + 'static,
    {
        self.routes.insert(
            path.into(),
            Box::new(move |req| Box::pin(handler(req))),
        );
        self
    }

    pub async fn handle(&self, req: Request) -> Response {
        let path = req.path.clone();

        let handler = match self.routes.get(&path) {
            Some(h) => h,
            None => return Response::not_found(),
        };

        let next = Next {
            middlewares: &self.middlewares,
            handler,
        };
        next.run(req).await
    }
}

/// A service health checker that runs periodic checks.
pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
}

pub trait HealthCheck: Send + Sync {
    fn check(&self) -> HealthStatus;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthStatus {
    pub healthy: bool,
    pub message: String,
}

/// Simple health check implementation
pub struct SimpleHealthCheck {
    name: String,
    checker: Box<dyn Fn() -> bool + Send + Sync>,
}

impl SimpleHealthCheck {
    pub fn new(name: impl Into<String>, checker: impl Fn() -> bool + Send + Sync + 'static) -> Self {
        SimpleHealthCheck {
            name: name.into(),
            checker: Box::new(checker),
        }
    }

    pub fn check(&self) -> HealthStatus {
        let healthy = (self.checker)();
        HealthStatus {
            healthy,
            message: if healthy {
                format!("{}: OK", self.name)
            } else {
                format!("{}: FAILED", self.name)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_coordinator() {
        let coordinator = ShutdownCoordinator::new();
        let coord2 = coordinator.clone();

        let handle = tokio::spawn(async move {
            let mut rx = coord2.subscribe();
            rx.recv().await.unwrap();
            "shutdown complete"
        });

        tokio::time::sleep(Duration::from_millis(10)).await;
        coordinator.shutdown();

        let result = handle.await.unwrap();
        assert_eq!(result, "shutdown complete");
    }

    #[tokio::test]
    async fn test_shutdown_multiple_subscribers() {
        let coordinator = ShutdownCoordinator::new();
        let mut handles = Vec::new();

        for _ in 0..5 {
            let coord = coordinator.clone();
            handles.push(tokio::spawn(async move {
                let mut rx = coord.subscribe();
                rx.recv().await.ok();
            }));
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
        coordinator.shutdown();

        for h in handles {
            h.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_app_builder_basic() {
        let app = AppBuilder::new().route("/hello", |_req| async {
            Response::ok("Hello, World!")
        });

        let response = app
            .handle(Request {
                path: "/hello".to_string(),
                method: "GET".to_string(),
                headers: HashMap::new(),
                body: Vec::new(),
            })
            .await;

        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"Hello, World!");
    }

    #[tokio::test]
    async fn test_app_builder_not_found() {
        let app = AppBuilder::new().route("/hello", |_req| async {
            Response::ok("Hello")
        });

        let response = app
            .handle(Request {
                path: "/unknown".to_string(),
                method: "GET".to_string(),
                headers: HashMap::new(),
                body: Vec::new(),
            })
            .await;

        assert_eq!(response.status, 404);
    }

    #[tokio::test]
    async fn test_rate_limit_middleware() {
        let rate_limit = RateLimitMiddleware::new(2);

        // Process 3 requests through the rate limiter
        let req = Request {
            path: "/api".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        };

        let handler: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Send + Sync> =
            Box::new(|_req| Box::pin(async { Response::ok("ok") }));

        let middlewares: Vec<Box<dyn Middleware>> = vec![Box::new(RateLimitMiddleware::new(2))];

        // First two requests should succeed
        for _ in 0..2 {
            let next = Next {
                middlewares: &middlewares,
                handler: &*handler,
            };
            let response = next.run(req.clone()).await;
            assert_eq!(response.status, 200);
        }

        // Third request should be rate limited
        let next = Next {
            middlewares: &middlewares,
            handler: &*handler,
        };
        let response = next.run(req.clone()).await;
        assert_eq!(response.status, 429);
    }

    #[tokio::test]
    async fn test_simple_health_check() {
        let check = SimpleHealthCheck::new("db", || true);
        let status = check.check();
        assert!(status.healthy);
        assert!(status.message.contains("OK"));
    }

    #[tokio::test]
    async fn test_simple_health_check_failing() {
        let check = SimpleHealthCheck::new("db", || false);
        let status = check.check();
        assert!(!status.healthy);
        assert!(status.message.contains("FAILED"));
    }

    #[tokio::test]
    async fn test_response_builder() {
        let resp = Response::ok("data");
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, b"data");

        let resp = Response::not_found();
        assert_eq!(resp.status, 404);

        let resp = Response::internal_error("oops");
        assert_eq!(resp.status, 500);
    }

    #[tokio::test]
    async fn test_shutdown_with_timeout() {
        let coordinator = ShutdownCoordinator::new();
        let coord = coordinator.clone();

        let handle = tokio::spawn(async move {
            let mut rx = coord.subscribe();
            tokio::select! {
                _ = rx.recv() => "clean shutdown",
                _ = tokio::time::sleep(Duration::from_secs(5)) => "timeout",
            }
        });

        // Give the spawned task time to subscribe
        tokio::time::sleep(Duration::from_millis(10)).await;
        coordinator.shutdown();
        let result = handle.await.unwrap();
        assert_eq!(result, "clean shutdown");
    }

    #[tokio::test]
    async fn test_app_multiple_routes() {
        let app = AppBuilder::new()
            .route("/users", |_req| async { Response::ok("users list") })
            .route("/orders", |_req| async { Response::ok("orders list") })
            .route("/health", |_req| async { Response::ok("ok") });

        let req = |path: &str| Request {
            path: path.to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        };

        assert_eq!(app.handle(req("/users")).await.status, 200);
        assert_eq!(app.handle(req("/orders")).await.status, 200);
        assert_eq!(app.handle(req("/health")).await.status, 200);
        assert_eq!(app.handle(req("/unknown")).await.status, 404);
    }
}
