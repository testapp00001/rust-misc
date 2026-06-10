# Module 13: Secret Management

> "Secrets are the keys to the kingdom. Lose one, and an attacker owns everything it protects."

## Overview

Secret management is the discipline of storing, distributing, rotating, and auditing sensitive credentials. This module covers:

- **Environment variable security**: Reading secrets from env vars, pitfalls of `.env` files
- **Secret scanning**: Detecting hardcoded API keys, passwords, and tokens in source code
- **Vault integration**: HashiCorp Vault patterns — lease-based retrieval, auto-rotation
- **Secret rotation**: Rotating secrets without downtime using graceful rollover
- **Sealed secrets**: Encrypting secrets for safe storage in version control
- **Secret injection**: Runtime injection patterns — never bake secrets into images or binaries
- **Memory protection**: Zeroize on drop, guard pages, preventing secrets from leaking in core dumps
- **Secret sharing**: Shamir's Secret Sharing — splitting a secret into N shares requiring K to reconstruct
- **Audit trails**: Logging who accessed what secret and when
- **Incident response**: What to do when a secret leaks — rotate, assess blast radius, notify

## Key Concepts

### The Secret Lifecycle

```
Create → Store → Distribute → Use → Rotate → Revoke → Audit
```

Every secret must go through this lifecycle. Skipping any step creates risk.

### Never Hardcode Secrets

```rust
// DANGEROUS — secret in source code, visible in git history forever
const API_KEY: &str = "sk-1234567890abcdef";

// DANGEROUS — secret compiled into the binary
let password = "hunter2";

// SAFE — read from environment at runtime
let api_key = std::env::var("API_KEY").expect("API_KEY must be set");
```

### `.env` Files Are Dangerous

`.env` files are a common way to manage secrets in development, but they create real risks:

1. **Committed to git** — `.gitignore` must exclude them; one mistake leaks everything
2. **Shared via chat/email** — developers copy-paste them around
3. **Left on production servers** — `.env` files linger after deployment
4. **No rotation** — the same `.env` file may be used for months or years
5. **No audit trail** — who accessed the `.env` file? When?

**Better alternatives**: Environment variables injected by your orchestrator, a secrets manager (Vault, AWS Secrets Manager), or sealed secrets.

### Zeroize on Drop

Secrets in memory can leak through:
- Core dumps
- Memory swapping to disk
- Process memory inspection
- Use-after-free bugs

The `zeroize` crate overwrites memory before it's freed:

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
struct ApiKey {
    key: Vec<u8>,  // Automatically zeroed when dropped
}
```

### Secret Rotation

Secrets should be rotated regularly. The key challenge is rotating without downtime:

```
Phase 1: Generate new secret, keep old one active
Phase 2: Update all consumers to use new secret
Phase 3: Verify all consumers are using new secret
Phase 4: Revoke old secret
```

This "graceful rollover" ensures no service interruption during rotation.

### Vault Integration Patterns

HashiCorp Vault (and similar systems) provide:
- **Lease-based secrets**: Secrets have a TTL and auto-expire
- **Dynamic secrets**: Generated on-demand (e.g., database credentials per-service)
- **Audit logging**: Every access is logged
- **Encryption as a service**: Encrypt/decrypt without exposing keys

```text
App → "I need a DB credential" → Vault → Generates unique credential (TTL: 1h)
                                        → Returns credential + lease
                                        → Auto-revokes after 1 hour
```

### Shamir's Secret Sharing

Split a secret into N shares where any K shares can reconstruct it, but K-1 shares reveal nothing:

```text
Secret: "master-key-12345"
Split into 5 shares, threshold 3

Share 1: a1b2c3...
Share 2: d4e5f6...
Share 3: g7h8i9...
Share 4: j0k1l2...
Share 5: m3n4o5...

Any 3 shares → reconstruct original secret
Any 2 shares → learn nothing about the secret
```

## Attack Patterns

### Hardcoded Secret Discovery

Attackers scan public repositories, Docker images, and compiled binaries for hardcoded secrets. Tools like `trufflehog`, `gitleaks`, and `detect-secrets` automate this.

### Environment Variable Exposure

Environment variables can leak through:
- `/proc/<pid>/environ` on Linux
- Debug endpoints (`/debug/vars` in Go)
- Error messages that include environment context
- Process listing tools
- Container inspection (`docker inspect`)

### Secret Sprawl

As systems grow, secrets proliferate: database passwords, API keys, TLS certificates, signing keys, encryption keys. Without centralized management, you end up with:
- Secrets in 10 different places
- No idea which secrets are still in use
- No rotation schedule
- No audit trail

### Memory Disclosure

Even well-managed secrets can leak from memory:
- **Core dumps**: A crashed process dumps all memory, including secrets
- **Swap**: Secrets written to disk if the system swaps
- **Cold boot attacks**: RAM retains data briefly after power loss
- **Spectre/Meltdown**: Speculative execution can leak memory contents

## Rust-Specific Tips

1. Use `secrecy::SecretString` for passwords — it prevents `Debug` output and implements `Zeroize` on drop
2. Use `zeroize::ZeroizeOnDrop` for custom secret types
3. Use `std::env::var()` for environment variables — never hardcode
4. Use `ring::aead` for encrypting secrets at rest
5. Avoid `println!("{:?}", secret)` — the `secrecy` crate blocks this
6. Use `secrecy::ExposeSecret` to explicitly access the inner value — forces conscious decision

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_env_var_secrets.rs` | Environment variables | Reading secrets from env, .env pitfalls |
| 02 | `p02_secret_scanning.rs` | Secret scanning | Detecting hardcoded keys, passwords, tokens |
| 03 | `p03_vault_integration.rs` | Vault integration | Lease-based secret retrieval, auto-rotation |
| 04 | `p04_secret_rotation.rs` | Secret rotation | Rotate without downtime, graceful rollover |
| 05 | `p05_sealed_secrets.rs` | Sealed secrets | Encrypt secrets for version control storage |
| 06 | `p06_secret_injection.rs` | Secret injection | Runtime injection, don't bake into binaries |
| 07 | `p07_memory_protection.rs` | Memory protection | Zeroize on drop, guard pages |
| 08 | `p08_secret_sharing.rs` | Secret sharing | Shamir's Secret Sharing scheme |
| 09 | `p09_audit_secret_access.rs` | Audit trails | Logging secret access events |
| 10 | `p10_secret_leak_response.rs` | Incident response | Rotate, assess blast radius, notify |

## Running Tests

```bash
# Test your implementation
cargo test -p 13-secret-management

# Test the reference solution
cargo test -p 13-secret-management --features solution

# Run a specific exercise's tests
cargo test -p 13-secret-management p01
```

## References

- [OWASP Secret Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
- [HashiCorp Vault Documentation](https://www.vaultproject.io/docs)
- [Shamir's Secret Sharing (Wikipedia)](https://en.wikipedia.org/wiki/Shamir%27s_secret_sharing)
- [zeroize crate](https://docs.rs/zeroize)
- [secrecy crate](https://docs.rs/secrecy)
- [NIST SP 800-57: Key Management](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)
