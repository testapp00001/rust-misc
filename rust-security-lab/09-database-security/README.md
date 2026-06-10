# Module 09: Database Security

> "Your database is the crown jewel. Protect it at every layer: the query, the storage, the connection, and the access."

## Overview

Database security spans the full lifecycle of data: how it enters the database
(parameterized queries), how it is stored (encryption at rest, field-level
encryption), how it is backed up (encrypted backups), who can see it (row-level
security, data masking), and how access is tracked (audit logging). This module
covers all ten layers with hands-on Rust exercises.

## The Golden Rules

1. **NEVER concatenate user input into SQL.** Use parameterized queries / prepared
   statements. SQL injection is still the #1 web vulnerability after 25 years.
2. **Encrypt sensitive fields before storing.** Even if an attacker dumps the
   database, they get ciphertext, not plaintext.
3. **Encrypt backups.** An unencrypted backup on a shared drive defeats every
   encryption-at-rest measure in the live database.
4. **Log every access to sensitive data.** If you cannot prove who read what and
   when, you cannot detect or investigate breaches.
5. **Use TLS for every database connection.** Network sniffing is trivial on
   shared infrastructure.

## Common Mistakes

| Mistake | Why It's Bad | Fix |
|---------|--------------|-----|
| String formatting SQL | SQL injection gives full DB access | Parameterized queries only |
| Storing SSNs/CCs plaintext | DB dump exposes everything | Field-level AES-GCM encryption |
| Unencrypted backups | Backup theft = full breach | Encrypt with AES-256, store key separately |
| No audit log | Cannot detect or prove breach | Log all reads/writes with user + timestamp |
| Trusting network perimeter | Lateral movement after compromise | TLS + cert pinning for DB connections |
| Same credentials for all envs | Dev leak exposes production | Separate credentials per environment |
| No data masking in dev/staging | Developers see real PII | Mask/hash sensitive fields in non-prod |
| Running migrations as root | Schema changes can drop tables | Least-privilege migration user |
| No row-level security | Any app user sees all rows | RLS policies or application-level filtering |
| Hardcoded DB passwords | Source code leak = DB access | Use env vars or secret manager |

## Learning Path

```
p01  Parameterized queries     (prevent SQL injection at the query layer)
p02  SQL injection attacks      (understand the enemy: login bypass, UNION injection)
p03  Field-level encryption     (encrypt sensitive columns with AES-GCM)
p04  Encryption at rest         (database-level encryption concepts)
p05  Secure backups             (encrypting database backups)
p06  Audit logging              (who accessed what, when)
p07  Data masking               (protect PII in non-production environments)
p08  Secure migrations          (safe schema changes)
p09  Connection security        (TLS, certificate validation)
p10  Row-level security         (restrict row access per user)
```

## Quick Test

```bash
# Test your implementation
cargo test -p database_security

# Test reference solution
cargo test -p database_security --features solution
```

## References

- [OWASP SQL Injection Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html)
- [OWASP Database Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Database_Security_Cheat_Sheet.html)
- [NIST SP 800-92: Guide to Computer Security Log Management](https://csrc.nist.gov/publications/detail/sp/800-92/final)
- [CIS Database Benchmarks](https://www.cisecurity.org/benchmark/database_benchmarks)
- [PostgreSQL Row-Level Security](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)
- [AES-GCM Field-Level Encryption (OWASP)](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
