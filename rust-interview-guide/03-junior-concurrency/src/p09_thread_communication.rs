/// Problem: Thread Communication
///
/// Master advanced thread communication patterns.
///
/// Key Concepts:
/// - Channels
/// - Shared state
/// - Signaling
/// - Synchronization
/// - Event-driven patterns

use std::sync::{Arc, Mutex, mpsc, Barrier, Condvar};
use std::thread;
use std::time::Duration;

/// Problem 1: Request-response pattern
/// Implement request-response with channels
pub fn request_response() -> Vec<i32> {
    let (req_tx, req_rx) = mpsc::channel();
    let (res_tx, res_rx) = mpsc::channel();

    // Server
    thread::spawn(move || {
        for req in req_rx {
            let response = req * 2;
            res_tx.send(response).unwrap();
        }
    });

    // Client
    let mut results = Vec::new();
    for i in 0..5 {
        req_tx.send(i).unwrap();
        let response = res_rx.recv().unwrap();
        results.push(response);
    }

    results
}

/// Problem 2: Broadcast pattern
/// Broadcast message to multiple receivers (simulated)
pub fn broadcast() -> Vec<Vec<i32>> {
    let (tx, rx) = mpsc::channel();

    // Sender
    thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
        }
    });

    // Collect all values
    let all_values: Vec<i32> = rx.iter().collect();

    // Distribute to receivers
    let chunk_size = (all_values.len() + 2) / 3;
    all_values.chunks(chunk_size).map(|c| c.to_vec()).collect()
}

/// Problem 3: Select pattern (simulated)
/// Simulate select on multiple channels
pub fn select_pattern() -> Vec<i32> {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    thread::spawn(move || {
        for i in 0..3 {
            tx1.send(i).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    });

    thread::spawn(move || {
        for i in 10..13 {
            tx2.send(i).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    });

    let mut results = Vec::new();
    results.extend(rx1.iter());
    results.extend(rx2.iter());
    results
}

/// Problem 4: Barrier synchronization
/// Synchronize threads at a barrier
pub fn barrier_sync() -> Vec<i32> {
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = vec![];

    for i in 0..3 {
        let barrier = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            // Do some work
            let result = i * 10;

            // Wait for all threads
            barrier.wait();

            result
        });
        handles.push(handle);
    }

    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

/// Problem 5: Condition variable signaling
/// Signal between threads with Condvar
pub fn condvar_signal() -> i32 {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair_clone = Arc::clone(&pair);

    let handle = thread::spawn(move || {
        let (lock, cvar) = &*pair_clone;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
    });

    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }

    handle.join().unwrap();
    42
}

/// Problem 6: Event-driven pattern
/// Implement event-driven communication
pub fn event_driven() -> Vec<String> {
    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    // Event producer
    let tx_clone = tx.clone();
    handles.push(thread::spawn(move || {
        for i in 0..3 {
            tx_clone.send(format!("event_{}", i)).unwrap();
        }
    }));

    // Another event producer
    handles.push(thread::spawn(move || {
        for i in 0..3 {
            tx.send(format!("message_{}", i)).unwrap();
        }
    }));
    // tx is moved here, so the channel will close when both producers finish

    // Event consumer
    let mut results = Vec::new();
    for event in rx {
        results.push(event);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    results
}

/// Problem 7: Pipeline with feedback
/// Pipeline with feedback loop
pub fn pipeline_feedback() -> Vec<i32> {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let (tx3, rx3) = mpsc::channel();

    // Stage 1: Generate
    thread::spawn(move || {
        for i in 0..5 {
            tx1.send(i).unwrap();
        }
    });

    // Stage 2: Process
    thread::spawn(move || {
        for val in rx1 {
            tx2.send(val * 2).unwrap();
        }
    });

    // Stage 3: Collect
    thread::spawn(move || {
        for val in rx2 {
            tx3.send(val + 1).unwrap();
        }
    });

    rx3.iter().collect()
}

/// Problem 8: Shared event bus
/// Implement a shared event bus
pub fn shared_event_bus() -> Vec<String> {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    // Producers
    for i in 0..3 {
        let events = Arc::clone(&events);
        let handle = thread::spawn(move || {
            let mut events = events.lock().unwrap();
            events.push(format!("event_{}", i));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

let result =     events.lock().unwrap().clone(); result
}

/// Problem 9: Thread signaling
/// Signal threads to stop
pub fn thread_signaling() -> Vec<i32> {
    let running = Arc::new(Mutex::new(true));
    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    // Worker thread
    let running_clone = Arc::clone(&running);
    let handle = thread::spawn(move || {
        let mut i = 0;
        loop {
            {
                let running = running_clone.lock().unwrap();
                if !*running {
                    break;
                }
            }
            tx.send(i).unwrap();
            i += 1;
            thread::sleep(Duration::from_millis(10));
        }
    });
    handles.push(handle);

    // Let it run for a bit
    thread::sleep(Duration::from_millis(50));

    // Signal to stop
    {
        let mut running = running.lock().unwrap();
        *running = false;
    }

    for handle in handles {
        handle.join().unwrap();
    }

    rx.iter().collect()
}

/// Problem 10: Thread coordination
/// Coordinate multiple threads
pub fn thread_coordination() -> i32 {
    let result = Arc::new(Mutex::new(0));
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = vec![];

    for i in 0..3 {
        let result = Arc::clone(&result);
        let barrier = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            // Do some work
            let contribution = i * 10;

            // Wait for all threads
            barrier.wait();

            // Combine results
            let mut result = result.lock().unwrap();
            *result += contribution;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = *result.lock().unwrap(); result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_response() {
        assert_eq!(request_response(), vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_broadcast() {
        let results = broadcast();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_select_pattern() {
        let results = select_pattern();
        assert_eq!(results.len(), 6);
    }

    #[test]
    fn test_barrier_sync() {
        let results = barrier_sync();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_condvar_signal() {
        assert_eq!(condvar_signal(), 42);
    }

    #[test]
    fn test_event_driven() {
        let results = event_driven();
        assert_eq!(results.len(), 6);
    }

    #[test]
    fn test_pipeline_feedback() {
        assert_eq!(pipeline_feedback(), vec![1, 3, 5, 7, 9]);
    }

    #[test]
    fn test_shared_event_bus() {
        let results = shared_event_bus();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_thread_signaling() {
        let results = thread_signaling();
        assert!(results.len() > 0);
    }

    #[test]
    fn test_thread_coordination() {
        assert_eq!(thread_coordination(), 30); // 0 + 10 + 20
    }
}
