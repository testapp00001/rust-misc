# 🐛 Real-World Errors Encountered

This document records actual errors encountered while building the Rust Interview Guide project, with detailed explanations and solutions.

## Table of Contents

1. [Syntax Errors](#syntax-errors)
2. [Lifetime Errors](#lifetime-errors)
3. [Concurrency Errors](#concurrency-errors)
4. [Type Errors](#type-errors)
5. [Test Failures](#test-failures)
6. [Build Errors](#build-errors)

---

## Syntax Errors

### 1. Missing closing brace in enum

**Error:**
```
error: mismatched closing delimiter: `}`
```

**Code:**
```rust
pub enum AppError {
    ParseError(String),
    ValidationError(String),
    NotFoundError(String,  // Missing closing parenthesis
}
```

**Fix:**
```rust
pub enum AppError {
    ParseError(String),
    ValidationError(String),
    NotFoundError(String),  // Add closing parenthesis
}
```

**Lesson:** Always check matching delimiters when you see `mismatched closing delimiter`.

---

### 2. Space in function name

**Error:**
```
error[E0425]: cannot find value `is` in this scope
```

**Code:**
```rust
pub fn is suitable_for_performance(&self) -> bool {
    // ...
}
```

**Fix:**
```rust
pub fn is_suitable_for_performance(&self) -> bool {
    // ...
}
```

**Lesson:** Function names cannot contain spaces. Use underscores.

---

### 3. Typo in field name

**Error:**
```
error[E0609]: no field `criters` on type `&mut RegistryFactory`
```

**Code:**
```rust
self.criters.insert(name.to_string(), creator);
```

**Fix:**
```rust
self.creators.insert(name.to_string(), creator);
```

**Lesson:** Typos in field names cause "no field" errors. Double-check spelling.

---

## Lifetime Errors

### 1. MutexGuard dropped while borrowed

**Error:**
```
error[E0597]: `counter` does not live long enough
```

**Code:**
```rust
let counter = Arc::new(Mutex::new(0));
// ... spawn threads ...
*counter.lock().unwrap()
```

**Why:** The `MutexGuard` is temporary and gets dropped at the end of the expression, but the borrow continues.

**Fix:**
```rust
let counter = Arc::new(Mutex::new(0));
// ... spawn threads ...
let result = *counter.lock().unwrap();
result
```

**Lesson:** Store the lock result in a variable to extend its lifetime.

---

### 2. RwLock guard dropped while borrowed

**Error:**
```
error[E0597]: `data` does not live long enough
```

**Code:**
```rust
let data = Arc::new(RwLock::new(vec![1, 2, 3]));
// ... spawn threads ...
data.read().unwrap().iter().sum()
```

**Fix:**
```rust
let data = Arc::new(RwLock::new(vec![1, 2, 3]));
// ... spawn threads ...
let result = data.read().unwrap().iter().sum();
result
```

**Lesson:** Same pattern - store the guard in a variable.

---

### 3. Reference outlives value

**Error:**
```
error[E0597]: `x` does not live long enough
```

**Code:**
```rust
let r;
{
    let x = 5;
    r = &x;
}
println!("{}", r);
```

**Fix:**
```rust
let x = 5;
let r = &x;
println!("{}", r);
```

**Lesson:** Ensure referenced values live long enough.

---

## Concurrency Errors

### 1. Receiver cannot be shared between threads

**Error:**
```
error[E0277]: `std::sync::mpsc::Receiver<i32>` cannot be shared between threads safely
```

**Code:**
```rust
let (tx, rx) = mpsc::channel();
let rx = Arc::new(rx);

for _ in 0..3 {
    let rx = Arc::clone(&rx);
    thread::spawn(move || {
        rx.recv(); // ERROR
    });
}
```

**Why:** `mpsc::Receiver` is not `Sync`.

**Fix:**
```rust
let (tx, rx) = mpsc::channel();

// Single receiver
let results: Vec<_> = rx.iter().collect();
```

**Lesson:** Use single receiver or `crossbeam` channel for multiple consumers.

---

### 2. JoinHandle type mismatch

**Error:**
```
error[E0308]: mismatched types
expected `JoinHandle<i32>`, found `JoinHandle<()>`
```

**Code:**
```rust
let mut handles = vec![];
handles.push(thread::spawn(|| 42));      // JoinHandle<i32>
handles.push(thread::spawn(|| {}));      // JoinHandle<()>
```

**Fix:**
```rust
let mut handles_i32 = vec![];
let mut handles_unit = vec![];

handles_i32.push(thread::spawn(|| 42));
handles_unit.push(thread::spawn(|| {}));
```

**Lesson:** All elements in a vector must have the same type.

---

### 3. Moved value in loop

**Error:**
```
error[E0382]: use of moved value: `data`
```

**Code:**
```rust
let data = vec![1, 2, 3, 4, 5];
for i in 0..data.len() {
    thread::spawn(move || {
        data[i] * 2 // ERROR: data moved in previous iteration
    });
}
```

**Fix:**
```rust
let data = Arc::new(vec![1, 2, 3, 4, 5]);
for i in 0..5 {
    let data = Arc::clone(&data);
    thread::spawn(move || {
        data[i] * 2
    });
}
```

**Lesson:** Use `Arc` for shared ownership across threads.

---

### 4. Thread pool deadlock

**Error:** Test hangs forever

**Code:**
```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Wrong: doesn't signal workers to stop
        for worker in self.workers.drain(..) {
            worker.join().unwrap();
        }
    }
}
```

**Fix:**
```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop sender to signal workers
        let sender = std::mem::replace(&mut self.sender, mpsc::channel().0);
        drop(sender);

        // Now workers will exit their loops
        for worker in self.workers.drain(..) {
            worker.join().unwrap();
        }
    }
}
```

**Lesson:** Always close channels to signal receivers to stop.

---

## Type Errors

### 1. Type annotations needed

**Error:**
```
error[E0282]: type annotations needed
```

**Code:**
```rust
let x = "42".parse().unwrap();
```

**Fix:**
```rust
let x: i32 = "42".parse().unwrap();
// or
let x = "42".parse::<i32>().unwrap();
```

---

### 2. Trait not implemented

**Error:**
```
error[E0599]: no method named `clone` found for struct `Person`
```

**Code:**
```rust
struct Person {
    name: String,
}

let p = Person { name: "Alice".to_string() };
let p2 = p.clone(); // ERROR
```

**Fix:**
```rust
#[derive(Clone)]
struct Person {
    name: String,
}
```

---

### 3. Cannot implement Copy for type with String

**Error:**
```
error[E0204]: the trait `Copy` cannot be implemented for this type
```

**Code:**
```rust
#[derive(Copy, Clone)]
struct MyStruct {
    data: String, // ERROR: String is not Copy
}
```

**Fix:**
```rust
#[derive(Clone)] // Only Clone, not Copy
struct MyStruct {
    data: String,
}
```

**Lesson:** `Copy` only works for types that are entirely stack-allocated.

---

## Test Failures

### 1. Wrong expected value

**Error:**
```
assertion `left == right` failed
left: 10
right: 13
```

**Code:**
```rust
#[test]
fn test_rwlock() {
    assert_eq!(rwlock(), 13); // Wrong expected value
}
```

**Fix:**
```rust
#[test]
fn test_rwlock() {
    assert_eq!(rwlock(), 10); // Correct value
}
```

**Lesson:** Verify your expected values by tracing through the logic.

---

### 2. Stack overflow in tests

**Error:**
```
thread has overflowed its stack
```

**Code:**
```rust
pub fn box_large_data() -> Box<[i32; 1000000]> {
    Box::new([0; 1000000]) // Too large for stack
}
```

**Fix:**
```rust
pub fn box_large_data() -> Box<[i32; 1000]> {
    Box::new([0; 1000]) // Smaller size
}
```

**Lesson:** Large arrays should be heap-allocated or reduced in size.

---

### 3. Integer overflow

**Error:**
```
attempt to add with overflow
```

**Code:**
```rust
let mut sum = 0;
for i in 0..1000000 {
    sum += i; // Overflow for i32
}
```

**Fix:**
```rust
let mut sum: i64 = 0; // Use larger type
for i in 0..1000000 {
    sum += i;
}
```

---

## Build Errors

### 1. Missing dependency

**Error:**
```
error[E0432]: unresolved import `serde_json`
```

**Fix:** Add to `Cargo.toml`:
```toml
[dependencies]
serde_json = "1"
```

---

### 2. Duplicate module definition

**Error:**
```
error[E0428]: the name `tests` is defined multiple times
```

**Fix:** Remove duplicate `mod tests` blocks.

---

### 3. Unstable feature

**Error:**
```
error[E0658]: use of unstable library feature
```

**Fix:** Use stable alternatives or enable the feature:
```rust
#![feature(some_feature)]
```

---

## 📋 Summary Table

| Error Type | Common Cause | Quick Fix |
|------------|--------------|-----------|
| `mismatched closing delimiter` | Missing `}` or `)` | Check bracket matching |
| `cannot move out of borrowed content` | Moving from reference | Use `clone()` |
| `does not live long enough` | Temporary dropped too early | Store in variable |
| `cannot be shared between threads` | Not `Sync` | Use `Mutex` or single receiver |
| `mismatched types` | Wrong type | Check function signatures |
| `trait bound not satisfied` | Missing derive | Add `#[derive(...)]` |
| `type annotations needed` | Can't infer type | Add explicit type |
| `use of moved value` | Value already moved | Use `Arc` or `clone()` |

---

## 🎓 Key Takeaways

1. **Read error messages carefully** - They tell you exactly what's wrong
2. **Check bracket matching** - Many syntax errors are mismatched brackets
3. **Understand ownership** - Move vs borrow is fundamental
4. **Store lock guards** - Don't let them drop too early
5. **Use `Arc` for shared data** - When multiple threads need access
6. **Close channels** - To signal receivers to stop
7. **Verify test expectations** - Trace through the logic
8. **Watch for overflow** - Use appropriate integer sizes

---

**Remember:** Every error is a learning opportunity. Rust's compiler catches many bugs that other languages would let slip through!
