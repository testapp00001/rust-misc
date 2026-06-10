# Module 08: Secure Local Storage

Encrypting files, secure deletion, temp file safety, and protecting data at rest.

## Learning Objectives

By the end of this module, you will understand:

- How to encrypt and decrypt files using AES-256-GCM
- Secure file deletion techniques (overwriting before unlinking)
- Creating and managing secure temporary files
- Unix filesystem permissions for security (0600, 0700)
- Designing encrypted container formats
- Deriving encryption keys from passwords using Argon2id
- OS keychain integration concepts
- Full-disk vs file-level encryption tradeoffs
- Encrypting configuration files securely

## Lessons

| # | Lesson | Description |
|---|--------|-------------|
| 01 | File Encryption | Encrypt files with AES-256-GCM |
| 02 | File Decryption | Decrypt files, verify authentication tags |
| 03 | Secure Deletion | Overwrite with random data before unlinking |
| 04 | Temp File Security | Secure temp files with restrictive permissions |
| 05 | Filesystem Permissions | Unix file permissions (0600, 0700) |
| 06 | Encrypted Container | Simple container format (header + encrypted data) |
| 07 | Key Derivation from Password | Argon2id + salt for password-based key derivation |
| 08 | OS Keychain Integration | macOS Keychain, Windows Credential Manager, Linux Secret Service |
| 09 | Data at Rest | Full-disk vs file-level encryption concepts |
| 10 | Secure Config Files | Encrypting configuration files |

## Quick Start

```bash
# Test your implementation
cargo test -p secure_local_storage

# Test reference solution
cargo test -p secure_local_storage --features solution
```

## Prerequisites

- Module 01: Cryptographic Primitives (hashing, HMAC)
- Module 02: Symmetric Encryption (AES-GCM, ChaCha20)
- Module 07: Password Security (Argon2id)

## Key Concepts

### AES-256-GCM File Encryption

AES-GCM provides authenticated encryption: confidentiality + integrity + authenticity.
When encrypting files, we prepend the nonce (12 bytes) to the ciphertext so the
decryptor can extract it. The 16-byte authentication tag is appended automatically.

### Secure Deletion

Simply calling `unlink()` or `delete()` doesn't erase file contents — the data
remains on disk until overwritten. Secure deletion overwrites the file with random
data (ideally multiple passes) before unlinking.

### Temp File Security

Temporary files are often created in `/tmp` with world-readable permissions.
Secure temp files must:
- Use restrictive permissions (0600)
- Be created in a private directory
- Be automatically deleted when no longer needed

### Argon2id Key Derivation

Never use a raw password as an encryption key. Argon2id is a memory-hard KDF
that resists GPU/ASIC attacks. It takes a password + salt and produces a
cryptographically strong key.

## Dependencies

- `ring` — Cryptographic primitives
- `aes-gcm` — AES-256-GCM authenticated encryption
- `rand` — Cryptographically secure random number generation
- `zeroize` — Secure memory zeroing
- `secrecy` — Secret wrapper types
- `serde` / `serde_json` — Serialization for container formats
- `base64` / `hex` — Encoding utilities
- `argon2` — Password-based key derivation
