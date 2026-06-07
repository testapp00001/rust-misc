/// Problem: Async Basics
///
/// Master Rust's async/await system.
///
/// Key Concepts:
/// - async fn
/// - .await
/// - Future trait
/// - Async blocks
/// - Tokio runtime

/// Problem 1: Basic async function
/// Create a simple async function
pub async fn basic_async() -> i32 {
    42
}

/// Problem 2: Async with delay
/// Simulate async work
pub async fn async_with_delay() -> i32 {
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    42
}

/// Problem 3: Async with Result
/// Return Result from async
pub async fn async_with_result() -> Result<i32, String> {
    Ok(42)
}

/// Problem 4: Async with Option
/// Return Option from async
pub async fn async_with_option() -> Option<i32> {
    Some(42)
}

/// Problem 5: Async block
/// Use async block
pub async fn async_block() -> i32 {
    let future = async { 42 };
    future.await
}

/// Problem 6: Async with parameters
/// Pass parameters to async function
pub async fn async_with_params(x: i32, y: i32) -> i32 {
    x + y
}

/// Problem 7: Async with mutable state
/// Use mutable state in async
pub async fn async_with_mutable_state() -> i32 {
    let mut x = 0;
    x += 1;
    x
}

/// Problem 8: Async with String
/// Return String from async
pub async fn async_string() -> String {
    "hello".to_string()
}

/// Problem 9: Async with Vec
/// Return Vec from async
pub async fn async_vec() -> Vec<i32> {
    vec![1, 2, 3]
}

/// Problem 10: Async with HashMap
/// Return HashMap from async
pub async fn async_hashmap() -> std::collections::HashMap<String, i32> {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_string(), 42);
    map
}

/// Problem 11: Async with loop
/// Use loop in async
pub async fn async_loop() -> i32 {
    let mut sum = 0;
    for i in 0..5 {
        sum += i;
    }
    sum
}

/// Problem 12: Async with match
/// Use match in async
pub async fn async_match(x: Option<i32>) -> i32 {
    match x {
        Some(v) => v,
        None => 0,
    }
}

/// Problem 13: Async with if let
/// Use if let in async
pub async fn async_if_let(x: Option<i32>) -> i32 {
    if let Some(v) = x {
        v
    } else {
        0
    }
}

/// Problem 14: Async with closures
/// Use closures in async
pub async fn async_with_closure() -> i32 {
    let add = |x, y| x + y;
    add(1, 2)
}

/// Problem 15: Async with trait objects
/// Use trait objects in async
pub trait AsyncProcessor {
    async fn process(&self, input: i32) -> i32;
}

pub struct DoubleProcessor;

impl AsyncProcessor for DoubleProcessor {
    async fn process(&self, input: i32) -> i32 {
        input * 2
    }
}

pub async fn async_with_trait_object() -> i32 {
    let processor = DoubleProcessor;
    processor.process(21).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_async() {
        assert_eq!(basic_async().await, 42);
    }

    #[tokio::test]
    async fn test_async_with_delay() {
        assert_eq!(async_with_delay().await, 42);
    }

    #[tokio::test]
    async fn test_async_with_result() {
        assert_eq!(async_with_result().await, Ok(42));
    }

    #[tokio::test]
    async fn test_async_with_option() {
        assert_eq!(async_with_option().await, Some(42));
    }

    #[tokio::test]
    async fn test_async_block() {
        assert_eq!(async_block().await, 42);
    }

    #[tokio::test]
    async fn test_async_with_params() {
        assert_eq!(async_with_params(20, 22).await, 42);
    }

    #[tokio::test]
    async fn test_async_with_mutable_state() {
        assert_eq!(async_with_mutable_state().await, 1);
    }

    #[tokio::test]
    async fn test_async_string() {
        assert_eq!(async_string().await, "hello");
    }

    #[tokio::test]
    async fn test_async_vec() {
        assert_eq!(async_vec().await, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_async_hashmap() {
        let map = async_hashmap().await;
        assert_eq!(map.get("key"), Some(&42));
    }

    #[tokio::test]
    async fn test_async_loop() {
        assert_eq!(async_loop().await, 10);
    }

    #[tokio::test]
    async fn test_async_match() {
        assert_eq!(async_match(Some(42)).await, 42);
        assert_eq!(async_match(None).await, 0);
    }

    #[tokio::test]
    async fn test_async_if_let() {
        assert_eq!(async_if_let(Some(42)).await, 42);
        assert_eq!(async_if_let(None).await, 0);
    }

    #[tokio::test]
    async fn test_async_with_closure() {
        assert_eq!(async_with_closure().await, 3);
    }

    #[tokio::test]
    async fn test_async_with_trait_object() {
        assert_eq!(async_with_trait_object().await, 42);
    }
}
