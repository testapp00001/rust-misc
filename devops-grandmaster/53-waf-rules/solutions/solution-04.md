# Solution 04: CSRF Token Validation

## Part A: Protection Mechanisms

| Protection | How It Prevents CSRF | Limitations |
|-----------|---------------------|-------------|
| Anti-CSRF Token | Server embeds an unpredictable token in each form. The attacker cannot read the token because the same-origin policy prevents reading cross-origin responses. Without the token, the forged request is rejected. | Does not protect APIs that use tokens in headers (CORS might allow the header). Token must be unpredictable and bound to the session. Does not protect GET-based CSRF. |
| SameSite Cookie Attribute | `SameSite=Strict` prevents the browser from sending the cookie with any cross-origin request. `SameSite=Lax` blocks cross-origin POST but allows GET from external links. | `Lax` still allows GET-based CSRF. `Strict` breaks legitimate external links (user must re-authenticate after clicking a link from an email). Browser support varies. |
| Origin/Referer Header Validation | Server checks that the `Origin` or `Referer` header matches its own domain. Cross-origin requests from `evil.com` will have `Origin: https://evil.com`. | Privacy tools and proxies strip these headers. Some browsers send `Origin: null` in certain contexts. Cannot distinguish same-site from same-origin. |
| Custom Request Header | APIs require a custom header (e.g., `X-Requested-With: XMLHttpRequest`). Simple HTML forms cannot set custom headers, so the attacker's forged form will not include it. | Only works for APIs (not traditional form submissions). CORS configuration must allow the custom header. Does not protect against XSS (which runs in the same origin). |
| Double Submit Cookie | Server sets a random value as both a cookie and a form parameter. On submission, server verifies they match. Attacker cannot read the cookie value to set the parameter. | Vulnerable if the attacker can set cookies (via a subdomain or cookie injection). Requires HTTPS to prevent cookie interception. Must use a cryptographically random value. |

### Why This Works

Each mechanism exploits a different property of the browser security model:

- **Anti-CSRF tokens** exploit the same-origin policy: `evil.com` cannot read the token from `bank.com`'s response.
- **SameSite cookies** exploit the browser's cookie handling rules: the browser itself refuses to send the cookie cross-origin.
- **Origin headers** exploit the browser's automatic header inclusion: the browser always sends the correct Origin.
- **Custom headers** exploit the limitation of HTML forms: `<form>` can only set `Content-Type`, not custom headers.
- **Double submit** exploits cookie isolation: the attacker cannot read the cookie to include its value in the request.

### Common Mistakes

- **Using only one mechanism.** Each has weaknesses. Defense in depth means combining them.
- **Putting the CSRF token in a cookie.** If the token is in a cookie, the browser sends it automatically, defeating the purpose. The token must be in the form body or a custom header.
- **Using GET for state-changing operations.** GET requests should never change state. All state changes must use POST/PUT/DELETE.

---

## Part B: WAF-Level CSRF Token Validation

```apacheconf
# Rule 1: Require CSRF token on POST/PUT/DELETE (provided in exercise)
SecRule REQUEST_METHOD "@rx ^(POST|PUT|PATCH|DELETE)$" \
  "id:4001,phase:1,chain,block,msg:'CSRF: Missing anti-CSRF token',severity:HIGH"
  SecRule &ARGS:csrf_token "@eq 0" \
    "chain"
    SecRule &REQUEST_HEADERS:X-CSRF-Token "@eq 0" ""

# Rule 2: Token format validation (at least 32 alphanumeric chars)
SecRule ARGS:csrf_token|REQUEST_HEADERS:X-CSRF-Token \
  "!@rx ^[a-zA-Z0-9+/=_-]{32,}$" \
  "id:4002,phase:1,chain,block,msg:'CSRF: Invalid token format',severity:HIGH"
  SecRule REQUEST_METHOD "@rx ^(POST|PUT|PATCH|DELETE)$" ""

# Rule 3: High-risk endpoint additional validation
SecRule REQUEST_URI "@rx ^/api/(transfer|account/delete|password/change)" \
  "id:4003,phase:1,chain,block,msg:'CSRF: High-risk endpoint missing double protection',severity:CRITICAL"
  SecRule REQUEST_METHOD "@rx ^POST$" \
    "chain"
    SecRule &ARGS:csrf_token "@eq 0" \
      "chain"
      SecRule &REQUEST_HEADERS:X-CSRF-Token "@eq 0" ""

# Rule 4: JSON API CSRF protection
SecRule REQUEST_METHOD "@rx ^(POST|PUT|PATCH|DELETE)$" \
  "id:4004,phase:1,chain,block,msg:'CSRF: JSON API missing CSRF header',severity:HIGH"
  SecRule REQUEST_HEADERS:Content-Type "@beginsWith application/json" \
    "chain"
    SecRule &REQUEST_HEADERS:X-CSRF-Token "@eq 0" ""

# Rule 5: Origin header validation on state-changing requests
SecRule REQUEST_METHOD "@rx ^(POST|PUT|PATCH|DELETE)$" \
  "id:4010,phase:1,chain,block,msg:'CSRF: Invalid Origin header',severity:HIGH"
  SecRule REQUEST_HEADERS:Origin "!@beginsWith https://app.example.com" \
    "chain"
    SecRule REQUEST_HEADERS:Origin "!@streq null" ""

# Rule 6: Block state-changing requests with no Origin AND no Referer
SecRule REQUEST_METHOD "@rx ^(POST|PUT|PATCH|DELETE)$" \
  "id:4011,phase:1,chain,block,msg:'CSRF: No Origin or Referer on state-changing request',severity:HIGH"
  SecRule &REQUEST_HEADERS:Origin "@eq 0" \
    "chain"
    SecRule &REQUEST_HEADERS:Referer "@eq 0" ""

# Rule 7: Validate Referer when Origin is missing
SecRule REQUEST_METHOD "@rx ^(POST|PUT|PATCH|DELETE)$" \
  "id:4012,phase:1,chain,block,msg:'CSRF: Invalid Referer header',severity:MEDIUM"
  SecRule &REQUEST_HEADERS:Origin "@eq 0" \
    "chain"
    SecRule REQUEST_HEADERS:Referer "!@beginsWith https://app.example.com" ""

# Rule 8: Block requests from known malicious origins
SecRule REQUEST_HEADERS:Origin|REQUEST_HEADERS:Referer \
  "@pmFromFile malicious-origins.txt" \
  "id:4013,phase:1,block,msg:'CSRF: Request from known malicious origin',severity:CRITICAL"
```

### Why This Works

The rules implement a layered CSRF defense at the WAF level:

1. **Token presence** (Rule 1): Ensures every state-changing request includes a CSRF token in the form body or a custom header.
2. **Token format** (Rule 2): Validates that the token looks like a real token (at least 32 alphanumeric characters), not a placeholder or empty value.
3. **High-risk endpoints** (Rule 3): Applies additional scrutiny to endpoints that perform sensitive operations.
4. **JSON APIs** (Rule 4): For API endpoints, the token must be in a custom header (since JSON APIs do not use form bodies for tokens).
5. **Origin validation** (Rules 5-7): Verifies that the request originates from the application domain.
6. **No Origin/Referer** (Rule 6): Blocks state-changing requests that have neither header, which is suspicious.
7. **Malicious origins** (Rule 8): Maintains a blocklist of known CSRF attack origins.

### Common Mistakes

- **Only checking for token presence.** The token must also be validated (format, freshness, session binding). A static string like "csrf-token" provides no protection.
- **Not handling Origin: null.** Some legitimate clients send `Origin: null` (sandboxed iframes, privacy browsers). Allow it but log it for monitoring.
- **Blocking all requests without Origin.** Some legitimate API clients (mobile apps, server-to-server) do not send Origin headers. Use a separate API key mechanism for these clients.

---

## Part C: SameSite Cookie Configuration

```
# Session cookie
Set-Cookie: session_id=abc123; SameSite=Lax; Secure; HttpOnly; Path=/; Domain=app.example.com

# CSRF token cookie (for double-submit pattern)
Set-Cookie: csrf_token=xyz789; SameSite=Lax; Secure; Path=/; Domain=app.example.com
```

| Attribute | Session Cookie | CSRF Cookie | Why |
|-----------|---------------|-------------|-----|
| SameSite | `Lax` | `Lax` | `Lax` blocks cross-origin POST (prevents CSRF) while allowing cross-origin GET navigations (preserves usability). `Strict` would block all cross-origin requests including legitimate links from emails and search engines. |
| Secure | Yes | Yes | Both cookies should only be sent over HTTPS. Without `Secure`, cookies can be intercepted over HTTP (e.g., during a redirect). |
| HttpOnly | Yes | **No** | The session cookie must be `HttpOnly` to prevent XSS from stealing it. The CSRF cookie must NOT be `HttpOnly` because JavaScript needs to read it for the double-submit pattern. |
| Path | `/` | `/` | Both cookies should be available for the entire application. Restricting the path could cause authentication failures on some endpoints. |
| Domain | `app.example.com` | `app.example.com` | Do not use `.example.com` (leading dot). This would allow subdomains to read the cookies, increasing the attack surface. |

### Why This Works

`SameSite=Lax` is the recommended default for most applications:

- **Cross-origin POST** (CSRF attack): Browser does NOT send the cookie. The server sees no session and rejects the request. Attack blocked.
- **Cross-origin GET** (user clicks link from email): Browser DOES send the cookie. User does not need to re-authenticate. Usability preserved.
- **Same-origin POST** (legitimate form submission): Browser DOES send the cookie. Normal application flow works.

The CSRF cookie intentionally omits `HttpOnly` because the double-submit pattern requires JavaScript to read the cookie value and include it as a request parameter or header. This is safe because the CSRF token is not a secret in the same way a session token is -- its security comes from the attacker's inability to set it on the correct domain.

### Common Mistakes

- **Using `SameSite=Strict` everywhere.** This breaks legitimate user flows (clicking links from emails, OAuth callbacks, third-party integrations).
- **Setting `HttpOnly` on the CSRF cookie.** This prevents JavaScript from reading it, breaking the double-submit pattern.
- **Using a broad domain scope.** `Domain=.example.com` allows any subdomain to access the cookies. If a subdomain is compromised, all cookies are exposed.

---

## Part D: Origin and Referer Validation Edge Cases

| Scenario | Expected Behavior | Reasoning |
|----------|------------------|-----------|
| Origin: `https://app.example.com` | Allow | Legitimate same-origin request. |
| Origin: `https://evil.com` | Block | Cross-origin state-changing request is a CSRF attack. |
| Origin: `null` | Allow with logging | Some privacy browsers and sandboxed iframes send `null`. Log for monitoring but do not block. Consider blocking for high-risk endpoints. |
| No Origin, Referer: `https://app.example.com/page` | Allow | Fall back to Referer when Origin is missing. Some tools strip Origin but preserve Referer. |
| No Origin, No Referer | Block | No way to verify request origin. Legitimate API clients should use API keys or custom headers instead. |
| Origin: `https://app.example.com.evil.com` | Block | Subdomain attack. `@beginsWith https://app.example.com` matches this. Use `@beginsWith https://app.example.com/` or `@streq https://app.example.com` for exact match. |

### Why This Works

The edge case handling follows the principle of defense in depth:

1. **Primary validation**: Origin header (most reliable, always set by browsers for cross-origin requests).
2. **Fallback validation**: Referer header (set by browsers but can be stripped by privacy tools).
3. **Default action**: Block (when neither header is present, deny the request).

The `null` Origin case is the most nuanced. It is sent by:
- Sandboxed iframes without the `allow-same-origin` flag
- Privacy-focused browsers (Tor Browser)
- `file://` protocol requests
- Redirects from HTTPS to HTTP

For most applications, allowing `null` with logging is the right balance. For financial or healthcare applications, blocking `null` on high-risk endpoints is appropriate.

### Common Mistakes

- **Using `@contains` instead of `@beginsWith`.** `@contains app.example.com` would match `https://evil.com/app.example.com`. Always use `@beginsWith` or exact match.
- **Not handling subdomain attacks.** `app.example.com.evil.com` starts with `app.example.com` but is not your domain. Use exact matching or verify the domain is followed by `/`, `:`, or end of string.
- **Blocking all requests without Origin.** Some legitimate clients (mobile apps, webhooks, server-to-server) do not send Origin headers. Provide an alternative authentication mechanism for these clients.

---

## Part E: Layered CSRF Defense Architecture

| Layer | What Specific Threats | If Bypassed | Interaction with Other Layers |
|-------|---------------------|-------------|------------------------------|
| WAF Rules (Edge) | Automated CSRF attacks, known malicious origins, missing tokens | Application framework validates tokens | WAF reduces load on application by blocking obvious attacks. Application validates the cryptographic token. |
| Application Framework | Token forgery, token reuse, session fixation | Cookie configuration prevents cross-origin cookie sending | Framework generates and validates tokens. Cookies ensure tokens are only sent with legitimate requests. |
| Cookie Configuration | Cross-origin credential sending | Response headers prevent clickjacking | SameSite prevents the browser from sending cookies cross-origin. CSP prevents the page from being framed. |
| Response Headers | Clickjacking (related attack), information leakage | WAF blocks known attack patterns | CSP `frame-ancestors` prevents iframe embedding. WAF blocks requests with suspicious patterns. |

### Why This Works

Each layer addresses a different aspect of the CSRF threat model:

- **WAF** catches the low-hanging fruit: automated attacks, missing tokens, known malicious origins. It operates at the edge, reducing load on the application.
- **Application** provides cryptographic verification: the token is bound to the session and cannot be forged. This is the primary defense.
- **Cookies** use the browser's built-in security: SameSite prevents the browser from sending credentials cross-origin. This is a browser-enforced defense that the attacker cannot bypass.
- **Response headers** prevent related attacks: clickjacking uses iframes to trick users into clicking on hidden elements. CSP `frame-ancestors` prevents this.

The key insight is that each layer operates independently. If the WAF is bypassed (e.g., the attacker has a valid-looking token), the application's cryptographic validation catches it. If the application's token validation fails (e.g., the token leaks), the SameSite cookie prevents the browser from sending the session. If the cookie is somehow sent (e.g., SameSite bypass in an old browser), the CSP prevents the page from being framed.

### Common Mistakes

- **Relying on a single layer.** No single defense is perfect. CSRF protection requires all four layers working together.
- **Not testing each layer independently.** Disable each layer one at a time and verify that the other layers still provide protection.
- **Forgetting about related attacks.** CSRF and clickjacking are related (both exploit cross-origin trust). Protect against both.

---

## Common Mistakes to Avoid

1. **Using GET for state-changing operations.** GET requests should be idempotent. All state changes must use POST/PUT/DELETE.

2. **Not validating the token cryptographically.** A random-looking string is not enough. The token must be bound to the session and verified server-side.

3. **Reusing tokens across sessions.** Each session should have a unique CSRF token. Reusing tokens allows session fixation attacks.

4. **Not protecting AJAX endpoints.** APIs that use JSON bodies are also vulnerable to CSRF if they rely on cookies for authentication. Require a custom header or token.

5. **Ignoring the Origin header.** The Origin header is the most reliable CSRF signal. A request from `evil.com` with `Origin: https://evil.com` is clearly a CSRF attack.

## Key Takeaway

CSRF protection requires multiple layers because no single mechanism is foolproof. Anti-CSRF tokens provide cryptographic verification. SameSite cookies leverage the browser's built-in security. Origin/Referer validation provides an additional signal. WAF rules enforce all of these at the edge. The goal is not to build one perfect defense but to create a system where the failure of any single layer does not compromise security.
