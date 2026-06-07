//! # Channel Patterns for Concurrent Communication
//!
//! Channels are the primary mechanism for message-passing concurrency in Rust.
//! They provide safe, typed communication between threads without shared state.
//!
//! ## Channel Types:
//!
//! | Type | Crate | Use Case |
//! |------|-------|----------|
//! | `mpsc::channel` | std | Single-producer, single-consumer |
//! | `mpsc::sync_channel` | std | Bounded single-producer, single-consumer |
//! | `crossbeam::channel` | crossbeam | Multi-producer, multi-consumer with select |
//! | `flume` | flpm | Async/sync hybrid channels |
//!
//! ## Patterns:
//!
//! - **Fan-out**: One producer, multiple consumers
//! - **Fan-in**: Multiple producers, one consumer
//! - **Pipeline**: Chain of stages connected by channels
//! - **Request-Response**: Pairs of channels for RPC-style communication
//! - **Select**: Wait on multiple channels simultaneously

use crossbeam::channel::{self, Receiver, Sender};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

/// A message that can be sent through a pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineMessage {
    Data(Vec<u8>),
    Flush,
    Shutdown,
}

/// Pipeline stage that processes messages and forwards them.
pub struct PipelineStage<I, O> {
    pub name: String,
    pub handler: Box<dyn Fn(I) -> Option<O> + Send + 'static>,
}

/// Builder for constructing message processing pipelines.
pub struct PipelineBuilder {
    stages: Vec<Box<dyn Fn(PipelineMessage) -> Option<PipelineMessage> + Send>>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn add_stage<F>(mut self, name: &str, handler: F) -> Self
    where
        F: Fn(PipelineMessage) -> Option<PipelineMessage> + Send + 'static,
    {
        self.stages.push(Box::new(handler));
        self
    }

    /// Build and run the pipeline, returning (input_sender, output_receiver).
    pub fn build(self) -> (Sender<PipelineMessage>, Receiver<PipelineMessage>) {
        let (input_tx, input_rx) = channel::unbounded();
        let (output_tx, output_rx) = channel::unbounded();

        thread::spawn(move || {
            let mut current_rx = input_rx;

            for stage in self.stages {
                let (next_tx, next_rx) = channel::unbounded();
                let handler = stage;

                let rx = current_rx;
                thread::spawn(move || {
                    for msg in rx.iter() {
                        if let Some(result) = handler(msg) {
                            if next_tx.send(result).is_err() {
                                break;
                            }
                        }
                    }
                });

                current_rx = next_rx;
            }

            // Forward final output
            for msg in current_rx.iter() {
                if output_tx.send(msg).is_err() {
                    break;
                }
            }
        });

        (input_tx, output_rx)
    }
}

/// Request-response channel pair for RPC-style communication.
pub struct RequestResponse<Req, Resp> {
    pub request_tx: Sender<(Req, Sender<Resp>)>,
    pub request_rx: Receiver<(Req, Sender<Resp>)>,
}

impl<Req: Send + 'static, Resp: Send + 'static> RequestResponse<Req, Resp> {
    pub fn new() -> Self {
        let (tx, rx) = channel::unbounded();
        Self {
            request_tx: tx,
            request_rx: rx,
        }
    }

    /// Create a client handle that sends requests and waits for responses.
    pub fn client(&self) -> RequestClient<Req, Resp> {
        RequestClient {
            tx: self.request_tx.clone(),
        }
    }

    /// Create a server handle that processes requests.
    pub fn server(&self) -> RequestServer<'_, Req, Resp> {
        RequestServer {
            rx: &self.request_rx,
        }
    }
}

pub struct RequestClient<Req, Resp> {
    tx: Sender<(Req, Sender<Resp>)>,
}

impl<Req: Send, Resp: Send> RequestClient<Req, Resp> {
    /// Send a request and wait for the response.
    pub fn call(&self, request: Req) -> Result<Resp, String> {
        let (resp_tx, resp_rx) = channel::unbounded();
        self.tx
            .send((request, resp_tx))
            .map_err(|_| "Channel closed".to_string())?;
        resp_rx
            .recv()
            .map_err(|_| "Response channel closed".to_string())
    }
}

pub struct RequestServer<'a, Req, Resp> {
    rx: &'a Receiver<(Req, Sender<Resp>)>,
}

impl<'a, Req: Send, Resp: Send> RequestServer<'a, Req, Resp> {
    /// Process incoming requests with the given handler.
    pub fn serve<F>(&self, handler: F)
    where
        F: Fn(Req) -> Resp,
    {
        for (request, response_tx) in self.rx.iter() {
            let response = handler(request);
            let _ = response_tx.send(response);
        }
    }
}

/// Fan-out pattern: distribute work across multiple workers.
pub fn fan_out<T: Send + 'static>(
    receiver: Receiver<T>,
    num_workers: usize,
) -> Vec<Receiver<T>> {
    // Create channels for each worker
    let mut senders = Vec::with_capacity(num_workers);
    let mut receivers = Vec::with_capacity(num_workers);
    for _ in 0..num_workers {
        let (tx, rx) = channel::unbounded();
        senders.push(tx);
        receivers.push(rx);
    }

    // Round-robin distribution
    thread::spawn(move || {
        for (i, item) in receiver.iter().enumerate() {
            let worker = i % num_workers;
            if senders[worker].send(item).is_err() {
                break;
            }
        }
    });

    receivers
}

/// Fan-in pattern: merge multiple channels into one.
pub fn fan_in<T: Send + 'static>(receivers: Vec<Receiver<T>>) -> Receiver<T> {
    let (output_tx, output_rx) = channel::unbounded();

    for rx in receivers {
        let tx = output_tx.clone();
        thread::spawn(move || {
            for item in rx.iter() {
                if tx.send(item).is_err() {
                    break;
                }
            }
        });
    }

    output_rx
}

/// Channel-based timer that sends periodic ticks.
pub fn ticker(duration: Duration) -> Receiver<()> {
    let (tx, rx) = channel::unbounded();
    thread::spawn(move || {
        loop {
            thread::sleep(duration);
            if tx.send(()).is_err() {
                break;
            }
        }
    });
    rx
}

/// Rate limiter using a channel-based token bucket.
pub struct TokenBucket {
    rx: Receiver<()>,
}

impl TokenBucket {
    pub fn new(tokens_per_second: u32) -> Self {
        let (tx, rx) = channel::bounded(tokens_per_second as usize);

        thread::spawn(move || {
            let interval = Duration::from_secs_f64(1.0 / tokens_per_second as f64);
            loop {
                thread::sleep(interval);
                if tx.send(()).is_err() {
                    break;
                }
            }
        });

        Self { rx }
    }

    /// Acquire a token, blocking until one is available.
    pub fn acquire(&self) -> Result<(), String> {
        self.rx.recv().map_err(|_| "Token bucket closed".to_string())
    }

    /// Try to acquire a token without blocking.
    pub fn try_acquire(&self) -> bool {
        self.rx.try_recv().is_ok()
    }
}

/// Channel-based event bus for publish-subscribe.
pub struct EventBus<T: Clone + Send + 'static> {
    subscribers: HashMap<String, Vec<Sender<T>>>,
}

impl<T: Clone + Send + 'static> EventBus<T> {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }

    /// Subscribe to a topic, returning a receiver.
    pub fn subscribe(&mut self, topic: &str) -> Receiver<T> {
        let (tx, rx) = channel::unbounded();
        self.subscribers
            .entry(topic.to_string())
            .or_insert_with(Vec::new)
            .push(tx);
        rx
    }

    /// Publish a message to a topic.
    pub fn publish(&self, topic: &str, message: T) {
        if let Some(subscribers) = self.subscribers.get(topic) {
            for subscriber in subscribers {
                let _ = subscriber.send(message.clone());
            }
        }
    }

    pub fn subscriber_count(&self, topic: &str) -> usize {
        self.subscribers
            .get(topic)
            .map(|s| s.len())
            .unwrap_or(0)
    }
}

/// Bounded channel with backpressure logging.
pub fn bounded_with_backpressure<T: Send + 'static>(
    capacity: usize,
    _name: &str,
) -> (Sender<T>, Receiver<T>, Receiver<u64>) {
    let (tx, rx) = channel::bounded(capacity);
    let (backpressure_tx, backpressure_rx) = channel::unbounded();

    // Simple wrapper that forwards messages
    let (inner_tx, inner_rx) = channel::bounded::<T>(capacity);

    thread::spawn(move || {
        for item in inner_rx.iter() {
            if tx.send(item).is_err() {
                break;
            }
            let _ = backpressure_tx.send(1);
        }
    });

    (inner_tx, rx, backpressure_rx)
}

/// Worker pool that processes items from a channel.
pub struct WorkerPool<T: Send + 'static> {
    handles: Vec<thread::JoinHandle<()>>,
    work_tx: Sender<T>,
}

impl<T: Send + 'static> WorkerPool<T> {
    pub fn new<F>(num_workers: usize, handler: F) -> Self
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        let (tx, rx) = channel::unbounded();
        let handler = std::sync::Arc::new(handler);

        let mut handles = Vec::new();
        for i in 0..num_workers {
            let rx = rx.clone();
            let handler = std::sync::Arc::clone(&handler);
            let handle = thread::Builder::new()
                .name(format!("worker-{}", i))
                .spawn(move || {
                    for item in rx.iter() {
                        handler(item);
                    }
                })
                .expect("Failed to spawn worker");
            handles.push(handle);
        }

        Self {
            handles,
            work_tx: tx,
        }
    }

    pub fn submit(&self, work: T) -> Result<(), String> {
        self.work_tx
            .send(work)
            .map_err(|_| "Worker pool shut down".to_string())
    }

    pub fn shutdown(self) {
        drop(self.work_tx);
        for handle in self.handles {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel;

    #[test]
    fn test_basic_channel_communication() {
        let (tx, rx) = channel::unbounded();
        tx.send("hello").unwrap();
        tx.send("world").unwrap();

        assert_eq!(rx.recv().unwrap(), "hello");
        assert_eq!(rx.recv().unwrap(), "world");
    }

    #[test]
    fn test_bounded_channel_backpressure() {
        let (tx, rx) = channel::bounded(2);
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        // Third send would block (we use try_send to test)
        assert!(tx.try_send(3).is_err());

        assert_eq!(rx.recv().unwrap(), 1);
        assert!(tx.try_send(3).is_ok()); // Now there's space
    }

    #[test]
    fn test_crossbeam_select() {
        let (tx1, rx1) = channel::unbounded();
        let (tx2, rx2) = channel::unbounded();

        tx1.send("from channel 1").unwrap();
        tx2.send("from channel 2").unwrap();

        let mut received = Vec::new();
        crossbeam::channel::select! {
            recv(rx1) -> msg => received.push(msg.unwrap()),
            recv(rx2) -> msg => received.push(msg.unwrap()),
        }
        // One message received via select
        assert_eq!(received.len(), 1);
    }

    #[test]
    fn test_pipeline_builder() {
        let (tx, rx) = PipelineBuilder::new()
            .add_stage("double", |msg| match msg {
                PipelineMessage::Data(mut data) => {
                    data.extend_from_slice(&data.clone());
                    Some(PipelineMessage::Data(data))
                }
                other => Some(other),
            })
            .build();

        tx.send(PipelineMessage::Data(vec![1, 2, 3])).unwrap();
        let result = rx.recv().unwrap();
        match result {
            PipelineMessage::Data(data) => assert_eq!(data, vec![1, 2, 3, 1, 2, 3]),
            _ => panic!("Expected Data message"),
        }
    }

    #[test]
    fn test_pipeline_shutdown() {
        let (tx, rx) = PipelineBuilder::new().build();
        tx.send(PipelineMessage::Shutdown).unwrap();
        let result = rx.recv().unwrap();
        assert_eq!(result, PipelineMessage::Shutdown);
    }

    #[test]
    fn test_request_response() {
        let rr = RequestResponse::<String, String>::new();
        let client = rr.client();

        // Destructure rr: keep receiver for server, drop rr's sender so only
        // client's clone remains. Channel closes when client is dropped.
        let RequestResponse { request_tx, request_rx } = rr;
        drop(request_tx);

        let server_handle = thread::spawn(move || {
            for (request, response_tx) in request_rx.iter() {
                let response = format!("Response to: {}", request);
                let _ = response_tx.send(response);
            }
        });

        let response = client.call("test request".into()).unwrap();
        assert_eq!(response, "Response to: test request");

        drop(client);
        server_handle.join().unwrap();
    }

    #[test]
    fn test_fan_in_pattern() {
        let (tx1, rx1) = channel::unbounded();
        let (tx2, rx2) = channel::unbounded();
        let (tx3, rx3) = channel::unbounded();

        let merged = fan_in(vec![rx1, rx2, rx3]);

        tx1.send(1).unwrap();
        tx2.send(2).unwrap();
        tx3.send(3).unwrap();

        drop(tx1);
        drop(tx2);
        drop(tx3);

        let mut results: Vec<i32> = merged.iter().collect();
        results.sort();
        assert_eq!(results, vec![1, 2, 3]);
    }

    #[test]
    fn test_ticker() {
        let rx = ticker(Duration::from_millis(10));
        // Should receive at least one tick within 50ms
        let result = rx.recv_timeout(Duration::from_millis(50));
        assert!(result.is_ok());
    }

    #[test]
    fn test_token_bucket() {
        let bucket = TokenBucket::new(100); // 100 tokens/sec
        // Should be able to acquire immediately (pre-filled)
        thread::sleep(Duration::from_millis(50));
        assert!(bucket.try_acquire());
    }

    #[test]
    fn test_event_bus() {
        let mut bus = EventBus::new();
        let rx1 = bus.subscribe("topic1");
        let rx2 = bus.subscribe("topic1");
        let rx3 = bus.subscribe("topic2");

        assert_eq!(bus.subscriber_count("topic1"), 2);
        assert_eq!(bus.subscriber_count("topic2"), 1);

        bus.publish("topic1", "message for topic1");
        assert_eq!(rx1.recv().unwrap(), "message for topic1");
        assert_eq!(rx2.recv().unwrap(), "message for topic1");

        // topic2 subscriber should not receive topic1 messages
        assert!(rx3.try_recv().is_err());
    }

    #[test]
    fn test_worker_pool() {
        let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let results_clone = std::sync::Arc::clone(&results);

        let pool = WorkerPool::new(4, move |item: i32| {
            results_clone.lock().unwrap().push(item);
        });

        for i in 0..100 {
            pool.submit(i).unwrap();
        }

        pool.shutdown();

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 100);
    }

    #[test]
    fn test_worker_pool_ordering_not_guaranteed() {
        let pool = WorkerPool::new(4, |_item: i32| {
            // Processing takes variable time
        });

        for i in 0..10 {
            pool.submit(i).unwrap();
        }

        pool.shutdown();
        // No assertion on order - work stealing means order is not guaranteed
    }

    #[test]
    fn test_channel_iter_drain() {
        let (tx, rx) = channel::unbounded();
        for i in 0..5 {
            tx.send(i).unwrap();
        }
        drop(tx);

        let results: Vec<i32> = rx.iter().collect();
        assert_eq!(results, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_channel_try_recv_non_blocking() {
        let (tx, rx) = channel::unbounded::<i32>();
        assert!(rx.try_recv().is_err()); // Empty channel

        tx.send(42).unwrap();
        assert_eq!(rx.try_recv().unwrap(), 42);
        assert!(rx.try_recv().is_err()); // Drained
    }
}
