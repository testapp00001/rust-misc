# Module 19: Error Security

> "Your error messages are a roadmap for attackers. Every stack trace, every SQL error,
> every 'user not found' vs 'wrong password' distinction tells them exactly how to break in."

## Overview

Error handling is one of the most overlooked attack surfaces in software security. A single
careless error message can reveal database schemas, file paths, internal IP addresses,
framework versions, or authentication logic. This module covers the full spectrum of
error-related vulnerabilities: information leakage, timing oracles, oracle attacks,
stack trace exposure, and secure error design patterns.

Errors serve two audiences with completely different needs:
- **Developers** need full context: stack traces, error codes, file paths, SQL queries
- **Users** need safe, generic messages: "Something went wrong" or "Invalid credentials"

The core challenge is building error systems that satisfy both without leaking secrets.

## The Golden Rules

1. **NEVER expose internal details to users.** Stack traces, file paths, SQL errors,
   and framework versions must stay server-side. They help attackers map your system.
2. **Use identical error messages for auth failures.** "User not found" vs "wrong password"
   is a classic oracle attack -- it lets attackers enumerate valid usernames.
3. **Make error handling timing-constant for security-critical paths.** If checking a
   valid user takes 50ms (DB lookup + bcrypt) but an invalid user takes 2ms (early return),
   attackers can distinguish them by measuring response time.
4. **Fail closed, not open.** When in doubt, deny access. A system that defaults to
   "allow" on error is an attacker's dream. Prefer `Err` over a permissive fallback.
5. **Log everything, expose nothing.** Detailed errors go to logs (internal). Generic
   messages go to users (external). Never the other way around.
6. **Don't reveal which validation failed first.** Returning "email is invalid" then
   "password too short" lets attackers enumerate valid emails by changing only the password.

## Common Mistakes

| Mistake | Why It's Bad | Fix |
|---------|--------------|-----|
| SQL error in response | Reveals DB schema, table names, queries | Catch DB errors, return generic "server error" |
| Stack trace in production | Reveals file paths, framework, versions | Disable `RUST_BACKTRACE` in production |
| "User not found" vs "wrong password" | Username enumeration oracle | Use "Invalid credentials" for both |
| Early return on invalid user | Timing oracle (fast vs slow path) | Always run full auth flow |
| Different errors for different validations | Field enumeration oracle | Return single "validation failed" message |
| `unwrap()` in production code | Panics with file:line info exposed | Use `Result` with proper error handling |
| Default-allow on error | Fail-open: auth bypass on crash | Default-deny: return `Err` or reject |
| Logging only user-facing errors | Missing attack signals in audit trail | Log all security events at WARN/INFO |
| Revealing which field failed | Attackers probe one field at a time | Aggregate all validation errors |
| Using `debug!()` for auth failures | Missed brute-force detection | Use `warn!()` for security events |

## Error-Based Oracle Attacks

An oracle attack occurs when different error responses reveal information about internal
state. The most famous example is authentication:

```
# VULNERABLE: Different messages for different failure modes
if user_exists(username):
    if password_correct(password):
        return Ok("Welcome!")
    else:
        return Err("Wrong password")     # <-- Attacker knows username is valid
else:
    return Err("User not found")         # <-- Attacker knows username is invalid

# SECURE: Same message regardless of failure mode
if !user_exists(username) || !password_correct(password):
    return Err("Invalid credentials")    # <-- No information leaked
```

Timing oracles are subtler. Even if the message is identical, the *time* to respond
can reveal information:

```
# VULNERABLE: Early return on invalid user
async fn authenticate(user: &str, pass: &str) -> Result<()> {
    let u = db.find_user(user).await?;    // 50ms if user exists, 2ms if not
    verify_password(pass, &u.hash)?;       // 100ms bcrypt
    Ok(())
}
# Attacker measures: 52ms = invalid user, 150ms = valid user

# SECURE: Constant-time regardless of user existence
async fn authenticate(user: &str, pass: &str) -> Result<()> {
    let u = db.find_user(user).await;
    let dummy_hash = "$argon2id$v=19$m=4096$t=3$p=1$dummy"; // placeholder
    let hash = u.map(|u| u.hash).unwrap_or(dummy_hash.to_string());
    verify_password(pass, &hash)?;  // Always runs bcrypt
    Ok(())
}
```

## Fail-Closed vs Fail-Open

| Behavior | Example | Security |
|----------|---------|----------|
| **Fail-open** | Auth middleware returns `Ok(())` on error | DANGEROUS -- grants access on crash |
| **Fail-closed** | Auth middleware returns `Err` on error | SAFE -- denies access on crash |
| **Fail-closed (explicit)** | `Err(AuthError::AccessDenied)` with audit log | BEST -- denies + alerts |

```rust
// DANGEROUS: Fail-open
fn check_permission(user: &User, resource: &str) -> bool {
    match db.query_acl(user, resource) {
        Ok(acl) => acl.has_access(),
        Err(_) => true,  // <-- GRANTS ACCESS ON DATABASE ERROR
    }
}

// SAFE: Fail-closed
fn check_permission(user: &User, resource: &str) -> Result<bool> {
    let acl = db.query_acl(user, resource)?;  // <-- Propagates error, denies access
    Ok(acl.has_access())
}
```

## Learning Path

```
p01  Information leakage        (stack traces, file paths, SQL errors in responses)
p02  Secure error types         (internal details vs user-facing messages)
p03  Panic safety               (don't leak secrets in panic messages, catch_unwind)
p04  Error oracle               ("user not found" vs "wrong password")
p05  Timing oracle              (constant-time auth regardless of user existence)
p06  Error enumeration          (generic errors for all auth failures)
p07  Stack trace exposure       (disable RUST_BACKTRACE in production)
p08  Error aggregation          (don't reveal which validation failed first)
p09  Secure defaults            (fail closed, deny by default)
p10  Error logging              (log details internally, return generic to user)
```

## Quick Test

```bash
# Test your implementation
cargo test -p 19-error-security

# Test reference solution
cargo test -p 19-error-security --features solution
```

## References

- [OWASP Error Handling Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Error_Handling_Cheat_Sheet.html)
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [OWASP Testing for Error Codes](https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/08-Testing_for_Error_Handling/)
- [CWE-209: Information Exposure Through Error Messages](https://cwe.mitre.org/data/definitions/209.html)
- [CWE-204: Observable Response Discrepancy](https://cwe.mitre.org/data/definitions/204.html)
- [CWE-208: Observable Timing Discrepancy](https://cwe.mitre.org/data/definitions/208.html)
