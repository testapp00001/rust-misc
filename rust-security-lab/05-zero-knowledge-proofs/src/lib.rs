//! # Module 05: Zero-Knowledge Proofs
//!
//! Prove you know something without revealing it — the foundation of privacy-preserving
//! cryptography. This module builds ZK protocols from basic primitives (hashing, modular
//! arithmetic) to teach the core concepts.
//!
//! ## Learning Path
//! 1. Start with `p01_zk_fundamentals` — understand the three properties
//! 2. Learn Schnorr proofs and commitment schemes
//! 3. Build up to range proofs, set membership, and circuit-based ZK
//! 4. Finish with Bulletproofs overview
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 05-zero-knowledge-proofs              # Test your implementation
//! cargo test -p 05-zero-knowledge-proofs --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_zk_fundamentals;
#[cfg(not(feature = "solution"))]
pub mod p02_schnorr_proof;
#[cfg(not(feature = "solution"))]
pub mod p03_commitment_schemes;
#[cfg(not(feature = "solution"))]
pub mod p04_sigma_protocols;
#[cfg(not(feature = "solution"))]
pub mod p05_range_proofs;
#[cfg(not(feature = "solution"))]
pub mod p06_proof_of_knowledge;
#[cfg(not(feature = "solution"))]
pub mod p07_non_interactive_proofs;
#[cfg(not(feature = "solution"))]
pub mod p08_zk_set_membership;
#[cfg(not(feature = "solution"))]
pub mod p09_zk_boolean_circuits;
#[cfg(not(feature = "solution"))]
pub mod p10_bulletproofs_intro;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_zk_fundamentals.rs"]
pub mod p01_zk_fundamentals;
#[cfg(feature = "solution")]
#[path = "solution/p02_schnorr_proof.rs"]
pub mod p02_schnorr_proof;
#[cfg(feature = "solution")]
#[path = "solution/p03_commitment_schemes.rs"]
pub mod p03_commitment_schemes;
#[cfg(feature = "solution")]
#[path = "solution/p04_sigma_protocols.rs"]
pub mod p04_sigma_protocols;
#[cfg(feature = "solution")]
#[path = "solution/p05_range_proofs.rs"]
pub mod p05_range_proofs;
#[cfg(feature = "solution")]
#[path = "solution/p06_proof_of_knowledge.rs"]
pub mod p06_proof_of_knowledge;
#[cfg(feature = "solution")]
#[path = "solution/p07_non_interactive_proofs.rs"]
pub mod p07_non_interactive_proofs;
#[cfg(feature = "solution")]
#[path = "solution/p08_zk_set_membership.rs"]
pub mod p08_zk_set_membership;
#[cfg(feature = "solution")]
#[path = "solution/p09_zk_boolean_circuits.rs"]
pub mod p09_zk_boolean_circuits;
#[cfg(feature = "solution")]
#[path = "solution/p10_bulletproofs_intro.rs"]
pub mod p10_bulletproofs_intro;
