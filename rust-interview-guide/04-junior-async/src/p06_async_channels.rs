/// Problem: Async Channels
///
/// Master async channels.
///
/// Key Concepts:
/// - mpsc channels
/// - oneshot channels
/// - broadcast channels
/// - watch channels
/// - Channel patterns

use tokio::sync::{mpsc, oneshot, broadcast, watch};
use tokio::time::{sleep, Duration};

/// Problem 1: Basic mpsc channel
/// Use basic mpsc channel
pub async fn basic_mpsc() -> Vec<i32> {
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

/// Problem 2: Multiple producers
/// Use multiple producers
pub async fn multiple_producers() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);
    let mut handles = vec![];

    for i in 0..3 {
        let tx = tx.clone();
        let handle = tokio::spawn(async move {
            for j in 0..3 {
                tx.send(i * 10 + j).await.unwrap();
            }
        });
        handles.push(handle);
    }

    drop(tx);

    let mut results = Vec::new();
    while let Some(val) = rx.recv().await {
        results.push(val);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    results.sort();
    results
}

/// Problem 3: Bounded channel
/// Use bounded channel
pub async fn bounded_channel() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(2); // Small buffer

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

/// Problem 4: Oneshot channel
/// Use oneshot channel
pub async fn oneshot_channel() -> i32 {
    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        tx.send(42).unwrap();
    });

    rx.await.unwrap()
}

/// Problem 5: Broadcast channel
/// Use broadcast channel
pub async fn broadcast_channel() -> Vec<Vec<i32>> {
    let (tx, _) = broadcast::channel(10);
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

/// Problem 6: Watch channel
/// Use watch channel
pub async fn watch_channel() -> Vec<i32> {
    let (tx, mut rx) = watch::channel(0);

    tokio::spawn(async move {
        for i in 1..5 {
            sleep(Duration::from_millis(10)).await;
            tx.send(i).unwrap();
        }
    });

    let mut results = Vec::new();
    while rx.changed().await.is_ok() {
        let val = *rx.borrow();
        results.push(val);
    }
    results
}

/// Problem 7: Channel with error handling
/// Handle channel errors
pub async fn channel_error_handling() -> Result<Vec<i32>, String> {
    let (tx, mut rx) = mpsc::channel(10);

    tokio::spawn(async move {
        for i in 0..5 {
            if i == 3 {
                drop(tx);
                return;
            }
            tx.send(i).await.unwrap();
        }
    });

    let mut results = Vec::new();
    while let Some(val) = rx.recv().await {
        results.push(val);
    }
    Ok(results)
}

/// Problem 8: Channel with timeout
/// Use timeout with channels
pub async fn channel_with_timeout() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);

    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
            sleep(Duration::from_millis(10)).await;
        }
    });

    let mut results = Vec::new();
    loop {
        match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Some(val)) => results.push(val),
            _ => break,
        }
    }
    results
}

/// Problem 9: Channel with transformation
/// Transform messages in channel
pub async fn channel_transformation() -> Vec<i32> {
    let (tx1, mut rx1) = mpsc::channel(10);
    let (tx2, mut rx2) = mpsc::channel(10);

    // Producer
    tokio::spawn(async move {
        for i in 0..5 {
            tx1.send(i).await.unwrap();
        }
    });

    // Transformer
    tokio::spawn(async move {
        while let Some(val) = rx1.recv().await {
            tx2.send(val * 2).await.unwrap();
        }
    });

    // Consumer
    let mut results = Vec::new();
    while let Some(val) = rx2.recv().await {
        results.push(val);
    }
    results
}

/// Problem 10: Channel with backpressure
/// Handle backpressure
pub async fn channel_backpressure() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(2); // Small buffer

    // Fast producer
    tokio::spawn(async move {
        for i in 0..10 {
            tx.send(i).await.unwrap();
        }
    });

    // Slow consumer
    let mut results = Vec::new();
    while let Some(val) = rx.recv().await {
        results.push(val);
        sleep(Duration::from_millis(10)).await;
    }
    results
}

/// Problem 11: Channel with close
/// Close channel explicitly
pub async fn channel_close() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);

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

/// Problem 12: Channel with try_send
/// Use try_send for non-blocking send
pub async fn channel_try_send() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(2);

    tokio::spawn(async move {
        for i in 0..5 {
            loop {
                match tx.try_send(i) {
                    Ok(_) => break,
                    Err(_) => sleep(Duration::from_millis(1)).await,
                }
            }
        }
    });

    let mut results = Vec::new();
    while let Some(val) = rx.recv().await {
        results.push(val);
    }
    results
}

/// Problem 13: Channel with try_recv
/// Use try_recv for non-blocking receive
pub async fn channel_try_recv() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);

    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });

    sleep(Duration::from_millis(50)).await;

    let mut results = Vec::new();
    while let Ok(val) = rx.try_recv() {
        results.push(val);
    }
    results
}

/// Problem 14: Channel with capacity
/// Check channel capacity
pub async fn channel_capacity() -> usize {
    let (tx, rx) = mpsc::channel::<i32>(10);
    tx.send(1).await.unwrap();
    tx.send(2).await.unwrap();
    rx.len()
}

/// Problem 15: Channel with Receiver stream
/// Use Receiver as stream
pub async fn channel_as_stream() -> Vec<i32> {
    use futures::StreamExt;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_mpsc() {
        assert_eq!(basic_mpsc().await, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_multiple_producers() {
        let results = multiple_producers().await;
        assert_eq!(results.len(), 9);
    }

    #[tokio::test]
    async fn test_bounded_channel() {
        assert_eq!(bounded_channel().await, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_oneshot_channel() {
        assert_eq!(oneshot_channel().await, 42);
    }

    #[tokio::test]
    async fn test_broadcast_channel() {
        let results = broadcast_channel().await;
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_watch_channel() {
        let results = watch_channel().await;
        assert!(results.len() > 0);
    }

    #[tokio::test]
    async fn test_channel_error_handling() {
        let results = channel_error_handling().await.unwrap();
        assert_eq!(results, vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn test_channel_with_timeout() {
        assert_eq!(channel_with_timeout().await, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_channel_transformation() {
        assert_eq!(channel_transformation().await, vec![0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_channel_backpressure() {
        assert_eq!(channel_backpressure().await, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[tokio::test]
    async fn test_channel_close() {
        assert_eq!(channel_close().await, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_channel_try_send() {
        assert_eq!(channel_try_send().await, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_channel_try_recv() {
        assert_eq!(channel_try_recv().await, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_channel_capacity() {
        assert_eq!(channel_capacity().await, 2);
    }

    #[tokio::test]
    async fn test_channel_as_stream() {
        assert_eq!(channel_as_stream().await, vec![0, 1, 2, 3, 4]);
    }
}
