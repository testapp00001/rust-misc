# Module 7: Design Patterns

Rust-idiomatic design patterns that leverage the type system, ownership model, and
trait system. Covers builder, typestate, newtype, RAII, visitor, strategy, command,
observer, state machines, and dependency injection.

## Lesson Index

| # | File | Topic |
|---|------|-------|
| 1 | `p01_builder_pattern.rs` | Builder with required/optional fields, type-safe builders, derive_builder |
| 2 | `p02_typestate_pattern.rs` | Compile-time state machines, phantom types, state transitions at type level |
| 3 | `p03_newtype_pattern.rs` | Wrapper types, From/Into, Deref, preventing primitive obsession |
| 4 | `p04_raii_pattern.rs` | Resource management, Drop, scope guards, RAII wrappers |
| 5 | `p05_visitor_pattern.rs` | Trait-based visitors, enum dispatch, double dispatch, accept/visit |
| 6 | `p06_strategy_pattern.rs` | Trait objects vs generics for strategies, runtime strategy selection |
| 7 | `p07_command_pattern.rs` | Command trait, undo/redo, command queues, macro command |
| 8 | `p08_observer_pattern.rs` | Event systems, callbacks, subscriber lists, weak references |
| 9 | `p09_state_machine.rs` | Runtime state machines, enum-based states, transition tables, guards |
| 10 | `p10_dependency_injection.rs` | Constructor injection, service locator, trait-based DI, testing with DI |
