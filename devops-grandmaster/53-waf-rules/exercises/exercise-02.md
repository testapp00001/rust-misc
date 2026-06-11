# Exercise 02: SQL Injection WAF Rules

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Design WAF rules that detect and block SQL injection attacks at the edge before they reach your application. Build rules that cover common SQLi patterns while minimizing false positives on legitimate traffic.

## Scenario

Your application has a search API, a user login endpoint, and a product filter endpoint. All three construct SQL queries from user input. You need WAF rules to protect these endpoints while the development team migrates to parameterized queries.

---

### Part A: Identify the Attack Surface

Your application has these endpoints that accept user input:

```
GET  /api/search?q={user_input}
GET  /api/products?category={user_input}&price_min={user_input}&sort={user_input}
POST /api/login         (body: {"username": "{input}", "password": "{input}"})
GET  /api/users/{id}
POST /api/orders        (body: {"product_id": "{input}", "quantity": {input}})
```

For each endpoint, identify:
1. Which parameters are vulnerable to SQL injection
2. What type of SQL injection is most likely (union-based, boolean-blind, time-based, error-based)
3. What legitimate input looks like vs. malicious input

| Endpoint | Vulnerable Params | Injection Type | Legitimate Input Example | Malicious Input Example |
|----------|-------------------|----------------|-------------------------|------------------------|
| /api/search | ? | ? | ? | ? |
| /api/products | ? | ? | ? | ? |
| /api/login | ? | ? | ? | ? |
| /api/users/{id} | ? | ? | ? | ? |
| /api/orders | ? | ? | ? | ? |

<details><summary>Hint</summary>

String parameters (search, category, username) are most vulnerable because attackers can break out of string context with a single quote. Numeric parameters (id, quantity, price_min) can be vulnerable if the application does not validate that the input is actually a number. URL path parameters (/api/users/{id}) can also be vulnerable if the routing layer passes them directly to queries.

</details>

---

### Part B: Write Core SQLi Detection Rules

Write WAF rules in ModSecurity CRS syntax that detect SQL injection in request parameters. Start with the most common patterns.

**Rule 1: Detect SQL keywords in query parameters**

```apacheconf
# Block common SQL injection keywords in query string
SecRule ARGS_GET "@rx (?i)(\b(union|select|insert|update|delete|drop|alter|exec|execute)\b)" \
  "id:2001,phase:1,block,msg:'SQL Injection: SQL keyword in query parameter',\
   severity:CRITICAL,logdata:'%{MATCHED_VAR}'"
```

**Task:** Write rules for the following SQL injection patterns:

1. **SQL comment sequences** -- detect `--`, `/*`, `#` in user input
2. **String breakout** -- detect single quotes (`'`) followed by SQL operators
3. **UNION SELECT** -- detect the specific pattern of UNION-based injection
4. **Tautologies** -- detect always-true conditions like `OR 1=1`, `OR 'a'='a'`
5. **Time-based blind** -- detect `SLEEP()`, `WAITFOR`, `BENCHMARK()`, `pg_sleep()`

Write each rule below:

```apacheconf
# Rule 2: SQL comment sequences
# TODO

# Rule 3: String breakout with SQL operators
# TODO

# Rule 4: UNION SELECT pattern
# TODO

# Rule 5: Tautology detection
# TODO

# Rule 6: Time-based blind SQLi
# TODO
```

<details><summary>Hint</summary>

Use `@rx` for regex matching. The `(?i)` flag makes the pattern case-insensitive. For string breakout, look for a single quote followed by whitespace and then OR/AND/UNION. For tautologies, match patterns like `OR 1=1`, `OR '1'='1'`, `OR 1=1--`. For time-based, match function names like `sleep`, `benchmark`, `waitfor delay`, `pg_sleep`.

</details>

---

### Part C: Handle False Positives

Your rules from Part B will generate false positives. For each scenario below, explain why it is a false positive and how to fix the rule.

| Scenario | Why It Is a False Positive | How to Fix |
|----------|---------------------------|------------|
| User searches for "selection criteria" | The word "select" is in a legitimate search term | ? |
| User submits JSON with `"comment": "I dropped my phone"` | The word "drop" is in a natural sentence | ? |
| Product category is "women's clothing" | The single quote in "women's" triggers string breakout rule | ? |
| User searches for "C# programming" | The `#` character is part of a programming language name | ? |
| Sort parameter is "price-desc" | The `--` in "price-desc" looks like a SQL comment | ? |

<details><summary>Hint 1: Context-aware rules</summary>

Instead of matching keywords anywhere in the input, match them in specific contexts. For example, match `SELECT` only when preceded by a quote or semicolon, not when it appears as a standalone word. Use regex lookaheads and lookbehinds to add context.

</details>

<details><summary>Hint 2: Parameter-specific rules</summary>

Apply different rules to different parameters. The search parameter (`q`) is a free-text field and needs looser rules. The `id` parameter should only contain numbers. The `sort` parameter should match an allowlist of known values. Apply strict rules to structured parameters and lenient rules to free-text fields.

</details>

---

### Part D: Implement Parameter-Specific Rules

Write rules that apply different levels of strictness based on the parameter type:

1. **Numeric-only parameters** (id, quantity, price_min): Block anything that is not a number
2. **Allowlisted parameters** (sort, category): Block anything not in a known-good list
3. **Free-text parameters** (q, username, comment): Apply pattern-based SQLi detection with higher thresholds

```apacheconf
# Numeric parameter validation
# TODO: Block non-numeric values for id, quantity, price_min

# Allowlist validation
# TODO: Block invalid sort values (allow: price-asc, price-desc, name-asc, name-desc, newest)

# Free-text SQLi detection with reduced false positives
# TODO: Apply SQLi patterns but with context-aware matching
```

<details><summary>Hint</summary>

For numeric parameters, use `@rx ^[0-9]+$` to allow only digits. For allowlist, use `@within` or `@rx` with alternation. For free-text, combine multiple signals: require that SQL keywords appear near special characters (quotes, semicolons, comments) rather than matching keywords in isolation. Consider using anomaly scoring (increment a score per rule match) rather than blocking on a single rule.

</details>

---

### Part E: Test Your Rules

For each test case below, determine if your rules would block or allow the request, and whether that is the correct behavior.

| Test Case | Request | Expected Action | Your Rule Action | Correct? |
|-----------|---------|-----------------|-----------------|----------|
| 1 | `GET /api/search?q=union+select` | Block | ? | ? |
| 2 | `GET /api/search?q=selection+criteria` | Allow | ? | ? |
| 3 | `GET /api/users/1' OR '1'='1` | Block | ? | ? |
| 4 | `GET /api/users/42` | Allow | ? | ? |
| 5 | `GET /api/products?sort=price-asc` | Allow | ? | ? |
| 6 | `GET /api/products?sort=price;DROP TABLE--` | Block | ? | ? |
| 7 | `POST /api/login {"username":"admin'--","password":"x"}` | Block | ? | ? |
| 8 | `POST /api/login {"username":"admin","password":"p@ssw0rd!"}` | Allow | ? | ? |
| 9 | `GET /api/search?q=C%23+programming` | Allow | ? | ? |
| 10 | `GET /api/search?q='+UNION+SELECT+*+FROM+users--` | Block | ? | ? |

<details><summary>Hint</summary>

A good WAF rule set should block 1, 3, 6, 7, and 10 (true positives) while allowing 2, 4, 5, 8, and 9 (true negatives). If any legitimate request is blocked, the rule needs tuning. If any attack is allowed, the rule needs strengthening.

</details>

## Success Criteria

- [ ] At least 5 distinct SQL injection patterns covered by rules
- [ ] Rules use ModSecurity CRS syntax with correct `id`, `phase`, and `severity`
- [ ] False positive scenarios are addressed with specific fixes
- [ ] Parameter-specific rules differentiate between numeric, allowlisted, and free-text inputs
- [ ] At least 8 of 10 test cases produce the correct block/allow decision
- [ ] Can explain the trade-off between security (blocking attacks) and usability (allowing legitimate traffic)

## What You Should Understand After This Exercise

- SQL injection has many variants (union, boolean-blind, time-based, error-based) and rules must cover them all
- Pattern-based detection is necessary but insufficient -- context-aware rules reduce false positives
- Parameter validation (numeric checks, allowlists) is more reliable than payload pattern matching
- Anomaly scoring (multiple weak signals combining to a threshold) is better than single-rule blocking
- WAF rules are a stopgap while the application migrates to parameterized queries, not a permanent solution
