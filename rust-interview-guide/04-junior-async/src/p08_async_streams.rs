/// Problem: Async Streams
///
/// Master async streams.
///
/// Key Concepts:
/// - Stream trait
/// - Stream combinators
/// - Stream processing
/// - Custom streams
/// - Stream sinks

use futures::stream::{self, Stream, StreamExt};
use tokio::time::{sleep, Duration};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Problem 1: Basic stream
/// Create a basic stream
pub async fn basic_stream() -> Vec<i32> {
    let s = stream::iter(vec![1, 2, 3, 4, 5]);
    s.collect().await
}

/// Problem 2: Stream with map
/// Map stream elements
pub async fn stream_map() -> Vec<i32> {
    let s = stream::iter(vec![1, 2, 3, 4, 5]);
    s.map(|x| x * 2).collect().await
}

/// Problem 3: Stream with filter
/// Filter stream elements
pub async fn stream_filter() -> Vec<i32> {
    let s = stream::iter(vec![1, 2, 3, 4, 5]);
    s.filter(|x| futures::future::ready(x % 2 == 0)).collect().await
}

/// Problem 4: Stream with fold
/// Fold stream elements
pub async fn stream_fold() -> i32 {
    let s = stream::iter(vec![1, 2, 3, 4, 5]);
    s.fold(0, |acc, x| async move { acc + x }).await
}

/// Problem 5: Stream with for_each
/// Process each element
pub async fn stream_for_each() -> i32 {
    let mut sum = 0;
    let s = stream::iter(vec![1, 2, 3, 4, 5]);
    s.for_each(|x| {
        sum += x;
        async {}
    }).await;
    sum
}

/// Problem 6: Stream with take
/// Take first n elements
pub async fn stream_take() -> Vec<i32> {
    let s = stream::iter(vec![1, 2, 3, 4, 5]);
    s.take(3).collect().await
}

/// Problem 7: Stream with skip
/// Skip first n elements
pub async fn stream_skip() -> Vec<i32> {
    let s = stream::iter(vec![1, 2, 3, 4, 5]);
    s.skip(2).collect().await
}

/// Problem 8: Stream with chain
/// Chain two streams
pub async fn stream_chain() -> Vec<i32> {
    let s1 = stream::iter(vec![1, 2, 3]);
    let s2 = stream::iter(vec![4, 5, 6]);
    s1.chain(s2).collect().await
}

/// Problem 9: Stream with zip
/// Zip two streams
pub async fn stream_zip() -> Vec<(i32, i32)> {
    let s1 = stream::iter(vec![1, 2, 3]);
    let s2 = stream::iter(vec![4, 5, 6]);
    s1.zip(s2).collect().await
}

/// Problem 10: Stream with enumerate
/// Enumerate stream elements
pub async fn stream_enumerate() -> Vec<(usize, i32)> {
    let s = stream::iter(vec![10, 20, 30]);
    s.enumerate().collect().await
}

/// Problem 11: Stream with chunks
/// Chunk stream elements
pub async fn stream_chunks() -> Vec<Vec<i32>> {
    let s = stream::iter(vec![1, 2, 3, 4, 5]);
    s.chunks(2).collect().await
}

/// Problem 12: Stream with buffer_unordered
/// Process concurrently
pub async fn stream_buffer_unordered() -> Vec<i32> {
    let s = stream::iter(vec![1, 2, 3, 4, 5]);
    s.map(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * 2
    })
    .buffer_unordered(3)
    .collect()
    .await
}

/// Problem 13: Stream with buffer_ordered
/// Process concurrently but maintain order
pub async fn stream_buffer_ordered() -> Vec<i32> {
    let s = stream::iter(vec![1, 2, 3, 4, 5]);
    s.map(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * 2
    })
    .buffered(3)
    .collect()
    .await
}

/// Problem 14: Custom stream
/// Create a custom stream
pub struct CounterStream {
    count: i32,
    max: i32,
}

impl CounterStream {
    pub fn new(max: i32) -> Self {
        Self { count: 0, max }
    }
}

impl Stream for CounterStream {
    type Item = i32;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.count < self.max {
            let value = self.count;
            self.count += 1;
            Poll::Ready(Some(value))
        } else {
            Poll::Ready(None)
        }
    }
}

/// Problem 15: Stream with merge
/// Merge two streams (simulated)
pub async fn stream_merge() -> Vec<i32> {
    let s1 = stream::iter(vec![1, 3, 5]);
    let s2 = stream::iter(vec![2, 4, 6]);
    let mut result: Vec<i32> = s1.chain(s2).collect().await;
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_stream() {
        assert_eq!(basic_stream().await, vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_stream_map() {
        assert_eq!(stream_map().await, vec![2, 4, 6, 8, 10]);
    }

    #[tokio::test]
    async fn test_stream_filter() {
        assert_eq!(stream_filter().await, vec![2, 4]);
    }

    #[tokio::test]
    async fn test_stream_fold() {
        assert_eq!(stream_fold().await, 15);
    }

    #[tokio::test]
    async fn test_stream_for_each() {
        assert_eq!(stream_for_each().await, 15);
    }

    #[tokio::test]
    async fn test_stream_take() {
        assert_eq!(stream_take().await, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_stream_skip() {
        assert_eq!(stream_skip().await, vec![3, 4, 5]);
    }

    #[tokio::test]
    async fn test_stream_chain() {
        assert_eq!(stream_chain().await, vec![1, 2, 3, 4, 5, 6]);
    }

    #[tokio::test]
    async fn test_stream_zip() {
        assert_eq!(stream_zip().await, vec![(1, 4), (2, 5), (3, 6)]);
    }

    #[tokio::test]
    async fn test_stream_enumerate() {
        assert_eq!(stream_enumerate().await, vec![(0, 10), (1, 20), (2, 30)]);
    }

    #[tokio::test]
    async fn test_stream_chunks() {
        assert_eq!(stream_chunks().await, vec![vec![1, 2], vec![3, 4], vec![5]]);
    }

    #[tokio::test]
    async fn test_stream_buffer_unordered() {
        let mut result = stream_buffer_unordered().await;
        result.sort();
        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    }

    #[tokio::test]
    async fn test_stream_buffer_ordered() {
        assert_eq!(stream_buffer_ordered().await, vec![2, 4, 6, 8, 10]);
    }

    #[tokio::test]
    async fn test_custom_stream() {
        let s = CounterStream::new(5);
        let result: Vec<i32> = s.collect().await;
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_stream_merge() {
        let mut result = stream_merge().await;
        result.sort();
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }
}
