# 05 - Mid-Level Systems

## 🎯 Interview Focus
- File I/O
- Networking
- Serialization
- Error handling
- CLI tools

## 📚 Core Concepts

### 1. File I/O
```rust
use std::fs::File;
use std::io::{self, Read, Write};

let mut file = File::open("file.txt")?;
let mut contents = String::new();
file.read_to_string(&mut contents)?;
```

### 2. Serialization
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Data {
    name: String,
    age: u32,
}

let json = serde_json::to_string(&data)?;
let data: Data = serde_json::from_str(&json)?;
```

### 3. Error Handling
```rust
use anyhow::Result;

fn process() -> Result<()> {
    let data = std::fs::read_to_string("file.txt")?;
    // Process data
    Ok(())
}
```

## ❓ Common Interview Questions

### Q1: What's the difference between `anyhow` and `thiserror`?
**A:** `anyhow` is for application code (easy error handling). `thiserror` is for library code (custom error types).

### Q2: How do you handle errors in Rust?
**A:** Use `Result<T, E>` for recoverable errors, `panic!` for unrecoverable errors. Use `?` operator for propagation.

### Q3: What's serialization?
**A:** Converting data structures to a format (JSON, YAML, etc.) that can be stored or transmitted.

## 🧪 Problems

1. **p01_file_io** - Practice with file I/O
2. **p02_serialization** - Practice with serde
3. **p03_error_handling** - Practice with error handling
4. **p04_cli_tools** - Practice with CLI tools
5. **p05_networking** - Practice with networking
6. **p06_logging** - Practice with logging
7. **p07_configuration** - Practice with configuration
8. **p08_database** - Practice with database
9. **p09_testing** - Practice with testing
10. **p10_performance** - Practice with performance

## 💡 Rust Tips

1. **Use `anyhow`** for application error handling
2. **Use `thiserror`** for library error types
3. **Use `serde`** for serialization
4. **Use `clap`** for CLI tools
5. **Use `tracing`** for logging

## 🔗 Resources

- [File I/O](https://doc.rust-lang.org/book/ch12-02-reading-a-file.html)
- [Serde](https://serde.rs/)
- [Anyhow](https://docs.rs/anyhow/)
- [Thiserror](https://docs.rs/thiserror/)
