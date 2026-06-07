# 06 - Mid-Level Unsafe

## 🎯 Interview Focus
- Unsafe Rust
- Raw pointers
- FFI (Foreign Function Interface)
- Undefined behavior
- Memory safety

## 📚 Core Concepts

### 1. Unsafe Rust
```rust
unsafe {
    // Code that bypasses Rust's safety checks
    let raw = &mut data as *mut i32;
    *raw = 42;
}
```

### 2. Raw Pointers
```rust
let mut x = 42;
let raw = &mut x as *mut i32;
unsafe {
    *raw = 100;
}
```

### 3. FFI
```rust
extern "C" {
    fn abs(input: i32) -> i32;
}

let result = unsafe { abs(-42) };
```

### 4. Unsafe Traits
```rust
unsafe trait MyTrait {
    fn my_method(&self);
}

unsafe impl MyTrait for i32 {
    fn my_method(&self) {}
}
```

## ❓ Common Interview Questions

### Q1: When should you use `unsafe`?
**A:** Only when necessary: FFI, raw pointer manipulation, implementing unsafe traits, or performance-critical code that can't be expressed safely.

### Q2: What's undefined behavior?
**A:** Behavior that the Rust compiler doesn't guarantee. Examples: dereferencing null pointers, data races, violating memory safety.

### Q3: What's the difference between `unsafe` and `unsafe impl`?
**A:** `unsafe` is a block that allows unsafe operations. `unsafe impl` is used to implement unsafe traits.

## 🧪 Problems

1. **p01_unsafe_basics** - Practice with unsafe blocks
2. **p02_raw_pointers** - Practice with raw pointers
3. **p03_ffi** - Practice with FFI
4. **p04_unsafe_traits** - Practice with unsafe traits
5. **p05_memory_safety** - Practice with memory safety
6. **p06_unsafe_abstractions** - Practice with safe abstractions
7. **p07_unsafe_patterns** - Common unsafe patterns
8. **p08_unsafe_performance** - Performance with unsafe
9. **p09_unsafe_testing** - Testing unsafe code
10. **p10_unsafe_best_practices** - Best practices

## 💡 Rust Tips

1. **Minimize unsafe code** - Use it only when necessary
2. **Document safety invariants** - Explain why unsafe code is safe
3. **Use safe abstractions** - Wrap unsafe code in safe APIs
4. **Test unsafe code thoroughly** - Use Miri for detection
5. **Review unsafe code carefully** - Look for undefined behavior

## 🔗 Resources

- [Unsafe Rust](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)
- [FFI](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#extern-functions-and-calling-external-functions)
- [Miri](https://github.com/rust-lang/miri)
