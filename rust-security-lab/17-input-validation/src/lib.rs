//! # Module 17: Input Validation
//!
//! Type-driven validation, injection prevention, output encoding, and defense
//! in depth. This module covers the most common class of security vulnerabilities:
//! trusting user input.
//!
//! ## Learning Path
//! 1. Start with `p01_type_driven_validation` -- encode constraints in types
//! 2. Study injection attacks: SQL (p02), command (p03), XSS (p04)
//! 3. Learn URL and path defenses: SSRF (p05), path traversal (p06)
//! 4. Explore subtle attacks: Unicode (p07), ReDoS (p08), integer overflow (p09)
//! 5. Master defense in depth with input sanitization patterns (p10)
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 17-input-validation              # Test your implementation
//! cargo test -p 17-input-validation --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_type_driven_validation;
#[cfg(not(feature = "solution"))]
pub mod p02_sql_injection;
#[cfg(not(feature = "solution"))]
pub mod p03_command_injection;
#[cfg(not(feature = "solution"))]
pub mod p04_xss_prevention;
#[cfg(not(feature = "solution"))]
pub mod p05_ssrf_defense;
#[cfg(not(feature = "solution"))]
pub mod p06_path_traversal;
#[cfg(not(feature = "solution"))]
pub mod p07_unicode_attacks;
#[cfg(not(feature = "solution"))]
pub mod p08_regex_dos;
#[cfg(not(feature = "solution"))]
pub mod p09_integer_overflow;
#[cfg(not(feature = "solution"))]
pub mod p10_input_sanitization;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_type_driven_validation.rs"]
pub mod p01_type_driven_validation;
#[cfg(feature = "solution")]
#[path = "solution/p02_sql_injection.rs"]
pub mod p02_sql_injection;
#[cfg(feature = "solution")]
#[path = "solution/p03_command_injection.rs"]
pub mod p03_command_injection;
#[cfg(feature = "solution")]
#[path = "solution/p04_xss_prevention.rs"]
pub mod p04_xss_prevention;
#[cfg(feature = "solution")]
#[path = "solution/p05_ssrf_defense.rs"]
pub mod p05_ssrf_defense;
#[cfg(feature = "solution")]
#[path = "solution/p06_path_traversal.rs"]
pub mod p06_path_traversal;
#[cfg(feature = "solution")]
#[path = "solution/p07_unicode_attacks.rs"]
pub mod p07_unicode_attacks;
#[cfg(feature = "solution")]
#[path = "solution/p08_regex_dos.rs"]
pub mod p08_regex_dos;
#[cfg(feature = "solution")]
#[path = "solution/p09_integer_overflow.rs"]
pub mod p09_integer_overflow;
#[cfg(feature = "solution")]
#[path = "solution/p10_input_sanitization.rs"]
pub mod p10_input_sanitization;
