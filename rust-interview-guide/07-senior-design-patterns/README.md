# 07 - Senior Design Patterns

## 🎯 Interview Focus
- Builder pattern
- Observer pattern
- Strategy pattern
- State machine
- Repository pattern

## 📚 Core Concepts

### 1. Builder Pattern
```rust
struct QueryBuilder {
    table: String,
    conditions: Vec<String>,
}

impl QueryBuilder {
    fn new(table: &str) -> Self { ... }
    fn where_clause(self, condition: &str) -> Self { ... }
    fn build(self) -> String { ... }
}
```

### 2. Observer Pattern
```rust
trait Observer {
    fn update(&self, event: &Event);
}

struct EventSystem {
    observers: Vec<Box<dyn Observer>>,
}
```

### 3. Strategy Pattern
```rust
trait Strategy {
    fn execute(&self, data: &[i32]) -> i32;
}

struct Context {
    strategy: Box<dyn Strategy>,
}
```

## ❓ Common Interview Questions

### Q1: When would you use the Builder pattern?
**A:** When constructing complex objects with many optional parameters. It provides a fluent API and ensures objects are fully initialized.

### Q2: What's the Observer pattern?
**A:** A pattern where an object (subject) maintains a list of dependents (observers) and notifies them of state changes.

### Q3: What's the Strategy pattern?
**A:** A pattern that defines a family of algorithms, encapsulates each one, and makes them interchangeable.

## 🧪 Problems

1. **p01_builder** - Practice with builder pattern
2. **p02_observer** - Practice with observer pattern
3. **p03_strategy** - Practice with strategy pattern
4. **p04_state_machine** - Practice with state machines
5. **p05_repository** - Practice with repository pattern
6. **p06_factory** - Practice with factory pattern
7. **p07_command** - Practice with command pattern
8. **p08_iterator** - Practice with iterator pattern
9. **p09_decorator** - Practice with decorator pattern
10. **p10_adapter** - Practice with adapter pattern

## 💡 Rust Tips

1. **Use traits for polymorphism** - Rust's way of achieving dynamic dispatch
2. **Use enums for state machines** - Rust's enums are perfect for state machines
3. **Use builder for complex construction** - Especially with many optional fields
4. **Use RAII for resource management** - Drop trait for cleanup
5. **Use type system for safety** - Leverage Rust's type system

## 🔗 Resources

- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [Builder Pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html)
- [State Machine](https://hoverbear.org/blog/rust-state-machine-pattern/)
