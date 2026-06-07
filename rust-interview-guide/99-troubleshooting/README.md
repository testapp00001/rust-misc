# 🐛 Rust Common Errors & Solutions

A comprehensive guide documenting real errors encountered while building a large Rust project, with solutions and prevention tips.

## 📚 Table of Contents

1. [Ownership & Borrowing Errors](#1-ownership--borrowing-errors)
2. [Lifetime Errors](#2-lifetime-errors)
3. [Concurrency Errors](#3-concurrency-errors)
4. [Type System Errors](#4-type-system-errors)
5. [Async/Await Errors](#5-asyncawait-errors)
6. [Pattern Matching Errors](#6-pattern-matching-errors)
7. [Import & Module Errors](#7-import--module-errors)
8. [Test Failures](#8-test-failures)

---

## 1. Ownership & Borrowing Errors

### Error: `cannot move out of borrowed content`

**Example:**
```rust
let s = String::from("hello");
let r = &s;
let s2 = *r; // ERROR: cannot move out of borrowed content
```

**Why:** You can't move a value out of a reference. The reference only borrows the value.

**Solution:**
```rust
let s = String::from("hello");
let r = &s;
let s2 = r.clone(); // Clone instead of move
// Or use reference directly
let len = r.len();
```

**Prevention:** Use `clone()` when you need an owned copy, or work with references.

---

### Error: `cannot borrow as mutable because it is also borrowed as immutable`

**Example:**
```rust
let mut data = vec![1, 2, 3];
let first = &data[0];      // Immutable borrow
data.push(4);               // ERROR: cannot borrow as mutable
println!("{}", first);
```

**Why:** You can't mutate while there's an active immutable borrow.

**Solution:**
```rust
let mut data = vec![1, 2, 3];
let first = data[0];        // Copy the value instead of borrowing
data.push(4);               // Now OK
println!("{}", first);
```

**Prevention:** Limit borrow scopes, or copy small values instead of borrowing.

---

### Error: `cannot borrow data in a `&` reference as mutable`

**Example:**
```rust
let data = vec![1, 2, 3];
data.push(4); // ERROR: cannot borrow as mutable
```

**Why:** `data` is immutable. You need `mut` to modify.

**Solution:**
```rust
let mut data = vec![1, 2, 3];
data.push(4); // OK
```

---

### Error: `use of moved value`

**Example:**
```rust
let s = String::from("hello");
let s2 = s;
println!("{}", s); // ERROR: value used after move
```

**Why:** `String` doesn't implement `Copy`, so assignment moves the value.

**Solution:**
```rust
let s = String::from("hello");
let s2 = s.clone(); // Clone instead of move
println!("{}", s);   // Now OK
```

**Prevention:** Use `clone()` when you need multiple owners, or use references.

---

## 2. Lifetime Errors

### Error: `missing lifetime specifier`

**Example:**
```rust
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() { x } else { y }
}
```

**Why:** The compiler doesn't know how long the returned reference lives.

**Solution:**
```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

**Prevention:** Add lifetime annotations when returning references.

---

### Error: `lifetime mismatch`

**Example:**
```rust
struct Foo<'a> {
    data: &'a str,
}

impl<'a> Foo<'a> {
    fn get(&self) -> &str {
        self.data
    }
}
```

**Why:** The returned reference needs the same lifetime as `self`.

**Solution:**
```rust
impl<'a> Foo<'a> {
    fn get(&self) -> &'a str {
        self.data
    }
}
```

---

### Error: `borrowed value does not live long enough`

**Example:**
```rust
let r;
{
    let x = 5;
    r = &x;
} // x is dropped here
println!("{}", r); // ERROR: x doesn't live long enough
```

**Why:** The reference outlives the value it points to.

**Solution:**
```rust
let x = 5;
let r = &x;
println!("{}", r); // OK: x is still alive
```

**Prevention:** Ensure referenced values live long enough.

---

## 3. Concurrency Errors

### Error: `std::sync::mpsc::Receiver<i32> cannot be shared between threads safely`

**Example:**
```rust
use std::sync::{Arc, mpsc};
use std::thread;

let (tx, rx) = mpsc::channel();
let rx = Arc::new(rx);

for _ in 0..3 {
    let rx = Arc::clone(&rx);
    thread::spawn(move || {
        rx.recv(); // ERROR: Receiver is not Sync
    });
}
```

**Why:** `mpsc::Receiver` is not `Sync` (can't be shared between threads).

**Solution:**
```rust
// Option 1: Use a single receiver
let (tx, rx) = mpsc::channel();
// ... send from multiple threads ...
let results: Vec<_> = rx.iter().collect();

// Option 2: Use crossbeam channel (supports multiple receivers)
```

---

### Error: `MutexGuard` dropped while still borrowed

**Example:**
```rust
let data = Arc::new(Mutex::new(vec![1, 2, 3]));
let result = data.lock().unwrap().iter().sum::<i32>();
```

**Why:** The `MutexGuard` is dropped at the end of the expression, but the borrow continues.

**Solution:**
```rust
let data = Arc::new(Mutex::new(vec![1, 2, 3]));
let guard = data.lock().unwrap();
let result = guard.iter().sum::<i32>();
drop(guard); // Explicitly drop when done
```

**Prevention:** Store the lock guard in a variable.

---

### Error: `JoinHandle` type mismatch

**Example:**
```rust
let mut handles = vec![];
handles.push(thread::spawn(|| 42));      // JoinHandle<i32>
handles.push(thread::spawn(|| {}));      // JoinHandle<()>
```

**Why:** All elements in a vector must have the same type.

**Solution:**
```rust
// Separate different return types
let mut handles_i32 = vec![];
let mut handles_unit = vec![];

handles_i32.push(thread::spawn(|| 42));
handles_unit.push(thread::spawn(|| {}));
```

---

## 4. Type System Errors

### Error: `type annotations needed`

**Example:**
```rust
let x = "42".parse().unwrap();
```

**Why:** The compiler can't infer the target type.

**Solution:**
```rust
let x: i32 = "42".parse().unwrap();
// Or
let x = "42".parse::<i32>().unwrap();
```

---

### Error: `mismatched types`

**Example:**
```rust
fn foo() -> i32 {
    42
}

fn bar() {
    let x: () = foo(); // ERROR: expected (), found i32
}
```

**Solution:**
```rust
fn bar() {
    let x: i32 = foo(); // Correct type
}
```

---

### Error: `the trait bound is not satisfied`

**Example:**
```rust
struct MyStruct {
    data: Vec<i32>,
}

impl MyStruct {
    fn new() -> Self {
        Self { data: Vec::new() }
    }
}

let s = MyStruct::new();
s.clone(); // ERROR: Clone not implemented
```

**Solution:**
```rust
#[derive(Clone)]
struct MyStruct {
    data: Vec<i32>,
}
```

---

## 5. Async/Await Errors

### Error: `async fn` in public traits

**Example:**
```rust
trait MyTrait {
    async fn my_method(&self) -> i32;
}
```

**Why:** Async traits have limitations with `Send` bounds.

**Solution:**
```rust
// Option 1: Use `async-trait` crate
#[async_trait]
trait MyTrait {
    async fn my_method(&self) -> i32;
}

// Option 2: Return boxed future
trait MyTrait {
    fn my_method(&self) -> Pin<Box<dyn Future<Output = i32> + Send>>;
}
```

---

### Error: `cannot borrow data in an `Arc` as mutable`

**Example:**
```rust
let data = Arc::new(Receiver::new());
// ... try to call recv() ...
```

**Why:** `Arc` only gives shared access. Use `Mutex` for mutation.

**Solution:**
```rust
let data = Arc::new(Mutex::new(Receiver::new()));
let mut guard = data.lock().unwrap();
guard.recv();
```

---

## 6. Pattern Matching Errors

### Error: `non-exhaustive patterns`

**Example:**
```rust
enum Color { Red, Green, Blue }

match color {
    Color::Red => {},
    Color::Green => {},
    // Missing Blue!
}
```

**Solution:**
```rust
match color {
    Color::Red => {},
    Color::Green => {},
    Color::Blue => {},
    // Or use wildcard
    _ => {},
}
```

---

## 7. Import & Module Errors

### Error: `unresolved import`

**Example:**
```rust
use my_crate::MyStruct; // ERROR: my_crate not in dependencies
```

**Solution:** Add to `Cargo.toml`:
```toml
[dependencies]
my_crate = "0.1"
```

---

### Error: `the name `tests` is defined multiple times`

**Example:**
```rust
#[cfg(test)]
mod tests { /* ... */ }

// ... more code ...

#[cfg(test)]
mod tests { /* ... */ } // ERROR: duplicate
```

**Solution:** Combine into one `tests` module.

---

## 8. Test Failures

### Error: `assertion failed: left == right`

**Common Causes:**
1. Wrong expected value
2. Off-by-one errors
3. Floating point precision

**Solution:**
```rust
// Use approximate comparison for floats
assert!((result - expected).abs() < f64::EPSILON);

// Use proper assertions
assert_eq!(actual, expected);
assert!(condition);
```

---

### Error: Test timeout / deadlock

**Common Causes:**
1. Infinite loop
2. Channel not closed
3. Mutex deadlock

**Solution:**
```rust
// Use timeout
#[test]
fn test_with_timeout() {
    let result = std::thread::spawn(|| {
        // Your test code
    });
    
    // Wait with timeout
    std::thread::sleep(Duration::from_secs(5));
}
```

---

## 📋 Quick Reference

| Error | Cause | Fix |
|-------|-------|-----|
| `cannot move out of borrowed content` | Moving from reference | Use `clone()` |
| `cannot borrow as mutable` | Mutating while borrowed | Limit borrow scope |
| `use of moved value` | Value already moved | Use `clone()` or references |
| `missing lifetime specifier` | Returning reference | Add lifetime annotation |
| `lifetime mismatch` | Different lifetimes | Use same lifetime parameter |
| `cannot be shared between threads` | Not `Sync` | Use `Mutex` or single receiver |
| `type annotations needed` | Can't infer type | Add explicit type |
| `trait bound not satisfied` | Missing trait impl | Add `#[derive(...)]` |

---

## 🎓 Key Lessons

1. **Ownership is unique** - Each value has one owner
2. **Borrowing is temporary** - References don't take ownership
3. **Lifetimes are scopes** - They track how long references live
4. **Concurrency needs safety** - Use `Arc`, `Mutex`, channels
5. **Types must match** - Rust is statically typed
6. **Patterns must be exhaustive** - Cover all cases
7. **Tests must be reliable** - Avoid flaky tests

---

**Remember:** Rust's compiler is your friend. Read error messages carefully - they often tell you exactly how to fix the problem!
