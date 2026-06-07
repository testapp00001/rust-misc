# Module 15: Security

Cryptography with ring, secrets management, input validation, TLS configuration, and supply chain security.

## Lessons

| # | File | Topic |
|---|------|-------|
| 01 | `p01_cryptography_basics.rs` | ring crate, AES-GCM, HMAC, key derivation, secure random |
| 02 | `p02_password_hashing.rs` | Password hashing, salt generation, work factors, constant-time comparison |
| 03 | `p03_secrets_management.rs` | Zeroizing values, secrecy patterns, environment variables, vault integration |
| 04 | `p04_input_validation.rs` | Sanitization, injection prevention, URL validation, file path safety |
| 05 | `p05_jwt_authentication.rs` | JWT creation, verification, claims, expiration, refresh tokens |
| 06 | `p06_tls_configuration.rs` | TLS setup, certificate management, mTLS, certificate pinning |
| 07 | `p07_authorization.rs` | RBAC, ABAC, permission checking, middleware-based authorization |
| 08 | `p08_supply_chain_security.rs` | cargo audit, dependency verification, SBOM, reproducible builds |
| 09 | `p09_secure_coding.rs` | Timing attacks, constant-time comparison, integer overflow, panic safety |
| 10 | `p10_threat_modeling.rs` | Attack surface analysis, STRIDE, security review checklist, incident response |

## Running Tests

```bash
cargo test -p security_mastery
```
