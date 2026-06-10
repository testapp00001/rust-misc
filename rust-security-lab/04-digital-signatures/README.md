# Module 04: Digital Signatures

> "A digital signature proves who sent a message and that it wasn't tampered with. Without them, there is no trust on the internet."

## Overview

Digital signatures provide three critical security guarantees:
- **Authenticity**: The message came from the claimed sender
- **Integrity**: The message was not altered in transit
- **Non-repudiation**: The sender cannot deny having sent the message

This module covers Ed25519, ECDSA, and RSA signature schemes, along with attacks, defenses, and performance optimizations.

## Key Concepts

### How Digital Signatures Work

```
Signing:   message + private_key → signature
Verifying: message + signature + public_key → valid/invalid
```

The private key is kept secret. The public key is shared freely. Anyone with the public key can verify a signature, but only the private key holder can create one.

### Signature Scheme Comparison

| Property | Ed25519 | ECDSA (P-256) | RSA (PKCS#1 v1.5) |
|----------|---------|---------------|---------------------|
| Key size | 32 bytes | 32 bytes | 256+ bytes |
| Signature size | 64 bytes | 64 bytes | 256+ bytes |
| Speed | Very fast | Fast | Slow |
| Deterministic | Yes (RFC 8032) | No (needs RNG) | Yes |
| Batch verify | Yes | No | No |
| Standard | RFC 8032 | FIPS 186-4 | PKCS#1 |
| Use case | Modern apps, TLS 1.3 | Web PKI, certificates | Legacy, certificates |

### When to Use Which

| Scenario | Recommended |
|----------|-------------|
| New application, general purpose | Ed25519 |
| TLS certificates / Web PKI | ECDSA P-256 or P-384 |
| Government / compliance requirements | ECDSA (FIPS) or RSA |
| High-throughput verification | Ed25519 (batch verify) |
| Constrained embedded device | Ed25519 (small keys) |

## Attack Patterns

### Signature Malleability (ECDSA)
ECDSA signatures `(r, s)` and `(r, n - s)` are both valid for the same message. An attacker can flip a valid signature to a different valid one, potentially causing confusion in systems that track signatures by value. **Defense**: Enforce low-s normalization (s < n/2).

### Weak Random Nonces (ECDSA)
ECDSA requires a random nonce `k` for each signature. If `k` is predictable, reused, or biased, the private key can be recovered from just two signatures. This is the attack that broke Sony's PS3 signing key. **Defense**: Use RFC 6979 deterministic nonces.

### Signature Not Verified
The most common mistake: accepting a signature without actually verifying it, or verifying against the wrong public key. Always verify before trusting.

## Rust-Specific Tips

1. **Ed25519**: Use `ed25519-dalek` — it's the gold standard Rust implementation
2. **ECDSA**: Use `p256` crate with `ecdsa` for type-safe signature operations
3. **RSA**: Use `ring` for RSA signatures (it handles padding correctly)
4. **Serialization**: Serialize public keys and signatures as bytes, not strings
5. **Zeroize**: Private keys should be zeroized on drop (most crates handle this)

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_ed25519_signing.rs` | Ed25519 | Key generation, sign, verify |
| 02 | `p02_ecdsa_signing.rs` | ECDSA P-256 | ECDSA key generation, sign, verify |
| 03 | `p03_signature_verification.rs` | Verification | Handling invalid signatures safely |
| 04 | `p04_batch_verification.rs` | Batch verify | Ed25519 batch verification |
| 05 | `p05_threshold_signatures.rs` | Threshold | t-of-n signing concepts |
| 06 | `p06_signature_malleability.rs` | Malleability | ECDSA s vs n-s attack |
| 07 | `p07_deterministic_signatures.rs` | RFC 6979 | Deterministic ECDSA nonces |
| 08 | `p08_message_recovery.rs` | RSA recovery | Message recovery vs appendix |
| 09 | `p09_multi_signature.rs` | Multi-sig | Combining multiple signatures |
| 10 | `p10_signed_commitments.rs` | Commitments | Signatures + commitment schemes |

## References

- [RFC 8032 - Ed25519](https://tools.ietf.org/html/rfc8032)
- [FIPS 186-4 - DSA/ECDSA](https://csrc.nist.gov/publications/detail/fips/186/4/final)
- [RFC 6979 - Deterministic ECDSA](https://tools.ietf.org/html/rfc6979)
- [PKCS#1 v2.2 - RSA](https://tools.ietf.org/html/rfc8017)
- [Sony PS3 ECDSA Fail](https://www.schneier.com/blog/archives/2011/01/sony_ps3_securi.html)
