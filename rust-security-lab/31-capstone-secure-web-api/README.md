# Module 31: Capstone — Secure Web API

> "A single layer of security is a single point of failure. Defense in depth means every layer assumes every other layer has already been breached."

## Overview

This capstone project combines every security concept from the course into a single, production-grade secure web API. Each lesson builds one middleware layer. The final lesson composes them all into a defense-in-depth architecture that protects against real-world attack vectors.

**No single control is sufficient.** TLS protects transport but not the application. Authentication identifies the user but does not limit what they can do. Authorization restricts actions but does not validate input. Every layer fills gaps left by the others.

## Architecture: Middleware Pipeline

A secure API is a pipeline of middleware layers. Each request passes through every layer before reaching the handler. Each layer can reject the request early.

```
Client Request
    |
    v
+--------------------------+
| TLS (p01)                |  -- Encrypt transport, authenticate server
+--------------------------+
    |
    v
+--------------------------+
| Rate Limiting (p04)      |  -- Reject floods before processing
+--------------------------+
    |
    v
+--------------------------+
| Audit Logging (p07)      |  -- Log every request with context
+--------------------------+
    |
    v
+--------------------------+
| Request Signing (p06)    |  -- Verify integrity and authenticity
+--------------------------+
    |
    v
+--------------------------+
| Input Validation (p05)   |  -- Sanitize and reject malformed input
+--------------------------+
    |
    v
+--------------------------+
| Authentication (p02)     |  -- Identify who is making the request
+--------------------------+
    |
    v
+--------------------------+
| Authorization (p03)      |  -- Check if they are allowed
+--------------------------+
    |
    v
+--------------------------+
| Error Handling (p08)     |  -- Catch panics, return safe errors
+--------------------------+
    |
    v
+--------------------------+
| Handler                  |  -- Business logic
+--------------------------+
    |
    v
Client Response
```

## Defense in Depth

| Layer | Module | Protects Against |
|-------|--------|-----------------|
| TLS 1.3 | p01 | Eavesdropping, MITM, downgrade attacks |
| Authentication | p02 | Unauthorized access, token forgery |
| Authorization | p03 | Privilege escalation, IDOR |
| Rate Limiting | p04 | Brute force, DDoS, resource exhaustion |
| Input Validation | p05 | Injection, XSS, buffer overflows |
| Request Signing | p06 | Tampering, replay attacks |
| Audit Logging | p07 | Undetected breaches, compliance violations |
| Error Handling | p08 | Information disclosure, stack trace leaks |
| Secret Management | p09 | Credential exposure, key compromise |
| Full API | p10 | Everything above, composed together |

## Key Concepts

### TLS 1.3

TLS 1.3 is the current standard for encrypted transport. Key improvements over TLS 1.2:
- **1-RTT handshake** instead of 2-RTT (faster connections)
- **0-RTT resumption** for known clients (even faster)
- **Removed weak cipher suites** (no RSA key exchange, no CBC mode, no SHA-1)
- **Forward secrecy by default** (ephemeral Diffie-Hellman only)

### JWT Authentication

JSON Web Tokens provide stateless authentication. The server signs claims with a secret or private key. The client sends the token with each request. The server verifies the signature and checks expiry without any database lookup.

Critical validation steps: signature check, algorithm allowlist, expiry check, issuer/audience check.

### RBAC Authorization

Role-Based Access Control maps users to roles, and roles to permissions. Each endpoint declares required permissions. The authorization middleware checks whether the authenticated user's roles grant the required permission.

```
User -> [roles] -> Role -> [permissions] -> Permission
"alice" -> ["admin", "editor"] -> "admin" -> ["users:read", "users:write", "audit:read"]
```

### Rate Limiting

Three strategies:
- **Fixed Window**: Count requests per time window. Simple but has burst boundary issues.
- **Sliding Window**: Weighted average of current and previous window. Smoother.
- **Token Bucket**: Tokens refill at a constant rate. Each request consumes a token. Allows controlled bursts.

### Input Validation

Every input from the outside world is untrusted. Validate:
- **Request body**: Schema validation, field length limits, type checks
- **Query parameters**: Range checks, enumeration validation
- **Headers**: Content-Type enforcement, size limits
- **Path parameters**: Format validation, injection prevention

### HMAC Request Signing

Request signing ensures integrity and authenticity without TLS:
1. Client builds a canonical string from method, path, timestamp, body hash
2. Client computes HMAC-SHA256 of the canonical string with a shared secret
3. Client sends the signature in a header
4. Server recomputes and compares (constant-time)

This prevents tampering even if an attacker can observe the request.

### Audit Logging

Security-relevant events must be logged with enough context for incident response:
- Who (user ID, IP address, user agent)
- What (method, path, action)
- When (timestamp with timezone)
- Result (status code, error if any)
- Why (authorization decision)

Logs must be immutable, tamper-evident, and never contain secrets.

### Secure Error Handling

Internal errors must never leak to clients:
- Stack traces reveal implementation details
- Database errors reveal schema
- File paths reveal directory structure
- Version numbers reveal known vulnerabilities

Log the full error internally. Return a generic error with a correlation ID.

### Secret Management

Secrets (API keys, database passwords, TLS certificates) must:
- Never appear in source code or version control
- Be loaded from environment variables or secret managers
- Be rotated regularly without downtime
- Be zeroized from memory when no longer needed
- Use `secrecy::Secret<T>` to prevent accidental logging

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_tls_setup.rs` | TLS Configuration | Certificate loading, TLS 1.3, cipher suite selection |
| 02 | `p02_authentication_layer.rs` | JWT Authentication | Token validation, claims extraction, middleware pattern |
| 03 | `p03_authorization_layer.rs` | RBAC Authorization | Role-permission mapping, endpoint guards |
| 04 | `p04_rate_limiting.rs` | Rate Limiting | Token bucket, sliding window, per-user limits |
| 05 | `p05_input_validation.rs` | Input Validation | Request body, query params, header validation |
| 06 | `p06_request_signing.rs` | HMAC Request Signing | Canonical string, HMAC-SHA256, replay prevention |
| 07 | `p07_audit_logging.rs` | Audit Logging | Security event logging, structured context |
| 08 | `p08_error_handling.rs` | Secure Error Handling | Internal detail suppression, correlation IDs |
| 09 | `p09_secret_management.rs` | Secret Management | Env vars, zeroize, rotation, secrecy crate |
| 10 | `p10_full_api.rs` | Complete Secure API | All middleware composed into production-ready API |

## Quick Test

```bash
# Test your implementation
cargo test -p 31-capstone-secure-web-api

# Test reference solution
cargo test -p 31-capstone-secure-web-api --features solution

# Run a specific lesson
cargo test -p 31-capstone-secure-web-api p01_tls_setup
```

## References

- [OWASP API Security Top 10](https://owasp.org/www-project-api-security/)
- [JWT RFC 7519](https://tools.ietf.org/html/rfc7519)
- [TLS 1.3 RFC 8446](https://tools.ietf.org/html/rfc8446)
- [HMAC RFC 2104](https://tools.ietf.org/html/rfc2104)
- [NIST SP 800-63B: Digital Identity Guidelines](https://pages.nist.gov/800-63-3/sp800-63b.html)
- [OWASP Cheat Sheet Series](https://cheatsheetseries.owasp.org/)
