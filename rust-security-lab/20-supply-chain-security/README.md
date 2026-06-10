# Module 20: Supply Chain Security

Securing your dependencies, build pipeline, and release artifacts against supply chain attacks.

## Learning Objectives

- Audit dependencies for known vulnerabilities using `cargo-audit`
- Enforce license and dependency policies with `cargo-deny`
- Pin dependencies and understand `Cargo.lock` integrity
- Detect typosquatting attacks on crate names
- Generate and consume Software Bills of Materials (SBOM)
- Achieve and verify reproducible builds
- Understand build provenance and the SLSA framework
- Audit unsafe code usage with `cargo-geiger`
- Test minimum version resolution (MSRV-aware)
- Apply Sigstore concepts for release signing and verification

## Prerequisites

- Rust 1.70+ (edition 2021)
- Familiarity with Cargo and crate ecosystem
- Module 01 (Cryptographic Primitives) recommended

## Quick Start

```bash
# Test your implementation
cargo test -p supply_chain_security

# Test reference solutions
cargo test -p supply_chain_security --features solution

# Run a specific lesson
cargo test -p supply_chain_security p01_cargo_audit
```

## Lesson Map

| # | File | Topic | Key Concepts |
|---|------|-------|--------------|
| 01 | `p01_cargo_audit` | Vulnerability Checking | OSV database, advisory parsing, severity scoring |
| 02 | `p02_cargo_deny` | License & Dependency Compliance | License allowlists, duplicate detection, bans |
| 03 | `p03_dependency_pinning` | Dependency Pinning | Cargo.lock, semver ranges, lock file verification |
| 04 | `p04_typosquatting` | Typosquatting Detection | Edit distance, known-package matching, name analysis |
| 05 | `p05_sbom_generation` | SBOM Generation | CycloneDX/SPDX formats, dependency trees, hashing |
| 06 | `p06_reproducible_builds` | Reproducible Builds | Deterministic compilation, environment isolation |
| 07 | `p07_build_provenance` | Build Provenance & SLSA | SLSA levels, provenance attestations, verification |
| 08 | `p08_cargo_geiger` | Unsafe Code Audit | Unsafe detection, risk scoring, allow-lists |
| 09 | `p09_minimum_versions` | Minimum Version Resolution | MSRV testing, version floor strategies |
| 10 | `p10_sigstore` | Release Signing | Sigstore concepts, keyless signing, transparency logs |

## Attack Scenarios

Each lesson covers real-world attack vectors:

- **Dependency Confusion**: Malicious packages uploaded to public registries that shadow internal package names
- **Typosquatting**: Publishing crates with names similar to popular packages (e.g., `reqwests` vs `reqwest`)
- **Malicious Build Scripts**: `build.rs` files that exfiltrate data or inject backdoors
- **Compromised Maintainers**: Legitimate packages hijacked via stolen credentials
- **Lock File Manipulation**: Tampering with `Cargo.lock` to downgrade to vulnerable versions

## Tools Referenced

| Tool | Purpose |
|------|---------|
| `cargo-audit` | Check dependencies against RustSec advisory database |
| `cargo-deny` | Multi-purpose dependency linting (licenses, bans, sources) |
| `cargo-geiger` | Detect and measure `unsafe` code usage in dependencies |
| `cargo-auditable` | Embed dependency info into compiled binaries |
| `cyclonedx-bom` | Generate CycloneDX SBOM from Cargo projects |
| `cosign` | Sign and verify artifacts using Sigstore |

## Architecture

```
supply_chain_security/
  src/
    lib.rs              # Feature-flag gated module root
    p01_cargo_audit.rs  # Exercise: vulnerability checking
    p02_cargo_deny.rs   # Exercise: license compliance
    ...
    p10_sigstore.rs     # Exercise: release signing
    solution/
      p01_cargo_audit.rs  # Reference solution
      ...
```

All exercises use `todo!()` stubs. Implement the functions, run tests, then compare with solutions.
