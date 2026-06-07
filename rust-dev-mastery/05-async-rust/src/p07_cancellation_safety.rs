//! # Cancellation Safety
//!
//! When a future is dropped (cancelled), any work in progress is abandoned.
//! In `tokio::select!`, all branches that didn't complete are dropped. This can
//! cause data loss or resource leaks if not handled carefully.
//!
//! ## Key Concepts
//! - **Cancellation safety**: A future is cancellation-safe if dropping it mid-progress
//!   doesn't lose data or corrupt state
//! - **select! pitfall**: Only the winning branch completes; others are cancelled
//! - **Drop safety**: Implementing `Drop` to clean up resources
//! - **tokio::pin!**: Pinning futures to make them cancellation-safe in select!

use std::time::Duration;

/// Demonstrates the critical difference between cancellation-safe and
/// cancellation-unsafe patterns.
///
/// # Unsafe Pattern
/// ```ignore
/// tokio::select! {
///     value = channel.recv() => { process(value); }
///     _ = timeout => { /* channel.recv() is cancelled - message is lost! */ }
/// }
/// ```
///
/// # Safe Pattern
/// ```ignore
/// tokio::pin!(timeout_future);
/// loop {
///     tokio::select! {
///         value = channel.recv() => { process(value); break; }
///         _ = &mut timeout_future => { break; }
///     }
/// }
/// ```

/// A message processor that demonstrates proper cancellation handling.
/// It reads from a channel and processes messages, but handles cancellation
/// gracefully by draining remaining messages on shutdown.
pub struct SafeMessageProcessor {
    receiver: tokio::sync::mpsc::Receiver<String>,
    processed: Vec<String>,
}

impl SafeMessageProcessor {
    pub fn new(receiver: tokio::sync::mpsc::Receiver<String>) -> Self {
        SafeMessageProcessor {
            receiver,
            processed: Vec::new(),
        }
    }

    /// Processes messages until timeout, then drains remaining messages.
    /// This is the cancellation-safe pattern: no messages are lost.
    pub async fn run_with_timeout(&mut self, timeout: Duration) -> Vec<String> {
        let deadline = tokio::time::sleep(timeout);
        tokio::pin!(deadline);

        loop {
            tokio::select! {
                msg = self.receiver.recv() => {
                    match msg {
                        Some(msg) => {
                            self.processed.push(msg);
                        }
                        None => break, // Channel closed
                    }
                }
                _ = &mut deadline => {
                    // Timeout reached — don't break yet, drain remaining
                    break;
                }
            }
        }

        // Drain any remaining messages in the channel buffer
        while let Ok(msg) = self.receiver.try_recv() {
            self.processed.push(msg);
        }

        self.processed.clone()
    }

    /// Unsafe version that demonstrates what goes wrong without proper handling.
    /// Messages in the buffer at timeout time are lost.
    pub async fn run_unsafe(&mut self, timeout: Duration) -> Vec<String> {
        let deadline = tokio::time::sleep(timeout);
        tokio::pin!(deadline);

        // This is cancellation-unsafe: when deadline fires, any pending
        // recv() is cancelled and buffered messages are lost.
        tokio::select! {
            _ = &mut deadline => {
                // Messages still in channel buffer are lost!
            }
            _ = async {
                while let Some(msg) = self.receiver.recv().await {
                    self.processed.push(msg);
                }
            } => {}
        }

        self.processed.clone()
    }
}

/// A cancellation-safe transaction that can be rolled back if cancelled.
pub struct AsyncTransaction {
    id: u64,
    operations: Vec<Operation>,
    committed: bool,
    rolled_back: bool,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub description: String,
    pub applied: bool,
}

impl AsyncTransaction {
    pub fn new(id: u64) -> Self {
        AsyncTransaction {
            id,
            operations: Vec::new(),
            committed: false,
            rolled_back: false,
        }
    }

    pub async fn execute(&mut self, description: impl Into<String>) {
        let desc = description.into();
        tokio::time::sleep(Duration::from_millis(1)).await;
        self.operations.push(Operation {
            description: desc,
            applied: true,
        });
    }

    pub async fn commit(&mut self) {
        self.committed = true;
    }

    pub async fn rollback(&mut self) {
        for op in self.operations.iter_mut().rev() {
            op.applied = false;
        }
        self.rolled_back = true;
    }

    pub fn is_committed(&self) -> bool {
        self.committed
    }

    pub fn is_rolled_back(&self) -> bool {
        self.rolled_back
    }
}

impl Drop for AsyncTransaction {
    fn drop(&mut self) {
        if !self.committed && !self.rolled_back && !self.operations.is_empty() {
            // Synchronous rollback in Drop — can't await, but mark state
            // In production, you'd log a warning and schedule async cleanup
            eprintln!(
                "WARNING: Transaction {} dropped without commit/rollback. {} operations may be incomplete.",
                self.id,
                self.operations.len()
            );
            // Mark as rolled back for cleanup
            for op in self.operations.iter_mut().rev() {
                op.applied = false;
            }
            self.rolled_back = true;
        }
    }
}

/// A cancellation guard that runs a cleanup function when dropped.
/// This is the RAII pattern adapted for async resource cleanup.
pub struct CancellationGuard<F: FnMut()> {
    cleanup: Option<F>,
}

impl<F: FnMut()> CancellationGuard<F> {
    pub fn new(cleanup: F) -> Self {
        CancellationGuard {
            cleanup: Some(cleanup),
        }
    }

    /// Consume the guard without running the cleanup (e.g., on successful completion).
    pub fn disarm(mut self) {
        self.cleanup = None;
    }
}

impl<F: FnMut()> Drop for CancellationGuard<F> {
    fn drop(&mut self) {
        if let Some(mut cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

/// Demonstrates the select! loop pattern that is always cancellation-safe.
/// The key insight: pin the futures you'll re-use across iterations.
pub async fn safe_select_loop(
    mut data_rx: tokio::sync::mpsc::Receiver<i32>,
    mut control_rx: tokio::sync::mpsc::Receiver<()>,
    timeout: Duration,
) -> Vec<i32> {
    let mut results = Vec::new();
    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            biased; // Process data first if both are ready

            data = data_rx.recv() => {
                match data {
                    Some(value) => results.push(value),
                    None => break,
                }
            }
            _ = control_rx.recv() => {
                break; // Graceful shutdown signal
            }
            _ = &mut deadline => {
                break; // Timeout
            }
        }
    }

    results
}

/// Shows how to make an inherently cancellation-unsafe operation safe
/// by buffering and using a loop instead of a single select!.
pub async fn safe_channel_drain(
    rx: &mut tokio::sync::mpsc::Receiver<i32>,
    timeout: Duration,
) -> Vec<i32> {
    let mut items = Vec::new();
    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);

    // Use a loop with try_recv in the timeout branch
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Some(item) => items.push(item),
                    None => break,
                }
            }
            _ = &mut deadline => {
                // Drain remaining buffered items
                while let Ok(item) = rx.try_recv() {
                    items.push(item);
                }
                break;
            }
        }
    }

    items
}

/// A resource manager that tracks active resources and ensures cleanup
/// even when operations are cancelled.
pub struct ResourceManager {
    resources: Vec<String>,
    cleanup_log: Vec<String>,
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager {
            resources: Vec::new(),
            cleanup_log: Vec::new(),
        }
    }

    pub async fn acquire(&mut self, name: impl Into<String>) -> usize {
        let name = name.into();
        tokio::time::sleep(Duration::from_millis(1)).await;
        self.resources.push(name.clone());
        self.resources.len() - 1
    }

    pub async fn release(&mut self, index: usize) {
        if index < self.resources.len() {
            let name = self.resources[index].clone();
            tokio::time::sleep(Duration::from_millis(1)).await;
            self.cleanup_log.push(format!("Released: {name}"));
        }
    }

    pub async fn release_all(&mut self) {
        for i in (0..self.resources.len()).rev() {
            self.release(i).await;
        }
    }

    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    pub fn cleanup_log(&self) -> &[String] {
        &self.cleanup_log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_safe_message_processor_completes() {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tx.send("msg1".to_string()).await.unwrap();
        tx.send("msg2".to_string()).await.unwrap();
        tx.send("msg3".to_string()).await.unwrap();
        drop(tx);

        let mut processor = SafeMessageProcessor::new(rx);
        let result = processor.run_with_timeout(Duration::from_secs(5)).await;
        assert_eq!(result, vec!["msg1", "msg2", "msg3"]);
    }

    #[tokio::test]
    async fn test_safe_message_processor_timeout() {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        // Send messages
        tx.send("msg1".to_string()).await.unwrap();
        tx.send("msg2".to_string()).await.unwrap();

        let mut processor = SafeMessageProcessor::new(rx);
        let result = processor.run_with_timeout(Duration::from_millis(50)).await;

        // All sent messages should be captured (cancellation-safe)
        assert!(result.len() >= 2);
    }

    #[tokio::test]
    async fn test_safe_select_loop() {
        let (data_tx, data_rx) = tokio::sync::mpsc::channel(32);
        let (_ctrl_tx, ctrl_rx) = tokio::sync::mpsc::channel(1);

        data_tx.send(1).await.unwrap();
        data_tx.send(2).await.unwrap();
        data_tx.send(3).await.unwrap();
        drop(data_tx);

        let result = safe_select_loop(data_rx, ctrl_rx, Duration::from_secs(5)).await;
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_safe_select_loop_control_signal() {
        let (data_tx, data_rx) = tokio::sync::mpsc::channel(32);
        let (ctrl_tx, ctrl_rx) = tokio::sync::mpsc::channel(1);

        data_tx.send(1).await.unwrap();
        ctrl_tx.send(()).await.unwrap();

        let result = safe_select_loop(data_rx, ctrl_rx, Duration::from_secs(5)).await;
        // Only the first message should be captured before control signal
        assert_eq!(result, vec![1]);
    }

    #[tokio::test]
    async fn test_safe_channel_drain() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();

        let result = safe_channel_drain(&mut rx, Duration::from_millis(100)).await;
        assert!(result.contains(&1));
        assert!(result.contains(&2));
    }

    #[tokio::test]
    async fn test_cancellation_guard_disarm() {
        let cleaned = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cleaned_clone = cleaned.clone();

        {
            let guard = CancellationGuard::new(move || {
                cleaned_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            });
            guard.disarm(); // Don't run cleanup
        }

        assert!(!cleaned.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_cancellation_guard_drops() {
        let cleaned = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cleaned_clone = cleaned.clone();

        {
            let _guard = CancellationGuard::new(move || {
                cleaned_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            });
            // Guard dropped here, cleanup runs
        }

        assert!(cleaned.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        let mut tx = AsyncTransaction::new(1);
        tx.execute("insert users").await;
        tx.execute("update stats").await;
        tx.commit().await;

        assert!(tx.is_committed());
        assert!(!tx.is_rolled_back());
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let mut tx = AsyncTransaction::new(2);
        tx.execute("insert users").await;
        tx.rollback().await;

        assert!(tx.is_rolled_back());
        assert!(!tx.is_committed());
        // All operations should be marked as not applied
        for op in &tx.operations {
            assert!(!op.applied);
        }
    }

    #[tokio::test]
    async fn test_resource_manager() {
        let mut rm = ResourceManager::new();
        rm.acquire("db-conn").await;
        rm.acquire("cache-conn").await;

        assert_eq!(rm.resource_count(), 2);

        rm.release(0).await;
        assert_eq!(rm.cleanup_log(), &["Released: db-conn"]);
    }

    #[tokio::test]
    async fn test_resource_manager_release_all() {
        let mut rm = ResourceManager::new();
        rm.acquire("a").await;
        rm.acquire("b").await;
        rm.acquire("c").await;

        rm.release_all().await;
        assert_eq!(rm.cleanup_log().len(), 3);
    }

    #[tokio::test]
    async fn test_cancellation_with_select() {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let mut processor = SafeMessageProcessor::new(rx);

        // Spawn producer
        let producer = tokio::spawn(async move {
            for i in 0..10 {
                tx.send(format!("msg-{i}")).await.unwrap();
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        });

        // Run processor with a short timeout
        let result = processor.run_with_timeout(Duration::from_millis(30)).await;

        // Some messages should be captured
        assert!(!result.is_empty());

        // Clean up producer
        let _ = producer.await;
    }
}
