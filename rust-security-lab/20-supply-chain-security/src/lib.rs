//! # Module 20: Supply Chain Security
//!
//! Securing your dependencies, build pipeline, and release artifacts against supply chain attacks.
//!
//! ## Learning Path
//! 1. Start with `p01_cargo_audit` — learn to check for known vulnerabilities
//! 2. Move through dependency pinning, typosquatting detection, and SBOM generation
//! 3. Build up to build provenance (SLSA) and Sigstore release signing
//!
//! ## Quick Test
//! ```bash
//! cargo test -p supply_chain_security              # Test your implementation
//! cargo test -p supply_chain_security --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_cargo_audit;
#[cfg(not(feature = "solution"))]
pub mod p02_cargo_deny;
#[cfg(not(feature = "solution"))]
pub mod p03_dependency_pinning;
#[cfg(not(feature = "solution"))]
pub mod p04_typosquatting;
#[cfg(not(feature = "solution"))]
pub mod p05_sbom_generation;
#[cfg(not(feature = "solution"))]
pub mod p06_reproducible_builds;
#[cfg(not(feature = "solution"))]
pub mod p07_build_provenance;
#[cfg(not(feature = "solution"))]
pub mod p08_cargo_geiger;
#[cfg(not(feature = "solution"))]
pub mod p09_minimum_versions;
#[cfg(not(feature = "solution"))]
pub mod p10_sigstore;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_cargo_audit.rs"]
pub mod p01_cargo_audit;
#[cfg(feature = "solution")]
#[path = "solution/p02_cargo_deny.rs"]
pub mod p02_cargo_deny;
#[cfg(feature = "solution")]
#[path = "solution/p03_dependency_pinning.rs"]
pub mod p03_dependency_pinning;
#[cfg(feature = "solution")]
#[path = "solution/p04_typosquatting.rs"]
pub mod p04_typosquatting;
#[cfg(feature = "solution")]
#[path = "solution/p05_sbom_generation.rs"]
pub mod p05_sbom_generation;
#[cfg(feature = "solution")]
#[path = "solution/p06_reproducible_builds.rs"]
pub mod p06_reproducible_builds;
#[cfg(feature = "solution")]
#[path = "solution/p07_build_provenance.rs"]
pub mod p07_build_provenance;
#[cfg(feature = "solution")]
#[path = "solution/p08_cargo_geiger.rs"]
pub mod p08_cargo_geiger;
#[cfg(feature = "solution")]
#[path = "solution/p09_minimum_versions.rs"]
pub mod p09_minimum_versions;
#[cfg(feature = "solution")]
#[path = "solution/p10_sigstore.rs"]
pub mod p10_sigstore;
