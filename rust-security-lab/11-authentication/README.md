# Module 11: Authentication

> "Authentication answers one question: who are you? Get it wrong, and nothing else matters."

## Overview

Authentication is the process of verifying identity. This module covers the full modern
authentication landscape: tokens, sessions, multi-factor, and the protocols that wire them together.

- **JWT**: Self-contained tokens with signed claims — the lingua franca of API auth
- **Sessions**: Server-side state with opaque tokens — simple and revocable
- **OAuth2**: Delegated authorization — "Sign in with Google/GitHub"
- **MFA/TOTP**: Something you know + something you have — defense in depth
- **Passkeys/WebAuthn**: Phishing-resistant, passwordless authentication via FIDO2
- **Token lifecycle**: Creation, refresh, revocation, and secure logout

## Key Concepts

### JWT vs Sessions

| Property | JWT (Token-based) | Session (Cookie-based) |
|----------|-------------------|------------------------|
| State | Stateless — token carries all data | Stateful — server stores session |
| Storage | Client (localStorage, cookie) | Server (memory, Redis, DB) |
| Revocation | Hard — must use blocklist or short expiry | Easy — delete server-side entry |
| Scalability | Excellent — no shared state needed | Requires shared session store |
| Size | Larger (carries claims) | Small (just a session ID) |
| CSRF | Immune (sent via header) | Vulnerable (cookie auto-sent) |
| Best for | APIs, microservices, SPAs | Traditional web apps, high-security |

### OAuth2 Flows

```
Authorization Code Flow (most common, most secure):

1. Client  → Authorization Server:  "Please authorize me"
2. User    → Authorization Server:  Logs in, grants permission
3. Auth Server → Client:            Authorization code (short-lived)
4. Client  → Authorization Server:  Exchange code + secret for tokens
5. Auth Server → Client:            Access token + Refresh token

Why two tokens?
  Access token:  Short-lived (15min), sent with every API call
  Refresh token: Long-lived (days), used only to get new access tokens
```

### MFA Importance

Single-factor (password only) is insufficient:
- Passwords leak in breaches (billions compromised)
- Phishing captures passwords in real-time
- Credential stuffing reuses leaked passwords across sites

TOTP (RFC 6238) adds a second factor:
- Shared secret between server and authenticator app
- Code changes every 30 seconds
- Even if password leaks, attacker needs the TOTP secret

### Common JWT Attacks

| Attack | Description | Defense |
|--------|-------------|---------|
| `alg=none` | Change header to `{"alg":"none"}`, strip signature | Always validate `alg` against allowlist |
| Weak secret | Brute-force HMAC secret from token | Use 256-bit+ random secret |
| Key confusion | RS256→HS256, use public key as HMAC secret | Enforce expected algorithm |
| Claim injection | Modify `sub`, `role`, `exp` claims | Always verify signature first |
| Token reuse | Stolen token used by attacker | Short expiry + refresh rotation |

### Token Storage Security

| Storage | XSS Risk | CSRF Risk | Recommendation |
|---------|----------|-----------|----------------|
| `localStorage` | High — JS can read | Immune | Avoid for sensitive tokens |
| `sessionStorage` | High — JS can read | Immune | Avoid for sensitive tokens |
| `HttpOnly` cookie | Immune — JS cannot read | Mitigate with `SameSite` | Best for web apps |
| Memory (JS variable) | Low — cleared on refresh | Immune | Best for SPAs |

## Attack Patterns

### JWT alg=none Bypass

```json
// Original header
{"alg": "HS256", "typ": "JWT"}

// Attacker changes to
{"alg": "none", "typ": "JWT"}

// Server skips signature verification entirely!
```

**Defense**: Never trust the `alg` field from the token. Enforce expected algorithm server-side.

### Credential Stuffing

Attackers use leaked username/password pairs from breach A to attack service B.
- 15 billion credentials are circulating in underground markets
- Automated tools try thousands of combinations per minute

**Defense**: Rate limiting, breach detection (HaveIBeenPwned), mandatory MFA, anomaly detection.

## Rust-Specific Tips

1. Use `hmac` + `sha2` crates for HMAC-SHA256 JWT signing
2. Use `ring::hmac` for constant-time signature verification
3. Use `rand::thread_rng().gen::<[u8; 32]>()` for cryptographically secure random tokens
4. Use `secrecy::Secret<String>` to prevent accidental token logging
5. Use `zeroize::Zeroize` to clear sensitive data from memory
6. Always use constant-time comparison for token/signature verification

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_jwt_creation.rs` | JWT Creation | Claims, expiration, HMAC-SHA256 signing |
| 02 | `p02_jwt_verification.rs` | JWT Verification | Signature check, expiry, claim validation |
| 03 | `p03_jwt_attacks.rs` | JWT Attacks | alg=none bypass, weak secret, key confusion |
| 04 | `p04_session_tokens.rs` | Session Tokens | Cryptographic randomness, entropy |
| 05 | `p05_oauth2_flow.rs` | OAuth2 Flow | Authorization code exchange |
| 06 | `p06_totp_mfa.rs` | TOTP MFA | RFC 6238, time-based codes |
| 07 | `p07_passkey_concept.rs` | Passkeys/WebAuthn | FIDO2, public key credentials |
| 08 | `p08_credential_stuffing.rs` | Credential Stuffing | Rate limiting, breach detection |
| 09 | `p09_token_refresh.rs` | Token Refresh | Access + refresh token rotation |
| 10 | `p10_secure_logout.rs` | Secure Logout | Token revocation, session cleanup |

## Quick Test

```bash
# Test your implementation
cargo test -p 11-authentication

# Test reference solution
cargo test -p 11-authentication --features solution

# Run a specific lesson
cargo test -p 11-authentication p01_jwt_creation
```

## References

- [JWT RFC 7519](https://tools.ietf.org/html/rfc7519)
- [OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)
- [TOTP RFC 6238](https://tools.ietf.org/html/rfc6238)
- [WebAuthn W3C Spec](https://www.w3.org/TR/webauthn-2/)
- [JWT Best Practices RFC 8725](https://tools.ietf.org/html/rfc8725)
- [HaveIBeenPwned API](https://haveibeenpwned.com/API/v3)
