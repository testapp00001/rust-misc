# Module 25: Post-Quantum Cryptography

> "Quantum computers will not break all cryptography — but they will break the cryptography most of the internet relies on. The time to prepare is now, not after a quantum computer exists."

## Overview

Post-quantum cryptography (PQC) prepares for the day large-scale quantum computers can break today's public-key algorithms. This module covers:

- **Quantum Threat Model**: Shor's algorithm breaks RSA/ECC; Grover's halves symmetric key security
- **Lattice-Based Crypto**: Learning With Errors (LWE) — the mathematical foundation of most PQC
- **ML-KEM (Kyber)**: NIST-selected key encapsulation mechanism for key exchange
- **SPHINCS+**: Hash-based digital signatures — conservative, well-understood security
- **Hybrid Encryption**: Combining classical + PQC for defense in depth
- **Key Migration**: Transitioning existing systems from classical to PQC
- **Crypto Agility**: Designing systems so algorithms can be swapped easily
- **PQC in TLS**: How post-quantum algorithms integrate with TLS 1.3
- **Hash-Based Signatures**: Lamport, Merkle, WOTS+ — the building blocks
- **Risk Assessment**: "Harvest now, decrypt later" — why PQC matters TODAY

## Key Concepts

### The Quantum Threat
```
Classical Computer          Quantum Computer
RSA-2048: infeasible   →    RSA-2048: broken (Shor's)
AES-128: secure        →    AES-128: 64-bit security (Grover's)
AES-256: secure        →    AES-256: 128-bit security (Grover's)
ECDH-P256: secure      →    ECDH-P256: broken (Shor's)
```

### NIST PQC Standards (2024)
| Algorithm | Type | NIST Standard | Replaces |
|-----------|------|---------------|----------|
| ML-KEM (Kyber) | KEM | FIPS 203 | RSA key exchange, ECDH |
| ML-DSA (Dilithium) | Signature | FIPS 204 | RSA signatures, ECDSA |
| SLH-DSA (SPHINCS+) | Signature | FIPS 205 | Hash-based backup |

### Security Levels
| NIST Level | Classical Equival. | Quantum Equival. |
|------------|-------------------|------------------|
| 1 | AES-128 | AES-64 (Grover) |
| 3 | AES-192 | AES-96 (Grover) |
| 5 | AES-256 | AES-128 (Grover) |

## Attack Patterns

### Harvest Now, Decrypt Later
Nation-states are collecting encrypted traffic TODAY. When a quantum computer arrives, they can retroactively decrypt it. Any data that must remain secret for 10+ years needs PQC protection NOW.

### Shor's Algorithm
Given a quantum computer with enough qubits, Shor's algorithm factors large integers and computes discrete logarithms in polynomial time. This breaks:
- RSA (factoring)
- DH key exchange (discrete log)
- ECDH / ECDSA (elliptic curve discrete log)

### Grover's Algorithm
Provides a quadratic speedup for brute-force search. Effect on symmetric crypto:
- AES-128 → 64-bit security (use AES-256 for 128-bit quantum security)
- SHA-256 → 128-bit collision resistance (still adequate)

### Quantum-Safe vs Quantum-Resistant
- **Quantum-safe**: Algorithms believed to resist quantum attacks (lattice, hash-based, code-based)
- **Quantum-resistant**: Same meaning, used interchangeably
- **Hybrid**: Combining classical + PQC — if one breaks, the other still protects

## Rust-Specific Tips

### Feature Flag Gating
This module uses feature flags to switch between exercise stubs and reference solutions:
```rust
#[cfg(not(feature = "solution"))]
pub mod p01_quantum_threat;    // Your implementation

#[cfg(feature = "solution")]
#[path = "solution/p01_quantum_threat.rs"]
pub mod p01_quantum_threat;    // Reference solution
```

### No Native PQC in Rust Crypto Ecosystem (Yet)
Unlike AES-GCM or Ed25519, there is no de facto standard Rust crate for ML-KEM/ML-DSA as of 2024. This module focuses on CONCEPTS using simulated/homemade implementations. In production, you would use a vetted C/Rust implementation (e.g., `liboqs-rust`, `pqcrypto`).

### Constant-Time Operations
PQC implementations must be constant-time to prevent side-channel attacks. Use `subtle` crate or careful coding patterns. Lattice-based crypto involves many modular operations that can leak timing information.

## Quick Test

```bash
cargo test -p 25-post-quantum-crypto              # Test your implementation
cargo test -p 25-post-quantum-crypto --features solution  # Test reference solution
```

## Learning Path

1. **p01_quantum_threat** — Understand WHY we need PQC
2. **p02_lattice_based_crypto** — Learn the math behind most PQC
3. **p03_kyber_ml_kem** — Study the NIST-selected KEM
4. **p04_sphincs_plus** — Hash-based signatures
5. **p05_hybrid_encryption** — Defense in depth
6. **p06_key_migration** — Practical transition planning
7. **p07_crypto_agility** — Design for algorithm swaps
8. **p08_pqc_in_tls** — Real-world protocol integration
9. **p09_hash_based_sigs** — Deep dive into Lamport/Merkle/WOTS+
10. **p10_pqc_risk_assessment** — Business risk and prioritization
