# Module 4: Testing Mastery

Build a comprehensive testing strategy for Rust projects. Learn unit test
patterns, integration testing, property-based testing with proptest, fuzzing,
mocking with mockall, test fixtures, benchmarking with criterion, snapshot
testing, coverage analysis, and test organization best practices.

## Lesson Index

| # | File | Topic |
|---|------|-------|
| 1 | `p01_unit_test_patterns.rs` | Test organization, test modules, assertions, custom assert macros |
| 2 | `p02_integration_tests.rs` | tests/ directory, shared helpers, test fixtures, binary testing |
| 3 | `p03_property_based_testing.rs` | proptest, strategies, prop_assume!, shrinking, custom generators |
| 4 | `p04_fuzzing.rs` | cargo-fuzz, libfuzzer, arbitrary crate, fuzz targets, corpus management |
| 5 | `p05_mocking_patterns.rs` | mockall, MockAll, automock, expectations, predicate-based mocking |
| 6 | `p06_test_fixtures.rs` | Test setup/teardown, temporary directories, test data builders |
| 7 | `p07_benchmarking.rs` | criterion, benchmark groups, statistical analysis, flamegraphs |
| 8 | `p08_snapshot_testing.rs` | insta, snapshot reviews, inline snapshots, redaction |
| 9 | `p09_test_coverage.rs` | cargo-tarpaulin, llvm-cov, coverage reports, coverage CI integration |
| 10 | `p10_test_organization.rs` | Test hierarchy, test naming, test utilities module, test-only code |

## Running

```bash
cargo test -p testing_mastery
```
