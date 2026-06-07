# Module 12: Testing & Quality

Master Rust testing strategies, mocking, fuzzing, property-based testing, and benchmarks.

## Lesson Index

1. **p01_unit_testing.rs** - Test organization, assertions, test modules, test attributes, ignoring tests
2. **p02_integration_testing.rs** - tests/ directory, shared setup, test binaries, test fixtures
3. **p03_property_testing.rs** - proptest, QuickCheck, strategies, shrinking, property-based testing
4. **p04_fuzzing.rs** - cargo-fuzz, libfuzzer, fuzz targets, corpus management, coverage
5. **p05_mocking_patterns.rs** - Trait-based mocking, mock objects, dependency injection for testing
6. **p06_test_fixtures.rs** - Builder patterns for tests, factory functions, test data management
7. **p07_error_testing.rs** - Testing error paths, panic testing, Result testing, error propagation
8. **p08_benchmarking.rs** - criterion, bench functions, statistical analysis, flamegraphs
9. **p09_test_organization.rs** - Test modules, test helpers, shared utilities, test naming conventions
10. **p10_ci_quality.rs** - CI configuration, code coverage, clippy, rustfmt, documentation testing

## Key Concepts

- Rust's built-in test framework is powerful and zero-config for most cases
- Property-based testing finds edge cases that example-based tests miss
- Fuzzing discovers crashes and security vulnerabilities in unsafe code
- Good test organization scales with the codebase
