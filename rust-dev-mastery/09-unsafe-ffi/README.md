# Module 09: Unsafe Rust & FFI

Master unsafe Rust for systems programming, FFI with C/C++, and building safe abstractions
over unsafe code.

## Lesson Index

1. **p01_unsafe_basics.rs** - When to use unsafe, unsafe blocks, what unsafe unlocks, unsafe audit
2. **p02_raw_pointers.rs** - `*const T`, `*mut T`, dereferencing, pointer arithmetic, null checks
3. **p03_ffi_with_c.rs** - `extern "C"`, `#[repr(C)]`, calling C functions, passing strings, callbacks
4. **p04_bindgen_usage.rs** - bindgen setup, generating bindings, wrapper functions, build.rs integration
5. **p05_unsafe_traits.rs** - Send, Sync, unsafe trait impls, marker traits, safety invariants
6. **p06_memory_safety.rs** - Preventing UB, aliasing rules, uninitialized memory, alignment, provenance
7. **p07_unsafe_abstractions.rs** - Creating safe wrappers, encapsulation, safety documentation
8. **p08_unsafe_patterns.rs** - Common unsafe patterns, transmute, union, ManuallyDrop, MaybeUninit
9. **p09_unsafe_testing.rs** - Testing unsafe code, Miri, sanitizer, loom for concurrency
10. **p10_safety_documentation.rs** - SAFETY comments, invariant documentation, safety contracts, audit checklists

## Key Concepts

- **unsafe** does not mean "dangerous" -- it means "the compiler cannot verify safety; the programmer must"
- Every `unsafe` block is a trust boundary where the programmer guarantees safety invariants
- FFI is inherently unsafe because Rust cannot verify invariants across language boundaries
- Safe abstractions over unsafe code are the hallmark of well-designed Rust libraries
