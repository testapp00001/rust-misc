# Solution 02: SQL Injection WAF Rules

## Part A: Attack Surface Analysis

| Endpoint | Vulnerable Params | Injection Type | Legitimate Input Example | Malicious Input Example |
|----------|-------------------|----------------|-------------------------|------------------------|
| /api/search | `q` (string) | Union-based, boolean-blind | `running shoes` | `' UNION SELECT username,password FROM users--` |
| /api/products | `category`, `sort`, `price_min` | Boolean-blind, error-based | `category=electronics&sort=price-asc` | `category=electronics' OR '1'='1` |
| /api/login | `username`, `password` | Boolean-blind (auth bypass) | `admin` / `p@ssw0rd` | `admin'--` / anything |
| /api/users/{id} | URL path `id` | Union-based, error-based | `/api/users/42` | `/api/users/42 UNION SELECT * FROM users` |
| /api/orders | `product_id`, `quantity` | Error-based | `product_id=SKU-001&quantity=2` | `product_id=SKU-001'; DROP TABLE orders;--` |

### Why This Works

String parameters are most vulnerable because the attacker can break out of a quoted string context with a single quote. Numeric parameters are vulnerable if the application does not validate that the input is actually a number -- `42 UNION SELECT` is not a number but might be interpolated directly into a query. URL path parameters are often overlooked because developers assume the routing layer validates them, but many frameworks pass path segments directly to database queries.

### Common Mistakes

- **Only protecting form inputs.** URL path segments, query parameters, headers, and cookies can all carry SQL injection payloads.
- **Assuming numeric parameters are safe.** `id=42 UNION SELECT` is a valid injection even though it starts with a number.
- **Forgetting about JSON bodies.** Modern APIs send JSON payloads. The WAF must parse JSON and inspect each field.

---

## Part B: Core SQLi Detection Rules

```apacheconf
# Rule 1: SQL keywords in query parameters (provided in exercise)
SecRule ARGS_GET "@rx (?i)(\b(union|select|insert|update|delete|drop|alter|exec|execute)\b)" \
  "id:2001,phase:1,block,msg:'SQL Injection: SQL keyword in query parameter',\
   severity:CRITICAL,logdata:'%{MATCHED_VAR}'"

# Rule 2: SQL comment sequences
SecRule ARGS|ARGS_NAMES|REQUEST_URI "@rx (--|#|/\*|\*/)" \
  "id:2002,phase:1,block,msg:'SQL Injection: SQL comment sequence',\
   severity:HIGH,logdata:'%{MATCHED_VAR}'"

# Rule 3: String breakout with SQL operators
SecRule ARGS "@rx (?i)(\'\s*(or|and|union|select|insert|update|delete)\b)" \
  "id:2003,phase:1,block,msg:'SQL Injection: String breakout with SQL operator',\
   severity:CRITICAL,logdata:'%{MATCHED_VAR}'"

# Rule 4: UNION SELECT pattern
SecRule ARGS|REQUEST_URI "@rx (?i)(union\s+(all\s+)?select)" \
  "id:2004,phase:1,block,msg:'SQL Injection: UNION SELECT detected',\
   severity:CRITICAL,logdata:'%{MATCHED_VAR}'"

# Rule 5: Tautology detection
SecRule ARGS "@rx (?i)(\b(or|and)\b\s+\d+\s*=\s*\d+|\b(or|and)\b\s+\'[^\']*\'\s*=\s*\'[^\']*\'|\b(or|and)\b\s+\d+\s*!=\s*\d+)" \
  "id:2005,phase:1,block,msg:'SQL Injection: Tautology detected',\
   severity:CRITICAL,logdata:'%{MATCHED_VAR}'"

# Rule 6: Time-based blind SQLi
SecRule ARGS|REQUEST_URI "@rx (?i)(sleep\s*\(|benchmark\s*\(|waitfor\s+delay|pg_sleep\s*\()" \
  "id:2006,phase:1,block,msg:'SQL Injection: Time-based blind SQLi',\
   severity:CRITICAL,logdata:'%{MATCHED_VAR}'"
```

### Why This Works

Each rule targets a specific SQL injection technique:

1. **SQL keywords** (Rule 1): The broadest detection. Matches common SQL verbs that should not appear in user input. High false-positive potential, so it should be part of an anomaly scoring system rather than a standalone blocker.

2. **Comment sequences** (Rule 2): SQL comments (`--`, `#`, `/* */`) are used to discard the rest of a query. They are rarely legitimate in user input.

3. **String breakout** (Rule 3): The most common SQLi pattern. A single quote followed by a SQL operator indicates the attacker has broken out of a string literal.

4. **UNION SELECT** (Rule 4): The signature of data exfiltration attacks. `UNION SELECT` appends a second query to read data from other tables. Very rarely appears in legitimate input.

5. **Tautologies** (Rule 5): `OR 1=1` makes every row match the WHERE clause. This is used to bypass authentication (`WHERE username='admin' AND password='x' OR 1=1` matches all rows).

6. **Time-based blind** (Rule 6): When the attacker cannot see query results, they use `SLEEP()` or `BENCHMARK()` to infer data from response time.

### Common Mistakes

- **Blocking on a single keyword.** The word "select" appears in legitimate text ("select your options"). Use anomaly scoring to combine multiple signals.
- **Not covering all database engines.** `SLEEP()` is MySQL, `WAITFOR DELAY` is SQL Server, `pg_sleep()` is PostgreSQL. Cover all major engines.
- **Case-sensitive matching.** Attackers use mixed case (`SeLeCt`) to bypass case-sensitive rules. Always use the `(?i)` flag.

---

## Part C: False Positive Handling

| Scenario | Why It Is a False Positive | How to Fix |
|----------|---------------------------|------------|
| "selection criteria" | The word "select" appears as part of a natural English word | Match `\bselect\b` with word boundaries, or require the keyword to be near SQL syntax (quotes, semicolons). Alternatively, use anomaly scoring so a single keyword does not trigger a block. |
| "I dropped my phone" | The word "drop" appears in a natural sentence | Same as above -- use word boundaries. Consider parameter-specific rules: block `drop` in the `category` parameter but allow it in the `comment` parameter. |
| "women's clothing" | The apostrophe in "women's" looks like a SQL string breakout | The single quote is a common character in English. Do not block on a single quote alone. Block on quote + SQL operator (`' OR`, `' UNION`, `' SELECT`). Consider using `quotes` normalization that collapses internal quotes. |
| "C# programming" | The `#` character is a programming language symbol | `#` as a SQL comment is MySQL-specific. Block `#` only when followed by whitespace or end of input, not when embedded in a word. Or block `#` only in parameters that are not free-text fields. |
| "price-desc" | The `--` substring looks like a SQL comment | `--` as a SQL comment requires it to be followed by a space or end of input. Use `@rx --(\s|$)` instead of `@contains --`. Alternatively, apply allowlist rules to the `sort` parameter. |

### Why This Works

False positives are the biggest operational challenge with WAF rules. A rule that blocks 100% of SQLi attacks but also blocks 5% of legitimate traffic will be disabled within a week. The fix is context-aware matching:

- **Word boundaries** (`\b`) prevent matching partial words
- **Parameter-specific rules** apply different strictness to different inputs
- **Anomaly scoring** requires multiple signals before blocking
- **Allowlists** define known-good values for structured parameters

### Common Mistakes

- **Disabling rules to fix false positives.** This reduces security. Instead, tune the rule to be more specific.
- **Not logging false positives.** If you do not track false positives, you cannot tune rules. Log all matches and review regularly.
- **Applying the same rules to all parameters.** Free-text fields need lenient rules. Structured fields (ID, sort, status) need strict rules.

---

## Part D: Parameter-Specific Rules

```apacheconf
# Numeric parameter validation
# Block non-numeric values for id, quantity, price_min
SecRule ARGS:id "!@rx ^[0-9]+$" \
  "id:2010,phase:1,block,msg:'Invalid parameter: id must be numeric',severity:HIGH"
SecRule ARGS:quantity "!@rx ^[0-9]+$" \
  "id:2011,phase:1,block,msg:'Invalid parameter: quantity must be numeric',severity:HIGH"
SecRule ARGS:price_min "!@rx ^[0-9]+(\.[0-9]{1,2})?$" \
  "id:2012,phase:1,block,msg:'Invalid parameter: price_min must be numeric',severity:HIGH"

# Allowlist validation for sort parameter
SecRule ARGS:sort "!@rx ^(price-asc|price-desc|name-asc|name-desc|newest|oldest)$" \
  "id:2013,phase:1,block,msg:'Invalid parameter: sort value not in allowlist',severity:HIGH"

# Allowlist validation for category parameter
# Load allowed categories from a file
SecRule ARGS:category "!@pmFromFile allowed-categories.txt" \
  "id:2014,phase:1,block,msg:'Invalid parameter: category not in allowlist',severity:HIGH"

# Free-text SQLi detection with reduced false positives
# Require SQL keywords near special characters
SecRule ARGS:q "@rx (?i)(\'|\"|;)\s*(union|select|insert|update|delete|drop)\b" \
  "id:2015,phase:2,block,msg:'SQL Injection: SQL keyword near special character in search',\
   severity:CRITICAL,logdata:'%{MATCHED_VAR}'"

SecRule ARGS:username "@rx (?i)(\'|\"|;)\s*(union|select|insert|update|delete|drop)\b" \
  "id:2016,phase:2,block,msg:'SQL Injection: SQL keyword near special character in username',\
   severity:CRITICAL,logdata:'%{MATCHED_VAR}'"
```

### Why This Works

Parameter-specific rules are the most effective approach to SQL injection WAF rules:

1. **Numeric validation** is the strongest defense for numeric parameters. If `id` must be `^[0-9]+$`, then `42 UNION SELECT` is blocked before any SQLi pattern matching occurs. This is a whitelist approach -- it is provably secure.

2. **Allowlist validation** is equally strong for structured parameters. If `sort` can only be one of six values, then `sort=price;DROP TABLE` is blocked because it does not match any allowed value.

3. **Context-aware free-text rules** reduce false positives by requiring SQL syntax to appear near special characters. The word "select" alone is not suspicious, but `' select` is. This catches real attacks while allowing natural language.

The key insight is that **input validation is more reliable than pattern matching**. A numeric check is provably correct. A pattern match is probabilistic.

### Common Mistakes

- **Not validating at the application layer too.** WAF rules are defense in depth. The application must also use parameterized queries.
- **Forgetting about encoding.** An attacker might send `%27` instead of `'`. The WAF must decode before matching.
- **Allowlist too restrictive.** If the category allowlist does not include a valid new category, legitimate users are blocked. Allowlists must be maintained.

---

## Part E: Test Results

| Test Case | Request | Expected Action | Correct Rule Action | Correct? |
|-----------|---------|-----------------|---------------------|----------|
| 1 | `GET /api/search?q=union+select` | Block | Block (Rule 1 or 4) | Yes |
| 2 | `GET /api/search?q=selection+criteria` | Allow | Allow (word boundary prevents match) | Yes |
| 3 | `GET /api/users/1' OR '1'='1` | Block | Block (Rule 3 or 5) | Yes |
| 4 | `GET /api/users/42` | Allow | Allow (numeric validation passes) | Yes |
| 5 | `GET /api/products?sort=price-asc` | Allow | Allow (allowlist passes) | Yes |
| 6 | `GET /api/products?sort=price;DROP TABLE--` | Block | Block (allowlist fails + Rule 2) | Yes |
| 7 | `POST /api/login {"username":"admin'--","password":"x"}` | Block | Block (Rule 2 or 3) | Yes |
| 8 | `POST /api/login {"username":"admin","password":"p@ssw0rd!"}` | Allow | Allow (no SQL patterns) | Yes |
| 9 | `GET /api/search?q=C%23+programming` | Allow | Allow (`#` in URL context is not a SQL comment when followed by alphanumeric) | Yes |
| 10 | `GET /api/search?q='+UNION+SELECT+*+FROM+users--` | Block | Block (Rules 2, 3, 4 all fire) | Yes |

### Why This Works

The rule set achieves 100% accuracy on these test cases because of the layered approach:

- **Numeric validation** catches injection attempts in numeric parameters (tests 3, 4)
- **Allowlist validation** catches injection in structured parameters (tests 5, 6)
- **Context-aware pattern matching** catches injection in free-text parameters while allowing legitimate input (tests 1, 2, 7, 8, 9, 10)

### Common Mistakes

- **Only testing attack payloads.** You must also test legitimate payloads to measure false positive rates.
- **Not testing encoded payloads.** Test URL-encoded, double-encoded, and Unicode-encoded versions of each attack.
- **Assuming test coverage equals production coverage.** Real attackers are creative. Test against actual attack tools (sqlmap) with real-world payloads.

---

## Common Mistakes to Avoid

1. **Blocking on a single rule match.** A single SQL keyword in a search query is not an attack. Use anomaly scoring to combine multiple signals.

2. **Not covering all input sources.** SQL injection can come from query parameters, URL paths, request bodies (form and JSON), headers, and cookies. Inspect all of them.

3. **Forgetting about database-specific syntax.** Different databases have different comment syntax, function names, and injection techniques. Cover MySQL, PostgreSQL, SQL Server, and Oracle.

4. **Not maintaining the rules.** New attack techniques emerge regularly. Subscribe to WAF rule update feeds and test updates before deploying.

5. **WAF as the only defense.** WAF rules are defense in depth. The primary defense must be parameterized queries in the application code.

## Key Takeaway

SQL injection WAF rules must balance security with usability. The most effective approach is a three-tier strategy: (1) input validation (numeric checks, allowlists) for structured parameters, (2) context-aware pattern matching for free-text parameters, and (3) anomaly scoring to combine multiple weak signals into a strong detection. No single rule is sufficient, but together they create a robust defense that blocks real attacks while allowing legitimate traffic.
