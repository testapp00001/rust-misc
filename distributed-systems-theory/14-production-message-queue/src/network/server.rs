//! TCP server that accepts client connections and dispatches RPC requests.
//!
//! The server uses length-prefixed binary framing (see [`rpc::encode_message`])
//! and delegates request handling to a pluggable [`RequestHandler`] trait.
//! This keeps the networking layer decoupled from business logic -- the broker
//! implements `RequestHandler` to wire in storage, replication, and partitioning.
//!
//! # Architecture
//!
//! ```text
//! ┌────────────┐        TCP         ┌──────────────────┐      trait dispatch      ┌─────────────┐
//! │  Producer  │ ──── frames ────▶ │   NetworkServer   │ ──────────────────────▶ │   Broker     │
//! │  Consumer  │ ◀─── frames ──── │  (accept loop)     │ ◀────────────────────── │ (impl RH)    │
//! └────────────┘                   └──────────────────┘                          └─────────────┘
//! ```

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::error::{MqError, Result};
use crate::health::HealthMonitor;
use crate::metrics::Metrics;

use super::rpc::{decode_message, encode_message, RpcRequest, RpcResponse};

// ---------------------------------------------------------------------------
// RequestHandler trait
// ---------------------------------------------------------------------------

/// Application-level request handler.
///
/// Implement this trait to connect the networking layer to the broker's
/// business logic.  The server calls [`RequestHandler::handle`] for every
/// decoded request and sends the returned `RpcResponse` back to the client.
#[async_trait]
pub trait RequestHandler: Send + Sync + 'static {
    /// Process an RPC request and return a response.
    async fn handle(&self, request: RpcRequest) -> RpcResponse;
}

// ---------------------------------------------------------------------------
// DefaultRequestHandler
// ---------------------------------------------------------------------------

/// A minimal request handler that responds to health-check requests and
/// returns an error for everything else.
///
/// This is useful for testing the networking layer in isolation or as a
/// placeholder until the full broker implementation is wired in.
pub struct DefaultRequestHandler;

#[async_trait]
impl RequestHandler for DefaultRequestHandler {
    async fn handle(&self, request: RpcRequest) -> RpcResponse {
        match request {
            RpcRequest::Ping => RpcResponse::Pong,
            other => RpcResponse::Error(format!(
                "No handler registered for request variant: {:?}",
                std::mem::discriminant(&other)
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// NetworkServer
// ---------------------------------------------------------------------------

/// TCP server that listens for client connections and dispatches RPC requests.
///
/// The server is created with [`NetworkServer::new`] and started with
/// [`NetworkServer::start`].  Call [`NetworkServer::shutdown`] to gracefully
/// stop accepting new connections and drain in-flight requests.
pub struct NetworkServer {
    /// Bind address (`host:port`).
    addr: String,
    /// Maximum number of concurrent connections.
    max_connections: usize,
    /// Maximum size in bytes for a single RPC message frame.
    max_message_size: usize,
    /// Application request handler.
    handler: Arc<dyn RequestHandler>,
    /// Shared metrics collector.
    metrics: Arc<Metrics>,
    /// Shared health monitor.
    health: Arc<HealthMonitor>,
    /// Whether the server is currently running.
    running: AtomicBool,
    /// Number of active (in-flight) connections.
    active_connections: AtomicUsize,
    /// Sender for the shutdown broadcast channel.
    shutdown_tx: broadcast::Sender<()>,
}

impl NetworkServer {
    /// Create a new network server.
    ///
    /// Uses the [`DefaultRequestHandler`] which only responds to `Ping`.  Call
    /// [`NetworkServer::with_handler`] to supply a custom handler before
    /// starting the server.
    pub fn new(
        addr: String,
        max_connections: usize,
        metrics: Arc<Metrics>,
        health: Arc<HealthMonitor>,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            addr,
            max_connections,
            max_message_size: 16 * 1024 * 1024, // 16 MiB default
            handler: Arc::new(DefaultRequestHandler),
            metrics,
            health,
            running: AtomicBool::new(false),
            active_connections: AtomicUsize::new(0),
            shutdown_tx,
        }
    }

    /// Replace the default request handler with a custom implementation.
    ///
    /// Must be called before [`NetworkServer::start`].
    pub fn with_handler(mut self, handler: Arc<dyn RequestHandler>) -> Self {
        self.handler = handler;
        self
    }

    /// Set the maximum allowed RPC message size in bytes.
    ///
    /// Frames larger than this are rejected with an error response.  Must be
    /// called before [`NetworkServer::start`].
    pub fn with_max_message_size(mut self, max_message_size: usize) -> Self {
        self.max_message_size = max_message_size;
        self
    }

    /// Start listening for TCP connections.
    ///
    /// This method **blocks** the current task until the server is shut down
    /// (via [`NetworkServer::shutdown`] or a shutdown signal).
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr).await.map_err(|e| {
            MqError::Network(format!("Failed to bind to {}: {}", self.addr, e))
        })?;

        self.running.store(true, Ordering::SeqCst);
        info!(addr = %self.addr, "Network server listening");

        let max_conns = self.max_connections;
        let max_msg_size = self.max_message_size;
        let metrics = Arc::clone(&self.metrics);
        let health = Arc::clone(&self.health);
        let handler = Arc::clone(&self.handler);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, peer_addr)) => {
                            // Enforce connection limit.
                            let current_active =
                                self.active_connections.load(Ordering::SeqCst);
                            if current_active >= max_conns as usize {
                                warn!(
                                    peer = %peer_addr,
                                    active = current_active,
                                    max = max_conns,
                                    "Connection limit reached, rejecting"
                                );
                                // Send a brief error before dropping so the
                                // client knows why it was refused.
                                drop(stream);
                                continue;
                            }

                            self.active_connections.fetch_add(1, Ordering::SeqCst);
                            metrics.increment_connections();
                            health.set_connected_peers(
                                self.active_connections.load(Ordering::SeqCst) as u64,
                            );

                            let handler = Arc::clone(&handler);
                            let metrics = Arc::clone(&metrics);
                            let health = Arc::clone(&health);
                            let active = Arc::new(AtomicUsize::new(
                                self.active_connections.load(Ordering::SeqCst),
                            ));
                            let peer = peer_addr;

                            tokio::spawn(async move {
                                let result = handle_connection(
                                    stream,
                                    handler,
                                    &metrics,
                                    max_msg_size,
                                )
                                .await;

                                if let Err(e) = result {
                                    debug!(peer = %peer, error = %e, "Connection closed with error");
                                } else {
                                    debug!(peer = %peer, "Connection closed");
                                }

                                let remaining = active.fetch_sub(1, Ordering::SeqCst).saturating_sub(1);
                                metrics.decrement_connections();
                                health.set_connected_peers(remaining as u64);
                            });
                        }
                        Err(e) => {
                            // Transient accept errors (e.g. too many open files)
                            // should not kill the server.
                            error!(error = %e, "Failed to accept connection");
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received, stopping network server");
                    break;
                }
            }
        }

        self.running.store(false, Ordering::SeqCst);
        info!("Network server stopped");
        Ok(())
    }

    /// Signal the server to stop accepting new connections.
    ///
    /// In-flight requests are allowed to complete.  After this call, the
    /// [`NetworkServer::start`] future will resolve.
    pub async fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst);
        let _ = self.shutdown_tx.send(());
        info!("Shutdown signal sent");
    }

    /// Returns `true` if the server is currently accepting connections.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Return the number of active (in-flight) connections.
    pub fn active_connections(&self) -> usize {
        self.active_connections.load(Ordering::SeqCst)
    }

    /// Return the bound address.
    pub fn addr(&self) -> &str {
        &self.addr
    }
}

impl std::fmt::Debug for NetworkServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkServer")
            .field("addr", &self.addr)
            .field("max_connections", &self.max_connections)
            .field("max_message_size", &self.max_message_size)
            .field("running", &self.running.load(Ordering::Relaxed))
            .field(
                "active_connections",
                &self.active_connections.load(Ordering::Relaxed),
            )
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Connection handling
// ---------------------------------------------------------------------------

/// Handle a single TCP connection: read length-prefixed frames, dispatch each
/// request through the handler, and write back the response.
///
/// Returns `Ok(())` on clean disconnect (EOF) or `Err` on I/O / protocol
/// errors.
async fn handle_connection(
    mut stream: TcpStream,
    handler: Arc<dyn RequestHandler>,
    metrics: &Metrics,
    max_message_size: usize,
) -> Result<()> {
    let peer_addr = stream
        .peer_addr()
        .map_err(|e| MqError::Network(format!("Failed to get peer addr: {}", e)))?;

    debug!(peer = %peer_addr, "Connection established");

    let mut len_buf = [0u8; 4];

    loop {
        // --- Read length prefix ------------------------------------------------
        match stream.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                debug!(peer = %peer_addr, "Client disconnected (EOF)");
                return Ok(());
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
                debug!(peer = %peer_addr, "Connection reset by client");
                return Ok(());
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                debug!(peer = %peer_addr, "Broken pipe");
                return Ok(());
            }
            Err(e) => {
                return Err(MqError::Network(format!(
                    "Read error from {}: {}",
                    peer_addr, e
                )));
            }
        }

        let frame_len = u32::from_be_bytes(len_buf) as usize;

        // --- Enforce maximum message size ------------------------------------
        if frame_len > max_message_size {
            warn!(
                peer = %peer_addr,
                frame_len,
                max = max_message_size,
                "Frame exceeds maximum size"
            );
            let err_resp = RpcResponse::Error(format!(
                "Message too large: {} bytes (max: {} bytes)",
                frame_len, max_message_size
            ));
            write_response(&mut stream, &err_resp).await?;
            continue;
        }

        if frame_len == 0 {
            warn!(peer = %peer_addr, "Received zero-length frame");
            let err_resp = RpcResponse::Error("Empty frame".to_string());
            write_response(&mut stream, &err_resp).await?;
            continue;
        }

        // --- Read payload -----------------------------------------------------
        let mut payload = vec![0u8; frame_len];
        stream
            .read_exact(&mut payload)
            .await
            .map_err(|e| {
                MqError::Network(format!("Read payload error from {}: {}", peer_addr, e))
            })?;

        // --- Decode request ---------------------------------------------------
        let request: RpcRequest = match decode_message(&payload) {
            Ok(req) => req,
            Err(e) => {
                warn!(peer = %peer_addr, error = %e, "Failed to decode request");
                let err_resp = RpcResponse::Error(format!("Decode error: {}", e));
                write_response(&mut stream, &err_resp).await?;
                continue;
            }
        };

        // --- Dispatch and measure latency ------------------------------------
        let start = Instant::now();
        let response = handler.handle(request).await;
        let elapsed_us = start.elapsed().as_micros() as u64;

        metrics.record_latency("rpc_request", elapsed_us);
        if matches!(&response, RpcResponse::Error(_)) {
            metrics.record_error("rpc_error");
        }

        // --- Write response ---------------------------------------------------
        write_response(&mut stream, &response).await?;
    }
}

/// Encode and write a length-prefixed response to the stream.
async fn write_response(
    stream: &mut TcpStream,
    response: &RpcResponse,
) -> Result<()> {
    let frame = encode_message(response)?;
    stream
        .write_all(&frame)
        .await
        .map_err(|e| MqError::Network(format!("Write response error: {}", e)))?;
    stream
        .flush()
        .await
        .map_err(|e| MqError::Network(format!("Flush error: {}", e)))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Client helpers
// ---------------------------------------------------------------------------

/// Connect to a remote broker with a timeout.
pub async fn connect(
    addr: &str,
    timeout: std::time::Duration,
) -> Result<TcpStream> {
    let stream = tokio::time::timeout(timeout, TcpStream::connect(addr))
        .await
        .map_err(|_| {
            MqError::Timeout(format!(
                "Connection to {} timed out after {:?}",
                addr, timeout
            ))
        })?
        .map_err(|e| {
            MqError::Network(format!("Failed to connect to {}: {}", addr, e))
        })?;

    stream.set_nodelay(true).map_err(|e| {
        MqError::Network(format!(
            "Failed to set TCP_NODELAY on connection to {}: {}",
            addr, e
        ))
    })?;

    Ok(stream)
}

/// Send an RPC request over an existing connection and return the response.
///
/// This is a synchronous request-response helper suitable for simple clients.
/// For multiplexed or pipelined communication, build a custom framing layer.
pub async fn send_request(
    stream: &mut TcpStream,
    request: &RpcRequest,
) -> Result<RpcResponse> {
    // Write request.
    let frame = encode_message(request)?;
    stream
        .write_all(&frame)
        .await
        .map_err(|e| MqError::Network(format!("Write request error: {}", e)))?;
    stream
        .flush()
        .await
        .map_err(|e| MqError::Network(format!("Flush error: {}", e)))?;

    // Read response length.
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| MqError::Network(format!("Read response length error: {}", e)))?;

    let resp_len = u32::from_be_bytes(len_buf) as usize;
    let mut payload = vec![0u8; resp_len];
    stream
        .read_exact(&mut payload)
        .await
        .map_err(|e| MqError::Network(format!("Read response payload error: {}", e)))?;

    decode_message(&payload)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rpc::{
        Acks, BrokerInfo, FetchMessage, LogEntry, PartitionInfo, TopicInfo,
    };
    use std::time::Duration;

    /// A test handler that echoes request variants as recognizable responses.
    struct EchoHandler;

    #[async_trait]
    impl RequestHandler for EchoHandler {
        async fn handle(&self, request: RpcRequest) -> RpcResponse {
            match request {
                RpcRequest::Ping => RpcResponse::Pong,
                RpcRequest::Produce { topic: _, messages, .. } => {
                    RpcResponse::Produce {
                        offsets: (0..messages.len() as u64).collect(),
                    }
                }
                RpcRequest::Fetch { .. } => RpcResponse::Fetch {
                    messages: vec![FetchMessage {
                        offset: 0,
                        key: Some(b"echo-key".to_vec()),
                        value: b"echo-value".to_vec(),
                        timestamp: 1_000_000,
                        headers: vec![],
                    }],
                    high_watermark: 1,
                },
                RpcRequest::CreateTopic { .. } => RpcResponse::CreateTopic {
                    success: true,
                },
                RpcRequest::ListTopics => RpcResponse::ListTopics {
                    topics: vec![TopicInfo {
                        name: "test-topic".into(),
                        partitions: 3,
                        replication_factor: 1,
                    }],
                },
                RpcRequest::GetMetadata { .. } => RpcResponse::GetMetadata {
                    brokers: vec![BrokerInfo {
                        id: 1,
                        address: "127.0.0.1:9092".into(),
                        rack: None,
                    }],
                    partitions: vec![PartitionInfo {
                        topic: "test-topic".into(),
                        partition: 0,
                        leader: 1,
                        replicas: vec![1],
                        isr: vec![1],
                        high_watermark: 0,
                    }],
                },
                RpcRequest::AppendEntries {
                    term, leader_id: _, ..
                } => RpcResponse::AppendEntries {
                    term,
                    success: true,
                },
                RpcRequest::RequestVote { term, .. } => RpcResponse::RequestVote {
                    term,
                    vote_granted: true,
                },
                RpcRequest::InstallSnapshot { term, .. } => {
                    RpcResponse::InstallSnapshot {
                        term,
                        success: true,
                    }
                }
                _ => RpcResponse::Error("unhandled".into()),
            }
        }
    }

    /// Start a server on a random port and return the server handle and bound
    /// address.  The server runs in a background task.
    async fn start_test_server() -> (NetworkServer, String) {
        // Find a free port by briefly binding then releasing.
        let probe = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = probe.local_addr().unwrap().to_string();
        drop(probe);

        let metrics = Metrics::new();
        let health = HealthMonitor::shared();
        let server = NetworkServer::new(addr.clone(), 100, metrics, health)
            .with_handler(Arc::new(EchoHandler));

        // Clone the server state for the background task (the original
        // `server` handle is returned to the caller for shutdown).
        let bg = NetworkServer {
            addr: server.addr().to_string(),
            max_connections: server.max_connections,
            max_message_size: server.max_message_size,
            handler: Arc::clone(&server.handler),
            metrics: Arc::clone(&server.metrics),
            health: Arc::clone(&server.health),
            running: AtomicBool::new(false),
            active_connections: AtomicUsize::new(0),
            shutdown_tx: server.shutdown_tx.clone(),
        };

        tokio::spawn(async move {
            let _ = bg.start().await;
        });

        // Give the server time to bind the socket.
        tokio::time::sleep(Duration::from_millis(50)).await;

        (server, addr)
    }

    #[tokio::test]
    async fn ping_pong() {
        let (_server, addr) = start_test_server().await;

        let mut stream = connect(&addr, Duration::from_secs(5)).await.unwrap();
        let response = send_request(&mut stream, &RpcRequest::Ping).await.unwrap();
        assert!(matches!(response, RpcResponse::Pong));
    }

    #[tokio::test]
    async fn produce_and_fetch() {
        let (_server, addr) = start_test_server().await;
        let mut stream = connect(&addr, Duration::from_secs(5)).await.unwrap();

        // Produce.
        let produce_resp = send_request(
            &mut stream,
            &RpcRequest::Produce {
                topic: "orders".into(),
                partition: None,
                messages: vec![b"msg1".to_vec(), b"msg2".to_vec(), b"msg3".to_vec()],
                acks: Acks::Leader,
            },
        )
        .await
        .unwrap();

        if let RpcResponse::Produce { offsets } = produce_resp {
            assert_eq!(offsets, vec![0, 1, 2]);
        } else {
            panic!("expected Produce response, got {:?}", produce_resp);
        }

        // Fetch.
        let fetch_resp = send_request(
            &mut stream,
            &RpcRequest::Fetch {
                topic: "orders".into(),
                partition: 0,
                offset: 0,
                max_bytes: 1024,
            },
        )
        .await
        .unwrap();

        if let RpcResponse::Fetch {
            messages,
            high_watermark,
        } = fetch_resp
        {
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].key, Some(b"echo-key".to_vec()));
            assert_eq!(high_watermark, 1);
        } else {
            panic!("expected Fetch response");
        }
    }

    #[tokio::test]
    async fn create_topic() {
        let (_server, addr) = start_test_server().await;
        let mut stream = connect(&addr, Duration::from_secs(5)).await.unwrap();

        let resp = send_request(
            &mut stream,
            &RpcRequest::CreateTopic {
                topic: "events".into(),
                partitions: 6,
                replication_factor: 3,
            },
        )
        .await
        .unwrap();

        assert!(matches!(resp, RpcResponse::CreateTopic { success: true }));
    }

    #[tokio::test]
    async fn list_topics() {
        let (_server, addr) = start_test_server().await;
        let mut stream = connect(&addr, Duration::from_secs(5)).await.unwrap();

        let resp = send_request(&mut stream, &RpcRequest::ListTopics)
            .await
            .unwrap();

        if let RpcResponse::ListTopics { topics } = resp {
            assert_eq!(topics.len(), 1);
            assert_eq!(topics[0].name, "test-topic");
            assert_eq!(topics[0].partitions, 3);
        } else {
            panic!("expected ListTopics response");
        }
    }

    #[tokio::test]
    async fn raft_append_entries() {
        let (_server, addr) = start_test_server().await;
        let mut stream = connect(&addr, Duration::from_secs(5)).await.unwrap();

        let resp = send_request(
            &mut stream,
            &RpcRequest::AppendEntries {
                term: 5,
                leader_id: 1,
                prev_log_index: 10,
                prev_log_term: 4,
                entries: vec![LogEntry {
                    index: 11,
                    term: 5,
                    data: vec![1, 2, 3],
                }],
                leader_commit: 11,
            },
        )
        .await
        .unwrap();

        if let RpcResponse::AppendEntries { term, success } = resp {
            assert_eq!(term, 5);
            assert!(success);
        } else {
            panic!("expected AppendEntries response");
        }
    }

    #[tokio::test]
    async fn raft_request_vote() {
        let (_server, addr) = start_test_server().await;
        let mut stream = connect(&addr, Duration::from_secs(5)).await.unwrap();

        let resp = send_request(
            &mut stream,
            &RpcRequest::RequestVote {
                term: 3,
                candidate_id: 2,
                last_log_index: 20,
                last_log_term: 3,
            },
        )
        .await
        .unwrap();

        if let RpcResponse::RequestVote { term, vote_granted } = resp {
            assert_eq!(term, 3);
            assert!(vote_granted);
        } else {
            panic!("expected RequestVote response");
        }
    }

    #[tokio::test]
    async fn server_start_and_shutdown() {
        let metrics = Metrics::new();
        let health = HealthMonitor::shared();
        let server = NetworkServer::new(
            "127.0.0.1:0".to_string(),
            100,
            metrics,
            health,
        );

        assert!(!server.is_running());
        assert_eq!(server.active_connections(), 0);

        tokio::spawn({
            let s = NetworkServer {
                addr: server.addr().to_string(),
                max_connections: server.max_connections,
                max_message_size: server.max_message_size,
                handler: Arc::clone(&server.handler),
                metrics: Arc::clone(&server.metrics),
                health: Arc::clone(&server.health),
                running: AtomicBool::new(false),
                active_connections: AtomicUsize::new(0),
                shutdown_tx: server.shutdown_tx.clone(),
            };
            async move {
                let _ = s.start().await;
            }
        });

        tokio::time::sleep(Duration::from_millis(50)).await;
        server.shutdown().await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(!server.is_running());
    }

    #[tokio::test]
    async fn multiple_concurrent_clients() {
        let (_server, addr) = start_test_server().await;
        let mut handles = Vec::new();

        for i in 0..10 {
            let addr = addr.clone();
            handles.push(tokio::spawn(async move {
                let mut stream =
                    connect(&addr, Duration::from_secs(5)).await.unwrap();

                for j in 0..5 {
                    let resp = send_request(
                        &mut stream,
                        &RpcRequest::Produce {
                            topic: format!("topic-{}", i),
                            partition: None,
                            messages: vec![format!("msg-{}-{}", i, j).into_bytes()],
                            acks: Acks::None,
                        },
                    )
                    .await
                    .unwrap();

                    if let RpcResponse::Produce { offsets } = resp {
                        // EchoHandler returns one offset per message.
                        assert_eq!(offsets.len(), 1);
                    } else {
                        panic!("expected Produce response on iteration {}", j);
                    }
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[test]
    fn server_debug_format() {
        let metrics = Metrics::new();
        let health = HealthMonitor::shared();
        let server = NetworkServer::new(
            "127.0.0.1:9999".to_string(),
            512,
            metrics,
            health,
        );
        let debug = format!("{:?}", server);
        assert!(debug.contains("NetworkServer"));
        assert!(debug.contains("9999"));
        assert!(debug.contains("512"));
    }
}
