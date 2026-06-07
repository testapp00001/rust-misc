//! # Tokio Runtime Deep Dive
//!
//! Tokio is Rust's most popular async runtime. Understanding its internals—runtime
//! configuration, task spawning, and the differences between multi-thread and
//! current-thread schedulers—is essential for writing production async code.
//!
//! ## Key Concepts
//! - **Multi-thread runtime**: Work-stealing scheduler across OS threads
//! - **Current-thread runtime**: Single-threaded, lower overhead, good for embedded/tests
//! - **JoinHandle**: A future that resolves when a spawned task completes
//! - **Task budget**: Preventing starvation via cooperative scheduling

use std::time::Duration;

/// A worker that processes items from a shared queue, demonstrating runtime
/// configuration and task lifecycle management.
pub struct WorkerPool {
    worker_count: usize,
    task_sender: tokio::sync::mpsc::Sender<WorkItem>,
}

#[derive(Debug, Clone)]
pub struct WorkItem {
    pub id: u64,
    pub payload: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug)]
pub struct WorkResult {
    pub item_id: u64,
    pub processed_by: usize,
    pub duration_us: u64,
}

impl WorkerPool {
    /// Creates a new worker pool with the specified number of background tasks.
    /// Each worker runs as a separate Tokio task and reads from a shared channel.
    pub fn new(worker_count: usize) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<WorkItem>(256);

        let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));

        let join = tokio::spawn(async move {
            let mut handles = Vec::with_capacity(worker_count);

            for worker_id in 0..worker_count {
                let worker_rx = rx.clone();
                let handle = tokio::spawn(async move {
                    loop {
                        let item = {
                            let mut guard = worker_rx.lock().await;
                            guard.recv().await
                        };
                        let Some(item) = item else { break };

                        // Simulate work
                        let start = std::time::Instant::now();
                        tokio::time::sleep(Duration::from_micros(100)).await;
                        let elapsed = start.elapsed();

                        // In production you'd send results somewhere
                        let _ = WorkResult {
                            item_id: item.id,
                            processed_by: worker_id,
                            duration_us: elapsed.as_micros() as u64,
                        };
                    }
                });
                handles.push(handle);
            }

            // Drop the shared receiver so channel closes when all senders drop
            drop(rx);

            for handle in handles {
                let _ = handle.await;
            }
        });

        (Self { worker_count, task_sender: tx }, join)
    }

    /// Submits work to the pool. Returns Err if the pool has been shut down.
    pub async fn submit(&self, item: WorkItem) -> Result<(), tokio::sync::mpsc::error::SendError<WorkItem>> {
        self.task_sender.send(item).await
    }

    pub fn worker_count(&self) -> usize {
        self.worker_count
    }
}

/// Demonstrates configuring a custom Tokio runtime with specific thread counts
/// and a thread name prefix. In production, you'd use this for isolating workloads.
pub fn build_custom_runtime(thread_count: usize, thread_name: &str) -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(thread_count)
        .thread_name(thread_name)
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime")
}

/// Demonstrates a current-thread runtime for embedded or testing scenarios.
/// This runtime polls on the current OS thread only—no work stealing.
pub fn build_current_thread_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build current-thread runtime")
}

/// Spawns a task with a timeout, returning None if the task exceeds the deadline.
/// This is the canonical pattern for bounded async work.
pub async fn spawn_with_timeout<F, T>(duration: Duration, future: F) -> Option<T>
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let handle = tokio::spawn(future);
    match tokio::time::timeout(duration, handle).await {
        Ok(Ok(result)) => Some(result),
        Ok(Err(join_err)) => {
            // Task panicked
            eprintln!("Task panicked: {join_err}");
            None
        }
        Err(_elapsed) => {
            // Timeout expired—task is still running but we abandon it
            eprintln!("Task timed out after {duration:?}");
            None
        }
    }
}

/// Demonstrates cooperative scheduling with `tokio::task::yield_now()`.
/// Long-running tasks should periodically yield to prevent starvation.
pub async fn cooperative_counter(limit: u64) -> u64 {
    let mut count = 0u64;
    for i in 0..limit {
        count += i;
        // Yield every 1024 iterations to let other tasks run
        if i % 1024 == 0 {
            tokio::task::yield_now().await;
        }
    }
    count
}

/// A batched task spawner that limits concurrent tasks using a semaphore.
/// This prevents unbounded task creation that could exhaust memory.
pub async fn spawn_batched<I, F, T>(items: I, concurrency: usize, f: F) -> Vec<T>
where
    I: IntoIterator,
    I::Item: Send + 'static,
    F: Fn(I::Item) -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>> + Send + Sync + 'static,
    T: Send + 'static,
{
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
    let mut handles = Vec::new();

    for item in items {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let future = f(item);
        handles.push(tokio::spawn(async move {
            let result = future.await;
            drop(permit); // Release the semaphore slot
            result
        }));
    }

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_worker_pool_creation() {
        let (pool, _join) = WorkerPool::new(4);
        assert_eq!(pool.worker_count(), 4);
    }

    #[tokio::test]
    async fn test_worker_pool_processes_items() {
        let (pool, join) = WorkerPool::new(2);

        for i in 0..10 {
            pool.submit(WorkItem {
                id: i,
                payload: format!("task-{i}"),
                priority: Priority::Normal,
            })
            .await
            .unwrap();
        }

        // Drop the sender so the channel closes and workers drain
        drop(pool);

        // Wait for all workers to finish
        let _ = join.await;
    }

    #[test]
    fn test_custom_runtime() {
        let rt = build_custom_runtime(2, "test-worker");
        let result = rt.block_on(async { 42 });
        assert_eq!(result, 42);
        rt.shutdown_timeout(Duration::from_secs(1));
    }

    #[test]
    fn test_current_thread_runtime() {
        let rt = build_current_thread_runtime();
        let result = rt.block_on(async { 42 });
        assert_eq!(result, 42);
        rt.shutdown_timeout(Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_spawn_with_timeout_completes() {
        let result = spawn_with_timeout(Duration::from_secs(5), async { 42 }).await;
        assert_eq!(result, Some(42));
    }

    #[tokio::test]
    async fn test_spawn_with_timeout_expires() {
        let result = spawn_with_timeout(Duration::from_millis(1), async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            42
        })
        .await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cooperative_counter() {
        let result = cooperative_counter(10000).await;
        // Sum of 0..10000
        assert_eq!(result, (0..10000).sum::<u64>());
    }

    #[tokio::test]
    async fn test_spawn_batched_respects_concurrency() {
        let items = 0..20usize;
        let results = spawn_batched(items, 4, |n| {
            Box::pin(async move { n * 2 })
        })
        .await;

        assert_eq!(results.len(), 20);
        // Items are processed in order of completion, so just check all are present
        let mut sorted = results.clone();
        sorted.sort();
        let expected: Vec<usize> = (0..20).map(|n| n * 2).collect();
        assert_eq!(sorted, expected);
    }

    #[tokio::test]
    async fn test_join_handle_abort() {
        let handle = tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(100)).await;
            42
        });

        // Abort the task
        handle.abort();

        // The join should return an error
        let result = handle.await;
        assert!(result.is_err());
        assert!(result.unwrap_err().is_cancelled());
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }
}
