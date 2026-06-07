//! # Integration Testing
//!
//! Integration tests verify that multiple components work together correctly.
//! In Rust, integration tests live in the `tests/` directory and test the
//! public API of the crate.
//!
//! This module demonstrates patterns for integration testing within a
//! single crate (since we can't add files to tests/ from here).

use std::collections::HashMap;

/// A simple in-memory key-value store for demonstrating integration tests.
#[derive(Debug)]
pub struct KvStore {
    data: HashMap<String, Vec<u8>>,
    access_log: Vec<AccessLogEntry>,
}

#[derive(Debug, Clone)]
pub struct AccessLogEntry {
    pub key: String,
    pub operation: Operation,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Get,
    Set,
    Delete,
}

impl KvStore {
    pub fn new() -> Self {
        KvStore {
            data: HashMap::new(),
            access_log: Vec::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: Vec<u8>) {
        self.data.insert(key.to_string(), value);
        self.access_log.push(AccessLogEntry {
            key: key.to_string(),
            operation: Operation::Set,
            timestamp: self.access_log.len() as u64,
        });
    }

    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.data.get(key).map(|v| v.as_slice())
    }

    pub fn delete(&mut self, key: &str) -> bool {
        let existed = self.data.remove(key).is_some();
        if existed {
            self.access_log.push(AccessLogEntry {
                key: key.to_string(),
                operation: Operation::Delete,
                timestamp: self.access_log.len() as u64,
            });
        }
        existed
    }

    pub fn keys(&self) -> Vec<&str> {
        self.data.keys().map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn access_log(&self) -> &[AccessLogEntry] {
        &self.access_log
    }
}

/// A simple HTTP-like router for testing request handling.
#[derive(Debug)]
pub struct Router {
    routes: Vec<Route>,
}

#[derive(Debug)]
struct Route {
    method: String,
    path: String,
    handler_name: String,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Router {
    pub fn new() -> Self {
        Router { routes: Vec::new() }
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler_name: &str) {
        self.routes.push(Route {
            method: method.to_uppercase(),
            path: path.to_string(),
            handler_name: handler_name.to_string(),
        });
    }

    pub fn handle(&self, request: &Request) -> Response {
        for route in &self.routes {
            if route.method == request.method.to_uppercase() && route.path == request.path {
                return Response {
                    status: 200,
                    headers: HashMap::new(),
                    body: format!("Handled by {}", route.handler_name),
                };
            }
        }
        Response {
            status: 404,
            headers: HashMap::new(),
            body: "Not Found".to_string(),
        }
    }
}

/// A simple middleware chain for testing.
pub struct MiddlewareChain {
    middlewares: Vec<Box<dyn Fn(&Request) -> Option<Response>>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        MiddlewareChain {
            middlewares: Vec::new(),
        }
    }

    pub fn add<F>(&mut self, middleware: F)
    where
        F: Fn(&Request) -> Option<Response> + 'static,
    {
        self.middlewares.push(Box::new(middleware));
    }

    pub fn process(&self, request: &Request) -> Option<Response> {
        for middleware in &self.middlewares {
            if let Some(response) = middleware(request) {
                return Some(response);
            }
        }
        None
    }
}

/// Demonstrates testing with shared test fixtures.
pub struct TestFixtures;

impl TestFixtures {
    pub fn create_store_with_data() -> KvStore {
        let mut store = KvStore::new();
        store.set("name", b"Alice".to_vec());
        store.set("age", b"30".to_vec());
        store.set("city", b"NYC".to_vec());
        store
    }

    pub fn create_router() -> Router {
        let mut router = Router::new();
        router.add_route("GET", "/health", "health_handler");
        router.add_route("GET", "/users", "list_users");
        router.add_route("POST", "/users", "create_user");
        router.add_route("GET", "/users/:id", "get_user");
        router
    }

    pub fn create_request(method: &str, path: &str) -> Request {
        Request {
            method: method.to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration test: KvStore operations work together
    #[test]
    fn test_kvstore_full_workflow() {
        let mut store = KvStore::new();

        // Set values
        store.set("key1", b"value1".to_vec());
        store.set("key2", b"value2".to_vec());

        // Get values
        assert_eq!(store.get("key1"), Some(b"value1".as_slice()));
        assert_eq!(store.get("key2"), Some(b"value2".as_slice()));
        assert_eq!(store.get("nonexistent"), None);

        // List keys
        let mut keys = store.keys();
        keys.sort();
        assert_eq!(keys, vec!["key1", "key2"]);

        // Delete
        assert!(store.delete("key1"));
        assert!(!store.delete("nonexistent"));
        assert_eq!(store.len(), 1);
    }

    // Integration test: Router handles requests correctly
    #[test]
    fn test_router_request_handling() {
        let router = TestFixtures::create_router();

        let req = TestFixtures::create_request("GET", "/health");
        let resp = router.handle(&req);
        assert_eq!(resp.status, 200);
        assert!(resp.body.contains("health_handler"));

        let req = TestFixtures::create_request("GET", "/nonexistent");
        let resp = router.handle(&req);
        assert_eq!(resp.status, 404);
    }

    // Integration test: Middleware chain
    #[test]
    fn test_middleware_chain() {
        let mut chain = MiddlewareChain::new();

        // Auth middleware: reject requests without auth header
        chain.add(|req: &Request| {
            if !req.headers.contains_key("Authorization") {
                Some(Response {
                    status: 401,
                    headers: HashMap::new(),
                    body: "Unauthorized".to_string(),
                })
            } else {
                None
            }
        });

        // Test without auth
        let req = TestFixtures::create_request("GET", "/protected");
        let resp = chain.process(&req).unwrap();
        assert_eq!(resp.status, 401);

        // Test with auth
        let mut req = TestFixtures::create_request("GET", "/protected");
        req.headers
            .insert("Authorization".to_string(), "Bearer token".to_string());
        assert!(chain.process(&req).is_none());
    }

    // Integration test: Access logging
    #[test]
    fn test_access_logging() {
        let mut store = KvStore::new();
        store.set("key1", b"val1".to_vec());
        store.set("key2", b"val2".to_vec());
        store.delete("key1");

        let log = store.access_log();
        assert_eq!(log.len(), 3);
        assert_eq!(log[0].key, "key1");
        assert!(matches!(log[0].operation, Operation::Set));
        assert!(matches!(log[2].operation, Operation::Delete));
    }

    // Integration test: Using fixtures
    #[test]
    fn test_with_fixture() {
        let store = TestFixtures::create_store_with_data();
        assert_eq!(store.len(), 3);
        assert_eq!(store.get("name"), Some(b"Alice".as_slice()));
    }
}
