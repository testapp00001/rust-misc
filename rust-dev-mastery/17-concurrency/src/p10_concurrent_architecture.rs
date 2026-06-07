//! # Concurrent Architecture Patterns
//!
//! Designing concurrent applications requires choosing the right architecture
//! for the workload. This module covers thread-per-core, shared-nothing, event
//! loops, and hybrid architectures.
//!
//! ## Architecture Comparison:
//!
//! | Architecture | Best For | Contention | Complexity |
//! |-------------|----------|------------|------------|
//! | Thread-per-Core | CPU-bound, partitioned data | None | Low |
//! | Shared-Nothing | Message-passing systems | None | Medium |
//! | Event Loop | I/O-bound, many connections | Low | Medium |
//! | Work Stealing | Mixed workloads | Low | Low (with Rayon) |
//! | Actor Model | Complex state machines | None | High |

use crossbeam::channel::{self, Receiver, Sender};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Thread-per-core architecture: each core owns a partition of data.
/// No shared state between cores, communication via channels.
pub struct ThreadPerCore<T: Send + 'static> {
    cores: Vec<CoreHandle<T>>,
    num_cores: usize,
}

pub struct CoreHandle<T: Send> {
    core_id: usize,
    sender: Sender<CoreMessage<T>>,
    handle: Option<thread::JoinHandle<CoreStats>>,
}

enum CoreMessage<T> {
    Process(T),
    Query(Sender<CoreStats>),
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct CoreStats {
    pub core_id: usize,
    pub items_processed: u64,
    pub processing_time_us: u64,
}

impl<T: Send + 'static> ThreadPerCore<T> {
    pub fn new<F>(num_cores: usize, processor: F) -> Self
    where
        F: Fn(usize, T) + Send + Sync + 'static,
    {
        let processor = Arc::new(processor);
        let mut cores = Vec::with_capacity(num_cores);

        for core_id in 0..num_cores {
            let (tx, rx) = channel::unbounded();
            let processor = Arc::clone(&processor);

            let handle = thread::Builder::new()
                .name(format!("core-{}", core_id))
                .spawn(move || {
                    let mut items_processed = 0u64;
                    let start = Instant::now();

                    for msg in rx.iter() {
                        match msg {
                            CoreMessage::Process(item) => {
                                processor(core_id, item);
                                items_processed += 1;
                            }
                            CoreMessage::Query(resp_tx) => {
                                let _ = resp_tx.send(CoreStats {
                                    core_id,
                                    items_processed,
                                    processing_time_us: start.elapsed().as_micros() as u64,
                                });
                            }
                            CoreMessage::Shutdown => break,
                        }
                    }

                    CoreStats {
                        core_id,
                        items_processed,
                        processing_time_us: start.elapsed().as_micros() as u64,
                    }
                })
                .expect("Failed to spawn core thread");

            cores.push(CoreHandle {
                core_id,
                sender: tx,
                handle: Some(handle),
            });
        }

        Self { cores, num_cores }
    }

    /// Route an item to a specific core based on a partition function.
    pub fn send_to_core(&self, core_id: usize, item: T) {
        if core_id < self.num_cores {
            let _ = self.cores[core_id].sender.send(CoreMessage::Process(item));
        }
    }

    /// Distribute items round-robin across cores.
    pub fn distribute_round_robin(&self, items: Vec<T>) {
        for (i, item) in items.into_iter().enumerate() {
            let core = i % self.num_cores;
            let _ = self.cores[core].sender.send(CoreMessage::Process(item));
        }
    }

    /// Get statistics from all cores.
    pub fn stats(&self) -> Vec<CoreStats> {
        let mut stats = Vec::new();
        for core in &self.cores {
            let (tx, rx) = channel::bounded(1);
            let _ = core.sender.send(CoreMessage::Query(tx));
            if let Ok(stat) = rx.recv_timeout(Duration::from_millis(100)) {
                stats.push(stat);
            }
        }
        stats
    }

    pub fn shutdown(self) {
        for core in &self.cores {
            let _ = core.sender.send(CoreMessage::Shutdown);
        }
    }
}

/// Shared-nothing architecture with message passing between components.
pub struct SharedNothingSystem {
    components: HashMap<String, ComponentHandle>,
}

struct ComponentHandle {
    sender: Sender<ComponentMessage>,
    handle: Option<thread::JoinHandle<()>>,
}

enum ComponentMessage {
    Data(Vec<u8>),
    Request {
        data: Vec<u8>,
        reply_to: Sender<Vec<u8>>,
    },
    Shutdown,
}

impl SharedNothingSystem {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add_component<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(ComponentMessage) + Send + 'static,
    {
        let (tx, rx) = channel::unbounded();
        let handle = thread::spawn(move || {
            for msg in rx.iter() {
                handler(msg);
            }
        });

        self.components.insert(
            name.to_string(),
            ComponentHandle {
                sender: tx,
                handle: Some(handle),
            },
        );
    }

    pub fn send_to(&self, component: &str, data: Vec<u8>) {
        if let Some(handle) = self.components.get(component) {
            let _ = handle.sender.send(ComponentMessage::Data(data));
        }
    }

    pub fn request_from(
        &self,
        component: &str,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let handle = self
            .components
            .get(component)
            .ok_or_else(|| format!("Component '{}' not found", component))?;

        let (reply_tx, reply_rx) = channel::bounded(1);
        handle
            .sender
            .send(ComponentMessage::Request {
                data,
                reply_to: reply_tx,
            })
            .map_err(|_| "Component closed".to_string())?;

        reply_rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "Request timed out".to_string())
    }

    pub fn shutdown(self) {
        for (_, component) in self.components {
            let ComponentHandle { sender, handle } = component;
            let _ = sender.send(ComponentMessage::Shutdown);
            // Drop the sender before joining to close the channel and let the thread exit
            drop(sender);
            if let Some(handle) = handle {
                let _ = handle.join();
            }
        }
    }
}

/// Event loop for processing events sequentially within a thread.
pub struct EventLoop {
    event_tx: Sender<Box<dyn FnOnce() + Send>>,
    handle: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl EventLoop {
    pub fn new(name: &str) -> Self {
        let (tx, rx) = channel::unbounded::<Box<dyn FnOnce() + Send>>();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        let name = name.to_string();

        let handle = thread::Builder::new()
            .name(name)
            .spawn(move || {
                while running_clone.load(Ordering::Relaxed) {
                    match rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(task) => task(),
                        Err(channel::RecvTimeoutError::Timeout) => continue,
                        Err(channel::RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .expect("Failed to spawn event loop");

        Self {
            event_tx: tx,
            handle: Some(handle),
            running,
        }
    }

    /// Schedule a task on the event loop.
    pub fn schedule<F: FnOnce() + Send + 'static>(&self, task: F) {
        let _ = self.event_tx.send(Box::new(task));
    }

    /// Stop the event loop.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Pipeline architecture: chain of processing stages.
pub struct PipelineStage<I: Send + 'static, O: Send + 'static> {
    name: String,
    input: Receiver<I>,
    output: Sender<O>,
    process_fn: Box<dyn Fn(I) -> O + Send>,
}

impl<I: Send + 'static, O: Send + 'static> PipelineStage<I, O> {
    pub fn new<F>(name: &str, input: Receiver<I>, output: Sender<O>, process: F) -> Self
    where
        F: Fn(I) -> O + Send + 'static,
    {
        Self {
            name: name.into(),
            input,
            output,
            process_fn: Box::new(process),
        }
    }

    pub fn run(self) {
        for item in self.input.iter() {
            let result = (self.process_fn)(item);
            if self.output.send(result).is_err() {
                break;
            }
        }
    }
}

/// Fan-out/fan-in parallel processing pattern.
pub struct FanOutFanIn {
    num_workers: usize,
}

impl FanOutFanIn {
    pub fn new(num_workers: usize) -> Self {
        Self { num_workers }
    }

    /// Process items in parallel and collect results.
    pub fn process<T, R, F>(&self, items: Vec<T>, processor: F) -> Vec<R>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        let (work_tx, work_rx) = channel::unbounded();
        let (result_tx, result_rx) = channel::unbounded();
        let processor = Arc::new(processor);

        // Spawn workers
        for _ in 0..self.num_workers {
            let rx = work_rx.clone();
            let tx = result_tx.clone();
            let processor = Arc::clone(&processor);

            thread::spawn(move || {
                for item in rx.iter() {
                    let result = processor(item);
                    let _ = tx.send(result);
                }
            });
        }

        // Send work
        drop(result_tx); // Close result channel when workers finish
        for item in items {
            let _ = work_tx.send(item);
        }
        drop(work_tx); // Signal no more work

        // Collect results
        result_rx.iter().collect()
    }
}

/// Thread-safe metric collector for concurrent systems.
pub struct ConcurrentMetrics {
    counters: dashmap::DashMap<String, AtomicU64>,
    histograms: dashmap::DashMap<String, Vec<AtomicU64>>,
}

impl ConcurrentMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            counters: dashmap::DashMap::new(),
            histograms: dashmap::DashMap::new(),
        })
    }

    pub fn increment_counter(&self, name: &str) {
        self.counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters
            .get(name)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    pub fn record_histogram(&self, name: &str, value: u64) {
        self.histograms
            .entry(name.to_string())
            .or_insert_with(|| (0..10).map(|_| AtomicU64::new(0)).collect());
        if let Some(histogram) = self.histograms.get(name) {
            let bucket = (value as usize).min(histogram.len() - 1);
            histogram[bucket].fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_per_core_basic() {
        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = Arc::clone(&counter);

        let system = ThreadPerCore::new(4, move |_, _: u64| {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        let items: Vec<u64> = (0..100).collect();
        system.distribute_round_robin(items);
        thread::sleep(Duration::from_millis(50));

        assert_eq!(counter.load(Ordering::Relaxed), 100);
        system.shutdown();
    }

    #[test]
    fn test_thread_per_core_routing() {
        let results = Arc::new(std::sync::Mutex::new(HashMap::new()));
        let results_clone = Arc::clone(&results);

        let system = ThreadPerCore::new(4, move |core_id, item: String| {
            results_clone
                .lock()
                .unwrap()
                .entry(core_id)
                .or_insert_with(Vec::new)
                .push(item);
        });

        system.send_to_core(0, "item-0".into());
        system.send_to_core(1, "item-1".into());
        system.send_to_core(2, "item-2".into());
        thread::sleep(Duration::from_millis(50));

        let results = results.lock().unwrap();
        assert_eq!(results.get(&0).unwrap(), &vec!["item-0"]);
        assert_eq!(results.get(&1).unwrap(), &vec!["item-1"]);
        assert_eq!(results.get(&2).unwrap(), &vec!["item-2"]);
        system.shutdown();
    }

    #[test]
    fn test_thread_per_core_stats() {
        let system = ThreadPerCore::new(2, |_, _: u64| {
            thread::sleep(Duration::from_micros(10));
        });

        for i in 0..10u64 {
            system.send_to_core(0, i);
        }
        thread::sleep(Duration::from_millis(100));

        let stats = system.stats();
        assert!(!stats.is_empty());
        system.shutdown();
    }

    #[test]
    fn test_shared_nothing_system() {
        let mut system = SharedNothingSystem::new();

        system.add_component("processor", |msg| match msg {
            ComponentMessage::Data(data) => {
                // Process data
                let _ = String::from_utf8(data);
            }
            ComponentMessage::Request { data, reply_to } => {
                let response = data.into_iter().rev().collect();
                let _ = reply_to.send(response);
            }
            ComponentMessage::Shutdown => {}
        });

        system.send_to("processor", vec![1, 2, 3]);

        let response = system.request_from("processor", vec![4, 5, 6]).unwrap();
        assert_eq!(response, vec![6, 5, 4]);

        system.shutdown();
    }

    #[test]
    fn test_shared_nothing_request_response() {
        let mut system = SharedNothingSystem::new();

        system.add_component("echo", |msg| {
            if let ComponentMessage::Request { data, reply_to } = msg {
                let _ = reply_to.send(data);
            }
        });

        let response = system.request_from("echo", b"hello".to_vec()).unwrap();
        assert_eq!(response, b"hello");

        system.shutdown();
    }

    #[test]
    fn test_event_loop_basic() {
        let mut event_loop = EventLoop::new("test-loop");
        let result = Arc::new(AtomicU64::new(0));
        let result_clone = Arc::clone(&result);

        event_loop.schedule(move || {
            result_clone.store(42, Ordering::Relaxed);
        });

        thread::sleep(Duration::from_millis(50));
        assert_eq!(result.load(Ordering::Relaxed), 42);
    }

    #[test]
    fn test_event_loop_ordering() {
        let mut event_loop = EventLoop::new("ordering-test");
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));

        for i in 0..10 {
            let order = Arc::clone(&order);
            event_loop.schedule(move || {
                order.lock().unwrap().push(i);
            });
        }

        thread::sleep(Duration::from_millis(100));
        let order = order.lock().unwrap();
        // Event loop processes sequentially, so order should be preserved
        assert_eq!(*order, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn test_pipeline_stage() {
        let (tx1, rx1) = channel::unbounded();
        let (tx2, rx2) = channel::unbounded();

        let stage = PipelineStage::new("double", rx1, tx2, |x: i32| x * 2);

        thread::spawn(move || stage.run());

        tx1.send(5).unwrap();
        tx1.send(10).unwrap();
        drop(tx1);

        let results: Vec<i32> = rx2.iter().collect();
        assert_eq!(results, vec![10, 20]);
    }

    #[test]
    fn test_fan_out_fan_in() {
        let fof = FanOutFanIn::new(4);
        let items: Vec<i32> = (0..100).collect();
        let results = fof.process(items, |x| x * 2);

        let mut sorted = results;
        sorted.sort();
        let expected: Vec<i32> = (0..100).map(|x| x * 2).collect();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_concurrent_metrics() {
        let metrics = ConcurrentMetrics::new();
        let metrics_clone = Arc::clone(&metrics);

        let mut handles = vec![];
        for _ in 0..10 {
            let metrics = Arc::clone(&metrics_clone);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    metrics.increment_counter("requests");
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(metrics.get_counter("requests"), 1000);
    }

    #[test]
    fn test_concurrent_metrics_histogram() {
        let metrics = ConcurrentMetrics::new();
        metrics.record_histogram("latency", 10);
        metrics.record_histogram("latency", 50);
        metrics.record_histogram("latency", 100);

        assert_eq!(metrics.get_counter("nonexistent"), 0);
    }

    #[test]
    fn test_thread_per_core_round_robin() {
        let counts = Arc::new(std::sync::Mutex::new(HashMap::new()));
        let counts_clone = Arc::clone(&counts);

        let system = ThreadPerCore::new(4, move |core_id, _: i32| {
            *counts_clone
                .lock()
                .unwrap()
                .entry(core_id)
                .or_insert(0) += 1;
        });

        let items: Vec<i32> = (0..8).collect();
        system.distribute_round_robin(items);
        thread::sleep(Duration::from_millis(50));

        let counts = counts.lock().unwrap();
        // Each core should get 2 items (8 items / 4 cores)
        for count in counts.values() {
            assert_eq!(*count, 2);
        }
        system.shutdown();
    }

    #[test]
    fn test_fan_out_fan_in_empty() {
        let fof = FanOutFanIn::new(2);
        let results: Vec<i32> = fof.process(vec![], |x: i32| x);
        assert!(results.is_empty());
    }

    #[test]
    fn test_event_loop_stop() {
        let mut event_loop = EventLoop::new("stop-test");
        event_loop.schedule(|| {});
        event_loop.stop();
        // Should complete without hanging
    }
}
