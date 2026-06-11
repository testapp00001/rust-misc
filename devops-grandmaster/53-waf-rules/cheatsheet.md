# Cheatsheet: WAF Rules

## OWASP Top 10

| # | Vulnerability | Prevention |
|---|--------------|------------|
| 1 | Broken Access Control | RBAC, authorization checks |
| 2 | Cryptographic Failures | HTTPS, encryption at rest |
| 3 | Injection | Parameterized queries, input validation |
| 4 | Insecure Design | Threat modeling, security reviews |
| 5 | Security Misconfiguration | Hardening, minimal config |
| 6 | Vulnerable Components | Dependency scanning, updates |
| 7 | Auth Failures | MFA, session management |
| 8 | Data Integrity Failures | Digital signatures, CI/CD security |
| 9 | Logging Failures | Centralized logging, monitoring |
| 10 | SSRF | Input validation, allowlists |

## SQL Injection Prevention
```python
# BAD — vulnerable to SQL injection
query = f"SELECT * FROM users WHERE name = '{user_input}'"

# GOOD — parameterized query
query = "SELECT * FROM users WHERE name = %s"
cursor.execute(query, (user_input,))
```

## XSS Prevention
```python
# Content Security Policy header
response.headers['Content-Security-Policy'] = "default-src 'self'"

# Output encoding
from markupsafe import escape
safe_output = escape(user_input)
```

## CSRF Prevention
```python
# Flask
from flask_wtf.csrf import CSRFProtect
csrf = CSRFProtect(app)

# Token in form
<form>
    <input type="hidden" name="csrf_token" value="{{ csrf_token() }}">
</form>
```
