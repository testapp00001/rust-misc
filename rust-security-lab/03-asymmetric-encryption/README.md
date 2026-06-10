# Module 03: Asymmetric Encryption

> "Public-key cryptography solved the key distribution problem — but introduced a whole new class of implementation vulnerabilities."

## Overview

Asymmetric (public-key) encryption uses mathematically related key pairs: a **public key** that anyone can know, and a **private key** that must stay secret. This module covers:
- **RSA**: The classic public-key algorithm — encrypt, decrypt, sign, verify
- **Elliptic Curve Cryptography (ECC)**: Smaller keys, same security, faster operations
- **Key Exchange**: Diffie-Hellman, ECDH, X25519 — establishing shared secrets over insecure channels
- **Hybrid Encryption**: The real-world pattern — asymmetric for key exchange, symmetric for data
- **Key Encapsulation (KEM)**: Modern pattern for securely wrapping symmetric keys
- **Forward Secrecy**: Ephemeral keys so compromise of long-term keys doesn't expose past traffic

## Key Concepts

### Public Key vs Private Key
```
Private Key (secret)  →  generates  →  Public Key (shareable)
Public Key encrypts   →  only Private Key can decrypt
Private Key signs     →  anyone with Public Key can verify
```

### RSA vs ECC: When to Use Which
| Property | RSA | ECC (P-256 / X25519) |
|----------|-----|---------------------|
| Key size for 128-bit security | 3072 bits | 256 bits |
| Speed (key generation) | Slow | Fast |
| Speed (encryption) | Fast | Moderate |
| Speed (decryption) | Slow | Fast |
| Key exchange | RSA-KEM | ECDH / X25519 |
| Digital signatures | RSA-PSS | ECDSA / EdDSA |
| Legacy compatibility | Excellent | Growing |
| Modern recommendation | Avoid for new systems | Preferred |

**Rule of thumb**: Use ECC (X25519 for key exchange, Ed25519 for signatures) for new systems. RSA only when required for compatibility.

### Key Exchange Protocols
```
Diffie-Hellman (DH):    g^a mod p, g^b mod p  →  g^(ab) mod p
ECDH (P-256):           a·G, b·G              →  ab·G (on curve)
X25519:                 Curve25519 DH          →  32-byte shared secret
```

### Hybrid Encryption (The Real-World Pattern)
```
1. Alice generates ephemeral keypair
2. Alice sends public key to Bob
3. Bob generates ephemeral keypair
4. Both compute shared secret via ECDH/X25519
5. Derive symmetric key from shared secret (HKDF)
6. Encrypt data with AES-256-GCM using symmetric key
```
This is how TLS 1.3, Signal Protocol, and most modern systems work.

### Forward Secrecy
If an attacker records encrypted traffic today and compromises the private key years later:
- **Without forward secrecy**: All past traffic is decryptable
- **With forward secrecy**: Only the session whose ephemeral key was compromised is exposed

Achieved by generating **new ephemeral keypairs per session** and deleting them after use.

## Attack Patterns

### Chosen Ciphertext Attack (Bleichenbacher)
Against RSA without OAEP padding: an attacker sends modified ciphertexts and observes
whether decryption succeeds/fails. The error messages leak information about the plaintext.
**Defense**: Use OAEP padding (not textbook/PKCS#1 v1.5).

### Small Subgroup Attack
On elliptic curves, a point of small order can leak private key bits.
**Defense**: Validate that received points are on the curve, not the identity, and have the correct order.

### Weak Key Attack
RSA keys with small prime factors can be factored. Poor random number generators
produce predictable keys.
**Defense**: Use well-audited libraries with proper CSPRNG (ring, rsa crate).

### Key Confusion Attack
Using the same key for encryption and signing, or using a public key as a private key.
**Defense**: Use separate key pairs for each purpose; tag keys with their intended use.

## Rust-Specific Tips

1. Use `rsa` crate with OAEP padding — never use textbook RSA
2. Use `x25519-dalek` for key exchange — it's constant-time and widely audited
3. Use `p256` crate for ECDH when NIST compliance is needed
4. Always validate received public keys before using them
5. Use `zeroize` to clear private keys from memory when done
6. Use `secrecy` crate to prevent accidental logging of key material

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_rsa_basics.rs` | RSA fundamentals | Key generation, OAEP encrypt/decrypt |
| 02 | `p02_rsa_key_sizes.rs` | RSA key sizes | 2048 vs 4096, security vs performance |
| 03 | `p03_x25519_key_exchange.rs` | X25519 DH | Curve25519 key agreement |
| 04 | `p04_ecdh_key_exchange.rs` | ECDH P-256 | NIST curve key exchange |
| 05 | `p05_hybrid_encryption.rs` | Hybrid encryption | Asymmetric + symmetric combo |
| 06 | `p06_key_encapsulation.rs` | KEM pattern | Encapsulate shared secrets |
| 07 | `p07_chosen_ciphertext_attack.rs` | Bleichenbacher | Textbook RSA vulnerability |
| 08 | `p08_key_serialization.rs` | Key formats | PEM, DER, raw bytes |
| 09 | `p09_key_validation.rs` | Key validation | Curve points, subgroup, weak keys |
| 10 | `p10_forward_secrecy.rs` | Forward secrecy | Ephemeral keys per session |

## Quick Start

```bash
# Test your implementation
cargo test -p 03-asymmetric-encryption

# Test reference solution
cargo test -p 03-asymmetric-encryption --features solution

# Run a specific lesson
cargo test -p 03-asymmetric-encryption p01_rsa_basics
```

## References

- [RSA PKCS#1 v2.2 (OAEP)](https://datatracker.ietf.org/doc/html/rfc8017)
- [RFC 7748: Elliptic Curves for Security (X25519)](https://datatracker.ietf.org/doc/html/rfc7748)
- [NIST SP 800-56B: RSA Key Transport](https://csrc.nist.gov/publications/detail/sp/800-56b/rev-2/final)
- [Bleichenbacher's Attack](https://link.springer.com/chapter/10.1007/BFb0053459)
- [Signal Protocol (Double Ratchet)](https://signal.org/docs/specifications/doubleratchet/)
- [TLS 1.3 Key Exchange](https://datatracker.ietf.org/doc/html/rfc8446)
