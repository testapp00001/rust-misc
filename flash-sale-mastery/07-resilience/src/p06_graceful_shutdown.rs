//! # Exercise 06: Graceful Shutdown
//!
//! ## Learning Objective
//!
//! Implement graceful shutdown handling that drains in-flight requests and cleans
//! up resources before the process exits. A hard kill during a flash sale can leave
//! orders in an inconsistent state -- payment charged but order not recorded, or
//! stock decremented but order not created.
//!
//! ## Flash Sale Context
//!
//! During a flash sale, a deployment or scaling event might require shutting down
//! a server. If the server is killed mid-request, users see errors and data can
//! become inconsistent. A graceful shutdown: (1) stops accepting new requests,
//! (2) waits for in-flight requests to complete (with a timeout), (3) closes
//! database and Redis connections cleanly, (4) reports final metrics.
//!
//! ## Instructions
//!
//! 1. Implement `GracefulShutdown` struct that coordinates shutdown
//! 2. Implement `register_shutdown_signal()` that listens for SIGTERM/SIGINT
//! 3. Implement `wait_for_shutdown()` that blocks until shutdown is triggered
//! 4. Implement `drain()` that waits for in-flight operations to complete
//! 5. Use tokio's `CancellationToken` or `watch` channel for signal propagation
//!
//! ## Hints
//!
//! - Use `tokio::signal::ctrl_c()` for SIGINT handling
//! - Use `tokio::sync::watch` or `tokio_util::sync::CancellationToken` for signaling
//! - Track in-flight request count with `AtomicUsize`
//! - Use `tokio::select!` to wait for either all requests done or a timeout

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during graceful shutdown.
#[derive(Debug, Error)]
pub enum ShutdownError {
    #[error("shutdown timed out with {0} requests still in flight")]
    DrainTimeout(usize),
    #[error("shutdown already in progress")]
    AlreadyShuttingDown,
}

/// Coordinates graceful shutdown of a service.
///
/// Tracks in-flight requests and provides mechanisms to signal shutdown,
/// wait for the signal, and drain remaining work before exit.
pub struct GracefulShutdown {
    /// Whether a shutdown has been signaled
    shutdown_signaled: Arc<AtomicBool>,
    /// Number of currently in-flight requests
    in_flight: Arc<AtomicUsize>,
    // TODO: Add a tokio::sync::watch or similar channel for shutdown notification
}

impl GracefulShutdown {
    /// Create a new graceful shutdown coordinator.
    pub fn new() -> Self {
        // TODO: Initialize the shutdown coordinator
        todo!("Implement GracefulShutdown::new")
    }

    /// Get a handle for tracking in-flight requests.
    ///
    /// Returns an `InFlightGuard` that increments the count on creation
    /// and decrements on drop. Each request should hold a guard for its
    /// entire lifetime.
    pub fn in_flight_guard(&self) -> InFlightGuard {
        // TODO: Create and return an InFlightGuard
        todo!("Implement GracefulShutdown::in_flight_guard")
    }

    /// Register a shutdown signal handler.
    ///
    /// Spawns a tokio task that listens for Ctrl+C (SIGINT) and triggers
    /// the shutdown sequence. This should be called once at startup.
    pub fn register_shutdown_signal(&self) {
        // TODO: Spawn a task that:
        //   1. Waits for tokio::signal::ctrl_c()
        //   2. Sets the shutdown_signaled flag
        //   3. Notifies via the watch channel
        todo!("Implement GracefulShutdown::register_shutdown_signal")
    }

    /// Manually trigger a shutdown.
    ///
    /// Useful for testing or programmatic shutdown (e.g., from a health check).
    pub fn trigger_shutdown(&self) {
        // TODO: Set the shutdown flag and notify
        todo!("Implement GracefulShutdown::trigger_shutdown")
    }

    /// Wait until a shutdown signal is received.
    ///
    /// Returns immediately if shutdown was already signaled.
    pub async fn wait_for_shutdown(&self) {
        // TODO: Wait for the shutdown signal
        // If using a watch channel, wait for the value to change
        // If using a simple flag, poll with a short sleep interval
        todo!("Implement GracefulShutdown::wait_for_shutdown")
    }

    /// Drain in-flight requests, waiting up to `timeout` for them to complete.
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait for in-flight requests
    ///
    /// # Returns
    /// `Ok(())` if all requests completed, or `DrainTimeout` with the count
    /// of requests still in flight
    pub async fn drain(&self, timeout: Duration) -> Result<(), ShutdownError> {
        // TODO: Implement drain logic:
        //   1. Wait for in_flight count to reach 0, OR timeout
        //   2. Use tokio::time::timeout or tokio::select!
        //   3. If timeout, return DrainTimeout with remaining count
        todo!("Implement GracefulShutdown::drain")
    }

    /// Check if shutdown has been signaled.
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_signaled.load(Ordering::SeqCst)
    }

    /// Get the current number of in-flight requests.
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.load(Ordering::SeqCst)
    }
}

/// RAII guard that tracks an in-flight request.
///
/// Increments the in-flight counter on creation, decrements on drop.
pub struct InFlightGuard {
    in_flight: Arc<AtomicUsize>,
}

impl InFlightGuard {
    /// Create a new guard, incrementing the in-flight counter.
    fn new(in_flight: Arc<AtomicUsize>) -> Self {
        in_flight.fetch_add(1, Ordering::SeqCst);
        Self { in_flight }
    }
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_signal_received() {
        let shutdown = GracefulShutdown::new();
        assert!(!shutdown.is_shutting_down());

        shutdown.trigger_shutdown();
        assert!(shutdown.is_shutting_down());
    }

    #[tokio::test]
    async fn test_wait_for_shutdown() {
        let shutdown = Arc::new(GracefulShutdown::new());
        let s = shutdown.clone();

        // Trigger shutdown after a short delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            s.trigger_shutdown();
        });

        // Should unblock when shutdown is triggered
        tokio::time::timeout(Duration::from_millis(200), shutdown.wait_for_shutdown())
            .await
            .expect("wait_for_shutdown should complete");
    }

    #[tokio::test]
    async fn test_in_flight_guard_tracks_count() {
        let shutdown = GracefulShutdown::new();
        assert_eq!(shutdown.in_flight_count(), 0);

        let guard1 = shutdown.in_flight_guard();
        assert_eq!(shutdown.in_flight_count(), 1);

        let guard2 = shutdown.in_flight_guard();
        assert_eq!(shutdown.in_flight_count(), 2);

        drop(guard1);
        assert_eq!(shutdown.in_flight_count(), 1);

        drop(guard2);
        assert_eq!(shutdown.in_flight_count(), 0);
    }

    #[tokio::test]
    async fn test_drain_completes_when_empty() {
        let shutdown = GracefulShutdown::new();
        let result = shutdown.drain(Duration::from_millis(100)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_drain_waits_for_in_flight() {
        let shutdown = Arc::new(GracefulShutdown::new());
        let s = shutdown.clone();

        // Simulate an in-flight request
        let guard = shutdown.in_flight_guard();

        // Spawn a task that holds the guard and finishes after 50ms
        let handle = tokio::spawn(async move {
            let _g = guard;
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        // Drain should wait for the request to complete
        let result = s.drain(Duration::from_millis(500)).await;
        assert!(result.is_ok());
        assert_eq!(s.in_flight_count(), 0);

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_drain_timeout() {
        let shutdown = GracefulShutdown::new();
        let _guard = shutdown.in_flight_guard(); // Hold a guard indefinitely

        let result = shutdown.drain(Duration::from_millis(30)).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ShutdownError::DrainTimeout(count) => assert_eq!(count, 1),
            other => panic!("expected DrainTimeout, got: {other}"),
        }
    }

    #[tokio::test]
    async fn test_in_flight_guards_drop_on_completion() {
        let shutdown = Arc::new(GracefulShutdown::new());
        let s = shutdown.clone();

        let mut handles = Vec::new();
        for _ in 0..5 {
            let s = s.clone();
            handles.push(tokio::spawn(async move {
                let _guard = s.in_flight_guard();
                tokio::time::sleep(Duration::from_millis(20)).await;
            }));
        }

        // Wait for all tasks to complete
        for h in handles {
            h.await.unwrap();
        }

        // All guards should be dropped
        assert_eq!(s.in_flight_count(), 0);
    }
}
