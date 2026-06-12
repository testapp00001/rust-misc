//! # Solution 06: Graceful Shutdown
//!
//! Complete implementation of graceful shutdown handling that drains in-flight
//! requests and cleans up resources before process exit.

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
pub struct GracefulShutdown {
    shutdown_signaled: Arc<AtomicBool>,
    in_flight: Arc<AtomicUsize>,
    notify: Arc<tokio::sync::Notify>,
}

impl GracefulShutdown {
    /// Create a new graceful shutdown coordinator.
    pub fn new() -> Self {
        Self {
            shutdown_signaled: Arc::new(AtomicBool::new(false)),
            in_flight: Arc::new(AtomicUsize::new(0)),
            notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Get a handle for tracking in-flight requests.
    pub fn in_flight_guard(&self) -> InFlightGuard {
        InFlightGuard::new(self.in_flight.clone())
    }

    /// Register a shutdown signal handler.
    ///
    /// Spawns a tokio task that listens for Ctrl+C (SIGINT).
    pub fn register_shutdown_signal(&self) {
        let signaled = self.shutdown_signaled.clone();
        let notify = self.notify.clone();

        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                signaled.store(true, Ordering::SeqCst);
                notify.notify_waiters();
            }
        });
    }

    /// Manually trigger a shutdown.
    pub fn trigger_shutdown(&self) {
        self.shutdown_signaled.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    /// Wait until a shutdown signal is received.
    pub async fn wait_for_shutdown(&self) {
        if self.shutdown_signaled.load(Ordering::SeqCst) {
            return;
        }
        self.notify.notified().await;
    }

    /// Drain in-flight requests, waiting up to `timeout` for them to complete.
    pub async fn drain(&self, timeout: Duration) -> Result<(), ShutdownError> {
        let result = tokio::time::timeout(timeout, async {
            loop {
                let count = self.in_flight.load(Ordering::SeqCst);
                if count == 0 {
                    return;
                }
                // Poll every 1ms for in-flight requests to complete
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await;

        match result {
            Ok(()) => Ok(()),
            _ => {
                let remaining = self.in_flight.load(Ordering::SeqCst);
                Err(ShutdownError::DrainTimeout(remaining))
            }
        }
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

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard that tracks an in-flight request.
pub struct InFlightGuard {
    in_flight: Arc<AtomicUsize>,
}

impl InFlightGuard {
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

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            s.trigger_shutdown();
        });

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

        let guard = shutdown.in_flight_guard();

        let handle = tokio::spawn(async move {
            let _g = guard;
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        let result = s.drain(Duration::from_millis(500)).await;
        assert!(result.is_ok());
        assert_eq!(s.in_flight_count(), 0);

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_drain_timeout() {
        let shutdown = GracefulShutdown::new();
        let _guard = shutdown.in_flight_guard();

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

        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(s.in_flight_count(), 0);
    }
}
