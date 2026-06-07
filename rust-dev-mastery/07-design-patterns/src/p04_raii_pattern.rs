//! # RAII Pattern (Resource Acquisition Is Initialization)
//!
//! RAII ties resource lifetime to object lifetime. When an object is created,
//! it acquires a resource; when it's dropped, the resource is released. Rust's
//! ownership system makes RAII natural and reliable.
//!
//! ## Key Concepts
//! - **Drop trait**: Called automatically when an object goes out of scope
//! - **Scope guards**: Run cleanup code at scope exit
//! - **RAII wrappers**: Manage resources (files, connections, locks) safely
//! - **Guard types**: Ensure invariants are maintained

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A connection guard that returns the connection to the pool on drop.
pub struct ConnectionGuard {
    connection: Option<Connection>,
    pool: Arc<Mutex<Vec<Connection>>>,
}

#[derive(Debug)]
pub struct Connection {
    pub id: u64,
    pub host: String,
    pub created_at: Instant,
}

impl Connection {
    pub fn new(id: u64, host: impl Into<String>) -> Self {
        Connection {
            id,
            host: host.into(),
            created_at: Instant::now(),
        }
    }

    pub fn execute(&self, query: &str) -> Result<Vec<String>, String> {
        Ok(vec![format!("{}: {}", self.host, query)])
    }
}

pub struct ConnectionPool {
    connections: Arc<Mutex<Vec<Connection>>>,
    max_size: usize,
    next_id: u64,
}

impl ConnectionPool {
    pub fn new(max_size: usize) -> Self {
        ConnectionPool {
            connections: Arc::new(Mutex::new(Vec::new())),
            max_size,
            next_id: 1,
        }
    }

    pub fn acquire(&mut self) -> ConnectionGuard {
        let conn = {
            let mut pool = self.connections.lock().unwrap();
            pool.pop().unwrap_or_else(|| {
                let id = self.next_id;
                self.next_id += 1;
                Connection::new(id, "localhost:5432")
            })
        };

        ConnectionGuard {
            connection: Some(conn),
            pool: self.connections.clone(),
        }
    }

    pub fn available(&self) -> usize {
        self.connections.lock().unwrap().len()
    }
}

impl std::ops::Deref for ConnectionGuard {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        self.connection.as_ref().unwrap()
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            self.pool.lock().unwrap().push(conn);
        }
    }
}

/// A scope guard that runs a closure on drop.
pub struct ScopeGuard<F: FnOnce()> {
    cleanup: Option<F>,
}

impl<F: FnOnce()> ScopeGuard<F> {
    pub fn new(cleanup: F) -> Self {
        ScopeGuard {
            cleanup: Some(cleanup),
        }
    }

    /// Disarm the guard so it doesn't run the cleanup.
    pub fn disarm(mut self) {
        self.cleanup = None;
    }
}

impl<F: FnOnce()> Drop for ScopeGuard<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

/// A timer guard that measures the duration of a scope.
pub struct TimerGuard {
    label: String,
    start: Instant,
    log: Arc<Mutex<Vec<(String, Duration)>>>,
}

impl TimerGuard {
    pub fn new(label: impl Into<String>, log: Arc<Mutex<Vec<(String, Duration)>>>) -> Self {
        TimerGuard {
            label: label.into(),
            start: Instant::now(),
            log,
        }
    }
}

impl Drop for TimerGuard {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        self.log.lock().unwrap().push((self.label.clone(), elapsed));
    }
}

/// A temporary directory that is deleted when the guard is dropped.
pub struct TempDir {
    path: String,
    cleanup: bool,
}

impl TempDir {
    pub fn new(prefix: &str) -> Self {
        TempDir {
            path: format!("/tmp/{prefix}_{}", std::process::id()),
            cleanup: true,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    /// Keep the directory even after the guard is dropped.
    pub fn keep(mut self) -> String {
        self.cleanup = false;
        self.path.clone()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if self.cleanup {
            // In real code: std::fs::remove_dir_all(&self.path)
        }
    }
}

/// A read lock guard for a shared data structure.
pub struct ReadGuard<'a, T> {
    data: &'a T,
    _lock: std::sync::MutexGuard<'a, ()>,
}

/// A write lock guard that tracks modifications.
pub struct WriteGuard<'a, T> {
    data: &'a mut T,
    _lock: std::sync::MutexGuard<'a, ()>,
    dirty: bool,
}

impl<'a, T> WriteGuard<'a, T> {
    pub fn get(&self) -> &T {
        self.data
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.dirty = true;
        self.data
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

/// A reference-counted resource handle.
/// The resource is shared among all clones and cleaned up when the last handle drops.
pub struct ResourceHandle<T> {
    resource: Arc<Mutex<T>>,
    cleanup: Arc<dyn Fn(&mut T) + Send + Sync>,
}

impl<T> Clone for ResourceHandle<T> {
    fn clone(&self) -> Self {
        ResourceHandle {
            resource: self.resource.clone(),
            cleanup: self.cleanup.clone(),
        }
    }
}

impl<T> ResourceHandle<T> {
    pub fn new(resource: T, cleanup: impl Fn(&mut T) + Send + Sync + 'static) -> Self {
        ResourceHandle {
            resource: Arc::new(Mutex::new(resource)),
            cleanup: Arc::new(cleanup),
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.resource.lock().unwrap();
        f(&guard)
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.resource.lock().unwrap();
        f(&mut guard)
    }
}

impl<T> Drop for ResourceHandle<T> {
    fn drop(&mut self) {
        // Only run cleanup if this is the last reference
        if Arc::strong_count(&self.resource) == 1 {
            if let Ok(mut resource) = self.resource.lock() {
                (self.cleanup)(&mut resource);
            }
        }
    }
}

/// A stack of cleanup actions that execute in reverse order on drop.
pub struct CleanupStack {
    actions: Vec<Box<dyn FnOnce()>>,
}

impl CleanupStack {
    pub fn new() -> Self {
        CleanupStack {
            actions: Vec::new(),
        }
    }

    pub fn push<F: FnOnce() + 'static>(&mut self, action: F) {
        self.actions.push(Box::new(action));
    }
}

impl Drop for CleanupStack {
    fn drop(&mut self) {
        // Execute in reverse order (LIFO)
        for action in self.actions.drain(..).rev() {
            action();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool_acquire_release() {
        let mut pool = ConnectionPool::new(5);

        {
            let conn = pool.acquire();
            assert_eq!(conn.id, 1);
            assert_eq!(pool.available(), 0);
            // Connection is returned to pool on drop
        }

        assert_eq!(pool.available(), 1);
    }

    #[test]
    fn test_connection_guard_deref() {
        let mut pool = ConnectionPool::new(5);
        let conn = pool.acquire();
        let result = conn.execute("SELECT 1").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_scope_guard_runs() {
        let executed = Arc::new(Mutex::new(false));
        let flag = executed.clone();

        {
            let _guard = ScopeGuard::new(move || {
                *flag.lock().unwrap() = true;
            });
        }

        assert!(*executed.lock().unwrap());
    }

    #[test]
    fn test_scope_guard_disarm() {
        let executed = Arc::new(Mutex::new(false));
        let flag = executed.clone();

        {
            let guard = ScopeGuard::new(move || {
                *flag.lock().unwrap() = true;
            });
            guard.disarm();
        }

        assert!(!*executed.lock().unwrap());
    }

    #[test]
    fn test_timer_guard() {
        let log = Arc::new(Mutex::new(Vec::new()));

        {
            let _timer = TimerGuard::new("test_op", log.clone());
            std::thread::sleep(Duration::from_millis(10));
        }

        let entries = log.lock().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "test_op");
        assert!(entries[0].1 >= Duration::from_millis(8));
    }

    #[test]
    fn test_temp_dir() {
        let path;
        {
            let tmp = TempDir::new("test");
            path = tmp.path().to_string();
            assert!(path.contains("test"));
        }
        // TempDir dropped, would be cleaned up in real code
        let _ = path;
    }

    #[test]
    fn test_temp_dir_keep() {
        let path;
        {
            let tmp = TempDir::new("test");
            path = tmp.keep();
        }
        // Directory preserved because we called keep()
        let _ = path;
    }

    #[test]
    fn test_resource_handle() {
        let handle = ResourceHandle::new(vec![1, 2, 3], |v: &mut Vec<i32>| {
            v.clear();
        });

        let len = handle.with(|v| v.len());
        assert_eq!(len, 3);

        let handle2 = handle.clone();
        assert_eq!(handle2.with(|v| v.len()), 3);
    }

    #[test]
    fn test_cleanup_stack() {
        let log = Arc::new(Mutex::new(Vec::new()));

        {
            let mut stack = CleanupStack::new();

            let log1 = log.clone();
            stack.push(move || log1.lock().unwrap().push("first"));

            let log2 = log.clone();
            stack.push(move || log2.lock().unwrap().push("second"));

            let log3 = log.clone();
            stack.push(move || log3.lock().unwrap().push("third"));
        }

        // Should execute in reverse order
        let entries = log.lock().unwrap();
        assert_eq!(*entries, vec!["third", "second", "first"]);
    }

    #[test]
    fn test_multiple_timer_guards() {
        let log = Arc::new(Mutex::new(Vec::new()));

        {
            let _t1 = TimerGuard::new("step1", log.clone());
            std::thread::sleep(Duration::from_millis(5));

            let _t2 = TimerGuard::new("step2", log.clone());
            std::thread::sleep(Duration::from_millis(5));
        }

        let entries = log.lock().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "step2"); // Inner drops first
        assert_eq!(entries[1].0, "step1");
    }
}
