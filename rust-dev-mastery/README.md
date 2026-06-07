# 🦀 Rust Dev Mastery — The Ultimate Knowledge Course

> **From DSA to Staff Engineer.** Everything an experienced Rust developer knows, organized as 200 hands-on lessons across 20 modules.

## Who This Is For

You've completed the DSA section and interview guide. You can solve LeetCode problems in Rust. Now you need the knowledge that separates a junior from a senior from a staff engineer: how to architect real projects, handle errors gracefully, trace production issues, write tests that catch bugs, design APIs that developers love, and ship code that runs fast and stays running.

## How to Use

```bash
# Navigate to any module
cd 02-error-handling

# Read the theory
cat README.md

# Study each lesson (10 per module)
# Each file = one concept with explanation + working code + tests
cat src/p01_error_hierarchy.rs

# Run tests for a module
cargo test -p error_handling

# Run ALL tests across all 20 modules
cargo test --workspace

# Generate documentation
cargo doc --workspace --open
```

## Course Structure

Each module contains:
- **README.md** — Theory, patterns, real-world context, and lesson index
- **src/p01-p10.rs** — 10 lessons, each with:
  - Detailed doc comments explaining the concept
  - Production-quality code examples
  - Comprehensive test suite
  - Rust-specific tips and anti-patterns

## The 20 Modules

### 🔧 Foundation (Modules 1–4)

| # | Module | Lessons | What You'll Learn |
|---|--------|---------|-------------------|
| 01 | [Project Architecture](01-project-architecture/) | 10 | Workspace design, crate organization, feature flags, build profiles, dependency management, release engineering |
| 02 | [Error Handling](02-error-handling/) | 10 | Error hierarchies, thiserror, anyhow, client-facing errors, recovery strategies, panic handling, error testing |
| 03 | [Logging & Tracing](03-logging-tracing/) | 10 | tracing crate, structured logging, spans, events, distributed tracing, OpenTelemetry, production observability |
| 04 | [Testing](04-testing/) | 10 | Unit patterns, integration tests, property-based testing, fuzzing, mocking, benchmarks, snapshot testing, coverage |

### ⚡ Core Mastery (Modules 5–8)

| # | Module | Lessons | What You'll Learn |
|---|--------|---------|-------------------|
| 05 | [Async Rust](05-async-rust/) | 10 | Tokio deep dive, Future internals, streams, cancellation safety, async patterns, async testing, async architecture |
| 06 | [Performance & Profiling](06-performance-profiling/) | 10 | Flamegraphs, criterion, memory profiling, CPU optimization, SIMD, cache optimization, zero-cost abstractions |
| 07 | [Design Patterns](07-design-patterns/) | 10 | Builder, typestate, newtype, RAII, visitor, strategy, command, observer, state machines, dependency injection |
| 08 | [API Design](08-api-design/) | 10 | Public API design, trait-based APIs, generics vs dyn, versioning, backward compatibility, documentation |

### 🔬 Advanced (Modules 9–12)

| # | Module | Lessons | What You'll Learn |
|---|--------|---------|-------------------|
| 09 | [Unsafe & FFI](09-unsafe-ffi/) | 10 | Unsafe code patterns, FFI with C/C++, bindgen, unsafe traits, safety documentation, unsafe abstractions |
| 10 | [Macros](10-macros/) | 10 | macro_rules!, procedural macros, derive macros, attribute macros, build scripts, code generation, hygiene |
| 11 | [Serialization](11-serialization/) | 10 | Serde deep dive, custom implementations, JSON, Protobuf, schema evolution, performance, format design |
| 12 | [Networking & Web](12-networking-web/) | 10 | HTTP clients/servers, WebSocket, gRPC, REST design, middleware, authentication, rate limiting, protocol design |

### 🏗️ Systems (Modules 13–16)

| # | Module | Lessons | What You'll Learn |
|---|--------|---------|-------------------|
| 13 | [CLI Tools](13-cli-tools/) | 10 | clap, interactive prompts, progress bars, config management, cross-platform, signals, shell completion |
| 14 | [Database & Storage](14-database-storage/) | 10 | SQLx, connection pooling, migrations, ORM patterns, caching, embedded DBs, Redis, storage engines |
| 15 | [Security](15-security/) | 10 | Cryptography, secrets management, input validation, auth, TLS, supply chain security, threat modeling |
| 16 | [DevOps & CI/CD](16-devops-cicd/) | 10 | GitHub Actions, release workflows, cross-compilation, Docker, monitoring, deployment, infrastructure as code |

### 🎯 Expert (Modules 17–20)

| # | Module | Lessons | What You'll Learn |
|---|--------|---------|-------------------|
| 17 | [Concurrency](17-concurrency/) | 10 | Channels, shared state, lock-free structures, actor model, work stealing, parallel iterators, synchronization |
| 18 | [Memory Management](18-memory-management/) | 10 | Custom allocators, arenas, pool allocation, bump allocation, memory mapping, leak detection, memory profiling |
| 19 | [Interop & WASM](19-interop-wasm/) | 10 | PyO3, WebAssembly, C embedding, plugin systems, dynamic loading, cross-language testing, ABI stability |
| 20 | [Code Quality](20-code-quality/) | 10 | Clippy, rustfmt, documentation, code review, refactoring, technical debt, naming, metrics, style guide |

## Learning Paths

### 🚀 The Full Path (200 lessons, ~3–6 months)
Go in order. Each module builds on previous ones.

### ⚡ The "I Need This Now" Path
Pick the module matching your current project need:
- **Shipping a CLI?** → Modules 13, 1, 2
- **Building a web service?** → Modules 12, 14, 3, 5
- **Debugging production issues?** → Modules 3, 6, 4
- **Writing a library for others?** → Modules 8, 2, 4, 20
- **Performance critical code?** → Modules 6, 18, 17, 9

### 🏢 The "Staff Engineer" Path
Modules 1, 7, 8, 6, 3, 15, 16, 20 — architecture, design, and quality.

## Prerequisites

- Completed the DSA section (or equivalent Rust knowledge)
- Comfortable with ownership, borrowing, lifetimes
- Basic understanding of traits, generics, enums
- Rust installed (edition 2021)

## Stats

- **20 modules** covering every professional Rust topic
- **200 lessons** with production-quality code
- **200+ test suites** you can run and modify
- **Real-world patterns** used in production Rust codebases

---

**Master Rust. Build anything. Ship with confidence.** 🦀
