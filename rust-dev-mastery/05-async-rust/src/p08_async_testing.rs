//! # Async Testing
//!
//! Testing async code requires special patterns: `#[tokio::test]` for test harnesses,
//! time simulation for deterministic timing tests, and utilities for managing
//! concurrent test scenarios.
//!
//! ## Key Concepts
//! - **#[tokio::test]**: Macro that sets up a Tokio runtime for each test
//! - **tokio::time::pause()**: Simulates time without real sleeping
//! - **Test utilities**: Helper functions for common async test patterns
//! - **Deterministic testing**: Controlling time and ordering in tests

use std::time::Duration;

/// A simple async service for demonstrating test patterns.
pub struct CounterService {
    count: tokio::sync::RwLock<u64>,
    events: tokio::sync::mpsc::Sender<CounterEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CounterEvent {
    Incremented(u64),
    Reset,
}

impl CounterService {
    pub fn new() -> (Self, tokio::sync::mpsc::Receiver<CounterEvent>) {
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        (
            CounterService {
                count: tokio::sync::RwLock::new(0),
                events: tx,
            },
            rx,
        )
    }

    pub async fn increment(&self) -> u64 {
        let mut count = self.count.write().await;
        *count += 1;
        let value = *count;
        let _ = self.events.send(CounterEvent::Incremented(value)).await;
        value
    }

    pub async fn get(&self) -> u64 {
        *self.count.read().await
    }

    pub async fn reset(&self) {
        let mut count = self.count.write().await;
        *count = 0;
        let _ = self.events.send(CounterEvent::Reset).await;
    }
}

/// An async rate limiter that allows N requests per time window.
pub struct RateLimiter {
    max_requests: u32,
    window: Duration,
    timestamps: tokio::sync::Mutex<Vec<tokio::time::Instant>>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        RateLimiter {
            max_requests,
            window,
            timestamps: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn try_acquire(&self) -> bool {
        let now = tokio::time::Instant::now();
        let mut timestamps = self.timestamps.lock().await;

        // Remove expired timestamps
        timestamps.retain(|ts| now - *ts < self.window);

        if timestamps.len() < self.max_requests as usize {
            timestamps.push(now);
            true
        } else {
            false
        }
    }
}

/// A scheduled task runner that executes closures at fixed intervals.
pub struct Scheduler {
    tasks: Vec<ScheduledTask>,
}

struct ScheduledTask {
    name: String,
    interval: Duration,
    last_run: Option<tokio::time::Instant>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, name: impl Into<String>, interval: Duration) {
        self.tasks.push(ScheduledTask {
            name: name.into(),
            interval,
            last_run: None,
        });
    }

    /// Runs the scheduler for a specified duration, returning task names and execution counts.
    pub async fn run_for(&mut self, duration: Duration) -> Vec<(String, u32)> {
        let start = tokio::time::Instant::now();
        let mut counts: Vec<(String, u32)> = self
            .tasks
            .iter()
            .map(|t| (t.name.clone(), 0))
            .collect();

        loop {
            if tokio::time::Instant::now() - start >= duration {
                break;
            }

            let now = tokio::time::Instant::now();
            for (i, task) in self.tasks.iter_mut().enumerate() {
                let should_run = match task.last_run {
                    None => true,
                    Some(last) => now - last >= task.interval,
                };

                if should_run {
                    task.last_run = Some(now);
                    counts[i].1 += 1;
                }
            }

            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        counts
    }
}

/// Test utility: wait for a condition with timeout.
/// Returns Ok(()) when condition is true, Err if timeout expires.
pub async fn wait_for<F>(timeout: Duration, mut condition: F) -> Result<(), String>
where
    F: FnMut() -> bool,
{
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        if condition() {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(format!("Condition not met within {timeout:?}"));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Test utility: collect events from a channel within a timeout.
pub async fn collect_events<T>(
    rx: &mut tokio::sync::mpsc::Receiver<T>,
    timeout: Duration,
) -> Vec<T> {
    let mut events = Vec::new();
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }

        match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Some(event)) => events.push(event),
            _ => break,
        }
    }

    events
}

/// A mock HTTP client for testing async request handlers.
#[derive(Clone)]
pub struct MockHttpClient {
    responses: std::sync::Arc<tokio::sync::Mutex<Vec<(String, Result<String, String>)>>>,
    requests: std::sync::Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        MockHttpClient {
            responses: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
            requests: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    pub async fn enqueue_response(&self, url: &str, response: Result<String, String>) {
        self.responses
            .lock()
            .await
            .push((url.to_string(), response));
    }

    pub async fn get(&self, url: &str) -> Result<String, String> {
        self.requests.lock().await.push(url.to_string());

        let mut responses = self.responses.lock().await;
        if let Some(idx) = responses.iter().position(|(u, _)| u == url) {
            let (_, response) = responses.remove(idx);
            response
        } else {
            Err(format!("No mock response for {url}"))
        }
    }

    pub async fn request_count(&self) -> usize {
        self.requests.lock().await.len()
    }

    pub async fn requested_urls(&self) -> Vec<String> {
        self.requests.lock().await.clone()
    }
}

/// A retry handler with exponential backoff for testing.
pub async fn retry_handler<F, Fut, T, E>(
    max_attempts: u32,
    base_delay: Duration,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut delay = base_delay;

    for attempt in 0..max_attempts {
        match operation().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                if attempt + 1 == max_attempts {
                    return Err(e);
                }
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_counter_service_basic() {
        let (service, _rx) = CounterService::new();
        assert_eq!(service.get().await, 0);

        service.increment().await;
        service.increment().await;
        assert_eq!(service.get().await, 2);
    }

    #[tokio::test]
    async fn test_counter_service_reset() {
        let (service, _rx) = CounterService::new();
        service.increment().await;
        service.increment().await;
        service.reset().await;
        assert_eq!(service.get().await, 0);
    }

    #[tokio::test]
    async fn test_counter_service_events() {
        let (service, mut rx) = CounterService::new();

        service.increment().await;
        service.increment().await;

        let events = collect_events(&mut rx, Duration::from_millis(100)).await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], CounterEvent::Incremented(1));
        assert_eq!(events[1], CounterEvent::Incremented(2));
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(3, Duration::from_secs(1));

        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        assert!(!limiter.try_acquire().await); // Exceeds limit
    }

    #[tokio::test]
    async fn test_rate_limiter_recovers() {
        let limiter = RateLimiter::new(2, Duration::from_millis(50));

        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        assert!(!limiter.try_acquire().await);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn test_wait_for_condition_met() {
        let value = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let v = value.clone();

        // Set value after a delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            v.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        let result = wait_for(Duration::from_secs(1), || {
            value.load(std::sync::atomic::Ordering::SeqCst)
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_wait_for_timeout() {
        let result = wait_for(Duration::from_millis(50), || false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not met"));
    }

    #[tokio::test]
    async fn test_mock_http_client() {
        let client = MockHttpClient::new();

        client
            .enqueue_response("https://api.example.com/users", Ok("users data".into()))
            .await;
        client
            .enqueue_response(
                "https://api.example.com/orders",
                Err("server error".into()),
            )
            .await;

        let result = client.get("https://api.example.com/users").await;
        assert_eq!(result, Ok("users data".into()));

        let result = client.get("https://api.example.com/orders").await;
        assert_eq!(result, Err("server error".into()));

        assert_eq!(client.request_count().await, 2);
    }

    #[tokio::test]
    async fn test_mock_http_no_response() {
        let client = MockHttpClient::new();
        let result = client.get("unknown").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_handler_success_first() {
        let result = retry_handler(3, Duration::from_millis(1), || async {
            Ok::<_, String>(42)
        })
        .await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_retry_handler_eventual_success() {
        let attempt = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let a = attempt.clone();

        let result = retry_handler(3, Duration::from_millis(1), move || {
            let a = a.clone();
            async move {
                let n = a.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if n < 2 {
                    Err(format!("fail {n}"))
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_scheduler() {
        let mut scheduler = Scheduler::new();
        scheduler.add_task("fast", Duration::from_millis(10));
        scheduler.add_task("slow", Duration::from_millis(50));

        let counts = scheduler.run_for(Duration::from_millis(100)).await;

        let fast_count = counts.iter().find(|(n, _)| n == "fast").unwrap().1;
        let slow_count = counts.iter().find(|(n, _)| n == "slow").unwrap().1;

        assert!(fast_count > slow_count);
        assert!(fast_count > 5);
    }

    #[tokio::test]
    async fn test_concurrent_counter() {
        let (service, mut rx) = CounterService::new();
        let service = std::sync::Arc::new(service);

        // Drain events in background to prevent channel from filling up
        let drainer = tokio::spawn(async move {
            while rx.recv().await.is_some() {}
        });

        let mut handles = Vec::new();
        for _ in 0..10 {
            let svc = service.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..100 {
                    svc.increment().await;
                }
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(service.get().await, 1000);
        drop(service); // Drop service so channel closes
        let _ = drainer.await;
    }

    #[tokio::test]
    async fn test_collect_events_with_timeout() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();

        let events = collect_events(&mut rx, Duration::from_millis(50)).await;
        assert_eq!(events, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_collect_events_empty() {
        let (_tx, mut rx) = tokio::sync::mpsc::channel::<i32>(32);
        let events = collect_events(&mut rx, Duration::from_millis(50)).await;
        assert!(events.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_current_thread_runtime() {
        // Some tests benefit from single-threaded runtime
        let (tx, mut rx) = tokio::sync::watch::channel(0);

        let tx2 = tx.clone();
        tokio::spawn(async move {
            tx2.send(42).unwrap();
        });

        // With current_thread, the spawned task won't run until we await
        tokio::time::sleep(Duration::from_millis(1)).await;

        let _ = rx.changed().await;
        assert_eq!(*rx.borrow(), 42);
    }
}
