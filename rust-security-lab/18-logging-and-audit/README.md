# Module 18: Logging and Audit

> "Logs are the black box of your application. If they contain secrets, they're a treasure map for attackers."

## Overview

Logging is essential for debugging, monitoring, and incident response -- but it is
also one of the most common sources of data breaches. Developers routinely log
passwords, API keys, PII, and session tokens without realizing that log aggregation
systems (Splunk, ELK, Datadog) are high-value targets. This module covers secure
logging practices: what to log, what to redact, how to prevent log injection, and
how to build tamper-evident audit trails.

## The Golden Rules

1. **NEVER log secrets.** Passwords, API keys, tokens, and private keys must never
   appear in logs. A single misconfigured log line can expose your entire system.
2. **NEVER log raw PII.** Emails, SSNs, credit card numbers, and addresses must be
   redacted or masked before logging. GDPR fines can reach 4% of global revenue.
3. **Sanitize all user input in log messages.** CRLF injection (`\r\n`) can forge
   log entries, inject fake audit trails, or exploit log viewers (XSS in Kibana).
4. **Use structured logging.** JSON logs with consistent fields (timestamp, level,
   event, user_id, correlation_id) are machine-parseable and searchable.
5. **Build tamper-evident audit trails.** Hash chains and digital signatures make
   it possible to detect if someone deleted or modified log entries.

## Common Mistakes

| Mistake | Why It's Bad | Fix |
|---------|--------------|-----|
| Logging passwords | Log breach = credential breach | Never log; use redacted placeholders |
| Logging full credit card | PCI-DSS violation, massive fines | Mask: `**** **** **** 1234` |
| Logging full SSN | Identity theft risk | Mask: `*** **-**34` |
| Logging raw user input | CRLF injection, log forgery | Sanitize: strip `\r\n`, control chars |
| Using DEBUG level for auth failures | Missed attacks in production | Use WARN for auth failures |
| No correlation IDs | Can't trace requests across services | Generate UUID per request, propagate |
| Log retention forever | Compliance violation, storage cost | Define retention policy per log type |
| Deleting logs after incident | Obstruction of justice, SOC2 failure | Tamper-evident, append-only storage |
| Logging in plaintext over network | Man-in-the-middle can read logs | Use TLS for log shipping |
| No audit trail for admin actions | Can't detect insider threats | Log who, what, when, where, result |

## Log Injection Attack

The most underappreciated logging vulnerability. If user input flows directly into
log messages, an attacker can inject newlines to forge entries:

```
# User provides this as their username:
admin\r\n2024-01-15 INFO User admin logged in successfully

# Log output becomes:
2024-01-15 INFO Login attempt for user: admin
2024-01-15 INFO User admin logged in successfully  <-- FORGED
```

The forged entry looks legitimate. An auditor reviewing logs would believe `admin`
logged in. Defense: strip or escape all control characters from user input before
logging.

## Compliance Requirements

| Framework | Logging Requirement |
|-----------|-------------------|
| **GDPR** | Right to erasure -- must be able to delete user data from logs |
| **SOC2** | Audit trail for all access to sensitive data, tamper-evident logs |
| **HIPAA** | Access logs for all PHI, 6-year retention, encryption at rest |
| **PCI-DSS** | Never log full PAN, audit trail for all access to cardholder data |
| **SOX** | Audit trail for financial system changes, 7-year retention |

## Learning Path

```
p01  PII redaction              (mask emails, SSNs, credit cards)
p02  Structured logging         (JSON logs with consistent fields)
p03  Log injection              (CRLF injection, defense with sanitization)
p04  Tamper-evident logs        (hash chain, append-only, signed entries)
p05  Audit trail                (who, what, when, where, result)
p06  Sensitive data masking     (show partial values: ****1234)
p07  Log levels security        (auth failures at WARN, not DEBUG)
p08  Correlation IDs            (request tracing across services)
p09  Compliance logging         (GDPR, SOC2, HIPAA requirements)
p10  Log retention              (retention policies, secure deletion)
```

## Quick Test

```bash
# Test your implementation
cargo test -p 18-logging-and-audit

# Test reference solution
cargo test -p 18-logging-and-audit --features solution
```

## References

- [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)
- [OWASP Log Injection](https://owasp.org/www-community/attacks/Log_Injection)
- [GDPR Article 17: Right to Erasure](https://gdpr-info.eu/art-17-gdpr/)
- [PCI-DSS v4.0: Requirement 10](https://www.pcisecuritystandards.org/document_library/)
- [NIST SP 800-92: Guide to Computer Security Log Management](https://csrc.nist.gov/publications/detail/sp/800-92/final)
- [SOC2 Trust Services Criteria](https://us.aicpa.org/interestareas/frc/assuranceadvisoryservices/aaborgatrustservicescriteria)
