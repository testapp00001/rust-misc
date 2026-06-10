//! # Module 07: Password Security
//!
//! Hashing, salting, peppering, validation, brute-force defense, and secure
//! password reset flows. This module covers the entire password lifecycle.
//!
//! ## Learning Path
//! 1. Start with `p01_argon2id_hashing` -- the recommended algorithm
//! 2. Understand alternatives: bcrypt (p02), scrypt (p03)
//! 3. Learn why salting (p04) and peppering (p05) matter
//! 4. Study attacks: timing attacks (p06), brute force (p08)
//! 5. Build defenses: validation (p07), reset flow (p09), migration (p10)
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 07-password-security              # Test your implementation
//! cargo test -p 07-password-security --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_argon2id_hashing;
#[cfg(not(feature = "solution"))]
pub mod p02_bcrypt_hashing;
#[cfg(not(feature = "solution"))]
pub mod p03_scrypt_hashing;
#[cfg(not(feature = "solution"))]
pub mod p04_salting;
#[cfg(not(feature = "solution"))]
pub mod p05_pepper;
#[cfg(not(feature = "solution"))]
pub mod p06_timing_attack;
#[cfg(not(feature = "solution"))]
pub mod p07_password_validation;
#[cfg(not(feature = "solution"))]
pub mod p08_brute_force_defense;
#[cfg(not(feature = "solution"))]
pub mod p09_password_reset;
#[cfg(not(feature = "solution"))]
pub mod p10_password_migration;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_argon2id_hashing.rs"]
pub mod p01_argon2id_hashing;
#[cfg(feature = "solution")]
#[path = "solution/p02_bcrypt_hashing.rs"]
pub mod p02_bcrypt_hashing;
#[cfg(feature = "solution")]
#[path = "solution/p03_scrypt_hashing.rs"]
pub mod p03_scrypt_hashing;
#[cfg(feature = "solution")]
#[path = "solution/p04_salting.rs"]
pub mod p04_salting;
#[cfg(feature = "solution")]
#[path = "solution/p05_pepper.rs"]
pub mod p05_pepper;
#[cfg(feature = "solution")]
#[path = "solution/p06_timing_attack.rs"]
pub mod p06_timing_attack;
#[cfg(feature = "solution")]
#[path = "solution/p07_password_validation.rs"]
pub mod p07_password_validation;
#[cfg(feature = "solution")]
#[path = "solution/p08_brute_force_defense.rs"]
pub mod p08_brute_force_defense;
#[cfg(feature = "solution")]
#[path = "solution/p09_password_reset.rs"]
pub mod p09_password_reset;
#[cfg(feature = "solution")]
#[path = "solution/p10_password_migration.rs"]
pub mod p10_password_migration;
