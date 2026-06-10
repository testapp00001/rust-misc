# Module 01: Cryptographic Primitives

> "Hashing is the foundation of all cryptographic security. Get it wrong, and everything built on top collapses."

## Overview

Cryptographic primitives are the atomic building blocks of all security systems. This module covers:
- **Hashing**: SHA-2, SHA-3, BLAKE3 — one-way functions that produce fixed-size digests
- **HMAC**: Hash-based Message Authentication Code — verifying integrity AND authenticity
- **Encoding**: Base64, hex — converting binary data to text (NOT encryption!)
- **Checksums**: Detecting accidental data corruption
- **Common attacks**: Length extension, collision, timing attacks

## Key Concepts

### Hashing vs Encryption
```
Hashing:  data → hash     (ONE-WAY, cannot reverse)
Encryption: data → ciphertext → data (TWO-WAY, can decrypt)
```

### When to use what
| Need | Use |
|------|-----|
| Verify file integrity | SHA-256 or BLAKE3 |
| Password storage | Argon2id (Module 07), NOT raw hash |
| Message authentication | HMAC-SHA256 |
| Data fingerprinting | BLAKE3 (fast) or SHA-256 (standard) |
| Encode binary for URLs | Base64url |
| Encode binary for display | Hex |

### Hash Properties
1. **Deterministic**: Same input → same output
2. **Pre-image resistant**: Can't reverse a hash to get input
3. **Collision resistant**: Can't find two inputs with same hash
4. **Avalanche effect**: Small input change → completely different hash

## Attack Patterns

### Length Extension Attack
SHA-256 and SHA-512 are vulnerable. Given `hash(m)` and `len(m)`, an attacker can compute `hash(m || padding || m')` without knowing `m`. **Defense**: Use HMAC instead of `hash(key || message)`.

### Timing Attack on Comparison
String comparison (`==`) short-circuits on first differing byte. An attacker can measure response time to guess the hash byte-by-byte. **Defense**: Use constant-time comparison.

### Encoding ≠ Encryption
Base64/hex provide ZERO confidentiality. They're just number formats. A common mistake is "encoding" secrets with Base64 and thinking they're protected.

## Rust-Specific Tips

1. Use `ring::digest` for hashing — it's a binding to BoringSSL, heavily audited
2. Use `blake3` crate for high-performance hashing
3. Use `constant_time_eq` or `ring::constant_time::verify_slices_are_equal` for comparisons
4. Use `base64` crate with Engine API (v0.22+)
5. Never store raw hashes of passwords — use Argon2id (Module 07)

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_hashing_basics.rs` | SHA-256 and BLAKE3 | Hashing data, verifying integrity |
| 02 | `p02_sha3_and_variants.rs` | SHA-3 family | SHA3-256, SHA3-512, SHAKE |
| 03 | `p03_hmac_authentication.rs` | HMAC-SHA256 | Message authentication codes |
| 04 | `p04_length_extension_attack.rs` | Length extension | Why HMAC > hash(key\|\|msg) |
| 05 | `p05_constant_time_compare.rs` | Timing attacks | Constant-time comparison |
| 06 | `p06_base64_encoding.rs` | Base64 | Encoding binary as text |
| 07 | `p07_hex_encoding.rs` | Hex encoding | Hex representation and parsing |
| 08 | `p08_checksums.rs` | Checksums | CRC32, Adler-32 for integrity |
| 09 | `p09_merkle_tree.rs` | Merkle trees | Hash trees for verification |
| 10 | `p10_hash_based_commitment.rs` | Commitment schemes | Commit-reveal protocol |

## References

- [NIST SHA-3 Standard](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf)
- [BLAKE3 Paper](https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf)
- [HMAC RFC 2104](https://tools.ietf.org/html/rfc2104)
- [Timing Attacks](https://cr.yp.to/antiforgery/timing-20050414.pdf)
