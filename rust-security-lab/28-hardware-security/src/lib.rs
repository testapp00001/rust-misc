//! # Module 28: Hardware Security
//!
//! TPM, HSM, secure enclaves, attestation, hardware tokens, secure boot,
//! trusted execution, hardware RNG, tamper detection, and platform integration.
//!
//! ## Learning Path
//! 1. Start with `p01_tpm_basics` — understand roots of trust
//! 2. Progress through HSM, enclaves, and attestation
//! 3. Build up to hardware integration and platform abstraction
//!
//! ## Quick Test
//! ```bash
//! cargo test -p hardware_security              # Test your implementation
//! cargo test -p hardware_security --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_tpm_basics;
#[cfg(not(feature = "solution"))]
pub mod p02_hsm_concepts;
#[cfg(not(feature = "solution"))]
pub mod p03_secure_enclave;
#[cfg(not(feature = "solution"))]
pub mod p04_attestation;
#[cfg(not(feature = "solution"))]
pub mod p05_hardware_tokens;
#[cfg(not(feature = "solution"))]
pub mod p06_secure_boot;
#[cfg(not(feature = "solution"))]
pub mod p07_trusted_execution;
#[cfg(not(feature = "solution"))]
pub mod p08_hardware_rng;
#[cfg(not(feature = "solution"))]
pub mod p09_tamper_detection;
#[cfg(not(feature = "solution"))]
pub mod p10_hardware_integration;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_tpm_basics.rs"]
pub mod p01_tpm_basics;
#[cfg(feature = "solution")]
#[path = "solution/p02_hsm_concepts.rs"]
pub mod p02_hsm_concepts;
#[cfg(feature = "solution")]
#[path = "solution/p03_secure_enclave.rs"]
pub mod p03_secure_enclave;
#[cfg(feature = "solution")]
#[path = "solution/p04_attestation.rs"]
pub mod p04_attestation;
#[cfg(feature = "solution")]
#[path = "solution/p05_hardware_tokens.rs"]
pub mod p05_hardware_tokens;
#[cfg(feature = "solution")]
#[path = "solution/p06_secure_boot.rs"]
pub mod p06_secure_boot;
#[cfg(feature = "solution")]
#[path = "solution/p07_trusted_execution.rs"]
pub mod p07_trusted_execution;
#[cfg(feature = "solution")]
#[path = "solution/p08_hardware_rng.rs"]
pub mod p08_hardware_rng;
#[cfg(feature = "solution")]
#[path = "solution/p09_tamper_detection.rs"]
pub mod p09_tamper_detection;
#[cfg(feature = "solution")]
#[path = "solution/p10_hardware_integration.rs"]
pub mod p10_hardware_integration;
