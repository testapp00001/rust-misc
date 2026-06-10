//! # Module 25: Post-Quantum Cryptography
//!
//! Preparing for the quantum computing era: lattice-based crypto, hash-based signatures,
//! hybrid encryption, key migration, crypto agility, and PQC risk assessment.
//!
//! ## Learning Path
//! 1. Start with `p01_quantum_threat` — understand Shor's and Grover's algorithms
//! 2. Progress through lattice crypto, ML-KEM, and SPHINCS+
//! 3. Build practical skills: hybrid encryption, key migration, crypto agility
//! 4. Finish with TLS integration and risk assessment
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 25-post-quantum-crypto              # Test your implementation
//! cargo test -p 25-post-quantum-crypto --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_quantum_threat;
#[cfg(not(feature = "solution"))]
pub mod p02_lattice_based_crypto;
#[cfg(not(feature = "solution"))]
pub mod p03_kyber_ml_kem;
#[cfg(not(feature = "solution"))]
pub mod p04_sphincs_plus;
#[cfg(not(feature = "solution"))]
pub mod p05_hybrid_encryption;
#[cfg(not(feature = "solution"))]
pub mod p06_key_migration;
#[cfg(not(feature = "solution"))]
pub mod p07_crypto_agility;
#[cfg(not(feature = "solution"))]
pub mod p08_pqc_in_tls;
#[cfg(not(feature = "solution"))]
pub mod p09_hash_based_sigs;
#[cfg(not(feature = "solution"))]
pub mod p10_pqc_risk_assessment;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_quantum_threat.rs"]
pub mod p01_quantum_threat;
#[cfg(feature = "solution")]
#[path = "solution/p02_lattice_based_crypto.rs"]
pub mod p02_lattice_based_crypto;
#[cfg(feature = "solution")]
#[path = "solution/p03_kyber_ml_kem.rs"]
pub mod p03_kyber_ml_kem;
#[cfg(feature = "solution")]
#[path = "solution/p04_sphincs_plus.rs"]
pub mod p04_sphincs_plus;
#[cfg(feature = "solution")]
#[path = "solution/p05_hybrid_encryption.rs"]
pub mod p05_hybrid_encryption;
#[cfg(feature = "solution")]
#[path = "solution/p06_key_migration.rs"]
pub mod p06_key_migration;
#[cfg(feature = "solution")]
#[path = "solution/p07_crypto_agility.rs"]
pub mod p07_crypto_agility;
#[cfg(feature = "solution")]
#[path = "solution/p08_pqc_in_tls.rs"]
pub mod p08_pqc_in_tls;
#[cfg(feature = "solution")]
#[path = "solution/p09_hash_based_sigs.rs"]
pub mod p09_hash_based_sigs;
#[cfg(feature = "solution")]
#[path = "solution/p10_pqc_risk_assessment.rs"]
pub mod p10_pqc_risk_assessment;
