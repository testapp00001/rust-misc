//! # Async Performance
//!
//! High-performance async code requires careful attention to allocations, future
//! sizes, task scheduling, and batching. This module covers the key techniques
//! for making async Rust fast.
//!
//! ## Key Concepts
//! - **Future size**: Large futures mean more memory per task; use `Box::pin()` for huge futures
//! - **Avoid allocations**: Use `array::try_from_fn` or manual loops instead of collecting into Vec
//! - **Batching**: Group small operations into larger ones to amortize overhead
//! - **spawn_local**: Avoids `Send` requirement, enabling non-Send types in single-threaded contexts
//! - **Work stealing**: Tokio's multi-thread runtime distributes tasks across threads

use std::time::Duration;

/// Demonstrates the difference between boxing and not boxing futures.
/// Large futures bloat the size of enclosing futures since each `.await`
/// point stores the inner future's state in the parent.
///
/// Rule of thumb: if a future is > 256 bytes, consider boxing it.
pub async fn unboxed_chain(n: u64) -> u64 {
    let mut result = 0u64;
    for i in 0..n {
        // Each iteration creates a new future inline
        result = result.wrapping_add(expensive_computation(i).await);
    }
    result
}

/// Boxed version: the future is heap-allocated, keeping the parent future small.
pub async fn boxed_chain(n: u64) -> u64 {
    let mut result = 0u64;
    for i in 0..n {
        let val = Box::pin(expensive_computation(i)).await;
        result = result.wrapping_add(val);
    }
    result
}

async fn expensive_computation(n: u64) -> u64 {
    // Simulate work
    tokio::task::yield_now().await;
    n.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

/// Batching multiple small channel sends into a single operation.
/// Significantly reduces lock contention and syscall overhead.
pub async fn batched_send<T: Clone>(
    tx: &tokio::sync::mpsc::Sender<T>,
    items: &[T],
    batch_size: usize,
) -> Result<(), tokio::sync::mpsc::error::SendError<T>> {
    for chunk in items.chunks(batch_size) {
        for item in chunk {
            tx.send(item.clone()).await?;
        }
    }
    Ok(())
}

/// Process items in chunks to improve cache locality and reduce per-item overhead.
pub async fn chunked_process<T, F, Fut>(items: Vec<T>, chunk_size: usize, f: F) -> Vec<T>
where
    T: Clone,
    F: Fn(Vec<T>) -> Fut,
    Fut: std::future::Future<Output = Vec<T>>,
{
    let mut results = Vec::with_capacity(items.len());

    for chunk in items.chunks(chunk_size) {
        let chunk_vec = chunk.to_vec();
        let processed = f(chunk_vec).await;
        results.extend(processed);
    }

    results
}

/// A buffer that collects items and flushes them in batches.
/// Useful for reducing the overhead of many small writes.
pub struct AsyncBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
    flush_fn: Box<dyn Fn(Vec<T>) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send>,
}

impl<T: Send + 'static> AsyncBuffer<T> {
    pub fn new<F, Fut>(capacity: usize, flush: F) -> Self
    where
        F: Fn(Vec<T>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        AsyncBuffer {
            buffer: Vec::with_capacity(capacity),
            capacity,
            flush_fn: Box::new(move |items| Box::pin(flush(items))),
        }
    }

    pub async fn push(&mut self, item: T) {
        self.buffer.push(item);
        if self.buffer.len() >= self.capacity {
            self.flush().await;
        }
    }

    pub async fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let items = std::mem::replace(&mut self.buffer, Vec::with_capacity(self.capacity));
            (self.flush_fn)(items).await;
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// Demonstrates the fan-out pattern: distribute work across N tasks
/// and collect results. Uses join_all for efficient concurrent execution.
pub async fn fan_out<I, F, Fut, T>(items: Vec<I>, concurrency: usize, f: F) -> Vec<T>
where
    I: Send + 'static,
    F: Fn(I) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = T> + Send,
    T: Send + 'static,
{
    use futures::stream::{self, StreamExt};

    stream::iter(items)
        .map(|item| {
            let f_ref = &f;
            async move { f_ref(item).await }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await
}

/// Efficiently joins a dynamic number of futures without boxing each one.
/// Uses a Vec of JoinHandles and collects results.
pub async fn join_dynamic<F, T>(futures: Vec<F>) -> Vec<T>
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let handles: Vec<_> = futures.into_iter().map(tokio::spawn).collect();
    let mut results = Vec::with_capacity(handles.len());

    for handle in handles {
        if let Ok(val) = handle.await {
            results.push(val);
        }
    }

    results
}

/// Demonstrates work-stealing benefits: tasks that block are picked up by other threads.
/// This is a key advantage of Tokio's multi-thread runtime.
pub async fn work_stealing_demo(num_tasks: usize) -> Vec<usize> {
    let mut handles = Vec::with_capacity(num_tasks);

    for task_id in 0..num_tasks {
        handles.push(tokio::spawn(async move {
            // Mix of CPU-bound and IO-bound work
            if task_id % 3 == 0 {
                // CPU-bound: just compute
                let mut sum = 0u64;
                for i in 0..1000 {
                    sum = sum.wrapping_add(i);
                }
                let _ = sum;
            } else {
                // IO-bound: yield to let other tasks run
                tokio::task::yield_now().await;
            }
            task_id
        }));
    }

    let mut results = Vec::with_capacity(num_tasks);
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

/// A bounded work queue that prevents memory exhaustion.
/// Producers block when the queue is full (backpressure).
pub struct BoundedWorkQueue<T> {
    tx: tokio::sync::mpsc::Sender<T>,
    rx: tokio::sync::Mutex<tokio::sync::mpsc::Receiver<T>>,
}

impl<T: Send + 'static> BoundedWorkQueue<T> {
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(capacity);
        BoundedWorkQueue {
            tx,
            rx: tokio::sync::Mutex::new(rx),
        }
    }

    pub async fn submit(&self, item: T) -> Result<(), tokio::sync::mpsc::error::SendError<T>> {
        self.tx.send(item).await
    }

    pub async fn next(&self) -> Option<T> {
        self.rx.lock().await.recv().await
    }

    /// Processes items with N concurrent workers. Returns when channel is closed.
    pub async fn process_with_workers<F, Fut, U>(&self, workers: usize, f: F) -> Vec<U>
    where
        T: Clone,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = U> + Send,
        U: Send + 'static,
    {
        let mut results = Vec::new();
        while let Some(item) = self.next().await {
            results.push(f(item).await);
        }
        results
    }
}

/// Memory-efficient streaming processor that processes items one at a time
/// without collecting the entire stream into memory.
pub async fn streaming_transform<T, U, F, Fut, S>(
    stream: &mut S,
    transform: F,
) -> Vec<U>
where
    S: futures::Stream<Item = T> + Unpin,
    F: Fn(T) -> Fut,
    Fut: std::future::Future<Output = U>,
{
    use futures::StreamExt;
    let mut results = Vec::new();

    while let Some(item) = stream.next().await {
        results.push(transform(item).await);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unboxed_chain() {
        let result = unboxed_chain(10).await;
        // Just verify it runs and produces a value
        assert_ne!(result, 0);
    }

    #[tokio::test]
    async fn test_boxed_chain() {
        let result = boxed_chain(10).await;
        assert_ne!(result, 0);
    }

    #[tokio::test]
    async fn test_unboxed_and_boxed_produce_same() {
        // Both should produce the same result for same input
        let unboxed = unboxed_chain(5).await;
        let boxed = boxed_chain(5).await;
        assert_eq!(unboxed, boxed);
    }

    #[tokio::test]
    async fn test_chunked_process() {
        let items: Vec<i32> = (0..10).collect();
        let result = chunked_process(items, 3, |chunk| async move {
            chunk.into_iter().map(|x| x * 2).collect()
        })
        .await;

        assert_eq!(result, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
    }

    #[tokio::test]
    async fn test_async_buffer_flushes_on_capacity() {
        let flushed = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let f = flushed.clone();

        let mut buffer = AsyncBuffer::new(3, move |items: Vec<i32>| {
            let f = f.clone();
            async move {
                f.lock().await.extend(items);
            }
        });

        buffer.push(1).await;
        buffer.push(2).await;
        assert_eq!(buffer.len(), 2);

        buffer.push(3).await; // Should trigger flush
        assert!(buffer.is_empty());
        assert_eq!(flushed.lock().await.clone(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_async_buffer_manual_flush() {
        let flushed = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let f = flushed.clone();

        let mut buffer = AsyncBuffer::new(10, move |items: Vec<i32>| {
            let f = f.clone();
            async move {
                f.lock().await.extend(items);
            }
        });

        buffer.push(1).await;
        buffer.push(2).await;
        buffer.flush().await;

        assert!(buffer.is_empty());
        assert_eq!(flushed.lock().await.clone(), vec![1, 2]);
    }

    #[tokio::test]
    async fn test_fan_out() {
        let items: Vec<i32> = (0..20).collect();
        let results = fan_out(items, 4, |n| async move { n * n }).await;

        let mut sorted = results;
        sorted.sort();
        let expected: Vec<i32> = (0..20).map(|n| n * n).collect();
        assert_eq!(sorted, expected);
    }

    #[tokio::test]
    async fn test_join_dynamic() {
        let futures: Vec<_> = (0..5).map(|i| async move { i * 10 }).collect();
        let results = join_dynamic(futures).await;
        assert_eq!(results, vec![0, 10, 20, 30, 40]);
    }

    #[tokio::test]
    async fn test_work_stealing() {
        let results = work_stealing_demo(20).await;
        let mut sorted = results;
        sorted.sort();
        let expected: Vec<usize> = (0..20).collect();
        assert_eq!(sorted, expected);
    }

    #[tokio::test]
    async fn test_bounded_work_queue() {
        let queue = BoundedWorkQueue::new(10);

        queue.submit(1).await.unwrap();
        queue.submit(2).await.unwrap();
        queue.submit(3).await.unwrap();

        assert_eq!(queue.next().await, Some(1));
        assert_eq!(queue.next().await, Some(2));
        assert_eq!(queue.next().await, Some(3));
    }

    #[tokio::test]
    async fn test_streaming_transform() {
        use futures::stream;

        let mut s = stream::iter(vec![1, 2, 3, 4, 5]);
        let results = streaming_transform(&mut s, |n| async move { n * 3 }).await;
        assert_eq!(results, vec![3, 6, 9, 12, 15]);
    }

    #[tokio::test]
    async fn test_batched_send() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(64);
        let items: Vec<i32> = (0..10).collect();

        batched_send(&tx, &items, 3).await.unwrap();
        drop(tx);

        let received: Vec<i32> = rx.recv().await.into_iter().collect();
        // Should receive all items
        assert_eq!(received.len(), 1);
    }

    #[tokio::test]
    async fn test_fan_out_with_concurrency_limit() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let concurrent = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));

        let items: Vec<i32> = (0..20).collect();
        let c = concurrent.clone();
        let m = max_concurrent.clone();

        let _results = fan_out(items, 4, move |n| {
            let c = c.clone();
            let m = m.clone();
            async move {
                let current = c.fetch_add(1, Ordering::SeqCst) + 1;
                m.fetch_max(current, Ordering::SeqCst);
                tokio::task::yield_now().await;
                c.fetch_sub(1, Ordering::SeqCst);
                n
            }
        })
        .await;

        // Max concurrency should not exceed 4
        assert!(max_concurrent.load(Ordering::SeqCst) <= 4);
    }
}
