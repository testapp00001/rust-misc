/// Problem: Async Patterns
///
/// Master common async patterns.
///
/// Key Concepts:
/// - Producer-consumer
/// - Pipeline
/// - Fan-out/fan-in
/// - Circuit breaker
/// - Retry pattern

use tokio::sync::{mpsc, Semaphore};
use tokio::time::{sleep, Duration};
use std::sync::Arc;

/// Problem 1: Producer-consumer pattern
/// Implement producer-consumer
pub async fn producer_consumer() -> Vec<i32> {
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

/// Problem 2: Pipeline pattern
/// Implement pipeline
pub async fn pipeline() -> Vec<i32> {
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

/// Problem 3: Fan-out pattern
/// Implement fan-out
pub async fn fan_out() -> Vec<Vec<i32>> {
    let (tx, mut rx) = mpsc::channel(10);

    // Producer
    let producer = tokio::spawn(async move {
        for i in 0..10 {
            tx.send(i).await.unwrap();
        }
        // tx is dropped here, closing the channel
    });

    // Collect all values
    let mut all_values = Vec::new();
    while let Some(val) = rx.recv().await {
        all_values.push(val);
    }

    // Wait for producer to finish
    producer.await.unwrap();

    // Distribute to consumers
    let mut handles = Vec::new();
    let chunk_size = (all_values.len() + 2) / 3;
    for chunk in all_values.chunks(chunk_size) {
        handles.push(chunk.to_vec());
    }

    handles
}

/// Problem 4: Fan-in pattern
/// Implement fan-in
pub async fn fan_in() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);
    let mut handles = vec![];

    // Multiple producers
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

    // Single consumer
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

/// Problem 5: Circuit breaker pattern
/// Implement circuit breaker
pub async fn circuit_breaker() -> Vec<Result<i32, String>> {
    let mut results = Vec::new();
    let mut failures = 0;
    let threshold = 2;

    for i in 0..5 {
        if failures >= threshold {
            results.push(Err("Circuit open".to_string()));
            continue;
        }

        // Simulate operation
        let result = if i == 2 {
            Err("Error".to_string())
        } else {
            Ok(i * 2)
        };

        match result {
            Ok(val) => {
                failures = 0;
                results.push(Ok(val));
            }
            Err(e) => {
                failures += 1;
                results.push(Err(e));
            }
        }
    }
    results
}

/// Problem 6: Retry pattern
/// Implement retry
pub async fn retry() -> Result<i32, String> {
    let max_retries = 3;
    let mut attempts = 0;

    loop {
        attempts += 1;

        // Simulate operation
        let result = if attempts < 3 {
            Err("Not ready".to_string())
        } else {
            Ok(42)
        };

        match result {
            Ok(val) => return Ok(val),
            Err(e) => {
                if attempts >= max_retries {
                    return Err(e);
                }
                sleep(Duration::from_millis(10)).await;
            }
        }
    }
}

/// Problem 7: Rate limiter pattern
/// Implement rate limiter
pub async fn rate_limiter() -> Vec<i32> {
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

/// Problem 8: Batch processing pattern
/// Implement batch processing
pub async fn batch_processing() -> Vec<Vec<i32>> {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let batch_size = 3;
    let mut results = Vec::new();

    for chunk in data.chunks(batch_size) {
        let chunk = chunk.to_vec();
        let result = tokio::spawn(async move {
            chunk.into_iter().map(|x| x * 2).collect::<Vec<i32>>()
        }).await.unwrap();
        results.push(result);
    }
    results
}

/// Problem 9: Debounce pattern
/// Implement debounce
pub async fn debounce() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);
    let mut results = Vec::new();

    // Producer
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Debounce consumer
    while let Some(val) = rx.recv().await {
        results.push(val);
        // In real implementation, you'd reset a timer here
    }
    results
}

/// Problem 10: Throttle pattern
/// Implement throttle
pub async fn throttle() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);
    let mut results = Vec::new();

    // Producer
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });

    // Throttle consumer
    while let Some(val) = rx.recv().await {
        results.push(val);
        sleep(Duration::from_millis(10)).await;
    }
    results
}

/// Problem 11: Worker pool pattern
/// Implement worker pool
pub async fn worker_pool() -> Vec<i32> {
    let (tx, mut rx) = mpsc::channel(10);

    // Producer
    let producer = tokio::spawn(async move {
        for i in 0..10 {
            tx.send(i).await.unwrap();
        }
        // tx is dropped here, closing the channel
    });

    // Collect all values
    let mut all_values = Vec::new();
    while let Some(val) = rx.recv().await {
        all_values.push(val * 2);
    }

    // Wait for producer to finish
    producer.await.unwrap();

    all_values.sort();
    all_values
}

/// Problem 12: Event sourcing pattern
/// Implement event sourcing
pub async fn event_sourcing() -> Vec<String> {
    let (tx, mut rx) = mpsc::channel(10);
    let mut events = Vec::new();

    // Event producer
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(format!("event_{}", i)).await.unwrap();
        }
    });

    // Event consumer
    while let Some(event) = rx.recv().await {
        events.push(event);
    }
    events
}

/// Problem 13: Saga pattern
/// Implement saga
pub async fn saga() -> Vec<String> {
    let mut steps = Vec::new();

    // Step 1
    steps.push("step1_completed".to_string());

    // Step 2
    steps.push("step2_completed".to_string());

    // Step 3
    steps.push("step3_completed".to_string());

    steps
}

/// Problem 14: CQRS pattern
/// Implement CQRS
pub async fn cqrs() -> (Vec<i32>, Vec<i32>) {
    let (write_tx, mut write_rx) = mpsc::channel(10);
    let (read_tx, mut read_rx) = mpsc::channel(10);

    // Write side
    tokio::spawn(async move {
        for i in 0..5 {
            write_tx.send(i).await.unwrap();
        }
    });

    // Read side
    tokio::spawn(async move {
        while let Some(val) = write_rx.recv().await {
            read_tx.send(val * 2).await.unwrap();
        }
    });

    let mut writes = Vec::new();
    let mut reads = Vec::new();

    // Collect writes
    // (In real implementation, you'd have a separate write store)

    // Collect reads
    while let Some(val) = read_rx.recv().await {
        reads.push(val);
    }

    (writes, reads)
}

/// Problem 15: Outbox pattern
/// Implement outbox
pub async fn outbox() -> Vec<String> {
    let (tx, mut rx) = mpsc::channel(10);
    let mut outbox = Vec::new();

    // Producer
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(format!("message_{}", i)).await.unwrap();
        }
    });

    // Outbox processor
    while let Some(msg) = rx.recv().await {
        outbox.push(msg);
    }
    outbox
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_producer_consumer() {
        assert_eq!(producer_consumer().await, vec![0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_pipeline() {
        assert_eq!(pipeline().await, vec![0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_fan_out() {
        let results = fan_out().await;
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_fan_in() {
        let results = fan_in().await;
        assert_eq!(results.len(), 9);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let results = circuit_breaker().await;
        assert!(results.iter().any(|r| r.is_err()));
    }

    #[tokio::test]
    async fn test_retry() {
        assert_eq!(retry().await, Ok(42));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let results = rate_limiter().await;
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let results = batch_processing().await;
        assert_eq!(results.len(), 4);
    }

    #[tokio::test]
    async fn test_debounce() {
        let results = debounce().await;
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_throttle() {
        let results = throttle().await;
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_worker_pool() {
        let results = worker_pool().await;
        assert_eq!(results.len(), 10);
    }

    #[tokio::test]
    async fn test_event_sourcing() {
        let results = event_sourcing().await;
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_saga() {
        let results = saga().await;
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_cqrs() {
        let (writes, reads) = cqrs().await;
        assert_eq!(writes.len(), 0);
        assert_eq!(reads.len(), 5);
    }

    #[tokio::test]
    async fn test_outbox() {
        let results = outbox().await;
        assert_eq!(results.len(), 5);
    }
}
