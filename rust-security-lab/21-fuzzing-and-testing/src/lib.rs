//! # Module 21: Fuzzing and Testing
//!
//! Fuzzing, property-based testing, and security testing strategies.
//!
//! ## Learning Path
//! 1. Start with `p01_fuzz_basics` — understand fuzzing fundamentals
//! 2. Learn `p02_proptest_basics` — property-based testing with proptest
//! 3. Apply fuzzing to crypto (`p03`), parsers (`p04`), and differential testing (`p05`)
//! 4. Build up to test harnesses, regression suites, and CI integration
//!
//! ## Quick Test
//! ```bash
//! cargo test -p fuzzing_and_testing              # Test your implementation
//! cargo test -p fuzzing_and_testing --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_fuzz_basics;
#[cfg(not(feature = "solution"))]
pub mod p02_proptest_basics;
#[cfg(not(feature = "solution"))]
pub mod p03_crypto_fuzzing;
#[cfg(not(feature = "solution"))]
pub mod p04_parser_fuzzing;
#[cfg(not(feature = "solution"))]
pub mod p05_differential_fuzzing;
#[cfg(not(feature = "solution"))]
pub mod p06_coverage_guided;
#[cfg(not(feature = "solution"))]
pub mod p07_security_test_harness;
#[cfg(not(feature = "solution"))]
pub mod p08_regression_tests;
#[cfg(not(feature = "solution"))]
pub mod p09_mock_security;
#[cfg(not(feature = "solution"))]
pub mod p10_continuous_fuzzing;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_fuzz_basics.rs"]
pub mod p01_fuzz_basics;
#[cfg(feature = "solution")]
#[path = "solution/p02_proptest_basics.rs"]
pub mod p02_proptest_basics;
#[cfg(feature = "solution")]
#[path = "solution/p03_crypto_fuzzing.rs"]
pub mod p03_crypto_fuzzing;
#[cfg(feature = "solution")]
#[path = "solution/p04_parser_fuzzing.rs"]
pub mod p04_parser_fuzzing;
#[cfg(feature = "solution")]
#[path = "solution/p05_differential_fuzzing.rs"]
pub mod p05_differential_fuzzing;
#[cfg(feature = "solution")]
#[path = "solution/p06_coverage_guided.rs"]
pub mod p06_coverage_guided;
#[cfg(feature = "solution")]
#[path = "solution/p07_security_test_harness.rs"]
pub mod p07_security_test_harness;
#[cfg(feature = "solution")]
#[path = "solution/p08_regression_tests.rs"]
pub mod p08_regression_tests;
#[cfg(feature = "solution")]
#[path = "solution/p09_mock_security.rs"]
pub mod p09_mock_security;
#[cfg(feature = "solution")]
#[path = "solution/p10_continuous_fuzzing.rs"]
pub mod p10_continuous_fuzzing;
