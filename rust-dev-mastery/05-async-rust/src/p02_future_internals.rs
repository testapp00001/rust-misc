//! # Future Trait Internals
//!
//! Every `async fn` in Rust is compiled into a state machine that implements the
//! `Future` trait. Understanding how `Future`, `Poll`, `Waker`, and `Pin` work
//! under the hood is critical for writing efficient async code and diagnosing
//! performance issues.
//!
//! ## Key Concepts
//! - **Future::poll**: The runtime calls this to drive the future forward
//! - **Poll::Pending**: "I'm not done yet, wake me when something changes"
//! - **Poll::Ready(val)**: "Here's my result"
//! - **Waker**: A handle the runtime uses to re-schedule a future
//! - **Pin**: Prevents self-referential types from being moved in memory

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

/// A hand-rolled future that resolves after a specified duration.
/// This is the simplest possible custom Future implementation.
pub struct Delay {
    when: Instant,
}

impl Delay {
    pub fn new(duration: Duration) -> Self {
        Delay {
            when: Instant::now() + duration,
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            Poll::Ready(())
        } else {
            // Register the waker so we get called again.
            // In production runtimes, the waker is registered with a timer wheel.
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// A simple one-shot channel future that can be resolved from outside a future.
/// Demonstrates the producer-consumer pattern with wakers.
pub struct Oneshot<T> {
    inner: std::sync::Arc<std::sync::Mutex<OneshotInner<T>>>,
}

struct OneshotInner<T> {
    value: Option<T>,
    waker: Option<Waker>,
}

/// The sending half of a oneshot channel.
pub struct OneshotSender<T> {
    inner: std::sync::Arc<std::sync::Mutex<OneshotInner<T>>>,
}

pub fn oneshot<T>() -> (OneshotSender<T>, Oneshot<T>) {
    let inner = std::sync::Arc::new(std::sync::Mutex::new(OneshotInner {
        value: None,
        waker: None,
    }));
    (
        OneshotSender { inner: inner.clone() },
        Oneshot { inner },
    )
}

impl<T> OneshotSender<T> {
    pub fn send(self, value: T) {
        let mut inner = self.inner.lock().unwrap();
        inner.value = Some(value);
        if let Some(waker) = inner.waker.take() {
            waker.wake();
        }
    }
}

impl<T> Unpin for Oneshot<T> {}

impl<T> Future for Oneshot<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(value) = inner.value.take() {
            Poll::Ready(value)
        } else {
            inner.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Demonstrates a state-machine-style future with explicit states.
/// This is what the compiler generates from `async fn`, but by hand.
pub struct FetchStateMachine {
    state: FetchState,
}

enum FetchState {
    Init { url: String },
    Connecting,
    Reading { bytes_read: usize },
    Done,
}

impl FetchStateMachine {
    pub fn new(url: impl Into<String>) -> Self {
        FetchStateMachine {
            state: FetchState::Init { url: url.into() },
        }
    }
}

impl Future for FetchStateMachine {
    type Output = Result<Vec<u8>, FetchError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: FetchStateMachine is not structured in a way that requires pinning
        let this = unsafe { self.get_unchecked_mut() };

        loop {
            match &mut this.state {
                FetchState::Init { .. } => {
                    // Transition: Init -> Connecting
                    let FetchState::Init { url: _ } =
                        std::mem::replace(&mut this.state, FetchState::Connecting)
                    else {
                        unreachable!()
                    };
                    // In real code, start connection here
                }
                FetchState::Connecting => {
                    // Simulate: connection established, move to reading
                    this.state = FetchState::Reading { bytes_read: 0 };
                }
                FetchState::Reading { bytes_read } => {
                    let data = vec![1, 2, 3, 4, 5];
                    *bytes_read = data.len();
                    this.state = FetchState::Done;
                    return Poll::Ready(Ok(data));
                }
                FetchState::Done => panic!("polled after completion"),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FetchError {
    ConnectionFailed(String),
    Timeout,
}

/// A future that wraps another future and counts the number of polls.
/// This is useful for understanding how often a future is polled.
pub struct PollCounter<F> {
    inner: F,
    count: usize,
}

impl<F> PollCounter<F> {
    pub fn new(inner: F) -> Self {
        PollCounter { inner, count: 0 }
    }

    pub fn poll_count(&self) -> usize {
        self.count
    }
}

impl<F: Future + Unpin> Future for PollCounter<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.count += 1;
        Pin::new(&mut self.inner).poll(cx)
    }
}

/// Safe combinator: chains two futures sequentially (like `and_then`).
pub struct Then<FutA, FutB, F>
where
    FutA: Future,
    F: FnOnce(FutA::Output) -> FutB,
    FutB: Future,
{
    state: ThenState<FutA, FutB, F>,
}

enum ThenState<FutA, FutB, F>
where
    FutA: Future,
    F: FnOnce(FutA::Output) -> FutB,
    FutB: Future,
{
    First(FutA, Option<F>),
    Second(FutB),
    Done,
}

impl<FutA, FutB, F> Then<FutA, FutB, F>
where
    FutA: Future,
    F: FnOnce(FutA::Output) -> FutB,
    FutB: Future,
{
    pub fn new(first: FutA, f: F) -> Self {
        Then {
            state: ThenState::First(first, Some(f)),
        }
    }
}

impl<FutA, FutB, F> Future for Then<FutA, FutB, F>
where
    FutA: Future,
    FutB: Future,
    F: FnOnce(FutA::Output) -> FutB,
{
    type Output = FutB::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We never move the inner futures out of the pin
        let this = unsafe { self.get_unchecked_mut() };

        loop {
            match &mut this.state {
                ThenState::First(fut, _f) => {
                    let pinned = unsafe { Pin::new_unchecked(fut) };
                    match pinned.poll(cx) {
                        Poll::Ready(output) => {
                            // Transition to second state
                            let ThenState::First(_, Some(f)) =
                                std::mem::replace(&mut this.state, ThenState::Done)
                            else {
                                unreachable!()
                            };
                            let second = f(output);
                            this.state = ThenState::Second(second);
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                ThenState::Second(fut) => {
                    let pinned = unsafe { Pin::new_unchecked(fut) };
                    return pinned.poll(cx);
                }
                ThenState::Done => panic!("Future polled after completion"),
            }
        }
    }
}

/// An all-or-nothing future that polls a list of inner futures and collects
/// their results, but only once ALL are ready. Demonstrates manual pin management.
pub struct All<F: Future> {
    futures: Vec<Option<F>>,
    results: Vec<Option<F::Output>>,
}

impl<F: Future> All<F> {
    pub fn new(futures: Vec<F>) -> Self {
        let len = futures.len();
        All {
            futures: futures.into_iter().map(Some).collect(),
            results: (0..len).map(|_| None).collect(),
        }
    }
}

impl<F: Future + Unpin> Future for All<F> {
    type Output = Vec<F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let mut all_done = true;

        for i in 0..this.futures.len() {
            if this.results[i].is_some() {
                continue; // Already completed
            }
            if let Some(ref mut fut) = this.futures[i] {
                match Pin::new(fut).poll(cx) {
                    Poll::Ready(val) => {
                        this.results[i] = Some(val);
                        this.futures[i] = None;
                    }
                    Poll::Pending => {
                        all_done = false;
                    }
                }
            }
        }

        if all_done {
            let results: Vec<F::Output> = this.results.drain(..).map(|r| r.unwrap()).collect();
            Poll::Ready(results)
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_delay_resolves() {
        let start = Instant::now();
        Delay::new(Duration::from_millis(50)).await;
        assert!(start.elapsed() >= Duration::from_millis(40));
    }

    #[tokio::test]
    async fn test_oneshot_channel() {
        let (tx, rx) = oneshot::<i32>();

        tokio::spawn(async move {
            tx.send(42);
        });

        let value = rx.await;
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn test_oneshot_immediate_send() {
        let (tx, rx) = oneshot::<String>();
        tx.send("hello".to_string());
        let value = rx.await;
        assert_eq!(value, "hello");
    }

    #[tokio::test]
    async fn test_fetch_state_machine() {
        let result = FetchStateMachine::new("https://example.com").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_then_combinator() {
        let result = Then::new(async { 10 }, |x| async move { x * 2 }).await;
        assert_eq!(result, 20);
    }

    #[tokio::test]
    async fn test_chained_then() {
        let result = Then::new(async { 1 }, |a| {
            Then::new(async move { a + 1 }, |b| async move { b * 10 })
        })
        .await;
        assert_eq!(result, 20);
    }

    #[tokio::test]
    async fn test_poll_counter() {
        let counter = PollCounter::new(Box::pin(async { 42 }));
        let result = counter.await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_all_futures() {
        let futures: Vec<std::future::Ready<i32>> = vec![
            std::future::ready(1),
            std::future::ready(2),
            std::future::ready(3),
        ];
        let results = All::new(futures).await;
        assert_eq!(results, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_all_with_async() {
        let futs: Vec<Pin<Box<dyn Future<Output = i32>>>> = (0..5)
            .map(|i| Box::pin(async move { i * i }) as Pin<Box<dyn Future<Output = i32>>>)
            .collect();
        let results = All::new(futs).await;
        assert_eq!(results, vec![0, 1, 4, 9, 16]);
    }

    #[test]
    fn test_manual_poll_delay() {
        use std::task::{RawWaker, RawWakerVTable};

        fn noop_waker() -> Waker {
            fn noop(_: *const ()) {}
            fn clone(p: *const ()) -> RawWaker {
                RawWaker::new(p, &VTABLE)
            }
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
            unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
        }

        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        // Already-expired delay should resolve immediately
        let mut future = Delay::new(Duration::ZERO);
        let pinned = unsafe { Pin::new_unchecked(&mut future) };
        let result = pinned.poll(&mut cx);
        assert!(result.is_ready());

        // Far-future delay should be pending
        let mut future = Delay::new(Duration::from_secs(999));
        let pinned = unsafe { Pin::new_unchecked(&mut future) };
        let result = pinned.poll(&mut cx);
        assert!(result.is_pending());
    }

    #[test]
    fn test_manual_poll_oneshot() {
        use std::task::{RawWaker, RawWakerVTable};

        fn noop_waker() -> Waker {
            fn noop(_: *const ()) {}
            fn clone(p: *const ()) -> RawWaker {
                RawWaker::new(p, &VTABLE)
            }
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
            unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
        }

        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        let (tx, mut rx) = oneshot::<i32>();

        // Poll before send -> Pending
        let pinned = unsafe { Pin::new_unchecked(&mut rx) };
        assert!(pinned.poll(&mut cx).is_pending());

        // Send value
        tx.send(99);

        // Poll after send -> Ready
        let pinned = unsafe { Pin::new_unchecked(&mut rx) };
        assert_eq!(pinned.poll(&mut cx), Poll::Ready(99));
    }
}
