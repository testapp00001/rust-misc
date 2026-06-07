/// Problem: Producer Consumer
///
/// Master the producer-consumer pattern.
///
/// Key Concepts:
/// - Producer-consumer pattern
/// - Bounded channels
/// - Unbounded channels
/// - Multiple producers
/// - Multiple consumers

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Problem 1: Basic producer-consumer
/// Single producer, single consumer
pub fn basic_producer_consumer() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();

    // Producer
    thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
        }
    });

    // Consumer
    rx.iter().collect()
}

/// Problem 2: Multiple producers
/// Multiple producers, single consumer
pub fn multiple_producers() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    // Multiple producers
    for i in 0..3 {
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            for j in 0..3 {
                tx.send(i * 10 + j).unwrap();
            }
        });
        handles.push(handle);
    }

    drop(tx); // Drop original sender

    // Consumer
    let mut result: Vec<i32> = rx.iter().collect();
    result.sort();
    result
}

/// Problem 3: Bounded channel
/// Use bounded channel for backpressure
pub fn bounded_channel() -> Vec<i32> {
    let (tx, rx) = mpsc::sync_channel(2); // Buffer size 2

    // Producer
    thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
        }
    });

    // Consumer
    rx.iter().collect()
}

/// Problem 4: Producer with delay
/// Producer that sends with delay
pub fn producer_with_delay() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();

    // Producer with delay
    thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    });

    // Consumer
    rx.iter().collect()
}

/// Problem 5: Consumer with processing
/// Consumer that processes messages
pub fn consumer_with_processing() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();

    // Producer
    thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
        }
    });

    // Consumer with processing
    rx.iter().map(|x| x * 2).collect()
}

/// Problem 6: Producer-consumer with Result
/// Handle errors in producer-consumer
pub fn producer_consumer_result() -> Result<Vec<i32>, String> {
    let (tx, rx) = mpsc::channel();

    // Producer
    thread::spawn(move || {
        for i in 0..5 {
            if i == 3 {
                tx.send(Err("Error at 3".to_string())).unwrap();
            } else {
                tx.send(Ok(i)).unwrap();
            }
        }
    });

    // Consumer
    let results: Vec<Result<i32, String>> = rx.iter().collect();
    let mut values = Vec::new();
    for result in results {
        match result {
            Ok(v) => values.push(v),
            Err(e) => return Err(e),
        }
    }
    Ok(values)
}

/// Problem 7: Multiple consumers
/// Single producer, multiple consumers (simulated)
pub fn multiple_consumers() -> Vec<Vec<i32>> {
    let (tx, rx) = mpsc::channel();

    // Producer
    thread::spawn(move || {
        for i in 0..10 {
            tx.send(i).unwrap();
        }
    });

    // Collect all values
    let all_values: Vec<i32> = rx.iter().collect();

    // Distribute to consumers
    let chunk_size = (all_values.len() + 2) / 3;
    all_values.chunks(chunk_size).map(|c| c.to_vec()).collect()
}

/// Problem 8: Pipeline pattern
/// Chain of producers and consumers
pub fn pipeline_pattern() -> Vec<i32> {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    // Stage 1: Producer
    thread::spawn(move || {
        for i in 0..5 {
            tx1.send(i).unwrap();
        }
    });

    // Stage 2: Processor
    thread::spawn(move || {
        for val in rx1 {
            tx2.send(val * 2).unwrap();
        }
    });

    // Stage 3: Consumer
    rx2.iter().collect()
}

/// Problem 9: Producer-consumer with timeout
/// Handle timeouts
pub fn producer_consumer_timeout() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();

    // Producer
    thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    });

    // Consumer with timeout
    let mut result = Vec::new();
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(val) => result.push(val),
            Err(_) => break,
        }
    }
    result
}

/// Problem 10: Producer-consumer with backpressure
/// Handle backpressure with bounded channel
pub fn producer_consumer_backpressure() -> Vec<i32> {
    let (tx, rx) = mpsc::sync_channel(2); // Small buffer

    // Fast producer
    thread::spawn(move || {
        for i in 0..10 {
            tx.send(i).unwrap(); // Will block if buffer is full
        }
    });

    // Slow consumer
    let mut result = Vec::new();
    for val in rx {
        result.push(val);
        thread::sleep(Duration::from_millis(10)); // Slow processing
    }
    result
}

use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_producer_consumer() {
        assert_eq!(basic_producer_consumer(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_multiple_producers() {
        let result = multiple_producers();
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn test_bounded_channel() {
        assert_eq!(bounded_channel(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_producer_with_delay() {
        assert_eq!(producer_with_delay(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_consumer_with_processing() {
        assert_eq!(consumer_with_processing(), vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_producer_consumer_result() {
        assert!(producer_consumer_result().is_err());
    }

    #[test]
    fn test_multiple_consumers() {
        let result = multiple_consumers();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_pipeline_pattern() {
        assert_eq!(pipeline_pattern(), vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_producer_consumer_timeout() {
        assert_eq!(producer_consumer_timeout(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_producer_consumer_backpressure() {
        assert_eq!(producer_consumer_backpressure(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
