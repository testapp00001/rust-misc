/// Problem: Async Concurrency
///
/// Master async concurrency patterns.
///
/// Key Concepts:
/// - tokio::spawn
/// - tokio::join!
/// - tokio::select!
/// - Async channels
/// - Shared state

use tokio::time::{sleep, Duration};
use tokio::sync::{mpsc, Mutex, RwLock};
use std::sync::Arc;

/// Problem 1: Basic spawn
/// Spawn async tasks
pub async fn basic_spawn() -> i32 {
    let handle = tokio::spawn(async {
        42
    });
    handle.await.unwrap()
}

/// Problem 2: Multiple spawns
/// Spawn multiple tasks
pub async fn multiple_spawns() -> Vec<i32> {
    let mut handles = vec![];

    for i in 0..5 {
        let handle = tokio::spawn(async move {
            i * 2
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

/// Problem 3: Join multiple futures
/// Wait for all futures
pub async fn join_futures() -> (i32, i32, i32) {
    let future1 = async { 1 };
    let future2 = async { 2 };
    let future3 = async { 3 };

    tokio::join!(future1, future2, future3)
}

/// Problem 4: Select first future
/// Return first future to complete
pub async fn select_first() -> i32 {
    let future1 = async {
        sleep(Duration::from_millis(10)).await;
        1
    };
    let future2 = async {
        sleep(Duration::from_millis(5)).await;
        2
    };

    tokio::select! {
        val = future1 => val,
        val = future2 => val,
    }
}

/// Problem 5: Async channel
/// Use async channels
pub async fn async_channel() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);

    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });

    let mut results = Vec::new();
    while let Some(val) = rx.recv().await {
        results.push(val);
    }
    results
}

/// Problem 6: Shared state with Mutex
/// Use async Mutex
pub async fn shared_state_mutex() -> i32 {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..5 {
        let counter = Arc::clone(&counter);
        let handle = tokio::spawn(async move {
            let mut num = counter.lock().await;
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let result = *counter.lock().await;
    result
}

/// Problem 7: Shared state with RwLock
/// Use async RwLock
pub async fn shared_state_rwlock() -> i32 {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    // Readers
    for _ in 0..3 {
        let data = Arc::clone(&data);
        let handle = tokio::spawn(async move {
            let data = data.read().await;
            data.iter().sum::<i32>()
        });
        handles.push(handle);
    }

    // Writer
    let data_clone = Arc::clone(&data);
    let writer_handle = tokio::spawn(async move {
        let mut data = data_clone.write().await;
        data.push(4);
    });

    // Wait for writer
    writer_handle.await.unwrap();

    // Wait for readers
    for handle in handles {
        handle.await.unwrap();
    }

    let result = data.read().await.iter().sum();
    result
}

/// Problem 8: Async producer-consumer
/// Implement async producer-consumer
pub async fn async_producer_consumer() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);

    // Producer
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i * 2).await.unwrap();
        }
    });

    // Consumer
    let mut results = Vec::new();
    while let Some(val) = rx.recv().await {
        results.push(val);
    }
    results
}

/// Problem 9: Async pipeline
/// Implement async pipeline
pub async fn async_pipeline() -> Vec<i32> {
    let (tx1, mut rx1) = mpsc::channel(10);
    let (tx2, mut rx2) = mpsc::channel(10);

    // Stage 1
    tokio::spawn(async move {
        for i in 0..5 {
            tx1.send(i).await.unwrap();
        }
    });

    // Stage 2
    tokio::spawn(async move {
        while let Some(val) = rx1.recv().await {
            tx2.send(val * 2).await.unwrap();
        }
    });

    // Stage 3
    let mut results = Vec::new();
    while let Some(val) = rx2.recv().await {
        results.push(val);
    }
    results
}

/// Problem 10: Async broadcast
/// Use broadcast channel
pub async fn async_broadcast() -> Vec<Vec<i32>> {
    let (tx, _) = tokio::sync::broadcast::channel(10);
    let mut handles = vec![];

    // Sender
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        for i in 0..5 {
            tx_clone.send(i).unwrap();
        }
    });

    // Receivers
    for _ in 0..3 {
        let mut rx = tx.subscribe();
        let handle = tokio::spawn(async move {
            let mut results = Vec::new();
            while let Ok(val) = rx.recv().await {
                results.push(val);
            }
            results
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

/// Problem 11: Async with timeout
/// Add timeout to async operations
pub async fn async_with_timeout() -> Option<i32> {
    tokio::time::timeout(Duration::from_millis(100), async {
        sleep(Duration::from_millis(10)).await;
        42
    })
    .await
    .ok()
}

/// Problem 12: Async with cancellation
/// Cancel async tasks
pub async fn async_with_cancellation() -> Option<i32> {
    let handle = tokio::spawn(async {
        sleep(Duration::from_millis(100)).await;
        42
    });

    tokio::time::timeout(Duration::from_millis(10), handle)
        .await
        .ok()
        .and_then(|r| r.ok())
}

/// Problem 13: Async with rate limiting
/// Rate limit async operations
pub async fn async_rate_limit() -> Vec<i32> {
    let semaphore = Arc::new(tokio::sync::Semaphore::new(2));
    let mut handles = vec![];

    for i in 0..5 {
        let semaphore = Arc::clone(&semaphore);
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            i * 2
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

/// Problem 14: Async with barrier
/// Synchronize async tasks
pub async fn async_with_barrier() -> Vec<i32> {
    let barrier = Arc::new(tokio::sync::Barrier::new(3));
    let mut handles = vec![];

    for i in 0..3 {
        let barrier = Arc::clone(&barrier);
        let handle = tokio::spawn(async move {
            barrier.wait().await;
            i * 10
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

/// Problem 15: Async with notify
/// Use Notify for signaling
pub async fn async_with_notify() -> i32 {
    use tokio::sync::Notify;

    let notify = Arc::new(Notify::new());
    let notify_clone = Arc::clone(&notify);

    let handle = tokio::spawn(async move {
        notify_clone.notified().await;
        42
    });

    notify.notify_one();
    handle.await.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_spawn() {
        assert_eq!(basic_spawn().await, 42);
    }

    #[tokio::test]
    async fn test_multiple_spawns() {
        assert_eq!(multiple_spawns().await, vec![0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_join_futures() {
        assert_eq!(join_futures().await, (1, 2, 3));
    }

    #[tokio::test]
    async fn test_select_first() {
        assert_eq!(select_first().await, 2);
    }

    #[tokio::test]
    async fn test_async_channel() {
        assert_eq!(async_channel().await, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_shared_state_mutex() {
        assert_eq!(shared_state_mutex().await, 5);
    }

    #[tokio::test]
    async fn test_shared_state_rwlock() {
        assert_eq!(shared_state_rwlock().await, 16); // 1+2+3+4+6
    }

    #[tokio::test]
    async fn test_async_producer_consumer() {
        assert_eq!(async_producer_consumer().await, vec![0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_async_pipeline() {
        assert_eq!(async_pipeline().await, vec![0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_async_broadcast() {
        let results = async_broadcast().await;
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_async_with_timeout() {
        assert_eq!(async_with_timeout().await, Some(42));
    }

    #[tokio::test]
    async fn test_async_with_cancellation() {
        assert_eq!(async_with_cancellation().await, None);
    }

    #[tokio::test]
    async fn test_async_rate_limit() {
        let results = async_rate_limit().await;
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_async_with_barrier() {
        let results = async_with_barrier().await;
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_async_with_notify() {
        assert_eq!(async_with_notify().await, 42);
    }
}
