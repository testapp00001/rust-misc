/// Problem: Futures
///
/// Master Rust's Future trait.
///
/// Key Concepts:
/// - Future trait
/// - Poll method
/// - Waker
/// - Pin
/// - Future combinators

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// Problem 1: Basic Future
/// Implement a simple Future
pub struct ReadyFuture<T> {
    value: Option<T>,
}

impl<T> ReadyFuture<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Some(value),
        }
    }
}

impl<T> Future for ReadyFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We're not moving the value, just taking from Option
        let this = unsafe { self.get_unchecked_mut() };
        if let Some(value) = this.value.take() {
            Poll::Ready(value)
        } else {
            Poll::Pending
        }
    }
}

/// Problem 2: Delay Future
/// Implement a Future that completes after a delay
pub struct DelayFuture {
    until: Instant,
}

impl DelayFuture {
    pub fn new(duration: Duration) -> Self {
        Self {
            until: Instant::now() + duration,
        }
    }
}

impl Future for DelayFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.until {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Problem 3: Map Future
/// Transform a Future's output
pub struct MapFuture<F, T, U> {
    future: F,
    mapper: Box<dyn Fn(T) -> U>,
}

impl<F, T, U> MapFuture<F, T, U>
where
    F: Future<Output = T>,
{
    pub fn new(future: F, mapper: impl Fn(T) -> U + 'static) -> Self {
        Self {
            future,
            mapper: Box::new(mapper),
        }
    }
}

impl<F, T, U> Future for MapFuture<F, T, U>
where
    F: Future<Output = T> + Unpin,
{
    type Output = U;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.future).poll(cx) {
            Poll::Ready(value) => Poll::Ready((self.mapper)(value)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Problem 4: AndThen Future
/// Chain Futures
pub struct AndThenFuture<F1, F2, T> {
    future1: Option<F1>,
    future2: Option<Box<dyn Fn(T) -> F2>>,
}

impl<F1, F2, T> AndThenFuture<F1, F2, T>
where
    F1: Future<Output = T>,
    F2: Future,
{
    pub fn new(future1: F1, future2: impl Fn(T) -> F2 + 'static) -> Self {
        Self {
            future1: Some(future1),
            future2: Some(Box::new(future2)),
        }
    }
}

impl<F1, F2, T> Future for AndThenFuture<F1, F2, T>
where
    F1: Future<Output = T> + Unpin,
    F2: Future + Unpin,
{
    type Output = F2::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(future1) = &mut self.future1 {
            match Pin::new(future1).poll(cx) {
                Poll::Ready(value) => {
                    let future2 = (self.future2.take().unwrap())(value);
                    self.future1 = None;
                    self.future2 = None;
                    Pin::new(&mut Box::pin(future2)).poll(cx)
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}

/// Problem 5: Race Future
/// Return the first Future to complete
pub struct RaceFuture<F1, F2> {
    future1: F1,
    future2: F2,
}

impl<F1, F2> RaceFuture<F1, F2> {
    pub fn new(future1: F1, future2: F2) -> Self {
        Self { future1, future2 }
    }
}

impl<F1, F2, T> Future for RaceFuture<F1, F2>
where
    F1: Future<Output = T> + Unpin,
    F2: Future<Output = T> + Unpin,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Poll::Ready(value) = Pin::new(&mut self.future1).poll(cx) {
            return Poll::Ready(value);
        }
        if let Poll::Ready(value) = Pin::new(&mut self.future2).poll(cx) {
            return Poll::Ready(value);
        }
        Poll::Pending
    }
}

/// Problem 6: Join Future
/// Wait for all Futures to complete
pub struct JoinFuture<F1, F2, T1, T2> {
    future1: F1,
    future2: F2,
    result1: Option<T1>,
    result2: Option<T2>,
}

impl<F1, F2, T1, T2> JoinFuture<F1, F2, T1, T2>
where
    F1: Future<Output = T1>,
    F2: Future<Output = T2>,
{
    pub fn new(future1: F1, future2: F2) -> Self {
        Self {
            future1,
            future2,
            result1: None,
            result2: None,
        }
    }
}

impl<F1, F2, T1, T2> Future for JoinFuture<F1, F2, T1, T2>
where
    F1: Future<Output = T1> + Unpin,
    F2: Future<Output = T2> + Unpin,
{
    type Output = (T1, T2);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We're not moving the value, just accessing fields
        let this = unsafe { self.get_unchecked_mut() };

        if this.result1.is_none() {
            if let Poll::Ready(value) = Pin::new(&mut this.future1).poll(cx) {
                this.result1 = Some(value);
            }
        }

        if this.result2.is_none() {
            if let Poll::Ready(value) = Pin::new(&mut this.future2).poll(cx) {
                this.result2 = Some(value);
            }
        }

        if this.result1.is_some() && this.result2.is_some() {
            Poll::Ready((this.result1.take().unwrap(), this.result2.take().unwrap()))
        } else {
            Poll::Pending
        }
    }
}

/// Problem 7: Timeout Future
/// Add timeout to a Future
pub struct TimeoutFuture<F> {
    future: F,
    deadline: Instant,
}

impl<F> TimeoutFuture<F> {
    pub fn new(future: F, duration: Duration) -> Self {
        Self {
            future,
            deadline: Instant::now() + duration,
        }
    }
}

impl<F, T> Future for TimeoutFuture<F>
where
    F: Future<Output = T> + Unpin,
{
    type Output = Option<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.deadline {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.future).poll(cx) {
            Poll::Ready(value) => Poll::Ready(Some(value)),
            Poll::Pending => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

/// Problem 8: Lazy Future
/// Create a Future that computes on first poll
pub struct LazyFuture<F> {
    computation: Option<F>,
}

impl<F, T> LazyFuture<F>
where
    F: FnOnce() -> T,
{
    pub fn new(computation: F) -> Self {
        Self {
            computation: Some(computation),
        }
    }
}

impl<F, T> Future for LazyFuture<F>
where
    F: FnOnce() -> T,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We're not moving the value, just taking from Option
        let this = unsafe { self.get_unchecked_mut() };
        if let Some(computation) = this.computation.take() {
            Poll::Ready(computation())
        } else {
            Poll::Pending
        }
    }
}

/// Problem 9: Flatten Future
/// Flatten nested Futures
pub struct FlattenFuture<F> {
    future: Option<F>,
}

impl<F, T> FlattenFuture<F>
where
    F: Future<Output = T>,
{
    pub fn new(future: F) -> Self {
        Self {
            future: Some(future),
        }
    }
}

impl<F, T> Future for FlattenFuture<F>
where
    F: Future<Output = T> + Unpin,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(future) = &mut self.future {
            Pin::new(future).poll(cx)
        } else {
            Poll::Pending
        }
    }
}

/// Problem 10: Inspect Future
/// Inspect a Future's output
pub struct InspectFuture<F, T> {
    future: F,
    inspector: Box<dyn Fn(&T)>,
}

impl<F, T> InspectFuture<F, T>
where
    F: Future<Output = T>,
{
    pub fn new(future: F, inspector: impl Fn(&T) + 'static) -> Self {
        Self {
            future,
            inspector: Box::new(inspector),
        }
    }
}

impl<F, T> Future for InspectFuture<F, T>
where
    F: Future<Output = T> + Unpin,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.future).poll(cx) {
            Poll::Ready(value) => {
                (self.inspector)(&value);
                Poll::Ready(value)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ready_future() {
        let future = ReadyFuture::new(42);
        assert_eq!(future.await, 42);
    }

    #[tokio::test]
    async fn test_delay_future() {
        let future = DelayFuture::new(Duration::from_millis(10));
        future.await;
    }

    #[tokio::test]
    async fn test_map_future() {
        let future = ReadyFuture::new(21);
        let mapped = MapFuture::new(future, |x| x * 2);
        assert_eq!(mapped.await, 42);
    }

    #[tokio::test]
    async fn test_and_then_future() {
        let future = ReadyFuture::new(21);
        let chained = AndThenFuture::new(future, |x| ReadyFuture::new(x * 2));
        assert_eq!(chained.await, 42);
    }

    #[tokio::test]
    async fn test_race_future() {
        let future1 = ReadyFuture::new(1);
        let future2 = ReadyFuture::new(2);
        let race = RaceFuture::new(future1, future2);
        assert_eq!(race.await, 1);
    }

    #[tokio::test]
    async fn test_join_future() {
        let future1 = ReadyFuture::new(1);
        let future2 = ReadyFuture::new(2);
        let join = JoinFuture::new(future1, future2);
        assert_eq!(join.await, (1, 2));
    }

    #[tokio::test]
    async fn test_timeout_future() {
        let future = ReadyFuture::new(42);
        let timeout = TimeoutFuture::new(future, Duration::from_millis(100));
        assert_eq!(timeout.await, Some(42));
    }

    #[tokio::test]
    async fn test_lazy_future() {
        let future = LazyFuture::new(|| 42);
        assert_eq!(future.await, 42);
    }

    #[tokio::test]
    async fn test_flatten_future() {
        let future = ReadyFuture::new(42);
        let flattened = FlattenFuture::new(future);
        assert_eq!(flattened.await, 42);
    }

    #[tokio::test]
    async fn test_inspect_future() {
        let future = ReadyFuture::new(42);
        let inspected = InspectFuture::new(future, |value| {
            assert_eq!(*value, 42);
        });
        assert_eq!(inspected.await, 42);
    }
}
