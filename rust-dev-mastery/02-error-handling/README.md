# Module 2: Error Handling

Design robust error hierarchies for Rust applications. Learn when to use
thiserror vs anyhow, how to build custom error types, add context to errors,
create client-facing error responses, implement recovery strategies, and
test error paths thoroughly.

## Lesson Index

| # | File | Topic |
|---|------|-------|
| 1 | `p01_error_hierarchy.rs` | Designing error enums, grouping by domain, nesting errors |
| 2 | `p02_thiserror_patterns.rs` | Derive macros, #[from], #[source], #[context], display formatting |
| 3 | `p03_anyhow_patterns.rs` | anyhow::Result, context, chain, downcasting, when to use anyhow vs thiserror |
| 4 | `p04_custom_error_types.rs` | Manual error impl, Error trait, Display, Debug, source(), backtrace |
| 5 | `p05_error_context_chaining.rs` | Adding context, error chains, preserving root causes, wrap/with_context |
| 6 | `p06_client_facing_errors.rs` | API error responses, error serialization, hiding internals, status codes |
| 7 | `p07_error_recovery.rs` | Retry patterns, fallback strategies, partial success, error classification |
| 8 | `p08_panic_handling.rs` | catch_unwind, panic hooks, set_hook, abort vs unwind, panic safety |
| 9 | `p09_error_testing.rs` | Testing error paths, assert matches, error chain inspection, test helpers |
| 10 | `p10_error_documentation.rs` | Documenting errors, error enums as API contract, examples in docs |

## Running

```bash
cargo test -p error_handling
```
