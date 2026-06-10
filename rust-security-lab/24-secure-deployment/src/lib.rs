//! # Module 24: Secure Deployment
//!
//! Build it right, ship it safely, respond fast when things go wrong.
//!
//! ## Learning Path
//! 1. Start with `p01_reproducible_builds` — understand deterministic compilation
//! 2. Learn container security and SBOM generation
//! 3. Master release signing and build attestation
//! 4. Practice secret management, scanning, and hardening
//! 5. Finish with incident response procedures
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 24-secure-deployment              # Test your implementation
//! cargo test -p 24-secure-deployment --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_reproducible_builds;
#[cfg(not(feature = "solution"))]
pub mod p02_container_security;
#[cfg(not(feature = "solution"))]
pub mod p03_sbom_in_ci;
#[cfg(not(feature = "solution"))]
pub mod p04_release_signing;
#[cfg(not(feature = "solution"))]
pub mod p05_build_attestation;
#[cfg(not(feature = "solution"))]
pub mod p06_secret_injection_ci;
#[cfg(not(feature = "solution"))]
pub mod p07_image_scanning;
#[cfg(not(feature = "solution"))]
pub mod p08_supply_chain_verify;
#[cfg(not(feature = "solution"))]
pub mod p09_deployment_hardening;
#[cfg(not(feature = "solution"))]
pub mod p10_incident_response;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_reproducible_builds.rs"]
pub mod p01_reproducible_builds;
#[cfg(feature = "solution")]
#[path = "solution/p02_container_security.rs"]
pub mod p02_container_security;
#[cfg(feature = "solution")]
#[path = "solution/p03_sbom_in_ci.rs"]
pub mod p03_sbom_in_ci;
#[cfg(feature = "solution")]
#[path = "solution/p04_release_signing.rs"]
pub mod p04_release_signing;
#[cfg(feature = "solution")]
#[path = "solution/p05_build_attestation.rs"]
pub mod p05_build_attestation;
#[cfg(feature = "solution")]
#[path = "solution/p06_secret_injection_ci.rs"]
pub mod p06_secret_injection_ci;
#[cfg(feature = "solution")]
#[path = "solution/p07_image_scanning.rs"]
pub mod p07_image_scanning;
#[cfg(feature = "solution")]
#[path = "solution/p08_supply_chain_verify.rs"]
pub mod p08_supply_chain_verify;
#[cfg(feature = "solution")]
#[path = "solution/p09_deployment_hardening.rs"]
pub mod p09_deployment_hardening;
#[cfg(feature = "solution")]
#[path = "solution/p10_incident_response.rs"]
pub mod p10_incident_response;
