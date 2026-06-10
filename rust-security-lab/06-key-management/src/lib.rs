//! # Module 06: Key Management
//!
//! Generating, deriving, rotating, and destroying cryptographic keys — the hardest
//! part of cryptography.
//!
//! ## Learning Path
//! 1. Start with `p01_key_generation` — understand CSPRNG-based key creation
//! 2. Learn HKDF and PBKDF2 for key derivation
//! 3. Design key hierarchies and rotation strategies
//! 4. Master secure destruction and lifecycle management
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 06-key-management              # Test your implementation
//! cargo test -p 06-key-management --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_key_generation;
#[cfg(not(feature = "solution"))]
pub mod p02_hkdf_derivation;
#[cfg(not(feature = "solution"))]
pub mod p03_pbkdf2_derivation;
#[cfg(not(feature = "solution"))]
pub mod p04_key_hierarchy;
#[cfg(not(feature = "solution"))]
pub mod p05_key_rotation;
#[cfg(not(feature = "solution"))]
pub mod p06_key_escrow;
#[cfg(not(feature = "solution"))]
pub mod p07_key_versioning;
#[cfg(not(feature = "solution"))]
pub mod p08_hardware_backed_keys;
#[cfg(not(feature = "solution"))]
pub mod p09_key_destruction;
#[cfg(not(feature = "solution"))]
pub mod p10_key_lifecycle;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_key_generation.rs"]
pub mod p01_key_generation;
#[cfg(feature = "solution")]
#[path = "solution/p02_hkdf_derivation.rs"]
pub mod p02_hkdf_derivation;
#[cfg(feature = "solution")]
#[path = "solution/p03_pbkdf2_derivation.rs"]
pub mod p03_pbkdf2_derivation;
#[cfg(feature = "solution")]
#[path = "solution/p04_key_hierarchy.rs"]
pub mod p04_key_hierarchy;
#[cfg(feature = "solution")]
#[path = "solution/p05_key_rotation.rs"]
pub mod p05_key_rotation;
#[cfg(feature = "solution")]
#[path = "solution/p06_key_escrow.rs"]
pub mod p06_key_escrow;
#[cfg(feature = "solution")]
#[path = "solution/p07_key_versioning.rs"]
pub mod p07_key_versioning;
#[cfg(feature = "solution")]
#[path = "solution/p08_hardware_backed_keys.rs"]
pub mod p08_hardware_backed_keys;
#[cfg(feature = "solution")]
#[path = "solution/p09_key_destruction.rs"]
pub mod p09_key_destruction;
#[cfg(feature = "solution")]
#[path = "solution/p10_key_lifecycle.rs"]
pub mod p10_key_lifecycle;
