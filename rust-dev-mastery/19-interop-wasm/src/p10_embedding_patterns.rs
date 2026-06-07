//! # Embedding Patterns
//!
//! Embedding Rust in other runtimes (Python, Node.js, Ruby, etc.) requires
//! careful mapping of threads, memory, and GC interactions. This module
//! covers patterns for safe embedding.
//!
//! ## Key Challenges:
//!
//! - **Thread mapping**: Rust threads vs. runtime threads
//! - **GC integration**: Preventing Rust data from being collected
//! - **Exception mapping**: Converting errors across boundaries
//! - **Memory ownership**: Who owns what, when

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Runtime adapter trait for embedding Rust in foreign runtimes.
pub trait RuntimeAdapter: Send + Sync {
    /// Get the runtime name.
    fn name(&self) -> &str;

    /// Convert a Rust error to a runtime exception.
    fn throw_error(&self, error: &str);

    /// Check if the runtime is shutting down.
    fn is_shutting_down(&self) -> bool;
}

/// Python runtime adapter.
pub struct PythonAdapter {
    shutting_down: std::sync::atomic::AtomicBool,
}

impl PythonAdapter {
    pub fn new() -> Self {
        Self {
            shutting_down: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl RuntimeAdapter for PythonAdapter {
    fn name(&self) -> &str {
        "Python"
    }

    fn throw_error(&self, _error: &str) {
        // In real PyO3 code, this would call PyErr_SetString
    }

    fn is_shutting_down(&self) -> bool {
        self.shutting_down
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Thread-safe handle for managing Rust objects in a foreign runtime.
pub struct EmbeddedHandle<T: Send> {
    inner: Arc<Mutex<T>>,
    id: u64,
}

impl<T: Send> EmbeddedHandle<T> {
    pub fn new(value: T, id: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(value)),
            id,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn with<R, F>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.inner.lock().map_err(|e| e.to_string())?;
        Ok(f(&guard))
    }

    pub fn with_mut<R, F>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        Ok(f(&mut guard))
    }
}

impl<T: Send> Clone for EmbeddedHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            id: self.id,
        }
    }
}

/// Handle registry for managing embedded objects.
pub struct HandleRegistry<T: Send> {
    handles: HashMap<u64, EmbeddedHandle<T>>,
    next_id: u64,
}

impl<T: Send> HandleRegistry<T> {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn insert(&mut self, value: T) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.handles.insert(id, EmbeddedHandle::new(value, id));
        id
    }

    pub fn get(&self, id: u64) -> Option<&EmbeddedHandle<T>> {
        self.handles.get(&id)
    }

    pub fn remove(&mut self, id: u64) -> Option<EmbeddedHandle<T>> {
        self.handles.remove(&id)
    }

    pub fn count(&self) -> usize {
        self.handles.len()
    }
}

/// Exception bridge for converting between Rust and runtime errors.
pub struct ExceptionBridge {
    mappings: HashMap<String, String>,
}

impl ExceptionBridge {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert("NotFound".into(), "ResourceNotFoundError".into());
        mappings.insert("Validation".into(), "ValidationError".into());
        mappings.insert("Unauthorized".into(), "AuthenticationError".into());
        Self { mappings }
    }

    /// Map a Rust error type to a runtime exception type.
    pub fn map_exception(&self, rust_error: &str) -> &str {
        self.mappings
            .get(rust_error)
            .map(|s| s.as_str())
            .unwrap_or("RuntimeError")
    }

    pub fn register_mapping(&mut self, rust_type: &str, runtime_type: &str) {
        self.mappings
            .insert(rust_type.into(), runtime_type.into());
    }
}

/// GC-safe reference holder.
/// Prevents the garbage collector from collecting Rust-owned data.
pub struct GcSafeRef<T> {
    data: Arc<T>,
    prevent_gc: bool,
}

impl<T> GcSafeRef<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(data),
            prevent_gc: true,
        }
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn prevent_gc(&self) -> bool {
        self.prevent_gc
    }

    pub fn allow_gc(&mut self) {
        self.prevent_gc = false;
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }
}

/// Embedding configuration.
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub runtime: String,
    pub max_threads: usize,
    pub memory_limit_mb: usize,
    pub enable_gc_integration: bool,
    pub exception_handling: ExceptionHandling,
}

#[derive(Debug, Clone)]
pub enum ExceptionHandling {
    Propagate,
    Convert,
    Swallow,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            runtime: "unknown".into(),
            max_threads: 4,
            memory_limit_mb: 256,
            enable_gc_integration: true,
            exception_handling: ExceptionHandling::Propagate,
        }
    }
}

/// Thread pool for embedded Rust operations.
pub struct EmbeddedThreadPool {
    workers: Vec<std::thread::JoinHandle<()>>,
    sender: std::sync::mpsc::Sender<Box<dyn FnOnce() + Send>>,
}

impl EmbeddedThreadPool {
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver) =
            std::sync::mpsc::channel::<Box<dyn FnOnce() + Send>>();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::new();
        for _ in 0..num_threads {
            let rx = Arc::clone(&receiver);
            workers.push(std::thread::spawn(move || {
                loop {
                    let task = {
                        let guard = rx.lock().unwrap();
                        guard.recv()
                    };
                    match task {
                        Ok(task) => task(),
                        Err(_) => break,
                    }
                }
            }));
        }

        Self { workers, sender }
    }

    pub fn submit<F: FnOnce() + Send + 'static>(&self, task: F) {
        let _ = self.sender.send(Box::new(task));
    }

    pub fn shutdown(self) {
        drop(self.sender);
        for worker in self.workers {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_adapter() {
        let adapter = PythonAdapter::new();
        assert_eq!(adapter.name(), "Python");
        assert!(!adapter.is_shutting_down());
    }

    #[test]
    fn test_embedded_handle() {
        let handle = EmbeddedHandle::new(String::from("hello"), 1);
        assert_eq!(handle.id(), 1);

        let len = handle.with(|s| s.len()).unwrap();
        assert_eq!(len, 5);
    }

    #[test]
    fn test_embedded_handle_mut() {
        let handle = EmbeddedHandle::new(Vec::<i32>::new(), 1);
        handle.with_mut(|v| v.push(42)).unwrap();

        let val = handle.with(|v| v[0]).unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn test_embedded_handle_clone() {
        let handle1 = EmbeddedHandle::new(42u32, 1);
        let handle2 = handle1.clone();

        handle1.with_mut(|v| *v = 100).unwrap();
        let val = handle2.with(|v| *v).unwrap();
        assert_eq!(val, 100);
    }

    #[test]
    fn test_handle_registry() {
        let mut registry = HandleRegistry::new();
        let id1 = registry.insert("data1".to_string());
        let id2 = registry.insert("data2".to_string());

        assert_eq!(registry.count(), 2);
        assert!(registry.get(id1).is_some());
        assert!(registry.get(id2).is_some());

        registry.remove(id1);
        assert_eq!(registry.count(), 1);
        assert!(registry.get(id1).is_none());
    }

    #[test]
    fn test_exception_bridge() {
        let bridge = ExceptionBridge::new();
        assert_eq!(bridge.map_exception("NotFound"), "ResourceNotFoundError");
        assert_eq!(bridge.map_exception("Unknown"), "RuntimeError");
    }

    #[test]
    fn test_exception_bridge_register() {
        let mut bridge = ExceptionBridge::new();
        bridge.register_mapping("CustomError", "JavaScriptError");
        assert_eq!(bridge.map_exception("CustomError"), "JavaScriptError");
    }

    #[test]
    fn test_gc_safe_ref() {
        let gc_ref = GcSafeRef::new(42);
        assert_eq!(*gc_ref.get(), 42);
        assert!(gc_ref.prevent_gc());
        assert_eq!(gc_ref.strong_count(), 1);
    }

    #[test]
    fn test_gc_safe_ref_allow_gc() {
        let mut gc_ref = GcSafeRef::new(String::from("data"));
        assert!(gc_ref.prevent_gc());
        gc_ref.allow_gc();
        assert!(!gc_ref.prevent_gc());
    }

    #[test]
    fn test_embedding_config_default() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.max_threads, 4);
        assert!(config.enable_gc_integration);
    }

    #[test]
    fn test_embedding_config_clone() {
        let config = EmbeddingConfig {
            runtime: "python".into(),
            ..Default::default()
        };
        let cloned = config.clone();
        assert_eq!(cloned.runtime, "python");
    }

    #[test]
    fn test_embedded_thread_pool() {
        use std::sync::atomic::{AtomicU64, Ordering};

        let counter = Arc::new(AtomicU64::new(0));
        let pool = EmbeddedThreadPool::new(4);

        for _ in 0..100 {
            let counter = Arc::clone(&counter);
            pool.submit(move || {
                counter.fetch_add(1, Ordering::Relaxed);
            });
        }

        // Wait for tasks to complete
        std::thread::sleep(std::time::Duration::from_millis(100));
        pool.shutdown();

        assert_eq!(counter.load(Ordering::Relaxed), 100);
    }
}
