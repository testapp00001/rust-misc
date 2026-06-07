# 02 - Fresher Ownership

## 🎯 Interview Focus
- Ownership rules and semantics
- Borrowing (immutable and mutable)
- Lifetimes and annotations
- References and dereferencing
- Move semantics
- Clone and Copy traits
- Smart pointers (Box, Rc, RefCell)

## 📚 Core Concepts

### 1. Ownership Rules
```rust
// Rule 1: Each value has one owner
let s1 = String::from("hello");
let s2 = s1; // s1 is moved to s2, s1 is no longer valid

// Rule 2: When owner goes out of scope, value is dropped
{
    let s = String::from("hello");
    // s is valid here
} // s is dropped here
```

### 2. Borrowing
```rust
// Immutable borrow (can have multiple)
let s = String::from("hello");
let len = calculate_length(&s); // Borrow s

// Mutable borrow (only one at a time)
let mut s = String::from("hello");
change(&mut s); // Mutable borrow
```

### 3. Lifetimes
```rust
// Lifetime annotation
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Struct with lifetime
struct Excerpt<'a> {
    part: &'a str,
}
```

### 4. Move Semantics
```rust
// Stack data (Copy trait) - copied
let x = 5;
let y = x; // x is still valid

// Heap data (no Copy trait) - moved
let s1 = String::from("hello");
let s2 = s1; // s1 is no longer valid
```

### 5. Clone and Copy
```rust
// Clone - explicit deep copy
let s1 = String::from("hello");
let s2 = s1.clone(); // Both valid

// Copy - implicit copy for stack data
let x = 5;
let y = x; // Both valid (i32 implements Copy)
```

### 6. Smart Pointers
```rust
// Box<T> - heap allocation
let b = Box::new(5);

// Rc<T> - reference counting (multiple owners)
use std::rc::Rc;
let a = Rc::new(5);
let b = Rc::clone(&a);

// RefCell<T> - interior mutability
use std::cell::RefCell;
let x = RefCell::new(5);
*x.borrow_mut() += 1;
```

## ❓ Common Interview Questions

### Q1: What are the ownership rules?
**A:** 
1. Each value has one owner
2. When the owner goes out of scope, the value is dropped
3. You can have one mutable reference OR multiple immutable references

### Q2: What's the difference between `String` and `&str`?
**A:** `String` is owned (heap-allocated, growable). `&str` is a borrowed string slice (reference to a string).

### Q3: What's a lifetime?
**A:** A lifetime is the scope for which a reference is valid. The compiler uses lifetimes to ensure references don't outlive the data they point to.

### Q4: What's the difference between `clone()` and `Copy`?
**A:** `clone()` is an explicit deep copy. `Copy` is an implicit copy for stack data (like integers). `Copy` types are automatically copied when assigned.

### Q5: When would you use `Box<T>`?
**A:** Use `Box<T>` when you have a large amount of data on the heap, when you want to own a trait object, or when you have a recursive data structure.

## 🧪 Problems

1. **p01_ownership_basics** - Practice with ownership and moves
2. **p02_borrowing** - Practice with references
3. **p03_lifetimes** - Practice with lifetime annotations
4. **p04_move_semantics** - Practice with move and copy
5. **p05_clone_copy** - Practice with Clone and Copy traits
6. **p06_smart_pointers** - Practice with Box, Rc, RefCell
7. **p07_string_ownership** - Practice with String and &str
8. **p08_struct_ownership** - Practice with structs and ownership
9. **p09_closure_ownership** - Practice with closures and ownership
10. **p10_ownership_patterns** - Common ownership patterns

## 💡 Rust Tips

1. **Prefer borrowing** over ownership when possible
2. **Use `clone()` sparingly** - it's expensive
3. **Understand move semantics** - it's different from other languages
4. **Lifetimes can be tricky** - practice with examples
5. **Smart pointers have overhead** - use them when needed

## 🔗 Resources

- [Ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
- [References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
