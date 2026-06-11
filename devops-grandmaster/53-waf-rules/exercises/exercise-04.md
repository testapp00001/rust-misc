# Exercise 04: CSRF Token Validation

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Design and implement a CSRF protection system that combines WAF-level token validation with application-level defenses. Build rules that verify anti-CSRF tokens, enforce SameSite cookie attributes, and validate request origins.

## Scenario

Your application has multiple state-changing endpoints (transfer money, update profile, change password, delete account). The development team has implemented some CSRF protections but they are inconsistent. You need a WAF-level defense that enforces CSRF protection across all state-changing endpoints.

---

### Part A: Understand CSRF Attack Mechanics

A CSRF attack exploits the browser's automatic inclusion of cookies with cross-origin requests. Study the following attack flow:

```
1. User logs into bank.example.com (browser stores session cookie)
2. User visits evil-site.com (without logging out)
3. evil-site.com contains: <form action="https://bank.example.com/transfer" method="POST">
     <input name="to" value="attacker">
     <input name="amount" value="10000">
   </form>
   <script>document.forms[0].submit()</script>
4. Browser sends POST to bank.example.com WITH the session cookie
5. Server sees valid session and processes the transfer
```

For each protection mechanism below, explain how it prevents this attack and identify its limitations:

| Protection | How It Prevents CSRF | Limitations |
|-----------|---------------------|-------------|
| Anti-CSRF Token | ? | ? |
| SameSite Cookie Attribute | ? | ? |
| Origin/Referer Header Validation | ? | ? |
| Custom Request Header | ? | ? |
| Double Submit Cookie | ? | ? |

<details><summary>Hint</summary>

Anti-CSRF tokens are unpredictable values embedded in forms that the attacker cannot read (same-origin policy prevents reading the token). SameSite=Strict cookies are never sent with cross-origin requests. Origin/Referer headers tell the server where the request came from. Custom headers (like `X-Requested-With`) cannot be set in simple HTML forms. Double submit sends the token as both a cookie and a request parameter, then compares them.

</details>

---

### Part B: Implement WAF-Level CSRF Token Validation

Write WAF rules that enforce CSRF token presence and format on state-changing requests.

**Rule 1: Require CSRF token on POST/PUT/DELETE requests**

```apacheconf
# Enforce CSRF token on state-changing methods
SecRule REQUEST_METHOD "@rx ^(POST|PUT|PATCH|DELETE)$" \
  "id:4001,phase:1,chain,block,msg:'CSRF: Missing anti-CSRF token',severity:HIGH"
  SecRule &ARGS:csrf_token "@eq 0" \
    "chain"
    SecRule &REQUEST_HEADERS:X-CSRF-Token "@eq 0" ""
```

**Task:** Extend the CSRF protection with these rules:

1. **Token format validation** -- CSRF tokens should be at least 32 characters of alphanumeric content
2. **Token freshness** -- reject tokens older than the session timeout (use a timestamp component in the token)
3. **Endpoint-specific protection** -- apply stricter rules to high-risk endpoints (transfer, delete, password change)
4. **API endpoint protection** -- for API endpoints that use JSON bodies, validate the token in the header rather than the body

Write the rules:

```apacheconf
# Rule 2: Token format validation
# TODO: Verify token is at least 32 chars, alphanumeric

# Rule 3: High-risk endpoint protection
# TODO: Apply additional validation to /api/transfer, /api/account/delete, /api/password/change

# Rule 4: JSON API CSRF protection
# TODO: For endpoints with Content-Type: application/json, check X-CSRF-Token header

# Rule 5: Origin header validation
# TODO: For state-changing requests, verify Origin or Referer matches the application domain
```

<details><summary>Hint 1: Token format</summary>

Use `@rx ^[a-zA-Z0-9]{32,}$` to validate token format. The `&ARGS:csrf_token @eq 0` syntax checks if the parameter exists (count of 0 means missing).

</details>

<details><summary>Hint 2: Origin validation</summary>

Use `@beginsWith` to check that the Origin header starts with your domain. Be careful with subdomains. Reject requests with missing Origin headers on state-changing endpoints, but be aware that some legitimate clients (proxies, privacy tools) strip the Origin header.

</details>

---

### Part C: Design the SameSite Cookie Configuration

Write the `Set-Cookie` header configuration for your application's session cookie and CSRF token cookie. Explain each attribute.

```
# Session cookie
Set-Cookie: ???; ???; ???; ???

# CSRF token cookie (for double-submit pattern)
Set-Cookie: ???; ???; ???; ???
```

For each cookie, document:
1. The `SameSite` attribute value and why
2. The `Secure` attribute and why
3. The `HttpOnly` attribute and why
4. The `Path` attribute and why
5. The `Domain` attribute and why

| Attribute | Session Cookie | CSRF Cookie | Why |
|-----------|---------------|-------------|-----|
| SameSite | ? | ? | ? |
| Secure | ? | ? | ? |
| HttpOnly | ? | ? | ? |
| Path | ? | ? | ? |
| Domain | ? | ? | ? |

<details><summary>Hint</summary>

Session cookie: `SameSite=Strict` or `Lax`, `Secure`, `HttpOnly` (prevents XSS from reading it), `Path=/`, `Domain=app.example.com`. CSRF cookie: `SameSite=Strict` or `Lax`, `Secure`, but NOT `HttpOnly` (JavaScript needs to read it for the double-submit pattern). `SameSite=Lax` allows cookies with top-level navigations (GET requests from external links) but blocks them with cross-origin POST requests.

</details>

---

### Part D: Implement Origin and Referer Validation

Write WAF rules that validate the Origin and Referer headers on state-changing requests:

```apacheconf
# Rule: Validate Origin header on state-changing requests
SecRule REQUEST_METHOD "@rx ^(POST|PUT|PATCH|DELETE)$" \
  "id:4010,phase:1,chain,block,msg:'CSRF: Invalid Origin header',severity:HIGH"
  SecRule REQUEST_HEADERS:Origin "!@beginsWith https://app.example.com" ""

# Rule: Block state-changing requests with no Origin and no Referer
# TODO

# Rule: Validate Referer when Origin is missing
# TODO

# Rule: Block requests from known malicious origins
# TODO
```

Handle these edge cases:

| Scenario | Expected Behavior | Reasoning |
|----------|------------------|-----------|
| Origin: `https://app.example.com` | Allow | Legitimate same-origin request |
| Origin: `https://evil.com` | Block | Cross-origin state-changing request |
| Origin: `null` | ? | Some privacy browsers send `null` Origin |
| No Origin, Referer: `https://app.example.com/page` | ? | Some tools strip Origin but send Referer |
| No Origin, No Referer | ? | Could be legitimate API client or attack |
| Origin: `https://app.example.com.evil.com` | ? | Subdomain attack |

<details><summary>Hint</summary>

`Origin: null` is sent by some browsers for privacy-sensitive contexts (sandboxed iframes, file:// URLs). It can be legitimate but is also used by attackers. Consider allowing it only for specific endpoints. When Origin is missing, fall back to Referer validation. Always use `@beginsWith https://app.example.com` rather than `@contains` to prevent subdomain tricks. Block requests with no Origin AND no Referer on state-changing endpoints.

</details>

---

### Part E: Design the Complete CSRF Protection Architecture

Combine all the mechanisms into a layered CSRF protection strategy:

```
Layer 1: WAF Rules (edge)
    - Origin/Referer validation
    - CSRF token presence check
    - Token format validation

Layer 2: Application Framework (middleware)
    - Token generation and validation
    - Session binding

Layer 3: Cookie Configuration (browser)
    - SameSite attribute
    - Secure and HttpOnly flags

Layer 4: Response Headers (defense in depth)
    - CSP frame-ancestors
    - X-Frame-Options
```

For each layer, answer:
1. What specific threats does this layer address?
2. What happens if this layer is bypassed?
3. How does it interact with the other layers?

<details><summary>Hint</summary>

Layer 1 (WAF) catches obvious CSRF attempts at the edge without any application changes. Layer 2 (framework) provides cryptographic verification that the request originated from your site. Layer 3 (cookies) prevents the browser from sending credentials with cross-origin requests. Layer 4 (headers) prevents clickjacking, which is a related attack. If Layer 1 is bypassed (e.g., valid-looking token), Layer 2 catches it (token does not match). If Layer 2 fails (e.g., token leaked), Layer 3 prevents the cookie from being sent cross-origin.

</details>

## Success Criteria

- [ ] At least 4 CSRF protection mechanisms implemented in WAF rules
- [ ] Token validation rules check both presence and format
- [ ] SameSite cookie configuration is correct for both session and CSRF cookies
- [ ] Origin/Referer validation handles edge cases (null Origin, missing headers)
- [ ] High-risk endpoints have additional protection beyond baseline rules
- [ ] Can explain why CSRF protection requires multiple layers, not just WAF rules

## What You Should Understand After This Exercise

- CSRF exploits the browser's automatic cookie handling, not a vulnerability in the application code
- Anti-CSRF tokens work because the same-origin policy prevents attackers from reading them
- SameSite cookies are a browser-level defense that reduces but does not eliminate CSRF risk
- Origin/Referer validation is a useful defense but can be stripped by proxies and privacy tools
- Effective CSRF protection requires defense in depth: WAF, application, cookies, and headers working together
