# Module 14: API Security

> "An API is only as secure as its weakest endpoint. Every request is an attack surface."

## Overview

API security covers the techniques and patterns for protecting web APIs from abuse, forgery, and exploitation. This module covers:

- **Rate limiting**: Token bucket, sliding window, and per-user limits to prevent abuse
- **Request signing**: HMAC-based request signing to ensure integrity and authenticity
- **Webhook verification**: HMAC webhook signatures to verify callback authenticity
- **CORS security**: Cross-Origin Resource Sharing configuration and origin validation
- **CSRF protection**: Cross-Site Request Forgery defense with SameSite cookies and CSRF tokens
- **API key management**: Secure generation, hashing, and storage of API keys
- **Input size limits**: Request size limits for DoS prevention
- **JSON security**: Depth limits, type confusion, and safe parsing
- **GraphQL security**: Query depth limiting, complexity analysis, introspection control
- **API versioning**: Secure versioning strategies and deprecation handling

## Key Concepts

### Rate Limiting

Rate limiting controls how many requests a client can make in a given time window. Two primary algorithms:

**Token Bucket**: A bucket holds tokens that refill at a fixed rate. Each request consumes one token. When empty, requests are rejected.

```text
Capacity: 10 tokens
Refill rate: 1 token/second

Request 1-10: Allowed (bucket has tokens)
Request 11: Rejected (bucket empty)
After 5 seconds: 5 tokens refilled
Request 12-16: Allowed
```

**Sliding Window**: Tracks request timestamps within a rolling time window.

```text
Window: 60 seconds, Max: 100 requests

[12:00:00 ... 12:01:00] → 95 requests → Allowed
[12:00:30 ... 12:01:30] → 5 new requests → 100 total → Allowed
[12:00:45 ... 12:01:45] → 1 new request → 101 total → Rejected
```

### HMAC Request Signing

HMAC (Hash-based Message Authentication Code) ensures a request has not been tampered with and came from a trusted source.

```text
Client:
  signature = HMAC-SHA256(secret_key, method + path + timestamp + body_hash)
  Send: request + timestamp + signature

Server:
  expected = HMAC-SHA256(secret_key, method + path + timestamp + body_hash)
  if signature == expected AND timestamp is recent:
      Accept request
  else:
      Reject (401 Unauthorized)
```

The timestamp prevents replay attacks — the server rejects requests older than a threshold (e.g., 5 minutes).

### CORS (Cross-Origin Resource Sharing)

CORS controls which origins can access your API from a browser. Misconfigured CORS is a common vulnerability:

```rust
// DANGEROUS — allows any origin with credentials
Access-Control-Allow-Origin: *
Access-Control-Allow-Credentials: true

// SAFE — explicit origin whitelist
Access-Control-Allow-Origin: https://trusted-app.com
Access-Control-Allow-Credentials: true
```

### CSRF Protection

Cross-Site Request Forgery tricks authenticated users into making unwanted requests. Defenses:

1. **SameSite cookies**: `SameSite=Strict` or `SameSite=Lax` prevents cross-site cookie transmission
2. **CSRF tokens**: A random token in each form that the server validates
3. **Double-submit cookie**: Token in both cookie and header; attacker can't set both

### API Key Security

API keys must be:
- **Random**: Cryptographically random, not guessable
- **Hashed**: Store hashed keys, never plaintext (like passwords)
- **Scoped**: Limited permissions per key
- **Rotatable**: Easy to rotate without downtime

```text
Generation:  key = random_bytes(32) → base64 → "sk_live_abc123..."
Storage:     stored_hash = SHA256(key) → never store the raw key
Lookup:      hash(request_key) → compare against stored hashes
```

### JSON Security

JSON parsers can be exploited through:
- **Deeply nested objects**: Stack overflow via recursive parsing
- **Large payloads**: Memory exhaustion
- **Type confusion**: Unexpected types causing logic errors

### GraphQL Security

GraphQL APIs have unique risks:
- **Query depth attacks**: `user { friends { friends { friends { ... } } } }` — unbounded recursion
- **Introspection leaks**: Schema introspection reveals all types and fields
- **Batching attacks**: Multiple queries in one request to bypass rate limits

## Attack Patterns

### Rate Limit Bypass

Attackers bypass rate limiting by:
- Rotating IP addresses via botnets
- Using multiple API keys
- Manipulating request headers (`X-Forwarded-For`)
- Sending requests just below the limit

### CORS Misconfiguration

Common mistakes:
- Reflecting the `Origin` header without validation
- Using `*` with `Access-Control-Allow-Credentials: true`
- Allowing `null` origin (local files can send this)

### CSRF Exploitation

```html
<!-- Attacker's page -->
<form action="https://bank.com/transfer" method="POST">
  <input name="to" value="attacker-account">
  <input name="amount" value="10000">
</form>
<script>document.forms[0].submit();</script>
```

If the bank doesn't validate CSRF tokens, the victim's browser sends their session cookie with the form.

### API Key Theft

API keys leak through:
- Client-side JavaScript code
- URL parameters (logged in server access logs, browser history)
- Source control (committed accidentally)
- Error messages and logs

## Rust-Specific Tips

1. Use `ring::hmac` for HMAC operations — it's constant-time and well-audited
2. Use `rand::rngs::OsRng` for cryptographic randomness in API key generation
3. Use `HashMap` for rate limiter state — in production, use Redis or similar
4. Always use constant-time comparison (`ring::constant_time::verify_slices_are_equal`) for signature verification
5. Use `serde_json::Value` with depth checking to prevent deeply nested JSON attacks
6. For GraphQL, limit query depth at the parser level before execution

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_rate_limiting.rs` | Rate limiting | Token bucket, sliding window, per-user limits |
| 02 | `p02_request_signing.rs` | Request signing | HMAC sign body + timestamp, replay prevention |
| 03 | `p03_hmac_webhooks.rs` | Webhook verification | HMAC signature validation for callbacks |
| 04 | `p04_cors_security.rs` | CORS security | Origin validation, header configuration |
| 05 | `p05_csrf_protection.rs` | CSRF protection | SameSite cookies, CSRF token generation/validation |
| 06 | `p06_api_key_management.rs` | API key management | Generation, hashing, comparison |
| 07 | `p07_input_size_limits.rs` | Input size limits | Request body size limits, DoS prevention |
| 08 | `p08_json_security.rs` | JSON security | Depth limits, key count limits, safe parsing |
| 09 | `p09_graphql_security.rs` | GraphQL security | Query depth limiting, complexity scoring |
| 10 | `p10_api_versioning.rs` | API versioning | Secure version negotiation, deprecation |

## Running Tests

```bash
# Test your implementation
cargo test -p 14-api-security

# Test the reference solution
cargo test -p 14-api-security --features solution

# Run a specific exercise's tests
cargo test -p 14-api-security p01
```

## References

- [OWASP API Security Top 10](https://owasp.org/www-project-api-security/)
- [OWASP Cross-Site Request Forgery Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
- [MDN: CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS)
- [HMAC RFC 2104](https://datatracker.ietf.org/doc/html/rfc2104)
- [GraphQL Security Best Practices](https://www.apollographql.com/blog/graphql/security/9-ways-to-secure-your-graphql-api/)
- [Rate Limiting Algorithms](https://cloud.google.com/architecture/rate-limiting-strategies-techniques)
