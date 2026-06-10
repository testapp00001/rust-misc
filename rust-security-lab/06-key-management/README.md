# Module 06: Key Management

> "The hardest part of cryptography is not the math — it's key management. Every real-world crypto failure traces back to how keys were generated, stored, rotated, or destroyed."

## Overview

Key management is the discipline of handling cryptographic keys throughout their entire lifecycle. This module covers:

- **Key Generation**: Creating cryptographically secure random keys
- **Key Derivation**: HKDF (from master keys) and PBKDF2 (from passwords)
- **Key Hierarchy**: Designing master → domain → operational key trees
- **Key Rotation**: Re-encrypting data and retiring old keys
- **Key Escrow**: Splitting custody for disaster recovery
- **Key Versioning**: Managing multiple key versions in production
- **Hardware-Backed Keys**: TPM, HSM, and secure enclave concepts
- **Key Destruction**: Zeroizing memory, preventing forensic recovery
- **Key Lifecycle**: End-to-end lifecycle management

## Why Key Management Is the Hardest Part of Cryptography

Cryptographic algorithms (AES, RSA, ECDSA) are well-studied and mathematically sound. In practice, systems are almost never broken by cracking the algorithm — they're broken by:

1. **Hard-coded keys** in source code or config files
2. **Weak randomness** during key generation (predictable seeds)
3. **No key rotation** — one key used forever, increasing exposure window
4. **Poor storage** — keys in plaintext files, environment variables, or logs
5. **No key destruction** — deleted keys still in memory or disk sectors
6. **Key reuse** across contexts (same key for encryption and authentication)

## Key Types

| Type | Purpose | Example |
|------|---------|---------|
| **Symmetric key** | Encrypt/decrypt with same key | AES-256 key for data encryption |
| **Asymmetric key pair** | Public key encrypts, private key decrypts | RSA/ECDSA for signing |
| **Master key** | Root of trust, derives other keys | KMS master key |
| **Derived key** | Generated from master via KDF | Per-service encryption key |
| **Session key** | Short-lived, one communication session | TLS session key |
| **Data encryption key (DEK)** | Encrypts actual data | Key encrypting a database column |
| **Key encryption key (KEK)** | Encrypts other keys | Wraps DEKs for storage |

## Key Derivation Functions (KDF)

### HKDF (HMAC-based KDF)
- **Use case**: Deriving multiple keys from one high-entropy master key
- **Process**: Extract (condense entropy) → Expand (generate multiple keys)
- **Fast**: Designed for keys that are already strong
- **Standard**: RFC 5869

### PBKDF2 (Password-Based KDF)
- **Use case**: Deriving keys from low-entropy passwords
- **Process**: Repeatedly hash (HMAC) the password with a salt, thousands of times
- **Slow by design**: Makes brute-force attacks expensive
- **Standard**: NIST SP 800-132

### Argon2id
- **Use case**: Password hashing (winner of Password Hashing Competition)
- **Memory-hard**: Requires large amounts of RAM, defeating GPU attacks
- **Preferred** over PBKDF2 for password-based key derivation (see Module 07)

## Key Hierarchy Design Pattern

```
                    Master Key (MK)
                   /        |        \
            Domain Key  Domain Key  Domain Key
            (users)     (payments)  (logs)
           /     \         |          \
       DEK    DEK        DEK         DEK
      (u1)   (u2)      (pay1)      (log1)
```

**Principles:**
- Master key is used ONLY to derive/unwrap domain keys
- Domain keys derive operational keys for specific contexts
- Compromising one DEK doesn't affect others
- Rotate domain keys independently

## Key Rotation Strategies

1. **Periodic rotation**: Rotate every N days (e.g., 90 days)
2. **Event-based rotation**: Rotate on compromise suspicion, employee departure
3. **Versioned keys**: New data encrypted with new key, old data readable with old key
4. **Re-encryption**: Gradually re-encrypt old data with new key

## Common Mistakes

| Mistake | Risk | Fix |
|---------|------|-----|
| Hard-coded key in source | Key in every git clone forever | Use KMS, env vars, or secret managers |
| `rand::thread_rng()` for crypto | May not be CSPRNG on all platforms | Use `ring::SystemRandom` or `OsRng` |
| No key rotation | Breach exposes ALL historical data | Implement versioned key rotation |
| Key stored with data | Steal data = steal key | Separate key storage from data |
| Logging key material | Keys in log files | Zeroize after use, never log |
| Reusing key for sign + encrypt | Cryptographic weakness | One key, one purpose |

## Rust-Specific Tips

1. Use `ring::rand::SystemRandom` for key generation — it uses OS entropy
2. Use the `secrecy` crate's `Secret<T>` to prevent accidental key exposure (no Debug, no Display)
3. Use `zeroize` crate to securely erase key material from memory
4. Use `hkdf` crate for key derivation from master keys
5. Use `ring::pbkdf2` or `hmac` + `sha2` for password-based derivation

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_key_generation.rs` | Generating secure keys | `ring::SystemRandom`, CSPRNG |
| 02 | `p02_hkdf_derivation.rs` | HKDF key derivation | Extract-then-expand, multiple keys from one source |
| 03 | `p03_pbkdf2_derivation.rs` | PBKDF2 password-based KDF | Slow by design, salt, iterations |
| 04 | `p04_key_hierarchy.rs` | Key hierarchy design | Master → domain → operational keys |
| 05 | `p05_key_rotation.rs` | Key rotation | Versioned keys, re-encryption |
| 06 | `p06_key_escrow.rs` | Key escrow | Splitting key custody, M-of-N schemes |
| 07 | `p07_key_versioning.rs` | Key versioning | Multiple key versions, metadata |
| 08 | `p08_hardware_backed_keys.rs` | Hardware-backed keys | TPM, HSM, secure enclave concepts |
| 09 | `p09_key_destruction.rs` | Secure key destruction | Zeroize memory, prevent recovery |
| 10 | `p10_key_lifecycle.rs` | Complete lifecycle | Generation → distribution → storage → rotation → destruction |

## References

- [NIST SP 800-57: Key Management Recommendations](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)
- [RFC 5869: HKDF](https://tools.ietf.org/html/rfc5869)
- [NIST SP 800-132: PBKDF2](https://csrc.nist.gov/publications/detail/sp/800-132/final)
- [OWASP Key Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
- [Google Cloud KMS Best Practices](https://cloud.google.com/kms/docs/best-practices)
