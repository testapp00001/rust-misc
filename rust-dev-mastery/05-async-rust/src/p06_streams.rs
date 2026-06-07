//! # Async Streams
//!
//! Streams are the async equivalent of iterators: they produce a sequence of values
//! over time. The `Stream` trait in `futures` is the foundation, and combinator
//! methods allow transforming, filtering, and buffering streams.
//!
//! ## Key Concepts
//! - **Stream trait**: `poll_next()` returns `Poll<Option<Item>>`
//! - **StreamExt**: Ergonomic methods like `.map()`, `.filter()`, `.buffer_unordered()`
//! - **Backpressure**: Using bounded channels to slow producers when consumers are slow
//! - **Buffering**: `buffered()` and `buffer_unordered()` for concurrent processing

use futures::stream::{self, Stream, StreamExt};
use futures::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

/// A hand-rolled stream that yields incrementing values at a fixed interval.
/// Demonstrates implementing the Stream trait from scratch.
pub struct IntervalStream {
    interval: tokio::time::Interval,
    count: u64,
    max: Option<u64>,
}

impl IntervalStream {
    pub fn new(period: Duration, max: Option<u64>) -> Self {
        IntervalStream {
            interval: tokio::time::interval(period),
            count: 0,
            max,
        }
    }
}

impl Stream for IntervalStream {
    type Item = u64;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(max) = self.max {
            if self.count >= max {
                return Poll::Ready(None); // Stream exhausted
            }
        }

        match self.interval.poll_tick(cx) {
            Poll::Ready(_) => {
                let value = self.count;
                self.count += 1;
                Poll::Ready(Some(value))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A stream that batches individual items into groups of a specified size.
/// Useful for reducing the overhead of processing many small items.
pub struct BatchingStream<S: Stream> {
    inner: S,
    batch_size: usize,
    buffer: Vec<S::Item>,
}

impl<S: Stream> BatchingStream<S> {
    pub fn new(inner: S, batch_size: usize) -> Self {
        BatchingStream {
            inner,
            batch_size,
            buffer: Vec::with_capacity(batch_size),
        }
    }
}

impl<S> Stream for BatchingStream<S>
where
    S: Stream + Unpin,
    S::Item: Unpin,
{
    type Item = Vec<S::Item>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    self.buffer.push(item);
                    if self.buffer.len() >= self.batch_size {
                        let batch_size = self.batch_size;
                        let batch = std::mem::replace(
                            &mut self.buffer,
                            Vec::with_capacity(batch_size),
                        );
                        return Poll::Ready(Some(batch));
                    }
                }
                Poll::Ready(None) => {
                    if self.buffer.is_empty() {
                        return Poll::Ready(None);
                    } else {
                        let batch = std::mem::take(&mut self.buffer);
                        return Poll::Ready(Some(batch));
                    }
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Processes items from a stream with controlled concurrency and backpressure.
/// Uses `buffer_unordered` to process up to N items concurrently.
pub async fn process_with_backpressure<T, F, Fut, U>(
    input: impl Stream<Item = T>,
    concurrency: usize,
    processor: F,
) -> Vec<U>
where
    F: Fn(T) -> Fut + Clone,
    Fut: std::future::Future<Output = U>,
{
    input
        .map(move |item| {
            let proc = processor.clone();
            async move { proc(item).await }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await
}

/// Demonstrates merging multiple streams into one.
/// Items from all input streams are interleaved as they arrive.
pub async fn merge_streams<T: Send + 'static>(
    streams: Vec<impl Stream<Item = T> + Send + Unpin + 'static>,
) -> Vec<T> {
    let merged = stream::select_all(streams);
    merged.collect().await
}

/// Implements a rate-limited stream that yields items at a maximum rate.
pub struct RateLimitedStream<S: Stream> {
    inner: S,
    min_interval: Duration,
    last_yield: Option<tokio::time::Instant>,
}

impl<S: Stream> RateLimitedStream<S> {
    pub fn new(inner: S, min_interval: Duration) -> Self {
        RateLimitedStream {
            inner,
            min_interval,
            last_yield: None,
        }
    }
}

impl<S: Stream + Unpin> Stream for RateLimitedStream<S>
where
    S::Item: Unpin,
{
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check if enough time has passed since last yield
        if let Some(last) = self.last_yield {
            let now = tokio::time::Instant::now();
            if now - last < self.min_interval {
                // Schedule a wakeup after the remaining time
                let wake_at = last + self.min_interval;
                let delay = tokio::time::sleep_until(wake_at);
                tokio::pin!(delay);
                if delay.poll(cx).is_pending() {
                    return Poll::Pending;
                }
            }
        }

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(item) => {
                self.last_yield = Some(tokio::time::Instant::now());
                Poll::Ready(item)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Implements exponential moving average as a stream transformation.
/// Smooths out noisy data streams.
pub struct EmaStream<S: Stream<Item = f64>> {
    inner: S,
    alpha: f64,
    current: Option<f64>,
}

impl<S: Stream<Item = f64> + Unpin> EmaStream<S> {
    pub fn new(inner: S, alpha: f64) -> Self {
        assert!((0.0..=1.0).contains(&alpha), "alpha must be in [0, 1]");
        EmaStream {
            inner,
            alpha,
            current: None,
        }
    }
}

impl<S: Stream<Item = f64> + Unpin> Stream for EmaStream<S> {
    type Item = f64;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(value)) => {
                let ema = match self.current {
                    None => value,
                    Some(prev) => self.alpha * value + (1.0 - self.alpha) * prev,
                };
                self.current = Some(ema);
                Poll::Ready(Some(ema))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A stream multiplexer that distributes items across N output streams
/// using round-robin scheduling.
pub struct RoundRobinDemux<S: Stream> {
    inner: S,
    outputs: Vec<tokio::sync::mpsc::Sender<S::Item>>,
    current: usize,
}

impl<S: Stream + Unpin> RoundRobinDemux<S>
where
    S::Item: Clone,
{
    pub fn new(inner: S, num_outputs: usize, buffer: usize) -> (Self, Vec<tokio::sync::mpsc::Receiver<S::Item>>) {
        let mut outputs = Vec::with_capacity(num_outputs);
        let mut receivers = Vec::with_capacity(num_outputs);

        for _ in 0..num_outputs {
            let (tx, rx) = tokio::sync::mpsc::channel(buffer);
            outputs.push(tx);
            receivers.push(rx);
        }

        (
            RoundRobinDemux {
                inner,
                outputs,
                current: 0,
            },
            receivers,
        )
    }

    pub async fn run(&mut self) {
        while let Some(item) = self.inner.next().await {
            let idx = self.current % self.outputs.len();
            if self.outputs[idx].send(item).await.is_err() {
                // Receiver dropped, skip
            }
            self.current += 1;
        }
    }
}

/// Windowed stream: groups items into fixed-size windows with overlap.
pub struct WindowedStream<S: Stream> {
    inner: S,
    window_size: usize,
    step: usize,
    buffer: Vec<S::Item>,
}

impl<S: Stream + Unpin> WindowedStream<S>
where
    S::Item: Clone,
{
    pub fn new(inner: S, window_size: usize, step: usize) -> Self {
        WindowedStream {
            inner,
            window_size,
            step,
            buffer: Vec::new(),
        }
    }
}

impl<S: Stream + Unpin> Unpin for WindowedStream<S> {}

impl<S: Stream + Unpin> Stream for WindowedStream<S>
where
    S::Item: Clone,
{
    type Item = Vec<S::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            // If we have enough for a window, emit it
            if this.buffer.len() >= this.window_size {
                let window: Vec<S::Item> = this.buffer[..this.window_size].to_vec();
                // Advance by step
                let drain = this.step.min(this.buffer.len());
                this.buffer.drain(..drain);
                return Poll::Ready(Some(window));
            }

            match Pin::new(&mut this.inner).poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    this.buffer.push(item);
                }
                Poll::Ready(None) => {
                    if this.buffer.len() >= this.window_size {
                        // Emit remaining full windows
                        let window: Vec<S::Item> = this.buffer[..this.window_size].to_vec();
                        let drain = this.step.min(this.buffer.len());
                        this.buffer.drain(..drain);
                        return Poll::Ready(Some(window));
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_interval_stream() {
        let stream = IntervalStream::new(Duration::from_millis(5), Some(5));
        let items: Vec<u64> = stream.collect().await;
        assert_eq!(items, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_batching_stream() {
        let data = stream::iter(0..10);
        let batched = BatchingStream::new(data, 3);
        let batches: Vec<Vec<i32>> = batched.collect().await;

        assert_eq!(batches.len(), 4); // 3+3+3+1
        assert_eq!(batches[0], vec![0, 1, 2]);
        assert_eq!(batches[1], vec![3, 4, 5]);
        assert_eq!(batches[2], vec![6, 7, 8]);
        assert_eq!(batches[3], vec![9]);
    }

    #[tokio::test]
    async fn test_batching_exact_size() {
        let data = stream::iter(0..6);
        let batched = BatchingStream::new(data, 3);
        let batches: Vec<Vec<i32>> = batched.collect().await;

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0], vec![0, 1, 2]);
        assert_eq!(batches[1], vec![3, 4, 5]);
    }

    #[tokio::test]
    async fn test_process_with_backpressure() {
        let data = stream::iter(0..10);
        let results = process_with_backpressure(data, 4, |n: i32| async move { n * 2 }).await;

        let mut sorted = results;
        sorted.sort();
        assert_eq!(sorted, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
    }

    #[tokio::test]
    async fn test_merge_streams() {
        let s1 = stream::iter(vec![1, 2, 3]);
        let s2 = stream::iter(vec![4, 5, 6]);
        let s3 = stream::iter(vec![7, 8, 9]);

        let mut results = merge_streams(vec![s1, s2, s3]).await;
        results.sort();
        assert_eq!(results, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[tokio::test]
    async fn test_ema_stream() {
        let data = stream::iter(vec![10.0, 10.0, 10.0, 10.0, 10.0]);
        let ema: Vec<f64> = EmaStream::new(data, 0.5).collect().await;

        assert_eq!(ema.len(), 5);
        // First value = raw
        assert!((ema[0] - 10.0).abs() < 0.001);
        // All constant input should converge to same value
        for &v in &ema {
            assert!((v - 10.0).abs() < 0.001);
        }
    }

    #[tokio::test]
    async fn test_ema_stream_varying() {
        let data = stream::iter(vec![10.0, 20.0, 10.0, 20.0]);
        let ema: Vec<f64> = EmaStream::new(data, 0.5).collect().await;

        assert_eq!(ema.len(), 4);
        assert!((ema[0] - 10.0).abs() < 0.001);
        // EMA should be smoothed
        assert!((ema[1] - 15.0).abs() < 0.001); // 0.5*20 + 0.5*10
    }

    #[tokio::test]
    async fn test_round_robin_demux() {
        let data = stream::iter(0..9);
        let (mut demux, mut receivers) = RoundRobinDemux::new(data, 3, 10);

        demux.run().await;

        let r0: Vec<i32> = std::iter::from_fn(|| receivers[0].try_recv().ok()).collect();
        let r1: Vec<i32> = std::iter::from_fn(|| receivers[1].try_recv().ok()).collect();
        let r2: Vec<i32> = std::iter::from_fn(|| receivers[2].try_recv().ok()).collect();

        assert_eq!(r0, vec![0, 3, 6]);
        assert_eq!(r1, vec![1, 4, 7]);
        assert_eq!(r2, vec![2, 5, 8]);
    }

    #[tokio::test]
    async fn test_windowed_stream() {
        let data = stream::iter(0..6);
        let windows: Vec<Vec<i32>> = WindowedStream::new(data, 3, 1).collect().await;

        assert_eq!(windows.len(), 4); // [0,1,2], [1,2,3], [2,3,4], [3,4,5]
        assert_eq!(windows[0], vec![0, 1, 2]);
        assert_eq!(windows[1], vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_windowed_stream_step_equals_size() {
        let data = stream::iter(0..6);
        let windows: Vec<Vec<i32>> = WindowedStream::new(data, 3, 3).collect().await;

        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0], vec![0, 1, 2]);
        assert_eq!(windows[1], vec![3, 4, 5]);
    }

    #[tokio::test]
    async fn test_stream_chaining() {
        let result: Vec<i32> = stream::iter(0..20)
            .filter(|n| futures::future::ready(n % 2 == 0))
            .map(|n| n * 3)
            .take(5)
            .collect()
            .await;

        assert_eq!(result, vec![0, 6, 12, 18, 24]);
    }

    #[tokio::test]
    async fn test_stream_fold() {
        let sum: i32 = stream::iter(1..=100).fold(0, |acc, n| async move { acc + n }).await;
        assert_eq!(sum, 5050);
    }

    #[tokio::test]
    async fn test_empty_batching_stream() {
        let data = stream::iter(std::iter::empty::<i32>());
        let batched = BatchingStream::new(data, 5);
        let batches: Vec<Vec<i32>> = batched.collect().await;
        assert!(batches.is_empty());
    }
}
