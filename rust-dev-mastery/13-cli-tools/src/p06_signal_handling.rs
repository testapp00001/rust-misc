//! # Signal Handling
//!
//! CLI tools need to handle termination signals gracefully, cleaning up resources
//! and saving state before exiting. This lesson covers signal handling patterns,
//! graceful shutdown, and cleanup routines.
//!
//! ## Key Concepts
//! - SIGINT (Ctrl+C) and SIGTERM handling
//! - Graceful shutdown patterns
//! - Cleanup routines
//! - Timeout-based forced exit
//! - Signal-safe operations
//! - Atomic flag setting

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 1. Shutdown Signal
// ---------------------------------------------------------------------------

/// An atomic flag that signals graceful shutdown.
/// Multiple threads can check this flag safely.
#[derive(Debug)]
pub struct ShutdownSignal {
    shutdown_requested: AtomicBool,
    shutdown_count: AtomicU64,
    request_time: AtomicU64,
}

impl ShutdownSignal {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            shutdown_requested: AtomicBool::new(false),
            shutdown_count: AtomicU64::new(0),
            request_time: AtomicU64::new(0),
        })
    }

    /// Request shutdown.
    pub fn request_shutdown(&self) {
        let now = Instant::now().elapsed().as_millis() as u64;
        self.shutdown_requested.store(true, Ordering::SeqCst);
        self.shutdown_count.fetch_add(1, Ordering::SeqCst);
        self.request_time.store(now, Ordering::SeqCst);
    }

    /// Check if shutdown has been requested.
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    /// Get how many times shutdown was requested.
    pub fn shutdown_count(&self) -> u64 {
        self.shutdown_count.load(Ordering::SeqCst)
    }

    /// Reset the shutdown signal.
    pub fn reset(&self) {
        self.shutdown_requested.store(false, Ordering::SeqCst);
        self.shutdown_count.store(0, Ordering::SeqCst);
    }

    /// Create a future that resolves when shutdown is requested.
    /// (Simplified: returns true if already requested.)
    pub fn wait_for_shutdown(&self, timeout: Duration) -> bool {
        let start = Instant::now();
        while !self.is_shutdown_requested() {
            if start.elapsed() >= timeout {
                return false; // timeout
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        true
    }
}

// ---------------------------------------------------------------------------
// 2. Cleanup Registry
// ---------------------------------------------------------------------------

/// A cleanup action to run during shutdown.
pub struct CleanupAction {
    pub name: String,
    pub priority: u32, // lower = runs first
    pub action: CleanupFn,
}

impl std::fmt::Debug for CleanupAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CleanupAction")
            .field("name", &self.name)
            .field("priority", &self.priority)
            .finish()
    }
}

type CleanupFn = Box<dyn Fn() -> Result<(), String> + Send + Sync>;

impl CleanupAction {
    pub fn new(
        name: impl Into<String>,
        priority: u32,
        action: impl Fn() -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            priority,
            action: Box::new(action),
        }
    }
}

/// Registry of cleanup actions to run during shutdown.
pub struct CleanupRegistry {
    actions: Vec<CleanupAction>,
}

impl std::fmt::Debug for CleanupRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CleanupRegistry")
            .field("count", &self.actions.len())
            .finish()
    }
}

impl CleanupRegistry {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Register a cleanup action.
    pub fn register(&mut self, action: CleanupAction) {
        self.actions.push(action);
    }

    /// Run all cleanup actions in priority order.
    /// Returns a list of (name, error) for any that failed.
    pub fn run_all(&mut self) -> Vec<(String, String)> {
        self.actions.sort_by_key(|a| a.priority);

        let mut errors = Vec::new();
        for action in &self.actions {
            if let Err(e) = (action.action)() {
                errors.push((action.name.clone(), e));
            }
        }
        errors
    }

    pub fn action_count(&self) -> usize {
        self.actions.len()
    }
}

// ---------------------------------------------------------------------------
// 3. Graceful Shutdown Manager
// ---------------------------------------------------------------------------

/// Orchestrates graceful shutdown with timeout and forced exit.
#[derive(Debug)]
pub struct ShutdownManager {
    signal: Arc<ShutdownSignal>,
    cleanup: CleanupRegistry,
    _graceful_timeout: Duration,
    force_timeout: Duration,
}

#[derive(Debug, PartialEq)]
pub enum ShutdownResult {
    Graceful,
    Forced,
    Timeout,
}

impl ShutdownManager {
    pub fn new(
        signal: Arc<ShutdownSignal>,
        _graceful_timeout: Duration,
        force_timeout: Duration,
    ) -> Self {
        Self {
            signal,
            cleanup: CleanupRegistry::new(),
            _graceful_timeout,
            force_timeout,
        }
    }

    /// Register a cleanup action.
    pub fn register_cleanup(&mut self, action: CleanupAction) {
        self.cleanup.register(action);
    }

    /// Execute shutdown sequence.
    pub fn shutdown(&mut self) -> ShutdownResult {
        // If no shutdown requested, return immediately
        if !self.signal.is_shutdown_requested() {
            return ShutdownResult::Graceful;
        }

        // Check if we should force exit (double Ctrl+C)
        if self.signal.shutdown_count() >= 2 {
            return ShutdownResult::Forced;
        }

        // Run cleanup with timeout
        let cleanup_start = Instant::now();
        let errors = self.cleanup.run_all();
        let cleanup_duration = cleanup_start.elapsed();

        if !errors.is_empty() {
            eprintln!("Cleanup errors:");
            for (name, err) in &errors {
                eprintln!("  {name}: {err}");
            }
        }

        if cleanup_duration > self.force_timeout {
            return ShutdownResult::Timeout;
        }

        ShutdownResult::Graceful
    }

    pub fn signal(&self) -> &ShutdownSignal {
        &self.signal
    }
}

// ---------------------------------------------------------------------------
// 4. Signal Counter (for testing)
// ---------------------------------------------------------------------------

/// Counts signal occurrences for testing purposes.
#[derive(Debug)]
pub struct SignalCounter {
    sigint_count: AtomicU64,
    sigterm_count: AtomicU64,
    sighup_count: AtomicU64,
}

impl SignalCounter {
    pub fn new() -> Self {
        Self {
            sigint_count: AtomicU64::new(0),
            sigterm_count: AtomicU64::new(0),
            sighup_count: AtomicU64::new(0),
        }
    }

    pub fn record_sigint(&self) {
        self.sigint_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_sigterm(&self) {
        self.sigterm_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_sighup(&self) {
        self.sighup_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn sigint_count(&self) -> u64 {
        self.sigint_count.load(Ordering::SeqCst)
    }

    pub fn sigterm_count(&self) -> u64 {
        self.sigterm_count.load(Ordering::SeqCst)
    }

    pub fn sighup_count(&self) -> u64 {
        self.sighup_count.load(Ordering::SeqCst)
    }

    pub fn total_count(&self) -> u64 {
        self.sigint_count() + self.sigterm_count() + self.sighup_count()
    }

    pub fn reset(&self) {
        self.sigint_count.store(0, Ordering::SeqCst);
        self.sigterm_count.store(0, Ordering::SeqCst);
        self.sighup_count.store(0, Ordering::SeqCst);
    }
}

// ---------------------------------------------------------------------------
// 5. PID File Management
// ---------------------------------------------------------------------------

/// Manages a PID file for daemon processes.
#[derive(Debug)]
pub struct PidFile {
    path: String,
    pid: u32,
}

impl PidFile {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            pid: std::process::id(),
        }
    }

    /// Write the PID file.
    pub fn write(&self) -> Result<(), std::io::Error> {
        std::fs::write(&self.path, self.pid.to_string())
    }

    /// Read the PID from an existing PID file.
    pub fn read_pid(path: &str) -> Result<u32, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        content
            .trim()
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Check if a process with the given PID is running.
    pub fn is_process_running(pid: u32) -> bool {
        #[cfg(unix)]
        {
            // Sending signal 0 checks if process exists
            unsafe { libc_kill(pid as i32, 0) == 0 }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix, we can't easily check; assume running
            let _ = pid;
            true
        }
    }

    /// Remove the PID file.
    pub fn remove(&self) -> Result<(), std::io::Error> {
        if std::path::Path::new(&self.path).exists() {
            std::fs::remove_file(&self.path)
        } else {
            Ok(())
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }
}

#[cfg(unix)]
extern "C" {
    fn libc_kill(pid: i32, sig: i32) -> i32;
}

#[cfg(not(unix))]
unsafe fn libc_kill(_pid: i32, _sig: i32) -> i32 {
    -1
}

// ---------------------------------------------------------------------------
// 6. Signal Guard (RAII)
// ---------------------------------------------------------------------------

/// An RAII guard that runs cleanup when dropped.
pub struct SignalGuard {
    name: String,
    cleanup: Box<dyn Fn()>,
}

impl SignalGuard {
    pub fn new(name: impl Into<String>, cleanup: impl Fn() + 'static) -> Self {
        Self {
            name: name.into(),
            cleanup: Box::new(cleanup),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for SignalGuard {
    fn drop(&mut self) {
        (self.cleanup)();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[test]
    fn test_shutdown_signal_initial_state() {
        let signal = ShutdownSignal::new();
        assert!(!signal.is_shutdown_requested());
        assert_eq!(signal.shutdown_count(), 0);
    }

    #[test]
    fn test_shutdown_signal_request() {
        let signal = ShutdownSignal::new();
        signal.request_shutdown();
        assert!(signal.is_shutdown_requested());
        assert_eq!(signal.shutdown_count(), 1);
    }

    #[test]
    fn test_shutdown_signal_multiple_requests() {
        let signal = ShutdownSignal::new();
        signal.request_shutdown();
        signal.request_shutdown();
        signal.request_shutdown();
        assert_eq!(signal.shutdown_count(), 3);
    }

    #[test]
    fn test_shutdown_signal_reset() {
        let signal = ShutdownSignal::new();
        signal.request_shutdown();
        assert!(signal.is_shutdown_requested());

        signal.reset();
        assert!(!signal.is_shutdown_requested());
        assert_eq!(signal.shutdown_count(), 0);
    }

    #[test]
    fn test_shutdown_signal_wait_timeout() {
        let signal = ShutdownSignal::new();
        let result = signal.wait_for_shutdown(Duration::from_millis(50));
        assert!(!result); // should timeout
    }

    #[test]
    fn test_shutdown_signal_wait_immediate() {
        let signal = ShutdownSignal::new();
        signal.request_shutdown();
        let result = signal.wait_for_shutdown(Duration::from_secs(1));
        assert!(result); // should return immediately
    }

    #[test]
    fn test_cleanup_registry() {
        let mut registry = CleanupRegistry::new();
        assert_eq!(registry.action_count(), 0);

        registry.register(CleanupAction::new("action1", 1, || Ok(())));
        registry.register(CleanupAction::new("action2", 2, || Ok(())));
        assert_eq!(registry.action_count(), 2);

        let errors = registry.run_all();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_cleanup_registry_priority_order() {
        use std::sync::Mutex;

        let order = Arc::new(Mutex::new(Vec::new()));
        let order_clone = order.clone();

        let mut registry = CleanupRegistry::new();

        let o = order.clone();
        registry.register(CleanupAction::new("low-priority", 10, move || {
            o.lock().unwrap().push("low");
            Ok(())
        }));

        let o = order.clone();
        registry.register(CleanupAction::new("high-priority", 1, move || {
            o.lock().unwrap().push("high");
            Ok(())
        }));

        registry.run_all();
        let recorded = order_clone.lock().unwrap();
        assert_eq!(*recorded, vec!["high", "low"]);
    }

    #[test]
    fn test_cleanup_registry_with_errors() {
        let mut registry = CleanupRegistry::new();
        registry.register(CleanupAction::new("ok", 1, || Ok(())));
        registry.register(CleanupAction::new("fail", 2, || {
            Err("something went wrong".into())
        }));

        let errors = registry.run_all();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "fail");
    }

    #[test]
    fn test_shutdown_manager_no_request() {
        let signal = ShutdownSignal::new();
        let mut manager = ShutdownManager::new(signal, Duration::from_secs(5), Duration::from_secs(10));
        let result = manager.shutdown();
        assert_eq!(result, ShutdownResult::Graceful);
    }

    #[test]
    fn test_shutdown_manager_graceful() {
        let signal = ShutdownSignal::new();
        signal.request_shutdown();

        let mut manager = ShutdownManager::new(signal.clone(), Duration::from_secs(5), Duration::from_secs(10));
        manager.register_cleanup(CleanupAction::new("test", 1, || Ok(())));

        let result = manager.shutdown();
        assert_eq!(result, ShutdownResult::Graceful);
    }

    #[test]
    fn test_shutdown_manager_forced() {
        let signal = ShutdownSignal::new();
        signal.request_shutdown();
        signal.request_shutdown(); // double request = forced

        let mut manager = ShutdownManager::new(signal, Duration::from_secs(5), Duration::from_secs(10));
        let result = manager.shutdown();
        assert_eq!(result, ShutdownResult::Forced);
    }

    #[test]
    fn test_signal_counter() {
        let counter = SignalCounter::new();
        counter.record_sigint();
        counter.record_sigint();
        counter.record_sigterm();
        counter.record_sighup();

        assert_eq!(counter.sigint_count(), 2);
        assert_eq!(counter.sigterm_count(), 1);
        assert_eq!(counter.sighup_count(), 1);
        assert_eq!(counter.total_count(), 4);
    }

    #[test]
    fn test_signal_counter_reset() {
        let counter = SignalCounter::new();
        counter.record_sigint();
        counter.record_sigterm();

        counter.reset();
        assert_eq!(counter.total_count(), 0);
    }

    #[test]
    fn test_pid_file_write_and_read() {
        let path = "/tmp/test_pid_file_{}.pid";
        let path = path.replace("{}", &std::process::id().to_string());

        let pid_file = PidFile::new(&path);
        pid_file.write().unwrap();

        let read_pid = PidFile::read_pid(&path).unwrap();
        assert_eq!(read_pid, std::process::id());

        pid_file.remove().unwrap();
        assert!(!std::path::Path::new(&path).exists());
    }

    #[test]
    fn test_pid_file_remove_nonexistent() {
        let pid_file = PidFile::new("/tmp/nonexistent_pid_file_12345.pid");
        // Removing non-existent PID file should succeed
        assert!(pid_file.remove().is_ok());
    }

    #[test]
    fn test_signal_guard() {
        use std::sync::atomic::AtomicBool;

        let dropped = Arc::new(AtomicBool::new(false));
        let dropped_clone = dropped.clone();

        {
            let _guard = SignalGuard::new("test-guard", move || {
                dropped_clone.store(true, Ordering::SeqCst);
            });
            assert_eq!(_guard.name(), "test-guard");
            assert!(!dropped.load(Ordering::SeqCst));
        }

        // Guard dropped, cleanup should have run
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[test]
    fn test_signal_guard_multiple() {
        use std::sync::Mutex;

        let order = Arc::new(Mutex::new(Vec::new()));

        {
            let o = order.clone();
            let _g1 = SignalGuard::new("first", move || {
                o.lock().unwrap().push("first");
            });

            let o = order.clone();
            let _g2 = SignalGuard::new("second", move || {
                o.lock().unwrap().push("second");
            });
            // g2 dropped first (LIFO), then g1
        }

        let recorded = order.lock().unwrap();
        assert_eq!(*recorded, vec!["second", "first"]);
    }

    #[test]
    fn test_pid_file_current_pid() {
        let pid_file = PidFile::new("/tmp/test.pid");
        assert_eq!(pid_file.pid(), std::process::id());
    }
}
