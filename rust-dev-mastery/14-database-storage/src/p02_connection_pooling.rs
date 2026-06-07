//! # Connection Pooling
//!
//! Connection pools manage a set of reusable database connections, avoiding the
//! overhead of establishing new connections for each query. This lesson covers
//! pool configuration, health checks, and monitoring.
//!
//! ## Key Concepts
//! - Why connection pooling matters
//! - Pool sizing strategies
//! - Connection lifecycle (creation, validation, eviction)
//! - Health checks and dead connection detection
//! - Pool statistics and monitoring
//! - Timeout and retry configuration

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 1. Connection Pool Configuration
// ---------------------------------------------------------------------------

/// Configuration for a connection pool.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of idle connections to maintain.
    pub min_idle: u32,
    /// Maximum number of connections in the pool.
    pub max_size: u32,
    /// How long a connection can be idle before being closed.
    pub idle_timeout: Duration,
    /// Maximum lifetime of a connection.
    pub max_lifetime: Duration,
    /// How long to wait for a connection before timing out.
    pub acquire_timeout: Duration,
    /// How often to run health checks on idle connections.
    pub health_check_interval: Duration,
    /// SQL to run for connection validation.
    pub test_query: String,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_idle: 2,
            max_size: 10,
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            acquire_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(60),
            test_query: "SELECT 1".into(),
        }
    }
}

impl PoolConfig {
    /// Calculate recommended pool size based on CPU cores.
    /// Common formula: connections = (core_count * 2) + effective_spindle_count
    pub fn recommended_size(cpu_cores: u32) -> u32 {
        (cpu_cores * 2) + 1
    }

    pub fn validate(&self) -> Result<(), PoolConfigError> {
        if self.min_idle > self.max_size {
            return Err(PoolConfigError::MinIdleExceedsMax);
        }
        if self.max_size == 0 {
            return Err(PoolConfigError::ZeroMaxSize);
        }
        if self.acquire_timeout.is_zero() {
            return Err(PoolConfigError::ZeroTimeout);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PoolConfigError {
    #[error("min_idle cannot exceed max_size")]
    MinIdleExceedsMax,
    #[error("max_size must be greater than 0")]
    ZeroMaxSize,
    #[error("acquire_timeout must be greater than 0")]
    ZeroTimeout,
}

// ---------------------------------------------------------------------------
// 2. Connection Wrapper
// ---------------------------------------------------------------------------

/// A database connection with metadata.
#[derive(Debug)]
pub struct Connection {
    pub id: u64,
    pub created_at: Instant,
    pub last_used: Instant,
    pub last_checked: Instant,
    pub query_count: u64,
    pub is_valid: bool,
}

impl Connection {
    pub fn new(id: u64) -> Self {
        let now = Instant::now();
        Self {
            id,
            created_at: now,
            last_used: now,
            last_checked: now,
            query_count: 0,
            is_valid: true,
        }
    }

    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    pub fn idle_time(&self) -> Duration {
        self.last_used.elapsed()
    }

    pub fn is_expired(&self, max_lifetime: Duration) -> bool {
        self.age() > max_lifetime
    }

    pub fn is_idle_expired(&self, idle_timeout: Duration) -> bool {
        self.idle_time() > idle_timeout
    }

    pub fn record_use(&mut self) {
        self.last_used = Instant::now();
        self.query_count += 1;
    }

    pub fn mark_checked(&mut self) {
        self.last_checked = Instant::now();
    }
}

// ---------------------------------------------------------------------------
// 3. Connection Pool
// ---------------------------------------------------------------------------

/// A generic connection pool implementation.
#[derive(Debug)]
pub struct ConnectionPool {
    config: PoolConfig,
    idle: VecDeque<Connection>,
    active_count: u32,
    next_id: u64,
    stats: PoolStats,
}

#[derive(Debug, Default, Clone)]
pub struct PoolStats {
    pub total_created: u64,
    pub total_destroyed: u64,
    pub total_acquired: u64,
    pub total_released: u64,
    pub total_timeout: u64,
    pub total_health_check_failures: u64,
}

impl PoolStats {
    pub fn active_connections(&self) -> u64 {
        self.total_created - self.total_destroyed
    }
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Result<Self, PoolConfigError> {
        config.validate()?;

        let mut pool = Self {
            config,
            idle: VecDeque::new(),
            active_count: 0,
            next_id: 1,
            stats: PoolStats::default(),
        };

        // Pre-create minimum idle connections
        for _ in 0..pool.config.min_idle {
            let conn = pool.create_connection();
            pool.idle.push_back(conn);
        }

        Ok(pool)
    }

    /// Acquire a connection from the pool.
    pub fn acquire(&mut self) -> Result<Connection, PoolError> {
        // Try to get an idle connection
        while let Some(conn) = self.idle.pop_front() {
            // Check if connection is expired
            if conn.is_expired(self.config.max_lifetime)
                || conn.is_idle_expired(self.config.idle_timeout)
            {
                self.destroy_connection(conn);
                continue;
            }

            if !conn.is_valid {
                self.destroy_connection(conn);
                continue;
            }

            self.active_count += 1;
            self.stats.total_acquired += 1;
            return Ok(conn);
        }

        // No idle connections; try to create a new one
        if self.total_connections() < self.config.max_size {
            let conn = self.create_connection();
            self.active_count += 1;
            self.stats.total_acquired += 1;
            return Ok(conn);
        }

        // Pool is full
        self.stats.total_timeout += 1;
        Err(PoolError::Timeout)
    }

    /// Release a connection back to the pool.
    pub fn release(&mut self, mut conn: Connection) {
        self.active_count = self.active_count.saturating_sub(1);
        self.stats.total_released += 1;
        conn.mark_checked();
        self.idle.push_back(conn);
    }

    /// Run health checks on idle connections.
    pub fn health_check(&mut self) -> u32 {
        let mut checked = 0u32;
        let mut to_remove = Vec::new();

        for (i, conn) in self.idle.iter_mut().enumerate() {
            if conn.idle_time() < self.config.health_check_interval {
                continue;
            }

            conn.mark_checked();
            checked += 1;

            // Simulate health check (in real impl, execute test_query)
            if conn.is_expired(self.config.max_lifetime) {
                to_remove.push(i);
                self.stats.total_health_check_failures += 1;
            }
        }

        // Remove expired connections (reverse order to preserve indices)
        for i in to_remove.into_iter().rev() {
            if let Some(conn) = self.idle.remove(i) {
                self.destroy_connection(conn);
            }
        }

        // Ensure minimum idle connections
        while self.idle.len() < self.config.min_idle as usize
            && self.total_connections() < self.config.max_size
        {
            let conn = self.create_connection();
            self.idle.push_back(conn);
        }

        checked
    }

    /// Get the total number of connections (idle + active).
    pub fn total_connections(&self) -> u32 {
        self.idle.len() as u32 + self.active_count
    }

    /// Get the number of idle connections.
    pub fn idle_connections(&self) -> u32 {
        self.idle.len() as u32
    }

    /// Get the number of active connections.
    pub fn active_connections(&self) -> u32 {
        self.active_count
    }

    /// Get pool statistics.
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// Get the pool configuration.
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    fn create_connection(&mut self) -> Connection {
        let id = self.next_id;
        self.next_id += 1;
        self.stats.total_created += 1;
        Connection::new(id)
    }

    fn destroy_connection(&mut self, _conn: Connection) {
        self.stats.total_destroyed += 1;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("timed out waiting for available connection")]
    Timeout,
    #[error("pool is closed")]
    Closed,
    #[error("connection creation failed: {0}")]
    CreationFailed(String),
}

// ---------------------------------------------------------------------------
// 4. Pool Builder
// ---------------------------------------------------------------------------

/// Fluent builder for connection pools.
pub struct PoolBuilder {
    config: PoolConfig,
}

impl PoolBuilder {
    pub fn new() -> Self {
        Self {
            config: PoolConfig::default(),
        }
    }

    pub fn min_idle(mut self, min: u32) -> Self {
        self.config.min_idle = min;
        self
    }

    pub fn max_size(mut self, max: u32) -> Self {
        self.config.max_size = max;
        self
    }

    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.config.idle_timeout = timeout;
        self
    }

    pub fn max_lifetime(mut self, lifetime: Duration) -> Self {
        self.config.max_lifetime = lifetime;
        self
    }

    pub fn acquire_timeout(mut self, timeout: Duration) -> Self {
        self.config.acquire_timeout = timeout;
        self
    }

    pub fn build(self) -> Result<ConnectionPool, PoolConfigError> {
        ConnectionPool::new(self.config)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.min_idle, 2);
        assert_eq!(config.max_size, 10);
        assert_eq!(config.test_query, "SELECT 1");
    }

    #[test]
    fn test_pool_config_recommended_size() {
        assert_eq!(PoolConfig::recommended_size(1), 3);
        assert_eq!(PoolConfig::recommended_size(4), 9);
        assert_eq!(PoolConfig::recommended_size(8), 17);
    }

    #[test]
    fn test_pool_config_validation() {
        let config = PoolConfig::default();
        assert!(config.validate().is_ok());

        let config = PoolConfig {
            min_idle: 20,
            max_size: 10,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        let config = PoolConfig {
            max_size: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_pool_creation() {
        let pool = ConnectionPool::new(PoolConfig {
            min_idle: 3,
            max_size: 10,
            ..Default::default()
        })
        .unwrap();

        assert_eq!(pool.idle_connections(), 3);
        assert_eq!(pool.active_connections(), 0);
        assert_eq!(pool.total_connections(), 3);
    }

    #[test]
    fn test_pool_acquire_and_release() {
        let mut pool = ConnectionPool::new(PoolConfig {
            min_idle: 2,
            max_size: 5,
            ..Default::default()
        })
        .unwrap();

        let conn = pool.acquire().unwrap();
        assert_eq!(pool.active_connections(), 1);
        assert_eq!(pool.idle_connections(), 1);

        pool.release(conn);
        assert_eq!(pool.active_connections(), 0);
        assert_eq!(pool.idle_connections(), 2);
    }

    #[test]
    fn test_pool_acquire_creates_new_when_needed() {
        let mut pool = ConnectionPool::new(PoolConfig {
            min_idle: 0,
            max_size: 5,
            ..Default::default()
        })
        .unwrap();

        let _c1 = pool.acquire().unwrap();
        let _c2 = pool.acquire().unwrap();
        let _c3 = pool.acquire().unwrap();

        assert_eq!(pool.total_connections(), 3);
    }

    #[test]
    fn test_pool_max_size_enforced() {
        let mut pool = ConnectionPool::new(PoolConfig {
            min_idle: 0,
            max_size: 2,
            ..Default::default()
        })
        .unwrap();

        let _c1 = pool.acquire().unwrap();
        let _c2 = pool.acquire().unwrap();
        assert!(pool.acquire().is_err());
    }

    #[test]
    fn test_pool_reuse_released_connections() {
        let mut pool = ConnectionPool::new(PoolConfig {
            min_idle: 0,
            max_size: 5,
            ..Default::default()
        })
        .unwrap();

        let conn1 = pool.acquire().unwrap();
        let id1 = conn1.id;
        pool.release(conn1);

        let conn2 = pool.acquire().unwrap();
        assert_eq!(conn2.id, id1); // same connection reused
    }

    #[test]
    fn test_pool_stats() {
        let mut pool = ConnectionPool::new(PoolConfig {
            min_idle: 0,
            max_size: 10,
            ..Default::default()
        })
        .unwrap();

        let c1 = pool.acquire().unwrap();
        let c2 = pool.acquire().unwrap();
        pool.release(c1);
        pool.release(c2);

        let stats = pool.stats();
        assert_eq!(stats.total_acquired, 2);
        assert_eq!(stats.total_released, 2);
    }

    #[test]
    fn test_pool_health_check() {
        let mut pool = ConnectionPool::new(PoolConfig {
            min_idle: 2,
            max_size: 10,
            health_check_interval: Duration::from_millis(0), // always check
            ..Default::default()
        })
        .unwrap();

        let checked = pool.health_check();
        assert!(checked > 0);
    }

    #[test]
    fn test_pool_builder() {
        let pool = PoolBuilder::new()
            .min_idle(3)
            .max_size(20)
            .idle_timeout(Duration::from_secs(300))
            .build()
            .unwrap();

        assert_eq!(pool.config().min_idle, 3);
        assert_eq!(pool.config().max_size, 20);
    }

    #[test]
    fn test_connection_metadata() {
        let mut conn = Connection::new(1);
        assert_eq!(conn.id, 1);
        assert_eq!(conn.query_count, 0);
        assert!(conn.is_valid);
        assert!(conn.age() < Duration::from_secs(1));

        conn.record_use();
        assert_eq!(conn.query_count, 1);
    }

    #[test]
    fn test_connection_expiration() {
        let conn = Connection::new(1);
        assert!(!conn.is_expired(Duration::from_secs(3600)));
        assert!(!conn.is_idle_expired(Duration::from_secs(3600)));
    }

    #[test]
    fn test_pool_stats_active_connections() {
        let mut stats = PoolStats::default();
        assert_eq!(stats.active_connections(), 0);

        stats.total_created = 5;
        stats.total_destroyed = 2;
        assert_eq!(stats.active_connections(), 3);
    }
}
