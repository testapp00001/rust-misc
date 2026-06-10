# Module 16: Serialization Security

> "Deserialization of untrusted data is the #1 risk in the OWASP Top 10 for a reason -- it hands the attacker the keys to your application's internals."

## Overview

Serialization and deserialization are the invisible plumbing of every modern
application. Every JSON API, every config file, every message queue relies on
converting bytes into objects. When those bytes come from an attacker, the
results range from denial-of-service (stack overflow from deeply nested JSON)
to remote code execution (YAML tag abuse, Java object deserialization).

This module covers the attack surface of every major serialization format --
JSON, XML, YAML, Protobuf -- and teaches you to build defenses that hold
regardless of which format you choose.

## The Golden Rules

1. **NEVER deserialize into types with side effects.** If constructing your
   struct triggers database writes, network calls, or file operations, an
   attacker who controls the input controls your system.
2. **Use `deny_unknown_fields` in serde.** Unknown fields are not just noise --
   they are an attacker's foothold for type confusion and future exploit chains.
3. **Validate AFTER deserialization, not during.** Deserialization should parse;
   validation should check business rules. Mixing them hides security gaps.
4. **Limit recursion depth.** A 10 KB JSON blob with 100,000 nested arrays will
   stack-overflow most parsers. Set explicit depth limits.
5. **Never trust format-specific tags.** YAML `!!python/object/apply:os.system`
   and XML `<!ENTITY xxe SYSTEM "file:///etc/passwd">` are RCE payloads, not
   data.

## Why Deserialization Is So Dangerous

Deserialization turns untrusted bytes into live objects in your process. Unlike
SQL injection (where the attacker controls a query string) or XSS (where they
control HTML), deserialization attacks control **the shape of your object graph**.

| Attack Vector | Format | Impact |
|---------------|--------|--------|
| Type confusion | JSON, Protobuf | Logic bypass, privilege escalation |
| Deep nesting | JSON, XML | Stack overflow, denial of service |
| Tag abuse | YAML | Remote code execution |
| External entities | XML (XXE) | SSRF, file disclosure |
| Billion laughs | XML | Memory exhaustion, denial of service |
| Unknown fields | Protobuf | Silent data corruption |
| Default values | Protobuf | Authorization bypass |

## serde Safety Patterns

serde is Rust's serialization framework and is safer by default than most
language ecosystems (no arbitrary code execution during deserialization). But
"safer" is not "safe" -- you must still configure it correctly:

```rust
// UNSAFE: accepts any JSON, ignores unknown fields
#[derive(Deserialize)]
struct UserInput {
    name: String,
}

// SAFER: rejects unknown fields
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UserInput {
    name: String,
}
```

```rust
// UNSAFE: validates during deserialization (custom deserialize impl)
// This couples parsing to business logic and is hard to audit

// SAFER: deserialize first, validate second
let input: UserInput = serde_json::from_str(&raw)?;
validate(&input)?;  // Separate validation function
```

## Common Mistakes

| Mistake | Why It's Bad | Fix |
|---------|--------------|-----|
| No `deny_unknown_fields` | Attacker adds extra fields for type confusion | Add `#[serde(deny_unknown_fields)]` |
| No depth limit | Stack overflow from nested JSON | Use custom deserializer with depth counter |
| Deserializing XML without disabling external entities | XXE attack reads local files | Disable DTD processing entirely |
| Using YAML with default tags | `!!python/object` enables RCE | Use `serde_yaml` (safe), not `yaml-rust` with tags |
| Trusting protobuf default values | Missing field ≠ explicit zero | Always check `has_field()` or use `Option<T>` |
| Validating inside `Deserialize` impl | Hard to audit, couples concerns | Validate after deserialization |
| No size limit on input | Memory exhaustion | Cap input size before parsing |

## Learning Path

```
p01  Deserialization attacks    (type confusion, arbitrary code execution)
p02  Serde security             (deny_unknown_fields, validate after deserialization)
p03  JSON depth limiting        (prevent stack overflow via deeply nested JSON)
p04  Schema validation          (validate structure and types before processing)
p05  Format confusion attacks   (same data, different interpretation)
p06  Untrusted input            (never deserialize into types with side effects)
p07  Protobuf security          (unknown fields, default values, size limits)
p08  XML security               (XXE, billion laughs, SSRF via XML)
p09  YAML security              (YAML deserialization attacks, tag abuse)
p10  Safe default configurations (fail-closed, explicit over implicit)
```

## Quick Test

```bash
# Test your implementation
cargo test -p 16-serialization-security

# Test reference solution
cargo test -p 16-serialization-security --features solution
```

## References

- [OWASP Top 10:2021 A08 - Software and Data Integrity Failures](https://owasp.org/Top10/A08-2021-Software_and_Data_Integrity_Failures/)
- [OWASP Deserialization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Deserialization_Cheat_Sheet.html)
- [serde: Deny Unknown Fields](https://serde.rs/container-attrs.html#deny_unknown_fields)
- [CWE-502: Deserialization of Untrusted Data](https://cwe.mitre.org/data/definitions/502.html)
- [CWE-611: XML External Entity (XXE)](https://cwe.mitre.org/data/definitions/611.html)
- [JSON Bomb: Deeply Nested JSON Attacks](https://bishopfox.com/blog/json-interoperability-vulnerabilities)
- [YAML Deserialization Attacks](https://blog.skullsecurity.org/2022/theres-more-than-one-way-to-shell-a-server-yaml-deserialization)
