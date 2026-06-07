/// Problem: Message Passing
///
/// Master Rust's message passing with channels.
///
/// Key Concepts:
/// - mpsc channels
/// - Sender and Receiver
/// - Multiple producers
/// - Channel types
/// - Error handling

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Problem 1: Basic channel
/// Send and receive a message
pub fn basic_channel() -> i32 {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(42).unwrap();
    });

    rx.recv().unwrap()
}

/// Problem 2: Multiple messages
/// Send multiple messages
pub fn multiple_messages() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
        }
    });

    rx.iter().collect()
}

/// Problem 3: Multiple producers
/// Use multiple senders
pub fn multiple_producers() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    for i in 0..3 {
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            tx.send(i * 10).unwrap();
        });
        handles.push(handle);
    }

    drop(tx); // Drop original sender

    handles.into_iter().for_each(|h| h.join().unwrap());
    rx.iter().collect()
}

/// Problem 4: Channel with timeout
/// Use recv_timeout
pub fn channel_with_timeout() -> Option<i32> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        tx.send(42).unwrap();
    });

    rx.recv_timeout(Duration::from_millis(100)).ok()
}

/// Problem 5: Channel with try_recv
/// Non-blocking receive
pub fn channel_try_recv() -> Option<i32> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        tx.send(42).unwrap();
    });

    thread::sleep(Duration::from_millis(50));
    rx.try_recv().ok()
}

/// Problem 6: Channel with strings
/// Send strings through channels
pub fn channel_strings() -> String {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send("hello".to_string()).unwrap();
    });

    rx.recv().unwrap()
}

/// Problem 7: Channel with Result
/// Send Results through channels
pub fn channel_results() -> Result<i32, String> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(Ok(42)).unwrap();
    });

    rx.recv().unwrap()
}

/// Problem 8: Channel with Option
/// Send Options through channels
pub fn channel_options() -> Option<i32> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(Some(42)).unwrap();
    });

    rx.recv().unwrap()
}

/// Problem 9: Channel with custom type
/// Send custom types through channels
#[derive(Debug, PartialEq)]
pub struct Message {
    pub id: u32,
    pub content: String,
}

pub fn channel_custom_type() -> Message {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(Message {
            id: 1,
            content: "hello".to_string(),
        }).unwrap();
    });

    rx.recv().unwrap()
}

/// Problem 10: Channel with multiple receivers
/// Use channel for broadcast (simulated with single receiver)
pub fn channel_broadcast() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();

    // Sender
    thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
        }
    });

    // Single receiver collects all values
    rx.iter().collect()
}

/// Problem 11: Channel with sync_channel
/// Use bounded channel
pub fn sync_channel() -> Vec<i32> {
    let (tx, rx) = mpsc::sync_channel(2); // Buffer size 2

    thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
        }
    });

    rx.iter().collect()
}

/// Problem 12: Channel with error handling
/// Handle channel errors
pub fn channel_error_handling() -> Result<i32, String> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        // Sender is dropped without sending
        drop(tx);
    });

    rx.recv().map_err(|_| "Channel closed".to_string())
}

/// Problem 13: Channel with multiple senders and receivers
/// Complex channel pattern
pub fn channel_complex() -> Vec<i32> {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    // Sender 1
    thread::spawn(move || {
        for i in 0..3 {
            tx1.send(i).unwrap();
        }
    });

    // Sender 2
    thread::spawn(move || {
        for i in 10..13 {
            tx2.send(i).unwrap();
        }
    });

    // Collect from both channels
    let mut result: Vec<i32> = rx1.iter().collect();
    result.extend(rx2.iter());
    result
}

/// Problem 14: Channel with transformation
/// Transform messages in a pipeline
pub fn channel_pipeline() -> Vec<i32> {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    // Stage 1: Send numbers
    thread::spawn(move || {
        for i in 0..5 {
            tx1.send(i).unwrap();
        }
    });

    // Stage 2: Transform
    thread::spawn(move || {
        for val in rx1 {
            tx2.send(val * 2).unwrap();
        }
    });

    rx2.iter().collect()
}

/// Problem 15: Channel with select
/// Use select for multiple channels (simulated)
pub fn channel_select() -> Vec<i32> {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    thread::spawn(move || {
        tx1.send(1).unwrap();
        tx1.send(2).unwrap();
    });

    thread::spawn(move || {
        tx2.send(10).unwrap();
        tx2.send(20).unwrap();
    });

    let mut result = Vec::new();
    result.extend(rx1.iter());
    result.extend(rx2.iter());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_channel() {
        assert_eq!(basic_channel(), 42);
    }

    #[test]
    fn test_multiple_messages() {
        assert_eq!(multiple_messages(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_multiple_producers() {
        let result = multiple_producers();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_channel_with_timeout() {
        assert_eq!(channel_with_timeout(), Some(42));
    }

    #[test]
    fn test_channel_try_recv() {
        assert_eq!(channel_try_recv(), Some(42));
    }

    #[test]
    fn test_channel_strings() {
        assert_eq!(channel_strings(), "hello");
    }

    #[test]
    fn test_channel_results() {
        assert_eq!(channel_results(), Ok(42));
    }

    #[test]
    fn test_channel_options() {
        assert_eq!(channel_options(), Some(42));
    }

    #[test]
    fn test_channel_custom_type() {
        let msg = channel_custom_type();
        assert_eq!(msg.id, 1);
        assert_eq!(msg.content, "hello");
    }

    #[test]
    fn test_channel_broadcast() {
        let result = channel_broadcast();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_sync_channel() {
        assert_eq!(sync_channel(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_channel_error_handling() {
        assert!(channel_error_handling().is_err());
    }

    #[test]
    fn test_channel_complex() {
        let result = channel_complex();
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn test_channel_pipeline() {
        assert_eq!(channel_pipeline(), vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_channel_select() {
        let result = channel_select();
        assert_eq!(result.len(), 4);
    }
}
