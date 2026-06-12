//! Simple request-response RPC framework.
//!
//! This module provides an in-process RPC mechanism where handlers are registered
//! by method name and invoked synchronously when a request arrives. It demonstrates
//! the core pattern of remote procedure calls without involving actual networking.

use std::collections::HashMap;

/// An RPC request carrying a method name and a string payload.
#[derive(Debug, Clone)]
pub struct RPCRequest {
    /// Unique identifier for this request.
    pub id: u64,
    /// The method name to invoke.
    pub method: String,
    /// The serialized payload (e.g., JSON or bincode bytes as a string).
    pub payload: String,
}

/// An RPC response carrying a success flag and result data.
#[derive(Debug, Clone)]
pub struct RPCResponse {
    /// The id of the request this response corresponds to.
    pub id: u64,
    /// Whether the handler executed successfully.
    pub success: bool,
    /// The serialized result data.
    pub data: String,
}

/// An in-process RPC server that dispatches requests to registered handlers.
pub struct RPCServer {
    /// Map of method name to handler function.
    handlers: HashMap<String, Box<dyn Fn(&str) -> String>>,
    /// Total number of requests handled so far.
    request_count: usize,
}

impl RPCServer {
    /// Create a new RPC server with no handlers registered.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            request_count: 0,
        }
    }

    /// Register a handler for the given method name.
    ///
    /// The handler receives the request payload as a `&str` and returns the
    /// serialized response as a `String`.
    ///
    /// # Arguments
    ///
    /// * `method` - The method name this handler serves.
    /// * `handler` - The handler function.
    pub fn register_handler(&mut self, method: String, handler: Box<dyn Fn(&str) -> String>) {
        self.handlers.insert(method, handler);
    }

    /// Dispatch an incoming request to its registered handler.
    ///
    /// If no handler is registered for the method, the response indicates failure.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming RPC request.
    ///
    /// # Returns
    ///
    /// An `RPCResponse` with the handler's output or an error message.
    pub fn handle_request(&mut self, request: RPCRequest) -> RPCResponse {
        self.request_count += 1;
        if let Some(handler) = self.handlers.get(&request.method) {
            let data = handler(&request.payload);
            RPCResponse {
                id: request.id,
                success: true,
                data,
            }
        } else {
            RPCResponse {
                id: request.id,
                success: false,
                data: format!("no handler for method '{}'", request.method),
            }
        }
    }

    /// Return the total number of requests handled.
    pub fn request_count(&self) -> usize {
        self.request_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_request_with_handler() {
        let mut server = RPCServer::new();
        server.register_handler(
            "echo".into(),
            Box::new(|payload| format!("echo:{}", payload)),
        );

        let req = RPCRequest {
            id: 1,
            method: "echo".into(),
            payload: "hello".into(),
        };
        let resp = server.handle_request(req);
        assert!(resp.success);
        assert_eq!(resp.data, "echo:hello");
        assert_eq!(server.request_count(), 1);
    }

    #[test]
    fn test_handle_request_no_handler() {
        let mut server = RPCServer::new();
        let req = RPCRequest {
            id: 1,
            method: "unknown".into(),
            payload: "".into(),
        };
        let resp = server.handle_request(req);
        assert!(!resp.success);
        assert!(resp.data.contains("no handler"));
    }

    #[test]
    fn test_multiple_requests() {
        let mut server = RPCServer::new();
        server.register_handler(
            "add".into(),
            Box::new(|payload| {
                let parts: Vec<&str> = payload.split(',').collect();
                if parts.len() == 2 {
                    let a: i64 = parts[0].parse().unwrap_or(0);
                    let b: i64 = parts[1].parse().unwrap_or(0);
                    (a + b).to_string()
                } else {
                    "error".into()
                }
            }),
        );

        let req1 = RPCRequest {
            id: 1,
            method: "add".into(),
            payload: "3,4".into(),
        };
        let req2 = RPCRequest {
            id: 2,
            method: "add".into(),
            payload: "10,20".into(),
        };
        let r1 = server.handle_request(req1);
        let r2 = server.handle_request(req2);
        assert_eq!(r1.data, "7");
        assert_eq!(r2.data, "30");
        assert_eq!(server.request_count(), 2);
    }
}
