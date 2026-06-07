# 04 - Junior Async

## 🎯 Interview Focus
- Async/await syntax
- Futures and streams
- Tokio runtime
- Async error handling
- Concurrency in async

## 📚 Core Concepts

### 1. Async/Await
```rust
async fn fetch_data() -> String {
    // Async operations
    "data".to_string()
}

#[tokio::main]
async fn main() {
    let data = fetch_data().await;
}
```

### 2. Futures
```rust
use std::future::Future;

async fn my_future() -> i32 {
    42
}
```

### 3. Tokio Runtime
```rust
#[tokio::main]
async fn main() {
    // Tokio runtime handles async tasks
}

// Or manually
let rt = tokio::runtime::Runtime::new().unwrap();
rt.block_on(async {
    // Async code
});
```

### 4. Async Error Handling
```rust
async fn fetch() -> Result<String, reqwest::Error> {
    let response = reqwest::get("https://example.com").await?;
    let body = response.text().await?;
    Ok(body)
}
```

## ❓ Common Interview Questions

### Q1: What's the difference between async and threads?
**A:** Async is cooperative multitasking (tasks yield control). Threads are preemptive multitasking (OS switches between them). Async is more efficient for I/O-bound tasks.

### Q2: What's a Future?
**A:** A Future is a value that represents a computation that will complete in the future. It has a `poll` method that checks if the value is ready.

### Q3: What's the difference between `tokio::spawn` and `thread::spawn`?
**A:** `tokio::spawn` creates an async task on the Tokio runtime. `thread::spawn` creates an OS thread. Async tasks are lighter weight.

### Q4: What's Pinning?
**A:** Pinning ensures a value won't be moved in memory. This is important for self-referential structs in async code.

### Q5: What's the difference between `async` and `await`?
**A:** `async` creates a Future. `await` suspends execution until the Future completes.

## 🧪 Problems

1. **p01_async_basics** - Practice with async/await
2. **p02_futures** - Practice with Future trait
3. **p03_tokio_runtime** - Practice with Tokio
4. **p04_async_error_handling** - Practice with async errors
5. **p05_async_concurrency** - Practice with async concurrency
6. **p06_async_channels** - Practice with async channels
7. **p07_async_io** - Practice with async I/O
8. **p08_async_streams** - Practice with streams
9. **p09_async_patterns** - Common async patterns
10. **p10_async_best_practices** - Best practices

## 💡 Rust Tips

1. **Use `tokio::spawn`** for concurrent tasks
2. **Use `select!`** for racing futures
3. **Use `join!`** for parallel futures
4. **Avoid blocking** in async code
5. **Use `async move`** to capture variables

## 🔗 Resources

- [Async/Await](https://doc.rust-lang.org/book/ch20-01-async-await.html)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Futures Explained](https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html)
