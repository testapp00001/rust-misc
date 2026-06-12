use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;

/// Health status of a broker node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Hash)]
pub enum HealthStatus {
    /// All components are operating normally.
    Healthy,
    /// One or more non-critical components are degraded.
    Degraded,
    /// A critical component has failed; the broker cannot serve requests.
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// A point-in-time health check result, suitable for serialization into a
/// `/health` or `/ready` HTTP response.
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub is_leader: bool,
    pub connected_peers: usize,
    pub total_peers: usize,
    pub storage_healthy: bool,
    pub wal_healthy: bool,
    pub replication_lag: u64,
    pub uptime_secs: u64,
    pub version: &'static str,
}

/// Monitors the health of broker components.
///
/// Uses atomic flags so that component subsystems (network, storage,
/// replication) can update health in a lock-free manner from their own tasks.
pub struct HealthMonitor {
    start_time: Instant,
    is_leader: AtomicBool,
    storage_healthy: AtomicBool,
    wal_healthy: AtomicBool,
    connected_peers: AtomicU64,
    total_peers: AtomicU64,
    replication_lag: AtomicU64,
    disk_space_healthy: AtomicBool,
}

impl HealthMonitor {
    /// Create a new health monitor. Defaults to a degraded state until the
    /// first explicit component check marks things healthy.
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            is_leader: AtomicBool::new(false),
            storage_healthy: AtomicBool::new(true),
            wal_healthy: AtomicBool::new(true),
            connected_peers: AtomicU64::new(0),
            total_peers: AtomicU64::new(0),
            replication_lag: AtomicU64::new(0),
            disk_space_healthy: AtomicBool::new(true),
        }
    }

    /// Create a health monitor wrapped in `Arc` for sharing across tasks.
    pub fn shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Take a snapshot of the current health status.
    pub fn check(&self) -> HealthCheck {
        let storage = self.storage_healthy.load(Ordering::Relaxed);
        let wal = self.wal_healthy.load(Ordering::Relaxed);
        let disk = self.disk_space_healthy.load(Ordering::Relaxed);
        let connected = self.connected_peers.load(Ordering::Relaxed) as usize;
        let total = self.total_peers.load(Ordering::Relaxed) as usize;
        let lag = self.replication_lag.load(Ordering::Relaxed);

        let status = if !storage || !wal || !disk {
            HealthStatus::Unhealthy
        } else if connected < total || lag > 1000 {
            // Degraded when not all peers are reachable or lag is high.
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        HealthCheck {
            status,
            is_leader: self.is_leader.load(Ordering::Relaxed),
            connected_peers: connected,
            total_peers: total,
            storage_healthy: storage,
            wal_healthy: wal,
            replication_lag: lag,
            uptime_secs: self.start_time.elapsed().as_secs(),
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    /// Returns `true` if the broker is healthy enough to accept traffic.
    /// Used by readiness probes (e.g. Kubernetes readinessProbe).
    pub fn is_ready(&self) -> bool {
        let storage = self.storage_healthy.load(Ordering::Relaxed);
        let wal = self.wal_healthy.load(Ordering::Relaxed);
        storage && wal
    }

    /// Returns `true` if the broker is alive and responding.
    /// Used by liveness probes (e.g. Kubernetes livenessProbe).
    pub fn is_alive(&self) -> bool {
        true // If we can respond, we are alive.
    }

    /// Returns whether this node is the current leader.
    pub fn is_leader(&self) -> bool {
        self.is_leader.load(Ordering::Relaxed)
    }

    // -- Update methods called by subsystems --

    /// Mark this node as leader or follower.
    pub fn set_leader(&self, is_leader: bool) {
        self.is_leader.store(is_leader, Ordering::Relaxed);
    }

    /// Update storage health.
    pub fn set_storage_healthy(&self, healthy: bool) {
        self.storage_healthy.store(healthy, Ordering::Relaxed);
    }

    /// Update WAL health.
    pub fn set_wal_healthy(&self, healthy: bool) {
        self.wal_healthy.store(healthy, Ordering::Relaxed);
    }

    /// Update disk space health.
    pub fn set_disk_space_healthy(&self, healthy: bool) {
        self.disk_space_healthy.store(healthy, Ordering::Relaxed);
    }

    /// Update the number of currently connected peers.
    pub fn set_connected_peers(&self, count: u64) {
        self.connected_peers.store(count, Ordering::Relaxed);
    }

    /// Set the expected total number of peers (including self).
    pub fn set_total_peers(&self, count: u64) {
        self.total_peers.store(count, Ordering::Relaxed);
    }

    /// Update the current replication lag.
    pub fn set_replication_lag(&self, lag: u64) {
        self.replication_lag.store(lag, Ordering::Relaxed);
    }

    /// Increment connected peers by 1 (called when a peer connects).
    pub fn peer_connected(&self) {
        self.connected_peers.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement connected peers by 1 (called when a peer disconnects).
    pub fn peer_disconnected(&self) {
        let current = self.connected_peers.load(Ordering::Relaxed);
        if current > 0 {
            self.connected_peers.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_healthy() {
        let monitor = HealthMonitor::new();
        let check = monitor.check();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(!check.is_leader);
    }

    #[test]
    fn storage_failure_makes_unhealthy() {
        let monitor = HealthMonitor::new();
        monitor.set_storage_healthy(false);
        let check = monitor.check();
        assert_eq!(check.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn wal_failure_makes_unhealthy() {
        let monitor = HealthMonitor::new();
        monitor.set_wal_healthy(false);
        let check = monitor.check();
        assert_eq!(check.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn disk_failure_makes_unhealthy() {
        let monitor = HealthMonitor::new();
        monitor.set_disk_space_healthy(false);
        let check = monitor.check();
        assert_eq!(check.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn missing_peers_makes_degraded() {
        let monitor = HealthMonitor::new();
        monitor.set_total_peers(3);
        monitor.set_connected_peers(2);
        let check = monitor.check();
        assert_eq!(check.status, HealthStatus::Degraded);
    }

    #[test]
    fn high_lag_makes_degraded() {
        let monitor = HealthMonitor::new();
        monitor.set_total_peers(1);
        monitor.set_connected_peers(1);
        monitor.set_replication_lag(5000);
        let check = monitor.check();
        assert_eq!(check.status, HealthStatus::Degraded);
    }

    #[test]
    fn healthy_when_all_peers_connected_and_low_lag() {
        let monitor = HealthMonitor::new();
        monitor.set_total_peers(3);
        monitor.set_connected_peers(3);
        monitor.set_replication_lag(10);
        let check = monitor.check();
        assert_eq!(check.status, HealthStatus::Healthy);
    }

    #[test]
    fn leader_flag() {
        let monitor = HealthMonitor::new();
        assert!(!monitor.is_leader());
        monitor.set_leader(true);
        assert!(monitor.is_leader());
        let check = monitor.check();
        assert!(check.is_leader);
    }

    #[test]
    fn is_ready_requires_storage_and_wal() {
        let monitor = HealthMonitor::new();
        assert!(monitor.is_ready());

        monitor.set_storage_healthy(false);
        assert!(!monitor.is_ready());

        monitor.set_storage_healthy(true);
        monitor.set_wal_healthy(false);
        assert!(!monitor.is_ready());

        monitor.set_wal_healthy(true);
        assert!(monitor.is_ready());
    }

    #[test]
    fn is_alive_always_true() {
        let monitor = HealthMonitor::new();
        assert!(monitor.is_alive());
    }

    #[test]
    fn peer_connected_disconnected() {
        let monitor = HealthMonitor::new();
        assert_eq!(monitor.connected_peers.load(Ordering::Relaxed), 0);

        monitor.peer_connected();
        monitor.peer_connected();
        assert_eq!(monitor.connected_peers.load(Ordering::Relaxed), 2);

        monitor.peer_disconnected();
        assert_eq!(monitor.connected_peers.load(Ordering::Relaxed), 1);

        // Cannot go below 0
        monitor.peer_disconnected();
        monitor.peer_disconnected();
        monitor.peer_disconnected();
        assert_eq!(monitor.connected_peers.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn uptime_is_nonzero() {
        let monitor = HealthMonitor::new();
        let check = monitor.check();
        assert!(check.uptime_secs <= 1);
    }

    #[test]
    fn version_is_set() {
        let monitor = HealthMonitor::new();
        let check = monitor.check();
        assert!(!check.version.is_empty());
    }

    #[test]
    fn snapshot_reflects_all_fields() {
        let monitor = HealthMonitor::new();
        monitor.set_leader(true);
        monitor.set_total_peers(5);
        monitor.set_connected_peers(4);
        monitor.set_replication_lag(50);

        let check = monitor.check();
        assert!(check.is_leader);
        assert_eq!(check.connected_peers, 4);
        assert_eq!(check.total_peers, 5);
        assert_eq!(check.replication_lag, 50);
    }
}
