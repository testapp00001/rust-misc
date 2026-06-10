# 🛡️ Rust Security Lab — The Final Path of Rust Security Coding

> **"After completing this project, no security codebase will intimidate you."**

This is the definitive, exhaustive Rust security programming curriculum. It covers everything from cryptographic primitives to post-quantum cryptography, from memory safety to zero-knowledge proofs. Every lesson includes **attack demonstrations**, **correct defenses**, and **audit functions** — because understanding attacks is the only way to write truly secure code.

## Prerequisites

This project assumes you have completed:
- **rust-DSA** — Data structures and algorithms in Rust
- **rust-dev-mastery** — Professional Rust development patterns
- **rust-interview-guide** — Rust knowledge by career level

You should be comfortable with: ownership, lifetimes, traits, async/await, error handling, macros, testing, and basic concurrency.

## Teaching Methodology

Each lesson follows the **Attack → Defend → Audit** cycle:

1. **🔴 Attack**: Demonstrates the vulnerability with working exploit code
2. **🟢 Defend**: Shows the correct, secure implementation
3. **🔵 Audit**: Provides a function that detects the vulnerability
4. **📝 Exercise**: A `todo!()` stub for you to implement
5. **✅ Tests**: Validates correctness and attack resistance

Use the `solution` feature flag to toggle between your implementation and the reference:
```bash
# Test your implementation
cargo test -p 01-crypto-primitives

# Test the reference solution
cargo test -p 01-crypto-primitives --features solution
```

## Curriculum Overview

### Section 1: Cryptographic Foundations (Modules 01-05)
The bedrock of all security. Master hashing, encryption, signatures, and zero-knowledge proofs.

| # | Module | Description | Key Topics |
|---|--------|-------------|------------|
| 01 | [crypto-primitives](01-crypto-primitives/) | Hashing, HMAC, encoding | SHA-2, SHA-3, BLAKE3, HMAC, Base64, hex, checksums, length extension attacks |
| 02 | [symmetric-encryption](02-symmetric-encryption/) | Symmetric ciphers and authenticated encryption | AES-256-GCM, ChaCha20-Poly1305, nonces, IVs, key rotation, padding oracle attacks |
| 03 | [asymmetric-encryption](03-asymmetric-encryption/) | Public-key cryptography | RSA, X25519, ECDH, hybrid encryption, key encapsulation |
| 04 | [digital-signatures](04-digital-signatures/) | Signature schemes and verification | Ed25519, ECDSA, batch verification, threshold signatures |
| 05 | [zero-knowledge-proofs](05-zero-knowledge-proofs/) | Proving without revealing | Schnorr proofs, commitment schemes, Sigma protocols, Bulletproofs |

### Section 2: Data Protection (Modules 06-10)
Protect data at rest, in transit, and in databases.

| # | Module | Description | Key Topics |
|---|--------|-------------|------------|
| 06 | [key-management](06-key-management/) | Key lifecycle management | Key generation, HKDF, PBKDF2, key rotation, key hierarchy |
| 07 | [password-security](07-password-security/) | Secure credential storage | Argon2id, bcrypt, scrypt, salting, timing attacks |
| 08 | [secure-local-storage](08-secure-local-storage/) | File and disk encryption | File encryption, secure deletion, temp files, OS keychain |
| 09 | [database-security](09-database-security/) | Database protection | Encryption at rest, field-level encryption, SQL injection, audit logging |
| 10 | [data-in-transit](10-data-in-transit/) | Network data protection | TLS 1.3, certificate pinning, mTLS, DNS security, replay prevention |

### Section 3: Identity & Secrets (Modules 11-14)
Authentication, authorization, and secret management.

| # | Module | Description | Key Topics |
|---|--------|-------------|------------|
| 11 | [authentication](11-authentication/) | Identity verification | JWT/JWE, OAuth2, MFA/TOTP, passkeys/WebAuthn |
| 12 | [authorization](12-authorization/) | Access control | RBAC, ABAC, capability tokens, least privilege |
| 13 | [secret-management](13-secret-management/) | Secret lifecycle | Vault integration, env vars, secret rotation, sealed secrets |
| 14 | [api-security](14-api-security/) | API protection | Rate limiting, HMAC webhooks, CORS, CSRF, request signing |

### Section 4: Application Hardening (Modules 15-20)
Harden your application against every class of attack.

| # | Module | Description | Key Topics |
|---|--------|-------------|------------|
| 15 | [memory-security](15-memory-security/) | Secure memory handling | Zeroize, mlock, constant-time ops, guard pages, memory forensics |
| 16 | [serialization-security](16-serialization-security/) | Safe data parsing | Deserialization attacks, serde security, schema validation |
| 17 | [input-validation](17-input-validation/) | Input sanitization | Injection prevention, XSS, SSRF, path traversal, Unicode attacks |
| 18 | [logging-and-audit](18-logging-and-audit/) | Security logging | PII redaction, tamper-evident logs, audit trails |
| 19 | [error-security](19-error-security/) | Secure error handling | Information leakage, panic safety, error-based oracles |
| 20 | [supply-chain-security](20-supply-chain-security/) | Dependency security | cargo-audit, cargo-deny, SBOM, reproducible builds, Sigstore |

### Section 5: Security Engineering (Modules 21-24)
Testing, analysis, and deployment security.

| # | Module | Description | Key Topics |
|---|--------|-------------|------------|
| 21 | [fuzzing-and-testing](21-fuzzing-and-testing/) | Security testing | cargo-fuzz, proptest, property-based testing, differential fuzzing |
| 22 | [static-analysis](22-static-analysis/) | Code analysis | Clippy lints, cargo-geiger, Miri, Kani formal verification |
| 23 | [threat-modeling](23-threat-modeling/) | Risk assessment | STRIDE, attack trees, DREAD scoring, security design review |
| 24 | [secure-deployment](24-secure-deployment/) | Secure release | Reproducible builds, container security, SBOM, SLSA framework |

### Section 6: Advanced & Quantum (Modules 25-28)
Cutting-edge security topics.

| # | Module | Description | Key Topics |
|---|--------|-------------|------------|
| 25 | [post-quantum-crypto](25-post-quantum-crypto/) | Quantum-resistant crypto | Kyber/ML-KEM, SPHINCS+, hybrid PQC, migration strategies |
| 26 | [privacy-engineering](26-privacy-engineering/) | Privacy by design | Differential privacy, k-anonymity, GDPR implementation |
| 27 | [advanced-crypto-protocols](27-advanced-crypto-protocols/) | Advanced protocols | MPC, Shamir's secret sharing, homomorphic encryption |
| 28 | [hardware-security](28-hardware-security/) | Hardware trust | TPM, secure enclaves, HSM, attestation |

### Section 7: Capstone Projects (Modules 29-31)
Build complete, production-grade secure systems.

| # | Module | Description | Key Topics |
|---|--------|-------------|------------|
| 29 | [capstone-secure-messenger](29-capstone-secure-messenger/) | E2E encrypted messaging | Key exchange, forward secrecy, group chat, key verification |
| 30 | [capstone-encrypted-vault](30-capstone-encrypted-vault/) | Encrypted password vault | Master key derivation, secure storage, auto-lock, breach detection |
| 31 | [capstone-secure-web-api](31-capstone-secure-web-api/) | Secure web API | TLS, auth, rate limiting, input validation, audit logging |

## Quick Start

```bash
# List all modules
make list

# Test your implementation of a specific module
make test TOPIC=01-crypto-primitives

# Test the reference solution
make solution TOPIC=01-crypto-primitives

# Run all tests
make test-all
```

## Project Structure

```
rust-security-lab/
├── Cargo.toml              # Workspace definition
├── Makefile                # Build automation
├── README.md               # This file
├── 01-crypto-primitives/   # Module 1
│   ├── README.md           # Concept overview + attack patterns
│   ├── Cargo.toml          # Module dependencies
│   └── src/
│       ├── lib.rs          # Module declarations (feature-flag gated)
│       ├── p01_*.rs        # Exercise stubs (todo!())
│       └── solution/       # Reference implementations
│           └── p01_*.rs
├── 02-symmetric-encryption/
│   └── ...
├── ...
└── 31-capstone-secure-web-api/
    └── ...
```

## Learning Path

**Recommended order**: Follow the modules sequentially (01 → 31). Each section builds on the previous one.

**For experienced security practitioners**: You may skip to Section 4+ if you already have strong crypto fundamentals.

**Time estimate**: 150-300 hours for the complete curriculum, depending on your pace.

## Security Disclaimer

⚠️ **This project is for educational purposes.** The attack demonstrations are simplified for learning. Real-world attacks are more complex. Always use battle-tested, audited libraries for production code. Never roll your own cryptography in production.

## Contributing

This is a learning project. If you find errors or have suggestions, please open an issue.

## License

MIT
