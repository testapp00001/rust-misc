# Module 22: Static Analysis

Clippy is your first line of defense. Static analysis catches security bugs before
they reach production -- without running the code. This module covers the full
spectrum: from built-in Clippy lints to formal verification with Kani, from
dependency auditing to CI security gates that block merges on findings.

## Why Static Analysis?

Dynamic testing (fuzzing, pen testing) finds bugs at runtime. Static analysis finds
bugs at *compile time* or in CI. The earlier you catch a vulnerability, the cheaper
it is to fix:

```
Design  -->  Code  -->  Review  -->  Test  -->  Production
  ^           ^          ^           ^           ^
  $1          $5         $50         $500        $5000+
```

Static analysis operates at the "Code" and "Review" stages -- catching issues
before they even reach testing.

## Lessons

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_clippy_security` | Clippy Security Lints | `unwrap_used`, `expect_used`, `integer_arithmetic` |
| 02 | `p02_unsafe_audit` | Unsafe Code Audit | `cargo-geiger`, finding and minimizing `unsafe` |
| 03 | `p03_miri_basics` | Miri UB Detection | Detecting undefined behavior in unsafe code |
| 04 | `p04_kani_verification` | Kani Formal Verification | Proving properties about code mathematically |
| 05 | `p05_custom_lints` | Custom Clippy Lints | Project-specific security rules via `dylint` |
| 06 | `p06_dependency_auditing` | Dependency Auditing | `cargo-audit`, `cargo-deny` in CI |
| 07 | `p07_code_review_checklist` | Security Code Review | What to look for in security-focused reviews |
| 08 | `p08_semgrep_rust` | Semgrep Patterns | Finding security anti-patterns with pattern matching |
| 09 | `p09_taint_analysis` | Taint Analysis | Tracking untrusted data through code |
| 10 | `p10_ci_security_gate` | CI Security Gate | Blocking merges on security findings |

## Quick Start

```bash
# Test your implementation
cargo test -p 22-static-analysis

# Test reference solutions
cargo test -p 22-static-analysis --features solution
```

## Tool Installation

These tools are referenced in the lessons but installed separately:

```bash
# Clippy (comes with rustup)
rustup component add clippy

# Miri (for UB detection)
rustup +nightly component add miri

# cargo-geiger (unsafe code auditing)
cargo install cargo-geiger

# cargo-audit (dependency vulnerability scanning)
cargo install cargo-audit

# cargo-deny (supply chain policy enforcement)
cargo install cargo-deny

# Kani (formal verification) -- requires nightly
cargo install --locked kani-verifier
cargo kani setup
```

## The Security Static Analysis Stack

```
Layer 1: Clippy Lints          (every build, zero cost)
   |
Layer 2: Unsafe Audit          (weekly, cargo-geiger)
   |
Layer 3: Dependency Audit      (every PR, cargo-audit + cargo-deny)
   |
Layer 4: Semgrep / Custom      (every PR, pattern-based scanning)
   |
Layer 5: Miri                  (nightly, UB detection in unsafe code)
   |
Layer 6: Kani                  (release gates, formal verification)
   |
Layer 7: CI Security Gate      (merge blocker, combines all layers)
```

## Key Takeaways

1. **Clippy is not optional** -- enable security lints project-wide via `clippy.toml`
2. **`unsafe` is a liability** -- audit it, minimize it, verify it with Miri
3. **Dependencies are attack surface** -- audit on every PR, not just at release
4. **Formal verification catches what testing cannot** -- Kani proves absence of panics
5. **CI gates enforce policy** -- if it is not automated, it will be skipped

## Dependencies

- `ring` -- cryptographic operations (demonstrates unsafe audit target)
- `sha2` -- hash functions (demonstrates dependency audit)
- `zeroize` -- secure memory zeroing (demonstrates memory safety patterns)
