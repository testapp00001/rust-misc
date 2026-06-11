# Exercise 03: XSS Protection Rules

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Design WAF rules that detect and block Cross-Site Scripting (XSS) attacks across all three types: reflected, stored, and DOM-based. Build rules that handle encoding evasion, mutation XSS, and context-dependent payloads.

## Scenario

Your application is a social platform with user profiles, comments, search results, and a rich text editor. Users can post content that other users view. The development team has implemented some output encoding but it is inconsistent across the codebase. You need WAF rules as a defense-in-depth measure.

---

### Part A: Understand XSS Contexts

XSS payloads are injected into different HTML contexts. The detection strategy depends on where the payload appears.

For each context below, write an example XSS payload and explain why it is effective:

| HTML Context | Example Code | XSS Payload | Why It Works |
|-------------|-------------|-------------|-------------|
| HTML body | `<div>{user_input}</div>` | ? | ? |
| HTML attribute | `<input value="{user_input}">` | ? | ? |
| JavaScript variable | `var name = "{user_input}";` | ? | ? |
| URL parameter | `<a href="{user_input}">` | ? | ? |
| CSS value | `<div style="color:{user_input}">` | ? | ? |

<details><summary>Hint</summary>

In the HTML body, you need to close the tag or inject a new one: `</div><script>alert(1)</script>`. In an attribute, you need to break out of the attribute: `" onmouseover="alert(1)`. In JavaScript, you need to break out of the string: `"; alert(1);//`. In a URL, you can use the `javascript:` protocol. In CSS, you can use `expression()` (IE) or break out with `</style>`.

</details>

---

### Part B: Write XSS Detection Rules

Write WAF rules that detect XSS payloads across different contexts. Cover the following patterns:

**Rule 1: Script tag injection**

```apacheconf
# Block <script> tags in any parameter
SecRule ARGS|ARGS_NAMES|REQUEST_URI "@rx (?i)<script[\s>]" \
  "id:3001,phase:2,block,msg:'XSS: Script tag injection',severity:HIGH"
```

**Task:** Write rules for:

1. **Event handler injection** -- detect `onerror`, `onload`, `onmouseover`, `onclick`, `onfocus`, and other event handlers in user input
2. **HTML tag injection** -- detect `<iframe>`, `<object>`, `<embed>`, `<svg>`, `<img>`, `<body>`, `<input>`, `<details>`, `<marquee>` tags
3. **JavaScript protocol** -- detect `javascript:` and `data:` URIs in href/src attributes
4. **Encoded payloads** -- detect HTML entities (`&#`), Unicode escapes (`\u00`), and URL-encoded script tags
5. **Mutation XSS** -- detect patterns that become dangerous after browser HTML parsing (e.g., `<noscript><img title="</noscript><img src=x onerror=alert(1)>">`)

Write each rule:

```apacheconf
# Rule 2: Event handler injection
# TODO

# Rule 3: Dangerous HTML tags
# TODO

# Rule 4: JavaScript/data URI protocol
# TODO

# Rule 5: Encoded XSS payloads
# TODO

# Rule 6: Mutation XSS patterns
# TODO
```

<details><summary>Hint 1: Event handlers</summary>

Event handlers follow the pattern `on[eventname]="..."`. Use a regex that matches `on\w+\s*=` to catch all event handlers, not just the common ones. New event handlers are added to HTML regularly, so a broad pattern is more future-proof than an allowlist.

</details>

<details><summary>Hint 2: Encoded payloads</summary>

Attackers encode payloads to bypass pattern matching. URL encoding: `%3Cscript%3E`. HTML entities: `&#60;script&#62;`. Unicode: `<script>`. Double encoding: `%253C` (encode the `%` itself). Your WAF must decode input at multiple levels before matching patterns.

</details>

---

### Part C: Implement Context-Aware Response Headers

WAF rules block attacks, but the application should also set HTTP response headers that provide defense in depth. Write the HTTP security headers that complement WAF rules for XSS prevention.

For each header, write the value and explain what it does:

| Header | Value | What It Does |
|--------|-------|-------------|
| Content-Security-Policy | ? | ? |
| X-Content-Type-Options | ? | ? |
| X-XSS-Protection | ? | ? |
| X-Frame-Options | ? | ? |
| Referrer-Policy | ? | ? |

<details><summary>Hint</summary>

`Content-Security-Policy: default-src 'self'; script-src 'self'` prevents inline scripts and scripts from external domains. `X-Content-Type-Options: nosniff` prevents MIME-type sniffing. `X-XSS-Protection: 1; mode=block` enables the browser's built-in XSS filter (deprecated but still useful for older browsers). CSP is the most powerful XSS defense -- it can prevent script execution even if the WAF is bypassed.

</details>

---

### Part D: Handle Evasion Techniques

Attackers use creative techniques to bypass XSS filters. For each evasion below, explain how it bypasses a naive filter and how your rules should handle it.

| # | Evasion Technique | Example | How It Bypasses Naive Filter | How to Detect It |
|---|------------------|---------|----------------------------|-----------------|
| 1 | Mixed case | `<ScRiPt>` | Case-sensitive matching | ? |
| 2 | Null bytes | `<scr%00ipt>` | Filter stops at null byte | ? |
| 3 | Whitespace injection | `<script /src=...>` | Strict tag matching | ? |
| 4 | Tag fragmentation | `<scr ipt>` | Token-based matching | ? |
| 5 | Encoding layers | `&#x3C;script>` | Single-level decoding | ? |
| 6 | SVG-based | `<svg/onload=alert(1)>` | Only checking `<script>` tags | ? |
| 7 | Template injection | `{{constructor.constructor('alert(1)')()}}` | HTML-focused detection | ? |

<details><summary>Hint</summary>

Defense against evasion requires: (1) case-insensitive matching with `(?i)` flag, (2) removing null bytes and whitespace before matching, (3) normalizing at multiple encoding levels, (4) using broad tag/attribute patterns rather than exact matches, (5) covering all HTML contexts, not just `<script>` tags, (6) including template syntax patterns for frameworks like Angular and Vue.

</details>

---

### Part E: Design a Testing Strategy

Write a set of XSS test payloads that you would use to validate your WAF rules. Include at least 3 payloads per XSS type (reflected, stored, DOM-based) that test different evasion techniques.

```yaml
# xss-test-payloads.yaml
reflected_xss:
  - name: "Basic script tag"
    payload: ???
    should_block: true
  - name: "Event handler in attribute"
    payload: ???
    should_block: true
  - name: "Legitimate search term"
    payload: "best restaurants near me"
    should_block: false

stored_xss:
  # TODO: Add 3 test payloads

dom_based_xss:
  # TODO: Add 3 test payloads

evasion_tests:
  # TODO: Add 3 evasion test payloads
```

<details><summary>Hint</summary>

Reflected XSS test payloads: `<script>alert(1)</script>`, `"><img src=x onerror=alert(1)>`, `javascript:alert(1)`. Stored XSS: payloads that would be saved to a database and rendered on a different page. DOM-based: payloads that exploit client-side JavaScript like `#<script>alert(1)</script>` in the URL fragment.

</details>

## Success Criteria

- [ ] At least 5 distinct XSS patterns covered by WAF rules
- [ ] Rules handle at least 4 different evasion techniques
- [ ] Context-aware rules differentiate between HTML body, attributes, JavaScript, and URL contexts
- [ ] Security headers are correctly configured for defense in depth
- [ ] Test payloads cover reflected, stored, and DOM-based XSS
- [ ] Can explain why CSP is more powerful than WAF rules alone for XSS prevention

## What You Should Understand After This Exercise

- XSS has three variants (reflected, stored, DOM-based) that require different detection strategies
- HTML context determines the payload structure -- rules must account for body, attribute, JS, and URL contexts
- Evasion techniques exploit gaps between how filters parse input and how browsers parse HTML
- Content-Security-Policy is the strongest XSS defense because it prevents script execution regardless of injection
- WAF rules are defense in depth, not the primary XSS prevention mechanism -- output encoding is the primary defense
