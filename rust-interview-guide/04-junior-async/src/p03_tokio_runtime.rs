/// Problem: Tokio Runtime
///
/// Master Tokio runtime.
///
/// Key Concepts:
/// - Runtime creation
/// - Task spawning
/// - Handle
/// - Block on
/// - Runtime configuration

use tokio::runtime::Runtime;
use tokio::task;
use tokio::time::{sleep, Duration};

/// Problem 1: Basic runtime
/// Create a basic Tokio runtime
pub fn basic_runtime() -> i32 {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        42
    })
}

/// Problem 2: Runtime with multiple workers
/// Configure runtime with multiple workers
pub fn runtime_multi_worker() -> i32 {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut handles = vec![];

        for i in 0..5 {
            let handle = task::spawn(async move {
                i * 2
            });
            handles.push(handle);
        }

        let mut sum = 0;
        for handle in handles {
            sum += handle.await.unwrap();
        }
        sum
    })
}

/// Problem 3: Runtime with spawn
/// Spawn tasks on runtime
pub fn runtime_spawn() -> i32 {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let handle = task::spawn(async {
            42
        });
        handle.await.unwrap()
    })
}

/// Problem 4: Runtime with spawn_blocking
/// Use spawn_blocking for blocking operations
pub fn runtime_spawn_blocking() -> i32 {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let handle = task::spawn_blocking(|| {
            // Simulate blocking work
            std::thread::sleep(Duration::from_millis(10));
            42
        });
        handle.await.unwrap()
    })
}

/// Problem 5: Runtime with yield_now
/// Yield to other tasks
pub fn runtime_yield_now() -> i32 {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut sum = 0;
        for i in 0..5 {
            sum += i;
            task::yield_now().await;
        }
        sum
    })
}

/// Problem 6: Runtime with sleep
/// Use sleep in runtime
pub fn runtime_sleep() -> i32 {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        sleep(Duration::from_millis(10)).await;
        42
    })
}

/// Problem 7: Runtime with timeout
/// Use timeout in runtime
pub fn runtime_timeout() -> Option<i32> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        tokio::time::timeout(Duration::from_millis(100), async {
            42
        }).await.ok()
    })
}

/// Problem 8: Runtime with select
/// Use select in runtime
pub fn runtime_select() -> i32 {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let future1 = async {
            sleep(Duration::from_millis(10)).await;
            1
        };
        let future2 = async {
            sleep(Duration::from_millis(5)).await;
            2
        };

        tokio::select! {
            val = future1 => val,
            val = future2 => val,
        }
    })
}

/// Problem 9: Runtime with join
/// Use join in runtime
pub fn runtime_join() -> (i32, i32) {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let future1 = async { 1 };
        let future2 = async { 2 };

        let (val1, val2) = tokio::join!(future1, future2);
        (val1, val2)
    })
}

/// Problem 10: Runtime with handle
/// Get runtime handle
pub fn runtime_handle() -> i32 {
    let rt = Runtime::new().unwrap();
    let handle = rt.handle().clone();

    handle.block_on(async {
        42
    })
}

/// Problem 11: Runtime with current_thread
/// Use current_thread runtime
pub fn runtime_current_thread() -> i32 {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        42
    })
}

/// Problem 12: Runtime with custom configuration
/// Configure runtime
pub fn runtime_custom_config() -> i32 {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        42
    })
}

/// Problem 13: Runtime with Enter
/// Enter runtime context
pub fn runtime_enter() -> i32 {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    // Now we can spawn tasks
    let handle = task::spawn(async { 42 });
    rt.block_on(handle).unwrap()
}

/// Problem 14: Runtime with shutdown
/// Gracefully shutdown runtime
pub fn runtime_shutdown() -> i32 {
    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        42
    });
    rt.shutdown_background();
    result
}

/// Problem 15: Runtime with multiple tasks
/// Spawn multiple tasks
pub fn runtime_multiple_tasks() -> Vec<i32> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut handles = vec![];

        for i in 0..5 {
            let handle = task::spawn(async move {
                i * 2
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }
        results
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_runtime() {
        assert_eq!(basic_runtime(), 42);
    }

    #[test]
    fn test_runtime_multi_worker() {
        assert_eq!(runtime_multi_worker(), 20);
    }

    #[test]
    fn test_runtime_spawn() {
        assert_eq!(runtime_spawn(), 42);
    }

    #[test]
    fn test_runtime_spawn_blocking() {
        assert_eq!(runtime_spawn_blocking(), 42);
    }

    #[test]
    fn test_runtime_yield_now() {
        assert_eq!(runtime_yield_now(), 10);
    }

    #[test]
    fn test_runtime_sleep() {
        assert_eq!(runtime_sleep(), 42);
    }

    #[test]
    fn test_runtime_timeout() {
        assert_eq!(runtime_timeout(), Some(42));
    }

    #[test]
    fn test_runtime_select() {
        assert_eq!(runtime_select(), 2);
    }

    #[test]
    fn test_runtime_join() {
        assert_eq!(runtime_join(), (1, 2));
    }

    #[test]
    fn test_runtime_handle() {
        assert_eq!(runtime_handle(), 42);
    }

    #[test]
    fn test_runtime_current_thread() {
        assert_eq!(runtime_current_thread(), 42);
    }

    #[test]
    fn test_runtime_custom_config() {
        assert_eq!(runtime_custom_config(), 42);
    }

    #[test]
    fn test_runtime_enter() {
        assert_eq!(runtime_enter(), 42);
    }

    #[test]
    fn test_runtime_shutdown() {
        assert_eq!(runtime_shutdown(), 42);
    }

    #[test]
    fn test_runtime_multiple_tasks() {
        assert_eq!(runtime_multiple_tasks(), vec![0, 2, 4, 6, 8]);
    }
}
