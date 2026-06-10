# Module 15: Memory Security

> "Rust's memory safety guarantees protect you from buffer overflows and use-after-free — but they do NOT protect your secrets from being leaked through timing, memory dumps, or careless logging."

## Overview

Rust's ownership system prevents many classes of memory bugs, but **memory safety is not cryptographic safety**. Secrets can leak through:
- **Residual memory**: Freed memory still contains secret bytes; the allocator reuses it
- **Core dumps**: Crash files capture all process memory, including secrets
- **Swap space**: OS may write secret-containing pages to disk
- **Timing side-channels**: Variable-time operations reveal secret-dependent information
- **Logging/printing**: `Debug`/`Display` impls may expose secrets in logs
- **Memory forensics**: Attackers with physical access can read RAM contents

## Key Concepts

### Rust Memory Safety vs. Crypto Safety

```
Rust guarantees:                    Rust does NOT guarantee:
├── No buffer overflows             ├── Secrets are zeroed on drop
├── No use-after-free               ├── Constant-time operations
├── No data races                   ├── Protection from core dumps
├── No null pointer deref           ├── Protection from swap/pagefile
└── Single ownership                └── Secrets never appear in logs
```

### The Zeroize Pattern

Standard `Drop` implementation does NOT zero memory — it just marks it as free.
The `zeroize` crate provides a `Zeroize` trait that overwrites memory with zeros
BEFORE dropping, and uses compiler fences to prevent optimization from removing the wipe.

```rust
// WITHOUT zeroize: secret bytes remain in freed memory
let mut secret = vec![0x42u8; 32];
drop(secret); // memory still contains 0x42... until reused

// WITH zeroize: memory is overwritten before free
use zeroize::Zeroize;
let mut secret = vec![0x42u8; 32];
secret.zeroize(); // memory is now all zeros
drop(secret);
```

### The Secrecy Pattern

`Secret<T>` wraps sensitive values to prevent accidental exposure:

```rust
use secrecy::{Secret, ExposeSecret};

let password = Secret::new("hunter2".to_string());
// println!("{}", password);        // COMPILE ERROR — no Display/Debug impl
// let leaked = password.clone();   // COMPILE ERROR — no Clone impl
println!("{}", password.expose_secret()); // Must explicitly opt-in
```

### Timing Side-Channels

Variable-time comparison leaks information about where strings differ:

```
Attacker's guess:  "AAAA"
Actual secret:     "BAAA"
Comparison time:   ████████████████░░░░░░░░░░░░░░░░  (fast — fails at byte 0)

Attacker's guess:  "BAAA"
Actual secret:     "BAAA"
Comparison time:   ██████████████████████████████████  (slow — checks all bytes)
```

Defense: Always compare in constant time (check ALL bytes regardless of match).

## Attack Patterns

### Memory Residual Attack
After a program frees a buffer containing a secret, that memory is returned to the
allocator but NOT erased. Another allocation can reuse that memory and read the secret.
**Defense**: Use `zeroize` to overwrite before free, `mlock` to prevent swap.

### Core Dump Inspection
When a process crashes, the OS writes a core dump containing all mapped memory.
An attacker who obtains the core dump can extract secrets from it.
**Defense**: Disable core dumps (`setrlimit`), or encrypt them.

### Cache-Timing Attack
Modern CPUs have data-dependent cache behavior. By measuring cache access times,
an attacker can infer which table entries were accessed, leaking secret-dependent indices.
**Defense**: Use constant-time algorithms (no secret-dependent branches or memory access).

### Power Analysis (conceptual)
Hardware power consumption varies with data being processed. Simple Power Analysis (SPA)
and Differential Power Analysis (DPA) can extract cryptographic keys from hardware devices.
**Defense**: Hardware countermeasures, constant-time software implementations.

## Rust-Specific Tips

1. **`zeroize` crate**: Use `#[derive(Zeroize)]` on structs holding secrets; call `.zeroize()` or rely on `Drop` (with `#[zeroize(drop)]`)
2. **`secrecy` crate**: Wrap all passwords, keys, tokens in `Secret<T>` to prevent accidental logging
3. **`ring` crate**: Uses constant-time operations internally for crypto primitives
4. **`mlock`**: Use `libc::mlock` (or nix crate) to prevent secret pages from being swapped
5. **Never `#[derive(Debug)]` on secret-containing structs** — this is what `secrecy` prevents
6. **Stack vs Heap**: Small fixed-size secrets (keys, nonces) are safer on the stack — automatically cleaned up on scope exit, not subject to allocator reuse

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_zeroize_basics.rs` | Zeroize trait | Zero memory on drop, prevent optimization removal |
| 02 | `p02_secrecy_crate.rs` | Secret<T> wrapper | Prevent accidental printing/logging of secrets |
| 03 | `p03_constant_time_ops.rs` | Constant-time ops | Prevent timing side-channels in crypto code |
| 04 | `p04_mlock_memory.rs` | mlock/mprotect | Prevent secrets from being swapped to disk |
| 05 | `p05_guard_pages.rs` | Guard pages | Detect buffer overflows, stack canaries |
| 06 | `p06_memory_forensics_defense.rs` | Memory forensics | Minimize secret lifetime, zero on drop |
| 07 | `p07_core_dump_protection.rs` | Core dump handling | Disable/encrypt core dumps to protect secrets |
| 08 | `p08_stack_vs_heap.rs` | Stack vs heap | When to use stack (small, fixed) vs heap |
| 09 | `p09_secure_allocator.rs` | Secure allocator | malloc alternatives that zero on free |
| 10 | `p10_side_channel_defense.rs` | Side-channel defense | Cache-timing, branch prediction, power analysis |

## References

- [zeroize crate documentation](https://docs.rs/zeroize)
- [secrecy crate documentation](https://docs.rs/secrecy)
- [Constant-Time Implementations (BoringSSL)](https://boringssl.googlesource.com/boringssl/+/HEAD/CRYPTO_LIBRARY.md)
- [Timing Attacks on Implementations of Diffie-Hellman, RSA, DSS (Kocher 1996)](https://paulkocher.com/doc/TimingAttacks.pdf)
- [NIST SP 800-57: Key Management](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)
- [Memory Hardness for Password Hashing (Argon2)](https://www.password-hashing.net/argon2-specs.pdf)
