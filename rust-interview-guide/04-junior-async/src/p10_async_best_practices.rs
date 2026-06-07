/// Problem: Async Best Practices
///
/// Master async best practices.
///
/// Key Concepts:
/// - Error handling
/// - Cancellation safety
/// - Backpressure
/// - Resource management
/// - Performance

use tokio::time::{sleep, Duration};
use tokio::sync::{mpsc, Semaphore};
use std::sync::Arc;

/// Problem 1: Proper error handling
/// Handle errors in async
pub async fn proper_error_handling() -> Result<i32, String> {
    let result = async_operation().await?;
    Ok(result)
}

async fn async_operation() -> Result<i32, String> {
    Ok(42)
}

/// Problem 2: Cancellation safety
/// Write cancellation-safe code
pub async fn cancellation_safe() -> Option<i32> {
    let (tx, mut rx) = mpsc::channel(10);

    tokio::spawn(async move {
        tx.send(42).await.unwrap();
    });

    // Use select! for cancellation safety
    tokio::select! {
        val = rx.recv() => val,
        _ = sleep(Duration::from_millis(100)) => None,
    }
}

/// Problem 3: Backpressure handling
/// Handle backpressure
pub async fn backpressure_handling() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(2); // Small buffer

    // Producer with backpressure
    tokio::spawn(async move {
        for i in 0..5 {
            // This will block if buffer is full
            tx.send(i).await.unwrap();
        }
    });

    // Consumer
    let mut results = Vec::new();
    while let Some(val) = rx.recv().await {
        results.push(val);
    }
    results
}

/// Problem 4: Resource management
/// Manage resources properly
pub async fn resource_management() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);

    // Resource is automatically cleaned up when dropped
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
        // tx is dropped here, closing the channel
    });

    let mut results = Vec::new();
    while let Some(val) = rx.recv().await {
        results.push(val);
    }
    results
}

/// Problem 5: Avoid blocking
/// Avoid blocking in async code
pub async fn avoid_blocking() -> i32 {
    // Use spawn_blocking for blocking operations
    tokio::task::spawn_blocking(|| {
        // Blocking work
        std::thread::sleep(Duration::from_millis(10));
        42
    })
    .await
    .unwrap()
}

/// Problem 6: Proper timeout usage
/// Use timeouts properly
pub async fn proper_timeout() -> Option<i32> {
    tokio::time::timeout(Duration::from_millis(100), async {
        sleep(Duration::from_millis(10)).await;
        42
    })
    .await
    .ok()
}

/// Problem 7: Avoid clone overhead
/// Minimize cloning
pub async fn avoid_clone() -> Vec<i32> {
    let data = vec![1, 2, 3, 4, 5];

    // Use references when possible
    let sum: i32 = data.iter().sum();
    vec![sum]
}

/// Problem 8: Use appropriate channel types
/// Choose the right channel
pub async fn appropriate_channel() -> Vec<i32> {
    // Use bounded channel for backpressure
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

/// Problem 9: Proper task spawning
/// Spawn tasks properly
pub async fn proper_task_spawning() -> Vec<i32> {
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

/// Problem 10: Avoid nested async
/// Flatten async operations
pub async fn avoid_nested_async() -> i32 {
    // Don't nest async blocks unnecessarily
    let result = async { 42 }.await;
    result
}

/// Problem 11: Use select! properly
/// Use select! for racing
pub async fn use_select_properly() -> i32 {
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

/// Problem 12: Use join! properly
/// Use join! for parallel
pub async fn use_join_properly() -> (i32, i32) {
    let future1 = async { 1 };
    let future2 = async { 2 };

    tokio::join!(future1, future2)
}

/// Problem 13: Avoid memory leaks
/// Clean up resources
pub async fn avoid_memory_leaks() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);

    // Ensure sender is dropped
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
        // tx is dropped here
    });

    let mut results = Vec::new();
    while let Some(val) = rx.recv().await {
        results.push(val);
    }
    results
}

/// Problem 14: Use semaphore for limiting
/// Limit concurrency
pub async fn use_semaphore() -> Vec<i32> {
    let semaphore = Arc::new(Semaphore::new(2));
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

/// Problem 15: Proper cleanup
/// Clean up on shutdown
pub async fn proper_cleanup() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);

    let handle = tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });

    // Wait for task to complete
    handle.await.unwrap();

    let mut results = Vec::new();
    while let Some(val) = rx.recv().await {
        results.push(val);
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proper_error_handling() {
        assert_eq!(proper_error_handling().await, Ok(42));
    }

    #[tokio::test]
    async fn test_cancellation_safe() {
        assert_eq!(cancellation_safe().await, Some(42));
    }

    #[tokio::test]
    async fn test_backpressure_handling() {
        assert_eq!(backpressure_handling().await, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_resource_management() {
        assert_eq!(resource_management().await, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_avoid_blocking() {
        assert_eq!(avoid_blocking().await, 42);
    }

    #[tokio::test]
    async fn test_proper_timeout() {
        assert_eq!(proper_timeout().await, Some(42));
    }

    #[tokio::test]
    async fn test_avoid_clone() {
        assert_eq!(avoid_clone().await, vec![15]);
    }

    #[tokio::test]
    async fn test_appropriate_channel() {
        assert_eq!(appropriate_channel().await, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_proper_task_spawning() {
        assert_eq!(proper_task_spawning().await, vec![0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_avoid_nested_async() {
        assert_eq!(avoid_nested_async().await, 42);
    }

    #[tokio::test]
    async fn test_use_select_properly() {
        assert_eq!(use_select_properly().await, 2);
    }

    #[tokio::test]
    async fn test_use_join_properly() {
        assert_eq!(use_join_properly().await, (1, 2));
    }

    #[tokio::test]
    async fn test_avoid_memory_leaks() {
        assert_eq!(avoid_memory_leaks().await, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_use_semaphore() {
        let results = use_semaphore().await;
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_proper_cleanup() {
        assert_eq!(proper_cleanup().await, vec![0, 1, 2, 3, 4]);
    }
}
