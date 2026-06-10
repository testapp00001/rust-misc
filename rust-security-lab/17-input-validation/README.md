# Module 17: Input Validation

> "Never trust user input." -- The first rule of secure software engineering.

## Why Input Validation Matters

Input validation is the first line of defense against the most common class of
security vulnerabilities. The **OWASP Top 10** (2021) lists these directly caused
by insufficient input validation:

| Rank | Vulnerability                        | Input Validation Role              |
|------|--------------------------------------|------------------------------------|
| A01  | Broken Access Control                | Validate resource IDs, paths       |
| A03  | Injection (SQL, NoSQL, OS, LDAP)     | Parameterize queries, sanitize     |
| A07  | Cross-Site Scripting (XSS)           | Encode output, validate input      |
| A08  | Software and Data Integrity Failures | Validate types, ranges, formats    |
| A10  | Server-Side Request Forgery (SSRF)   | Validate URLs, block internal IPs  |

## Core Principle: Type-Driven Validation

Rust's type system is your greatest ally in input validation. Instead of
checking inputs at runtime and hoping you remembered every call site, encode
constraints **in the type itself**:

```rust
// BAD: Stringly-typed, easy to forget validation
fn process_email(email: &str) { /* hope someone validated this */ }

// GOOD: Type-driven, impossible to use without validation
struct Email(String);  // Can only be constructed through validation
impl Email {
    fn new(raw: &str) -> Result<Self, ValidationError> {
        if !raw.contains('@') { return Err(...); }
        Ok(Self(raw.to_string()))
    }
}
```

This module teaches you to build validation into your types so that invalid
states are **unrepresentable**.

## The Five-Step Security Pattern

Each lesson follows this pattern:

1. **Attack** -- Understand how the vulnerability works
2. **Defend** -- Learn the mitigation technique
3. **Audit** -- Review code for the vulnerability
4. **Exercise** -- Implement the defense yourself
5. **Test** -- Verify your implementation with comprehensive tests

## Lesson Map

| #   | Topic                    | Key Concept                                      |
|-----|--------------------------|--------------------------------------------------|
| 01  | Type-Driven Validation   | Newtypes, builder pattern, compile-time safety   |
| 02  | SQL Injection            | Parameterized queries, input sanitization        |
| 03  | Command Injection        | Never pass user input to shell, Command + args   |
| 04  | XSS Prevention           | HTML entity encoding, CSP, output encoding       |
| 05  | SSRF Defense             | URL validation, block internal IPs, allowlists   |
| 06  | Path Traversal           | Canonicalize paths, prefix checks, reject `../`  |
| 07  | Unicode Attacks          | Homoglyphs, RTL override, normalization          |
| 08  | ReDoS                    | Catastrophic backtracking, bounded regex         |
| 09  | Integer Overflow         | Size calculations, buffer lengths, checked math  |
| 10  | Input Sanitization       | Allowlist vs denylist, defense in depth          |

## Quick Start

```bash
# Run all exercises (will fail until you implement them)
cargo test -p 17-input-validation

# Run with reference solutions
cargo test -p 17-input-validation --features solution

# Run a specific lesson
cargo test -p 17-input-validation p01_type_driven_validation

# Run with verbose output
cargo test -p 17-input-validation -- --nocapture
```

## Defense in Depth

No single validation technique is sufficient. Real systems layer multiple
defenses:

```
User Input
    |
    v
[1. Type Validation]    -- Reject obviously invalid input early
    |
    v
[2. Business Rules]     -- Domain-specific validation
    |
    v
[3. Sanitization]       -- Clean input for storage
    |
    v
[4. Output Encoding]    -- Encode for the output context (HTML, SQL, shell)
    |
    v
[5. Security Headers]   -- CSP, X-Content-Type-Options, etc.
```

Each layer catches what the previous layer missed. An attacker who bypasses
the type system still faces output encoding. An attacker who bypasses encoding
still faces security headers.

## Dependencies

- `regex` -- Pattern matching for validation rules
- `ring` -- Cryptographic operations for secure comparisons
- `sha2` -- Hashing for integrity checks
- `url` -- URL parsing and validation

## Further Reading

- [OWASP Input Validation Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html)
- [OWASP SQL Injection Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html)
- [OWASP XSS Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)
- [CWE-20: Improper Input Validation](https://cwe.mitre.org/data/definitions/20.html)
- [The Rustonomicon: Encoding and Security](https://doc.rust-lang.org/nomicon/)
