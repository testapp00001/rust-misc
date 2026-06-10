# Module 21: Fuzzing and Testing

> "Testing shows the presence of bugs, never their absence. Fuzzing finds the ones you never thought to look for."

## Overview

Fuzzing is the practice of bombarding code with random, semi-random, and adversarial inputs to uncover bugs, crashes, and security vulnerabilities. This module covers:

- **Fuzzing fundamentals**: cargo-fuzz, libFuzzer, corpus management
- **Property-based testing**: proptest — defining invariants, shrinking failures
- **Crypto fuzzing**: Testing hash functions, encryption, encoding for robustness
- **Parser fuzzing**: JSON, XML, and custom parsers under adversarial input
- **Differential fuzzing**: Comparing two implementations for behavioral divergence
- **Coverage-guided fuzzing**: Understanding how modern fuzzers explore code paths
- **Security test harnesses**: Structured testing for auth, crypto, and input validation
- **Regression tests**: Capturing and replaying discovered vulnerabilities
- **Mock security components**: Isolating units under test from real crypto/network
- **Continuous fuzzing**: CI integration, long-running fuzz campaigns

## Key Concepts

### Why Fuzz Security Code?

Security code has a unique property: attackers actively search for inputs that cause unexpected behavior. Traditional unit tests check what you *thought* of. Fuzzing finds what you *didn't*.

```
Unit test:   Does sha256("hello") == expected?
Fuzz test:   Does sha256(arbitrary_bytes) ever panic? Return wrong length? Leak memory?
```

### Fuzzing vs Property-Based Testing

| Aspect | Fuzzing (cargo-fuzz) | Property-based (proptest) |
|--------|---------------------|--------------------------|
| Input generation | Random/mutational | Structured strategies |
| Execution | libFuzzer (C, fast) | Pure Rust |
| Coverage guidance | Yes | No |
| Shrinking | Built-in | Built-in |
| Use case | Crash/panic hunting | Invariant checking |
| CI friendly | Slow (needs time) | Fast (deterministic) |

### The Fuzzing Workflow

```
1. Write fuzz target (function that takes arbitrary bytes)
2. Run fuzzer with time limit: cargo fuzz run target_name -- -max_total_time=60
3. Fuzzer generates inputs, measures coverage, mutates winners
4. If crash found → minimized test case in artifacts/
5. Convert crash to regression test
6. Fix the bug
7. Re-run fuzzer to find more
```

### Security-Relevant Properties to Fuzz

1. **No panics**: Security code must NEVER panic on untrusted input
2. **Determinism**: Same input always produces same output
3. **Length bounds**: Output length matches specification
4. **Invertibility**: decrypt(encrypt(x)) == x
5. **No leaks**: Sensitive data doesn't appear in error messages
6. **Graceful degradation**: Invalid input returns error, not crash

## Rust-Specific Tips

1. `cargo fuzz` requires nightly Rust: `rustup install nightly`
2. Fuzz targets must take `&[u8]` — parse the bytes into your type
3. Use `#[cfg(fuzzing)]` to gate fuzz-specific code
4. proptest's `prop_assert!` gives better shrinking than `assert!`
5. Run fuzz tests with `--release` for 10x speed
6. Use `libfuzzer_sys::fuzz_target!` macro for cargo-fuzz targets

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_fuzz_basics.rs` | Fuzzing fundamentals | cargo-fuzz, libFuzzer, fuzz target structure |
| 02 | `p02_proptest_basics.rs` | Property-based testing | proptest strategies, shrinking, invariants |
| 03 | `p03_crypto_fuzzing.rs` | Crypto function fuzzing | Hash robustness, encoding edge cases |
| 04 | `p04_parser_fuzzing.rs` | Parser fuzzing | JSON/XML parsing under adversarial input |
| 05 | `p05_differential_fuzzing.rs` | Differential fuzzing | Comparing implementations for divergence |
| 06 | `p06_coverage_guided.rs` | Coverage-guided fuzzing | Code coverage, path exploration |
| 07 | `p07_security_test_harness.rs` | Security test harness | Structured security testing framework |
| 08 | `p08_regression_tests.rs` | Regression tests | Capturing and replaying bug-triggering inputs |
| 09 | `p09_mock_security.rs` | Mock security components | Isolating units from real crypto/network |
| 10 | `p10_continuous_fuzzing.rs` | CI integration | Automating fuzzing in CI pipelines |

## Quick Test

```bash
cargo test -p fuzzing_and_testing              # Test your implementation
cargo test -p fuzzing_and_testing --features solution  # Test reference solution
```

## References

- [cargo-fuzz book](https://rust-fuzz.github.io/book/)
- [proptest documentation](https://docs.rs/proptest/)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [OWASP Fuzzing Guide](https://owasp.org/www-community/Fuzzing)
- [Google OSS-Fuzz](https://google.github.io/oss-fuzz/)
