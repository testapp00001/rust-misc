//! # Module 27: Advanced Cryptographic Protocols
//!
//! Secret sharing, multi-party computation, homomorphic encryption, oblivious transfer,
//! secure aggregation, mental poker, commitments, zero-knowledge, blind signatures,
//! and anonymous credentials.
//!
//! ## Learning Path
//! 1. Start with `p01_secret_sharing` -- the foundation of threshold cryptography
//! 2. Progress through MPC, homomorphic concepts, and oblivious transfer
//! 3. Build up to mental poker, advanced commitments, and blind signatures
//!
//! ## Quick Test
//! ```bash
//! cargo test -p advanced_crypto_protocols              # Test your implementation
//! cargo test -p advanced_crypto_protocols --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_secret_sharing;
#[cfg(not(feature = "solution"))]
pub mod p02_multi_party_compute;
#[cfg(not(feature = "solution"))]
pub mod p03_homomorphic_concepts;
#[cfg(not(feature = "solution"))]
pub mod p04_oblivious_transfer;
#[cfg(not(feature = "solution"))]
pub mod p05_secure_aggregation;
#[cfg(not(feature = "solution"))]
pub mod p06_mental_poker;
#[cfg(not(feature = "solution"))]
pub mod p07_commitment_schemes_advanced;
#[cfg(not(feature = "solution"))]
pub mod p08_zero_knowledge_advanced;
#[cfg(not(feature = "solution"))]
pub mod p09_blind_signatures;
#[cfg(not(feature = "solution"))]
pub mod p10_anonymous_credentials;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_secret_sharing.rs"]
pub mod p01_secret_sharing;
#[cfg(feature = "solution")]
#[path = "solution/p02_multi_party_compute.rs"]
pub mod p02_multi_party_compute;
#[cfg(feature = "solution")]
#[path = "solution/p03_homomorphic_concepts.rs"]
pub mod p03_homomorphic_concepts;
#[cfg(feature = "solution")]
#[path = "solution/p04_oblivious_transfer.rs"]
pub mod p04_oblivious_transfer;
#[cfg(feature = "solution")]
#[path = "solution/p05_secure_aggregation.rs"]
pub mod p05_secure_aggregation;
#[cfg(feature = "solution")]
#[path = "solution/p06_mental_poker.rs"]
pub mod p06_mental_poker;
#[cfg(feature = "solution")]
#[path = "solution/p07_commitment_schemes_advanced.rs"]
pub mod p07_commitment_schemes_advanced;
#[cfg(feature = "solution")]
#[path = "solution/p08_zero_knowledge_advanced.rs"]
pub mod p08_zero_knowledge_advanced;
#[cfg(feature = "solution")]
#[path = "solution/p09_blind_signatures.rs"]
pub mod p09_blind_signatures;
#[cfg(feature = "solution")]
#[path = "solution/p10_anonymous_credentials.rs"]
pub mod p10_anonymous_credentials;
