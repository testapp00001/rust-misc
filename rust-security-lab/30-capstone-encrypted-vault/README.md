# Module 30: Capstone - Encrypted Password Vault

A capstone project that integrates every security concept from the course into a
working encrypted password vault. This module teaches you how real-world password
managers like 1Password, Bitwarden, and KeePass work under the hood.

## Why Build a Password Manager?

Password managers are one of the most important security tools. Understanding how
they work -- and how they can fail -- is critical for any security engineer. This
module walks through the complete architecture: from key derivation through
encrypted storage, secure memory handling, breach detection, and backup integrity.

## Architecture Overview

```
User Passphrase
      |
      v
+-----------------+
| Argon2id KDF    |  <-- Lesson 01: Memory-hard key derivation
| (salt + params) |
+-----------------+
      |
      v
  Master Key (32 bytes, stored in zeroizing memory)
      |
      +---> AES-256-GCM Encryption (Lessons 02-03)
      |         |
      |         v
      |     Encrypted Vault File (JSON)
      |       - nonce || ciphertext || auth_tag
      |       - entries: [{name, username, password, url, notes}, ...]
      |
      +---> Key Rotation (Lesson 07): re-derive, re-encrypt
      |
      +---> Backup Key (Lesson 09): separate encryption for exports
      |
      +---> Breach Check (Lesson 08): k-anonymity via HIBP API
```

## Learning Path

Each lesson builds on the previous ones. Complete them in order:

| # | File | Topic | Key Concepts |
|---|------|-------|--------------|
| 01 | `p01_master_key_derivation` | Master Key Derivation | Argon2id, salt, HKDF, key stretching |
| 02 | `p02_vault_encryption` | Vault Encryption | AES-256-GCM, nonce, authenticated encryption |
| 03 | `p03_vault_decryption` | Vault Decryption | Auth tag verification, tamper detection |
| 04 | `p04_entry_management` | Entry Management | CRUD operations on encrypted entries |
| 05 | `p05_secure_clipboard` | Secure Clipboard | Auto-clear, timeout, platform integration |
| 06 | `p06_auto_lock` | Auto-Lock | Inactivity timeout, zeroize keys on lock |
| 07 | `p07_key_rotation` | Key Rotation | Password change, re-encryption |
| 08 | `p08_breach_detection` | Breach Detection | k-anonymity, SHA-1 prefix, HaveIBeenPwned |
| 09 | `p09_secure_backup` | Secure Backup | Encrypted export/import, separate backup key |
| 10 | `p10_vault_integrity` | Vault Integrity | HMAC verification, version control, tamper log |

## Running the Exercises

```bash
# Test your implementation
cargo test -p 30-capstone-encrypted-vault

# Test the reference solution
cargo test -p 30-capstone-encrypted-vault --features solution
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `argon2` | Password-based key derivation (Argon2id) |
| `aes-gcm` | Authenticated encryption (AES-256-GCM) |
| `sha2` | SHA-256 for HMAC and hashing |
| `hkdf` | Key derivation from master key |
| `rand` | Cryptographic random number generation |
| `zeroize` | Secure memory zeroing |
| `secrecy` | Secret-wrapping types |
| `serde` / `serde_json` | Vault serialization |
| `base64` | Binary-to-text encoding for vault files |
| `chrono` | Timestamps for audit log and lock timeouts |

## Security Design Principles

1. **Defense in Depth**: Multiple layers -- encryption, authentication, integrity
2. **Least Privilege**: Keys only live as long as needed, then zeroized
3. **Memory Safety**: Zeroize all secrets, use secrecy wrappers
4. **Authenticated Encryption**: AES-256-GCM provides confidentiality + integrity
5. **Key Stretching**: Argon2id makes brute-force infeasible
6. **Zero Knowledge**: The vault file reveals nothing without the master passphrase
