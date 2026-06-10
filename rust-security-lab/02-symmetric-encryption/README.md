# Module 02: Symmetric Encryption

> "Symmetric encryption is the workhorse of modern cryptography. One key does everything — fast, efficient, and devastating when mishandled."

## Overview

Symmetric encryption uses a **single shared key** for both encryption and decryption. It is orders of magnitude faster than asymmetric encryption and is used to protect the bulk of data in transit (TLS) and at rest (disk encryption, databases).

This module covers:
- **AES-256-GCM**: The gold standard for authenticated encryption (hardware-accelerated on modern CPUs)
- **ChaCha20-Poly1305**: The software-friendly alternative (used in TLS 1.3, WireGuard, mobile)
- **Nonce management**: The single most common source of catastrophic failure in AEAD systems
- **Key rotation**: Limiting exposure when keys are compromised
- **Attacks**: Padding oracle, nonce reuse, chosen ciphertext

## What is Symmetric Encryption?

```
Encryption:  plaintext + key → ciphertext
Decryption:  ciphertext + key → plaintext
```

The same key is used for both operations. This means:
- Key distribution is the hard problem (how do you securely share the key?)
- It's fast enough for bulk data (AES-256 does ~1 GB/s in hardware)
- AEAD modes (GCM, Poly1305) also provide integrity and authenticity

## AES-256-GCM vs ChaCha20-Poly1305

| Property | AES-256-GCM | ChaCha20-Poly1305 |
|----------|-------------|-------------------|
| Block size | 128 bits | 512-bit state |
| Key size | 256 bits | 256 bits |
| Nonce size | 96 bits (12 bytes) | 96 bits (12 bytes) |
| Tag size | 128 bits (16 bytes) | 128 bits (16 bytes) |
| Hardware accel | AES-NI (x86, ARM) | None needed (fast in software) |
| Speed (with HW) | ~1-4 GB/s | ~1-3 GB/s |
| Speed (no HW) | ~100-200 MB/s | ~1-3 GB/s |
| Used in | TLS 1.3, IPsec, SSH | TLS 1.3, WireGuard, QUIC |
| Nonce misuse | Catastrophic | Catastrophic |

### When to Use Which

| Scenario | Recommendation |
|----------|---------------|
| Server-side (x86/ARM with AES-NI) | AES-256-GCM |
| Mobile / embedded (no AES-NI) | ChaCha20-Poly1305 |
| Performance-critical software path | ChaCha20-Poly1305 |
| Government / compliance requirements | AES-256-GCM (FIPS 140-2) |
| New project, no constraints | Either — both are excellent |

## Critical: Nonce Management

A **nonce** (number used once) must NEVER be repeated with the same key. This is the single most important rule in AEAD encryption.

### What Happens on Nonce Reuse (AES-GCM)
```
Given:  C1 = AES-GCM(key, nonce, P1)
        C2 = AES-GCM(key, nonce, P2)

Attacker computes: C1 XOR C2 = P1 XOR P2
```
With known plaintext (headers, predictable content), the attacker recovers ALL plaintext encrypted under that nonce. For GCM, nonce reuse also leaks the authentication key, allowing forgery of arbitrary messages.

### Nonce Strategies
| Strategy | When to Use |
|----------|------------|
| Counter | Single-device encryption, database |
| Random (96-bit) | Multi-device, short-lived keys |
| Derived (key + context) | Deterministic, protocol-level |

**Rule of thumb**: If you encrypt fewer than 2^32 messages with a random nonce, collision probability is negligible. For counter-based nonces, ensure monotonicity.

## Common Attacks

### 1. Nonce Reuse Attack
Reuse the same (key, nonce) pair. Destroys confidentiality AND authenticity for GCM.

### 2. Padding Oracle Attack
Exploits error messages that distinguish "bad padding" from "bad MAC" in CBC mode. Allows plaintext recovery one byte at a time. **Defense**: Use AEAD modes (GCM, Poly1305) which authenticate before decrypting.

### 3. Chosen Ciphertext Attack (CCA)
Attacker submits modified ciphertexts and observes decryption behavior. AEAD modes resist this by rejecting any tampered ciphertext.

## Rust-Specific Tips

1. **Use `aes-gcm` or `chacha20poly1305` crates** — both are well-audited and constant-time
2. **Use `ring` for higher-level AEAD** if you don't need fine-grained control
3. **Use `zeroize` to clear keys from memory** when done
4. **Use `secrecy::Secret` to wrap keys** — prevents accidental logging
5. **Never serialize nonces alongside ciphertext if using counters** — use a separate atomic counter
6. **Prefer 96-bit random nonces** for simplicity unless you need counter-based deterministic ordering

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_aes_gcm_basics.rs` | AES-256-GCM | Encrypt/decrypt with AEAD |
| 02 | `p02_chacha20_poly1305.rs` | ChaCha20-Poly1305 | Software AEAD alternative |
| 03 | `p03_nonce_management.rs` | Nonce strategies | Counter vs random nonces |
| 04 | `p04_key_rotation.rs` | Key rotation | Limiting key exposure |
| 05 | `p05_authenticated_encryption.rs` | AEAD deep dive | Encrypt-then-MAC vs AEAD |
| 06 | `p06_stream_vs_block.rs` | Streaming encryption | Chunked encryption for large data |
| 07 | `p07_padding_oracle_attack.rs` | Padding oracle | CBC vulnerability demo |
| 08 | `p08_nonce_reuse_attack.rs` | Nonce reuse | GCM/ChaCha20 nonce reuse exploit |
| 09 | `p09_additional_data.rs` | Associated data | AAD for binding context |
| 10 | `p10_hybrid_encryption.rs` | Hybrid encryption | Asymmetric key + symmetric data |

## Quick Test

```bash
# Test exercise stubs (will show todo!() panics)
cargo test -p 02-symmetric-encryption

# Test reference solutions
cargo test -p 02-symmetric-encryption --features solution
```

## References

- [NIST SP 800-38D (GCM)](https://csrc.nist.gov/publications/detail/sp/800-38d/final)
- [RFC 8439 (ChaCha20-Poly1305)](https://tools.ietf.org/html/rfc8439)
- [RFC 5116 (AEAD Interface)](https://tools.ietf.org/html/rfc5116)
- [Nonce-Disrespecting Adversaries](https://eprint.iacr.org/2016/475.pdf)
- [Crooked Crocodile Attack on GCM](https://eprint.iacr.org/2023/555.pdf)
