//! Connection pool for reusing TCP connections to broker peers.
//!
//! The pool maintains a set of idle connections per remote address.  When a
//! connection is requested the pool first tries to hand back an existing idle
//! connection; if none are available it creates a new one (subject to the
//! configured maximum).  Connections are returned to the pool after use so
//! they can be reused by subsequent requests.
//!
//! # Thread safety
//!
//! `ConnectionPool` uses `tokio::sync::Mutex` for the idle-connection map and
//! atomics for the active-connection counter, so it is safe to share across
//! async tasks via `Arc<ConnectionPool>`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// A pool of reusable TCP connections keyed by remote address.
pub struct ConnectionPool {
    /// Maximum number of connections the pool may hold open at any time
    /// (checked-out + idle).
    max_connections: usize,
    /// Idle connections grouped by remote address.
    idle: Arc<Mutex<HashMap<String, Vec<TcpStream>>>>,
    /// Number of connections currently checked out (not in `idle`).
    active_count: AtomicUsize,
    /// Running total of connections created (for diagnostics).
    total_created: AtomicUsize,
}

impl ConnectionPool {
    /// Create a new, empty connection pool.
    pub fn new(max_connections: usize) -> Self {
        Self {
            max_connections,
            idle: Arc::new(Mutex::new(HashMap::new())),
            active_count: AtomicUsize::new(0),
            total_created: AtomicUsize::new(0),
        }
    }

    /// Obtain a connection to `addr`.
    ///
    /// The pool first checks for an idle connection.  If none is available it
    /// creates a new TCP connection, subject to the pool's capacity limit.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool is at capacity or the TCP connect fails.
    pub async fn get_connection(
        &self,
        addr: &str,
    ) -> crate::error::Result<TcpStream> {
        // Fast path: try to reuse an idle connection.
        {
            let mut idle = self.idle.lock().await;
            if let Some(conns) = idle.get_mut(addr) {
                if let Some(conn) = conns.pop() {
                    self.active_count.fetch_add(1, Ordering::SeqCst);
                    debug!(
                        addr = %addr,
                        "Reused idle connection"
                    );
                    return Ok(conn);
                }
                // No idle connections for this addr; remove empty entry.
                idle.remove(addr);
            }
        }

        // Slow path: create a new connection.
        // Check capacity before connecting.
        let current_active = self.active_count.load(Ordering::SeqCst);
        let current_idle: usize = {
            let idle = self.idle.lock().await;
            idle.values().map(|v| v.len()).sum()
        };
        let total = current_active + current_idle;

        if total >= self.max_connections {
            warn!(
                active = current_active,
                idle = current_idle,
                max = self.max_connections,
                addr = %addr,
                "Connection pool at capacity"
            );
            return Err(crate::error::MqError::Network(format!(
                "Connection pool exhausted: {} active + {} idle = {} (max: {}). \
                 Failed to connect to {}",
                current_active,
                current_idle,
                total,
                self.max_connections,
                addr,
            )));
        }

        let stream = TcpStream::connect(addr).await.map_err(|e| {
            crate::error::MqError::Network(format!(
                "Failed to connect to {}: {}",
                addr, e
            ))
        })?;

        // Disable Nagle's algorithm for lower latency on small messages.
        stream.set_nodelay(true).map_err(|e| {
            crate::error::MqError::Network(format!(
                "Failed to set TCP_NODELAY on connection to {}: {}",
                addr, e
            ))
        })?;

        self.active_count.fetch_add(1, Ordering::SeqCst);
        self.total_created.fetch_add(1, Ordering::SeqCst);

        debug!(addr = %addr, "Created new connection");
        Ok(stream)
    }

    /// Return a connection to the pool for reuse.
    ///
    /// The connection will be stored in the idle map and handed out to future
    /// `get_connection` calls for the same address.
    pub async fn return_connection(&self, addr: String, conn: TcpStream) {
        self.active_count.fetch_sub(1, Ordering::SeqCst);

        // Cap idle connections per address to avoid unbounded growth.
        // Allow at most max_connections / 4 idle per address (minimum 1).
        let max_idle_per_addr =
            std::cmp::max(1, self.max_connections / 4);

        let mut idle = self.idle.lock().await;
        let conns = idle.entry(addr.clone()).or_default();
        if conns.len() < max_idle_per_addr {
            conns.push(conn);
            debug!(
                addr = %addr,
                idle_count = conns.len(),
                "Connection returned to pool"
            );
        } else {
            debug!(
                addr = %addr,
                "Idle connection limit reached, dropping connection"
            );
        }
    }

    /// Discard all idle connections for the given address.
    ///
    /// Useful when a remote peer is known to be unreachable.
    pub async fn remove_connections(&self, addr: &str) {
        let mut idle = self.idle.lock().await;
        if let Some(conns) = idle.remove(addr) {
            let count = conns.len();
            // Dropping the connections closes them.
            drop(conns);
            debug!(
                addr = %addr,
                count,
                "Removed all idle connections for address"
            );
        }
    }

    /// Return the number of connections currently checked out.
    pub fn active_count(&self) -> usize {
        self.active_count.load(Ordering::SeqCst)
    }

    /// Return the total number of connections created since the pool was
    /// instantiated (checked-out + idle + dropped).
    pub fn total_created(&self) -> usize {
        self.total_created.load(Ordering::SeqCst)
    }

    /// Return the total number of idle (available) connections across all
    /// addresses.
    pub async fn idle_count(&self) -> usize {
        let idle = self.idle.lock().await;
        idle.values().map(|v| v.len()).sum()
    }

    /// Close all idle connections and reset the pool.
    pub async fn clear(&self) {
        let mut idle = self.idle.lock().await;
        let total: usize = idle.values().map(|v| v.len()).sum();
        idle.clear();
        if total > 0 {
            debug!(dropped = total, "Cleared connection pool");
        }
    }
}

impl std::fmt::Debug for ConnectionPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionPool")
            .field("max_connections", &self.max_connections)
            .field("active_count", &self.active_count.load(Ordering::Relaxed))
            .field(
                "total_created",
                &self.total_created.load(Ordering::Relaxed),
            )
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    /// Start a dummy TCP server that accepts connections and holds them open.
    /// Returns the bound address and a join handle.
    async fn dummy_server() -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        // Hold the connection open until the task is dropped.
                        tokio::spawn(async move {
                            let mut buf = [0u8; 1];
                            let _ = stream.peek(&mut buf).await;
                        });
                    }
                    Err(_) => break,
                }
            }
        });
        (addr, handle)
    }

    #[tokio::test]
    async fn get_and_return_connection() {
        let (addr, _server) = dummy_server().await;
        let pool = ConnectionPool::new(10);

        let conn = pool.get_connection(&addr).await.unwrap();
        assert_eq!(pool.active_count(), 1);
        assert_eq!(pool.total_created(), 1);

        pool.return_connection(addr.clone(), conn).await;
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.idle_count().await, 1);
    }

    #[tokio::test]
    async fn reuse_idle_connection() {
        let (addr, _server) = dummy_server().await;
        let pool = ConnectionPool::new(10);

        // First connection: created from scratch.
        let conn = pool.get_connection(&addr).await.unwrap();
        pool.return_connection(addr.clone(), conn).await;
        assert_eq!(pool.total_created(), 1);

        // Second connection: should reuse the idle one.
        let _conn = pool.get_connection(&addr).await.unwrap();
        assert_eq!(pool.total_created(), 1); // No new connection created.
        assert_eq!(pool.active_count(), 1);
    }

    #[tokio::test]
    async fn connection_pool_capacity() {
        let (addr, _server) = dummy_server().await;
        let pool = ConnectionPool::new(2);

        let c1 = pool.get_connection(&addr).await.unwrap();
        let c2 = pool.get_connection(&addr).await.unwrap();

        // Pool is at capacity (2 active, 0 idle).
        let err = pool.get_connection(&addr).await;
        assert!(err.is_err());
        let err_msg = err.unwrap_err().to_string();
        assert!(err_msg.contains("exhausted"), "Error: {}", err_msg);

        // Return one; get should succeed again.
        pool.return_connection(addr.clone(), c1).await;
        let c3 = pool.get_connection(&addr).await.unwrap();
        assert_eq!(pool.active_count(), 2);

        // Clean up.
        pool.return_connection(addr.clone(), c2).await;
        pool.return_connection(addr.clone(), c3).await;
    }

    #[tokio::test]
    async fn remove_connections() {
        let (addr, _server) = dummy_server().await;
        let pool = ConnectionPool::new(10);

        let conn = pool.get_connection(&addr).await.unwrap();
        pool.return_connection(addr.clone(), conn).await;
        assert_eq!(pool.idle_count().await, 1);

        pool.remove_connections(&addr).await;
        assert_eq!(pool.idle_count().await, 0);
    }

    #[tokio::test]
    async fn clear_pool() {
        let (addr, _server) = dummy_server().await;
        let pool = ConnectionPool::new(10);

        let c1 = pool.get_connection(&addr).await.unwrap();
        let c2 = pool.get_connection(&addr).await.unwrap();
        pool.return_connection(addr.clone(), c1).await;
        pool.return_connection(addr.clone(), c2).await;

        assert_eq!(pool.idle_count().await, 2);
        pool.clear().await;
        assert_eq!(pool.idle_count().await, 0);
    }

    #[tokio::test]
    async fn active_count_tracking() {
        let (addr, _server) = dummy_server().await;
        let pool = ConnectionPool::new(10);

        assert_eq!(pool.active_count(), 0);

        let c1 = pool.get_connection(&addr).await.unwrap();
        assert_eq!(pool.active_count(), 1);

        let c2 = pool.get_connection(&addr).await.unwrap();
        assert_eq!(pool.active_count(), 2);

        pool.return_connection(addr.clone(), c1).await;
        assert_eq!(pool.active_count(), 1);

        pool.return_connection(addr.clone(), c2).await;
        assert_eq!(pool.active_count(), 0);
    }

    #[tokio::test]
    async fn multiple_addresses() {
        let (addr1, _s1) = dummy_server().await;
        let (addr2, _s2) = dummy_server().await;
        let pool = ConnectionPool::new(10);

        let c1 = pool.get_connection(&addr1).await.unwrap();
        let c2 = pool.get_connection(&addr2).await.unwrap();
        assert_eq!(pool.active_count(), 2);

        pool.return_connection(addr1.clone(), c1).await;
        pool.return_connection(addr2.clone(), c2).await;
        assert_eq!(pool.idle_count().await, 2);

        // Removing one address shouldn't affect the other.
        pool.remove_connections(&addr1).await;
        assert_eq!(pool.idle_count().await, 1);
    }

    #[test]
    fn debug_format() {
        let pool = ConnectionPool::new(512);
        let debug_str = format!("{:?}", pool);
        assert!(debug_str.contains("ConnectionPool"));
        assert!(debug_str.contains("512"));
    }
}
