//! # Module 04: Digital Signatures
//!
//! Ed25519, ECDSA, RSA signatures — authenticity, integrity, and non-repudiation.
//!
//! ## Learning Path
//! 1. Start with `p01_ed25519_signing` — the modern default
//! 2. Learn ECDSA in `p02_ecdsa_signing` — the web PKI standard
//! 3. Progress through verification, attacks, and advanced topics
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 04-digital-signatures              # Test your implementation
//! cargo test -p 04-digital-signatures --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_ed25519_signing;
#[cfg(not(feature = "solution"))]
pub mod p02_ecdsa_signing;
#[cfg(not(feature = "solution"))]
pub mod p03_signature_verification;
#[cfg(not(feature = "solution"))]
pub mod p04_batch_verification;
#[cfg(not(feature = "solution"))]
pub mod p05_threshold_signatures;
#[cfg(not(feature = "solution"))]
pub mod p06_signature_malleability;
#[cfg(not(feature = "solution"))]
pub mod p07_deterministic_signatures;
#[cfg(not(feature = "solution"))]
pub mod p08_message_recovery;
#[cfg(not(feature = "solution"))]
pub mod p09_multi_signature;
#[cfg(not(feature = "solution"))]
pub mod p10_signed_commitments;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_ed25519_signing.rs"]
pub mod p01_ed25519_signing;
#[cfg(feature = "solution")]
#[path = "solution/p02_ecdsa_signing.rs"]
pub mod p02_ecdsa_signing;
#[cfg(feature = "solution")]
#[path = "solution/p03_signature_verification.rs"]
pub mod p03_signature_verification;
#[cfg(feature = "solution")]
#[path = "solution/p04_batch_verification.rs"]
pub mod p04_batch_verification;
#[cfg(feature = "solution")]
#[path = "solution/p05_threshold_signatures.rs"]
pub mod p05_threshold_signatures;
#[cfg(feature = "solution")]
#[path = "solution/p06_signature_malleability.rs"]
pub mod p06_signature_malleability;
#[cfg(feature = "solution")]
#[path = "solution/p07_deterministic_signatures.rs"]
pub mod p07_deterministic_signatures;
#[cfg(feature = "solution")]
#[path = "solution/p08_message_recovery.rs"]
pub mod p08_message_recovery;
#[cfg(feature = "solution")]
#[path = "solution/p09_multi_signature.rs"]
pub mod p09_multi_signature;
#[cfg(feature = "solution")]
#[path = "solution/p10_signed_commitments.rs"]
pub mod p10_signed_commitments;
