# 03 - Junior Concurrency

## 🎯 Interview Focus
- Threads and thread spawning
- Message passing with channels
- Shared state with Mutex and Arc
- Send and Sync traits
- Deadlock prevention
- Thread safety

## 📚 Core Concepts

### 1. Threads
```rust
use std::thread;

let handle = thread::spawn(|| {
    // Code runs in new thread
    println!("Hello from thread!");
});

handle.join().unwrap(); // Wait for thread to finish
```

### 2. Message Passing
```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel();
thread::spawn(move || {
    tx.send("hello").unwrap();
});

let received = rx.recv().unwrap();
```

### 3. Shared State
```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        let mut num = counter.lock().unwrap();
        *num += 1;
    });
    handles.push(handle);
}
```

### 4. Send and Sync
```rust
// Send: Can be transferred between threads
// Sync: Can be referenced from multiple threads
// Most types are Send and Sync
// Rc is neither Send nor Sync
// Arc is both Send and Sync
```

## ❓ Common Interview Questions

### Q1: What's the difference between `Rc` and `Arc`?
**A:** `Rc` is for single-threaded use (not thread-safe). `Arc` is for multi-threaded use (uses atomic operations for thread safety).

### Q2: What's a deadlock?
**A:** A deadlock occurs when two or more threads are blocked forever, each waiting for the other to release a resource.

### Q3: What's the difference between `Mutex` and `RwLock`?
**A:** `Mutex` allows only one thread to access the data at a time. `RwLock` allows multiple readers OR one writer.

### Q4: How do you prevent data races?
**A:** Rust's type system prevents data races at compile time. `Send` and `Sync` traits ensure thread safety.

### Q5: What's message passing?
**A:** Instead of sharing memory, threads communicate by sending messages through channels. This is often safer than shared state.

## 🧪 Problems

1. **p01_threads_basics** - Practice with thread spawning
2. **p02_message_passing** - Practice with channels
3. **p03_shared_state** - Practice with Mutex and Arc
4. **p04_thread_safety** - Practice with Send and Sync
5. **p05_deadlock_prevention** - Practice avoiding deadlocks
6. **p06_producer_consumer** - Classic concurrency pattern
7. **p07_thread_pool** - Build a simple thread pool
8. **p08_parallel_computation** - Parallel algorithms
9. **p09_thread_communication** - Advanced communication patterns
10. **p10_concurrent_data_structures** - Thread-safe data structures

## 💡 Rust Tips

1. **Prefer message passing** over shared state
2. **Use `Arc` instead of `Rc`** for multi-threaded code
3. **Lock granularity matters** - lock for the shortest time possible
4. **Avoid nested locks** to prevent deadlocks
5. **Rust prevents data races** at compile time

## 🔗 Resources

- [Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Using Threads](https://doc.rust-lang.org/book/ch16-01-threads.html)
- [Message Passing](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
- [Shared State](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
