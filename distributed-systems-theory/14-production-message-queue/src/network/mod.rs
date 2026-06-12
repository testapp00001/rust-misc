//! Networking layer for the message queue broker.
//!
//! This module provides:
//!
//! - **[`rpc`]** -- A binary RPC protocol with length-prefixed framing over
//!   TCP.  All request and response types are defined here and serialized with
//!   `bincode`.
//! - **[`connection_pool`]** -- A pool of reusable TCP connections to broker
//!   peers, reducing connection-setup overhead for inter-node replication.
//! - **[`server`]** -- A TCP server that accepts client connections, decodes
//!   RPC frames, and dispatches requests through a pluggable [`RequestHandler`]
//!   trait.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use production_message_queue::network::{NetworkServer, RequestHandler};
//! use production_message_queue::network::rpc::{RpcRequest, RpcResponse};
//! use production_message_queue::metrics::Metrics;
//! use production_message_queue::health::HealthMonitor;
//!
//! struct MyHandler;
//!
//! #[async_trait::async_trait]
//! impl RequestHandler for MyHandler {
//!     async fn handle(&self, request: RpcRequest) -> RpcResponse {
//!         match request {
//!             RpcRequest::Ping => RpcResponse::Pong,
//!             _ => RpcResponse::Error("not implemented".into()),
//!         }
//!     }
//! }
//!
//! # tokio_test::block_on(async {
//! let metrics = Metrics::new();
//! let health = HealthMonitor::shared();
//! let server = NetworkServer::new(
//!     "127.0.0.1:9092".into(),
//!     1024,
//!     metrics,
//!     health,
//! )
//! .with_handler(Arc::new(MyHandler));
//!
//! // server.start().await.unwrap();
//! # });
//! ```

pub mod connection_pool;
pub mod rpc;
pub mod server;

// Re-export the most commonly used types at the module root for convenience.
pub use rpc::{
    decode_message, encode_message, Acks, BrokerInfo, FetchMessage, LogEntry,
    PartitionInfo, RpcRequest, RpcResponse, TopicInfo,
};
pub use server::{connect, send_request, NetworkServer, RequestHandler};
