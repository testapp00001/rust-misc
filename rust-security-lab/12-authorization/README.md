# Module 12: Authorization

> "Authentication answers WHO you are. Authorization answers WHAT you can do. Confusing the two is a critical security mistake."

## Overview

Authorization is the gatekeeper of your system -- once a user is authenticated, authorization determines what they can access and what actions they can perform. This module covers:

- **RBAC**: Role-Based Access Control -- assign roles, map roles to permissions
- **ABAC**: Attribute-Based Access Control -- policies based on attributes
- **Capability Tokens**: Tokens that encode specific granted permissions
- **Permission Hierarchies**: Admin > Editor > Viewer inheritance chains
- **Least Privilege**: Grant only the minimum permissions needed
- **Authorization Bypass**: IDOR, path traversal, privilege escalation attacks
- **Resource Scoping**: Users access only their own resources
- **Policy Engines**: Evaluate allow/deny from configurable rules
- **Audit Logging**: Record who tried what and the decision made
- **Graceful Denial**: Reject requests without leaking information

## Key Concepts

### Authentication vs Authorization

```
Authentication: "Who are you?"    → Identity verification (Module 11)
Authorization:  "What can you do?" → Permission enforcement (this module)
```

| Concept | Authentication | Authorization |
|---------|---------------|---------------|
| Purpose | Verify identity | Enforce access control |
| Input | Credentials (password, token) | Identity + requested resource + action |
| Output | User identity / session | Allow / Deny decision |
| Failure | "Invalid credentials" | "Access denied" |
| Timing | Happens first | Happens after authentication |

### RBAC vs ABAC vs Capability Tokens

| Model | How It Works | Best For | Limitation |
|-------|-------------|----------|------------|
| **RBAC** | Users assigned roles; roles have permissions | Simple org structures | Role explosion in complex systems |
| **ABAC** | Policies evaluate user/resource/env attributes | Fine-grained, context-aware rules | Complex to audit and debug |
| **Capability** | Tokens carry granted permissions directly | Delegation, microservices | Token revocation is hard |

### RBAC (Role-Based Access Control)
```
User → Role → Permissions
Alice → Admin → [read, write, delete, manage_users]
Bob   → Editor → [read, write]
Carol → Viewer → [read]
```

### ABAC (Attribute-Based Access Control)
```
Policy: ALLOW if user.department == "engineering" AND resource.classification <= "internal"
        AND time.hour BETWEEN 9 AND 17
```

### Capability Tokens
```
Token = { user: "alice", resource: "/docs/42", actions: ["read", "write"], expires: "2025-01-01" }
```

### Least Privilege Principle
Every user, program, and process should operate with the minimum set of privileges needed. Start with NOTHING, grant only what is explicitly required.

### IDOR (Insecure Direct Object Reference)
```
GET /api/users/123/documents   ← Attacker changes 123 to 456
GET /api/users/456/documents   ← Accesses another user's data!
Defense: Always verify the authenticated user OWNS the requested resource.
```

## Attack Patterns

### Authorization Bypass via IDOR
Changing resource IDs in URLs to access other users' data. The server must verify ownership, not just authentication.

### Privilege Escalation
A regular user gaining admin access by:
- Modifying hidden form fields (`role=admin`)
- Accessing admin endpoints directly
- Exploiting role assignment logic flaws

### Path Traversal
Using `../` in file paths to escape resource boundaries:
```
GET /files/../../../etc/passwd
Defense: Canonicalize paths and verify they stay within allowed directories.
```

### Information Leakage in Error Messages
```
BAD:  "Access denied: user 'bob' is not in role 'admin'"
GOOD: "Access denied" (no details about why)
```

## Rust-Specific Tips

1. Use enums for roles and permissions -- exhaustive matching catches missing cases
2. Use `HashSet<Permission>` for efficient permission lookups
3. Implement `Display` for error types but limit internal details in production responses
4. Use serde for serializing/deserializing capability tokens
5. Use ring/sha2 for signing capability tokens to prevent tampering

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_rbac_basics.rs` | RBAC fundamentals | Define roles, assign permissions, check access |
| 02 | `p02_abac_basics.rs` | ABAC fundamentals | Policies based on attributes |
| 03 | `p03_capability_tokens.rs` | Capability tokens | Tokens encoding specific permissions |
| 04 | `p04_permission_hierarchy.rs` | Permission hierarchies | Admin > Editor > Viewer inheritance |
| 05 | `p05_least_privilege.rs` | Least privilege | Grant minimum necessary permissions |
| 06 | `p06_authorization_bypass.rs` | Authorization bypass | IDOR, path traversal, privilege escalation |
| 07 | `p07_resource_scoping.rs` | Resource scoping | Users access only own resources |
| 08 | `p08_policy_engine.rs` | Policy engine | Evaluate allow/deny from rules |
| 09 | `p09_audit_decisions.rs` | Audit logging | Log who tried what, allowed/denied |
| 10 | `p10_graceful_denial.rs` | Graceful denial | Reject without information leakage |

## References

- [OWASP Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html)
- [OWASP IDOR](https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/05-Authorization_Testing/04-Testing_for_Insecure_Direct_Object_References)
- [NIST RBAC Standard](https://csrc.nist.gov/publications/detail/conference-paper/2000/07/01/the-nist-model-for-role-based-access-control)
- [XACML - ABAC Standard](https://docs.oasis-open.org/xacml/3.0/xacml-3.0-core-spec-os-en.html)
- [Capability-based Security](https://en.wikipedia.org/wiki/Capability-based_security)
