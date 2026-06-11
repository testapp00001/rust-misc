# Solution 01: Attack Payload Recognition

## Part A: Classified Attack Payloads

| Request | Attack Type | Payload Location | What the Payload Does | Potential Damage |
|---------|------------|------------------|----------------------|-----------------|
| 1 | SQL Injection | Query parameter (`name`) | `admin'--` closes the string and comments out the rest of the query, bypassing password check | Authentication bypass, unauthorized access |
| 2 | XSS (Stored) | Request body (`comment`) | Injects `<script>` tag that sends cookies to attacker's server on page load | Session hijacking, account takeover |
| 3 | CSRF | Request body (form fields) | Cross-origin form submission exploiting user's active session | Unauthorized money transfer |
| 4 | SQL Injection (UNION) | Query parameter (`q`) | UNION SELECT appends a query that reads usernames, passwords, and SSNs from the users table | Full data exfiltration |
| 5 | XSS (Stored, Mutation) | Request body (`bio`) | `<img onerror>` executes JavaScript when image fails to load; uses Image object to exfiltrate cookies | Session hijacking, persistent XSS |
| 6 | XSS (Reflected) | Query parameter (`q`) | `<svg/onload>` triggers JavaScript execution when the SVG is rendered in the search results page | Session hijacking, redirect to malicious site |

### Why This Works

- **SQL injection** works by breaking out of a string literal with a single quote, then using SQL comment syntax (`--`) to discard the rest of the query. The `UNION SELECT` variant appends an entirely new query to read data from other tables.
- **XSS** works by injecting HTML/JavaScript into content that other users view. Stored XSS (requests 2 and 5) persists in the database and affects every user who views the page. Reflected XSS (request 6) is injected via URL parameters and only affects the victim who clicks the malicious link.
- **CSRF** works by exploiting the browser's automatic cookie handling. The victim's browser sends the request with their session cookie, making it appear legitimate to the server.

### Common Mistakes

- **Confusing XSS types.** Reflected XSS requires the victim to click a crafted link. Stored XSS affects everyone who views the page. Stored is more dangerous because it does not require social engineering.
- **Thinking CSRF needs a special payload.** CSRF exploits trust, not code injection. The payload is a normal HTTP request -- the attack is in its origin.
- **Ignoring mutation XSS.** Request 5 uses an `<img onerror>` handler inside a tag that might survive HTML sanitization if the sanitizer does not handle nested contexts correctly.

---

## Part B: Detection Strategies

| Attack Type | What to Inspect | Detection Patterns | Limitations |
|-------------|----------------|-------------------|-------------|
| SQL Injection | Query parameters, request body, URL path | Single quotes, SQL keywords (SELECT, UNION, INSERT, UPDATE, DELETE, DROP), comment sequences (`--`, `/*`, `#`), tautologies (`OR 1=1`), semicolons | Cannot detect blind SQLi that uses only alphanumeric characters (e.g., `BENCHMARK` with no special chars). False positives on natural language containing SQL keywords. |
| XSS | All user-controllable input (params, body, headers, URL) | `<script>`, `<img>`, `<svg>`, `<iframe>`, event handlers (`on*=`), `javascript:` URI, `document.cookie`, HTML entities | Cannot detect DOM-based XSS (payload never reaches the server). False positives on legitimate HTML in rich-text fields. Encoding evasion requires multi-level decoding. |
| CSRF | Request origin (Origin/Referer headers), presence of anti-CSRF token | Missing CSRF token on state-changing requests (POST/PUT/DELETE), Origin header not matching application domain, Referer from external domain | Cannot detect CSRF if the attacker has obtained a valid token (e.g., via XSS). Legitimate API clients may not send Origin/Referer headers. Privacy tools strip these headers. |

### Why This Works

Each attack exploits a different trust boundary, so detection must target the specific mechanism:
- SQL injection detection focuses on **query syntax** in user input
- XSS detection focuses on **executable content** in all input sources
- CSRF detection focuses on **request origin** and **token presence**, not payload content

### Common Mistakes

- **Using the same detection approach for all three.** SQL injection and XSS have characteristic payloads, but CSRF does not -- it requires structural validation (tokens, origin).
- **Only inspecting query parameters.** Attackers inject payloads in headers, cookies, URL paths, and request bodies. The WAF must inspect all input sources.
- **Ignoring the response.** Data leakage (SQLi results appearing in responses) can be detected by inspecting response bodies for database error messages or sensitive data patterns.

---

## Part C: Decoded Evasion Payloads

| # | Obfuscated Payload | Encoding Used | Decoded Attack | Attack Type |
|---|-------------------|---------------|----------------|-------------|
| 1 | `%27%20OR%201%3D1%20--` | URL encoding | `' OR 1=1 --` | SQL Injection |
| 2 | `<script>alert(String.fromCharCode(88,83,83))</script>` | JavaScript `String.fromCharCode()` | `<script>alert("XSS")</script>` (88=X, 83=S, 83=S) | XSS |
| 3 | `%3Csvg%20onload%3Dalert(1)%3E` | URL encoding | `<svg onload=alert(1)>` | XSS |
| 4 | `' OR 1=1#` | MySQL comment syntax | `' OR 1=1` (everything after `#` is a MySQL comment) | SQL Injection |
| 5 | `<img src="x" onerror="&#97;&#108;&#101;&#114;&#116;(1)">` | HTML entity encoding | `<img src="x" onerror="alert(1)">` (97=a, 108=l, 101=e, 114=r, 116=t) | XSS |

### Why This Works

Attackers use encoding to bypass pattern-matching WAF rules:

1. **URL encoding** (`%XX`) is the most common evasion. A filter looking for `<script>` will miss `%3Cscript%3E`. The WAF must URL-decode input before inspection.

2. **HTML entity encoding** (`&#NNN;`) represents characters as decimal codes. A filter looking for `alert` will miss `&#97;&#108;&#101;&#114;&#116;`. The WAF must HTML-decode input before inspection.

3. **JavaScript encoding** (`String.fromCharCode()`) constructs strings dynamically. A filter looking for `"XSS"` will miss `String.fromCharCode(88,83,83)`. This is harder to detect because the encoding is evaluated by the JavaScript engine, not a simple decoding step.

4. **Comment syntax variation** exploits differences between database engines. A filter blocking `--` might miss `#` (MySQL) or `/* */` (all SQL databases).

5. **Mixed encoding** combines multiple encoding layers. The payload in request 5 uses HTML entities inside an HTML attribute, which the browser decodes before executing the event handler.

### Common Mistakes

- **Single-level decoding.** Attackers double-encode payloads: `%253C` decodes to `%3C` which decodes to `<`. The WAF must decode at multiple levels.
- **Only blocking common patterns.** Attackers constantly discover new encoding tricks. A robust WAF normalizes input to a canonical form before matching.
- **Forgetting about browser parsing.** The browser parses HTML differently than a WAF. Mutation XSS exploits these differences -- the WAF sees harmless content, but the browser's parser transforms it into executable code.

---

## Common Mistakes to Avoid

1. **Relying solely on pattern matching.** Pattern matching is necessary but not sufficient. Attackers evolve their techniques faster than rule databases update. Combine WAF rules with application-level defenses (parameterized queries, output encoding, CSRF tokens).

2. **Not testing with real payloads.** Academic examples like `' OR 1=1 --` are the simplest cases. Real attacks use obfuscation, encoding, and creative bypasses. Test your WAF against actual attack tools (sqlmap, XSStrike).

3. **Ignoring false positives.** A WAF that blocks legitimate users is worse than no WAF -- users will demand it be disabled. Invest time in tuning rules to your application's traffic patterns.

4. **Treating WAF as a complete solution.** WAFs are defense in depth. They reduce risk but do not eliminate it. The application must still implement proper input validation, parameterized queries, output encoding, and CSRF tokens.

## Key Takeaway

Recognizing attack payloads is the first step in WAF rule design, but it is only the beginning. Each attack type exploits a different trust boundary, requires different detection strategies, and can be evaded with different techniques. Effective WAF rules must understand the attack mechanism, not just match patterns. The goal is not to build an impenetrable wall but to raise the cost of attack high enough that most attackers move on to easier targets.
