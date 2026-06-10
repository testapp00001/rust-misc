# Module 07: Password Security

> "If you store passwords, you hold the keys to people's digital lives. Treat them accordingly."

## Overview

Password security is one of the most commonly implemented -- and most commonly
butchered -- areas of application security. This module covers the complete
lifecycle: hashing, salting, peppering, validation, brute-force defense, reset
flows, and migration from legacy algorithms.

## The Golden Rules

1. **NEVER store passwords in plaintext.** A single SQL injection exposes every user.
2. **NEVER use fast hashes (SHA-256, MD5) for passwords.** An attacker with a GPU
   can try billions of SHA-256 hashes per second. A password that takes you 1 ms
   to check takes an attacker 0.0000001 ms.
3. **Use Argon2id.** It is the winner of the Password Hashing Competition (PHC) and
   the OWASP-recommended algorithm. It is memory-hard, meaning it resists GPU and
   ASIC attacks by requiring large amounts of RAM.
4. **Always use a unique salt per password.** Salts defeat rainbow tables -- precomputed
   lookup tables that trade time for space.
5. **Tune cost factors for your threat model.** Higher cost = more security, but also
   more latency for legitimate users. The goal is to make attacks expensive while
   keeping login under ~500 ms.

## Why Memory-Hard Functions Matter

A standard CPU can compute SHA-256 at billions of hashes per second. A GPU with
thousands of cores can do it even faster. But Argon2id and scrypt require a large
amount of RAM per hash. GPUs have limited memory per core, and ASICs cannot cheaply
replicate large SRAM banks. This levels the playing field:

| Algorithm   | Type         | GPU Resistance | ASIC Resistance |
|-------------|--------------|----------------|-----------------|
| SHA-256     | Fast hash    | None           | None            |
| MD5         | Fast hash    | None           | None            |
| bcrypt      | CPU-hard     | Moderate       | Moderate        |
| scrypt      | Memory-hard  | Good           | Good            |
| Argon2id    | Memory-hard  | Excellent      | Excellent       |

## Common Mistakes

| Mistake | Why It's Bad | Fix |
|---------|--------------|-----|
| Storing plaintext | Any DB leak exposes all passwords | Hash with Argon2id |
| Using SHA-256/MD5 | Billions of guesses/sec on GPU | Use Argon2id |
| No salt | Rainbow table attack | Unique random salt per password |
| Short salt | Reduces salt effectiveness | Use 16+ byte salt |
| Hardcoded salt | Same as no salt for all users | Generate randomly per password |
| Using SHA-1 | Broken (collision attacks found) | Never use SHA-1 for anything |
| Comparing hashes with `==` | Timing attack leaks hash bytes | Use constant-time comparison |
| No rate limiting | Brute force becomes feasible | Exponential backoff + lockout |
| Reusing password reset tokens | Token replay attacks | Single-use, time-limited tokens |

## Learning Path

```
p01  Argon2id hashing        (the recommended algorithm)
p02  bcrypt hashing           (widely used, tunable cost)
p03  scrypt hashing           (memory-hard alternative)
p04  Salting                  (why salts defeat rainbow tables)
p05  Pepper                   (application-wide secret)
p06  Timing attack            (measuring response time to guess hash)
p07  Password validation      (strength rules, breach check)
p08  Brute force defense      (rate limiting, lockout, backoff)
p09  Password reset           (secure token flow)
p10  Password migration       (MD5/SHA -> Argon2id)
```

## Quick Test

```bash
# Test your implementation
cargo test -p 07-password-security

# Test reference solution
cargo test -p 07-password-security --features solution
```

## References

- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [Argon2 Specification (PHC Winner)](https://github.com/P-H-C/phc-winner-argon2/blob/master/argon2-specs.pdf)
- [NIST SP 800-63B: Digital Identity Guidelines](https://pages.nist.gov/800-63-3/sp800-63b.html)
- [bcrypt Paper](https://www.usenix.org/legacy/event/usenix99/provos/provos.pdf)
- [scrypt Paper](https://www.tarsnap.com/scrypt/scrypt.pdf)
- [Have I Been Pwned API](https://haveibeenpwned.com/API/v3)
