# 01 - Fresher Basics

## 🎯 Interview Focus
- Variables, mutability, and shadowing
- Data types and type inference
- Control flow (if, match, loops)
- Functions and closures
- Structs and enums
- Pattern matching
- Collections (Vec, HashMap, String)
- Error handling (Result, Option)

## 📚 Core Concepts

### 1. Variables and Mutability
```rust
let x = 5;           // Immutable
let mut y = 10;      // Mutable
let x = x + 1;      // Shadowing (creates new binding)
const MAX: i32 = 100; // Constant (must annotate type)
```

### 2. Data Types
```rust
// Scalar types
let整数: i32 = 42;        // Signed integer
let浮点: f64 = 3.14;      // Floating point
let布尔: bool = true;     // Boolean
let字符: char = 'A';      // Character

// Compound types
let元组: (i32, f64, bool) = (1, 2.0, true);
let数组: [i32; 3] = [1, 2, 3];
```

### 3. Control Flow
```rust
// if expression
let x = if condition { 5 } else { 10 };

// match expression (like switch)
match value {
    1 => println!("one"),
    2..=5 => println!("two to five"),
    _ => println!("other"),
}

// Loops
loop { break; }           // Infinite loop
while condition { }       // While loop
for i in 0..10 { }       // For loop
```

### 4. Functions
```rust
fn add(a: i32, b: i32) -> i32 {
    a + b  // No semicolon = return value
}

// Closures
let add = |a, b| a + b;
let add = |a: i32, b: i32| -> i32 { a + b };
```

### 5. Structs and Enums
```rust
struct Point {
    x: f64,
    y: f64,
}

enum Direction {
    North,
    South,
    East,
    West,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### 6. Pattern Matching
```rust
match value {
    Some(x) if x > 0 => println!("Positive: {}", x),
    Some(x) => println!("Non-positive: {}", x),
    None => println!("None"),
}
```

### 7. Collections
```rust
// Vec (dynamic array)
let mut vec = Vec::new();
vec.push(1);
vec[0]  // Access by index

// HashMap
let mut map = HashMap::new();
map.insert("key", "value");
map.get("key")  // Returns Option<&V>

// String
let s = String::from("hello");
let s = "hello".to_string();
```

### 8. Error Handling
```rust
// Result<T, E>
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// Option<T>
fn find(arr: &[i32], target: i32) -> Option<usize> {
    arr.iter().position(|&x| x == target)
}

// ? operator
fn process() -> Result<i32, String> {
    let value = divide(10.0, 2.0)?;
    Ok(value as i32)
}
```

## ❓ Common Interview Questions

### Q1: What's the difference between `let` and `let mut`?
**A:** `let` creates an immutable binding (cannot change the value). `let mut` creates a mutable binding (can change the value).

### Q2: What's shadowing?
**A:** Shadowing allows you to declare a new variable with the same name as a previous variable. The new variable "shadows" the previous one.

### Q3: What's the difference between `String` and `&str`?
**A:** `String` is an owned, growable string. `&str` is a string slice (reference to a string). `String` can be converted to `&str` via `&s` or `s.as_str()`.

### Q4: What's the `?` operator?
**A:** The `?` operator unwraps a `Result` or `Option`. If it's `Err` or `None`, it returns early from the function. If it's `Ok` or `Some`, it unwraps the value.

### Q5: What's the difference between `panic!` and `Result`?
**A:** `panic!` crashes the program immediately. `Result` allows graceful error handling. Use `panic!` for programming errors, `Result` for expected failures.

## 🧪 Problems

1. **p01_variables_and_types** - Practice with variables, types, and type inference
2. **p02_control_flow** - Practice with if, match, and loops
3. **p03_functions_closures** - Practice with functions and closures
4. **p04_structs_enums** - Practice with structs and enums
5. **p05_pattern_matching** - Practice with pattern matching
6. **p06_collections** - Practice with Vec, HashMap, String
7. **p07_error_handling** - Practice with Result, Option, and ?
8. **p08_iterators** - Practice with iterator methods
9. **p09_traits_generics** - Practice with traits and generics
10. **p10_modules_visibility** - Practice with modules and visibility

## 💡 Rust Tips

1. **Prefer `&str` over `String`** in function parameters (more flexible)
2. **Use `clone()` sparingly** - it copies data, which can be expensive
3. **Pattern matching is powerful** - use it instead of if-else chains
4. **The `?` operator** makes error handling clean
5. **Iterators are idiomatic** - prefer `.iter().map().collect()` over loops

## 🔗 Resources

- [Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)
- [Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html)
- [Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html)
- [Structs](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
