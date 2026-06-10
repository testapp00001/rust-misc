//! # Module 01: Cryptographic Primitives
//!
//! Hashing, HMAC, encoding, and checksums — the foundation of all cryptographic security.
//!
//! ## Learning Path
//! 1. Start with `p01_hashing_basics` — understand SHA-256 and BLAKE3
//! 2. Progress through HMAC, attacks, and encoding
//! 3. Build up to Merkle trees and commitment schemes
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 01-crypto-primitives              # Test your implementation
//! cargo test -p 01-crypto-primitives --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_hashing_basics;
#[cfg(not(feature = "solution"))]
pub mod p02_sha3_and_variants;
#[cfg(not(feature = "solution"))]
pub mod p03_hmac_authentication;
#[cfg(not(feature = "solution"))]
pub mod p04_length_extension_attack;
#[cfg(not(feature = "solution"))]
pub mod p05_constant_time_compare;
#[cfg(not(feature = "solution"))]
pub mod p06_base64_encoding;
#[cfg(not(feature = "solution"))]
pub mod p07_hex_encoding;
#[cfg(not(feature = "solution"))]
pub mod p08_checksums;
#[cfg(not(feature = "solution"))]
pub mod p09_merkle_tree;
#[cfg(not(feature = "solution"))]
pub mod p10_hash_based_commitment;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_hashing_basics.rs"]
pub mod p01_hashing_basics;
#[cfg(feature = "solution")]
#[path = "solution/p02_sha3_and_variants.rs"]
pub mod p02_sha3_and_variants;
#[cfg(feature = "solution")]
#[path = "solution/p03_hmac_authentication.rs"]
pub mod p03_hmac_authentication;
#[cfg(feature = "solution")]
#[path = "solution/p04_length_extension_attack.rs"]
pub mod p04_length_extension_attack;
#[cfg(feature = "solution")]
#[path = "solution/p05_constant_time_compare.rs"]
pub mod p05_constant_time_compare;
#[cfg(feature = "solution")]
#[path = "solution/p06_base64_encoding.rs"]
pub mod p06_base64_encoding;
#[cfg(feature = "solution")]
#[path = "solution/p07_hex_encoding.rs"]
pub mod p07_hex_encoding;
#[cfg(feature = "solution")]
#[path = "solution/p08_checksums.rs"]
pub mod p08_checksums;
#[cfg(feature = "solution")]
#[path = "solution/p09_merkle_tree.rs"]
pub mod p09_merkle_tree;
#[cfg(feature = "solution")]
#[path = "solution/p10_hash_based_commitment.rs"]
pub mod p10_hash_based_commitment;
