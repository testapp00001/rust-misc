# 🚀 Quick Reference: Common Rust Errors

## Error → Solution Cheat Sheet

| Error Message | Cause | Solution |
|---------------|-------|----------|
| `cannot move out of borrowed content` | Moving from reference | Use `.clone()` |
| `cannot borrow as mutable` | Mutating while borrowed | Limit borrow scope |
| `use of moved value` | Value already moved | Use `.clone()` or `Arc` |
| `missing lifetime specifier` | Returning reference | Add `'a` lifetime |
| `does not live long enough` | Temporary dropped early | Store in variable |
| `cannot be shared between threads` | Not `Sync` | Use `Mutex` or single receiver |
| `type annotations needed` | Can't infer type | Add `::<Type>` |
| `trait bound not satisfied` | Missing trait impl | Add `#[derive(...)]` |
| `mismatched types` | Wrong type | Check function signatures |
| `non-exhaustive patterns` | Missing match arm | Add `_ => {}` |

---

## Ownership & Borrowing

### ❌ Wrong
```rust
let s = String::from("hello");
let r = &s;
let s2 = *r; // ERROR: cannot move out of borrowed content
```

### ✅ Right
```rust
let s = String::from("hello");
let r = &s;
let s2 = r.clone(); // Clone instead of move
```

---

### ❌ Wrong
```rust
let mut data = vec![1, 2, 3];
let first = &data[0];
data.push(4); // ERROR: cannot borrow as mutable
println!("{}", first);
```

### ✅ Right
```rust
let mut data = vec![1, 2, 3];
let first = data[0]; // Copy the value
data.push(4);
println!("{}", first);
```

---

## Lifetimes

### ❌ Wrong
```rust
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() { x } else { y }
}
```

### ✅ Right
```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

---

### ❌ Wrong
```rust
let r;
{
    let x = 5;
    r = &x;
}
println!("{}", r); // ERROR: x doesn't live long enough
```

### ✅ Right
```rust
let x = 5;
let r = &x;
println!("{}", r);
```

---

## Concurrency

### ❌ Wrong
```rust
let (tx, rx) = mpsc::channel();
let rx = Arc::new(rx);
for _ in 0..3 {
    let rx = Arc::clone(&rx);
    thread::spawn(move || {
        rx.recv(); // ERROR: Receiver not Sync
    });
}
```

### ✅ Right
```rust
let (tx, rx) = mpsc::channel();
// Single receiver
let results: Vec<_> = rx.iter().collect();
```

---

### ❌ Wrong
```rust
let data = Arc::new(Mutex::new(0));
let result = data.lock().unwrap() + 1; // ERROR: guard dropped
```

### ✅ Right
```rust
let data = Arc::new(Mutex::new(0));
let result = *data.lock().unwrap() + 1;
// Or store guard:
let guard = data.lock().unwrap();
let result = *guard + 1;
```

---

### ❌ Wrong
```rust
let data = vec![1, 2, 3, 4, 5];
for i in 0..data.len() {
    thread::spawn(move || {
        data[i] * 2 // ERROR: data moved
    });
}
```

### ✅ Right
```rust
let data = Arc::new(vec![1, 2, 3, 4, 5]);
for i in 0..5 {
    let data = Arc::clone(&data);
    thread::spawn(move || {
        data[i] * 2
    });
}
```

---

## Types

### ❌ Wrong
```rust
let x = "42".parse().unwrap(); // ERROR: type needed
```

### ✅ Right
```rust
let x: i32 = "42".parse().unwrap();
// or
let x = "42".parse::<i32>().unwrap();
```

---

### ❌ Wrong
```rust
struct MyStruct { data: i32 }
let s = MyStruct { data: 42 };
s.clone(); // ERROR: Clone not implemented
```

### ✅ Right
```rust
#[derive(Clone)]
struct MyStruct { data: i32 }
let s = MyStruct { data: 42 };
s.clone();
```

---

## Pattern Matching

### ❌ Wrong
```rust
enum Color { Red, Green, Blue }
match color {
    Color::Red => {},
    Color::Green => {},
    // Missing Blue!
}
```

### ✅ Right
```rust
match color {
    Color::Red => {},
    Color::Green => {},
    Color::Blue => {},
}
```

---

## Async/Await

### ❌ Wrong
```rust
trait MyTrait {
    async fn my_method(&self) -> i32; // Limitations
}
```

### ✅ Right
```rust
trait MyTrait {
    fn my_method(&self) -> Pin<Box<dyn Future<Output = i32> + Send>>;
}
```

---

## Common Patterns

### Builder Pattern
```rust
struct Builder {
    value: i32,
    name: String,
}

impl Builder {
    pub fn new() -> Self { /* ... */ }
    pub fn value(mut self, v: i32) -> Self { self.value = v; self }
    pub fn name(mut self, n: &str) -> Self { self.name = n.to_string(); self }
    pub fn build(self) -> Config { /* ... */ }
}
```

### RAII Pattern
```rust
struct Resource { name: String }

impl Drop for Resource {
    fn drop(&mut self) {
        println!("Releasing: {}", self.name);
    }
}
```

---

## Debugging Tips

1. **Read the error message** - Rust errors are very helpful
2. **Check bracket matching** - Many syntax errors
3. **Trace ownership** - Who owns what?
4. **Check lifetimes** - How long do references live?
5. **Verify types** - Do types match?
6. **Test incrementally** - Don't write too much at once

---

**Remember:** The Rust compiler is your friend. It catches bugs that other languages would miss!
