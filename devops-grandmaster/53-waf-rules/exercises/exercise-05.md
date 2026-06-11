# Exercise 05: Comprehensive WAF Policy Design

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Design a complete, production-grade WAF policy that combines SQL injection, XSS, and CSRF protection into a cohesive rule set. Implement anomaly scoring, logging, rate limiting, and bot detection. Deploy the policy using AWS WAF or a compatible ModSecurity/Coraza configuration.

## Scenario

You are deploying a WAF for a financial services application that handles sensitive user data and payment transactions. The application has a public API, a web frontend, and an admin portal. The WAF must protect all three while maintaining low latency and minimal false positives.

---

### Part A: Design the Rule Architecture

Design the overall WAF rule architecture. Instead of blocking on a single rule match, use anomaly scoring where each rule increments a score and the request is blocked when the score exceeds a threshold.

Draw the rule architecture:

```
REQUEST
  |
  +-- Phase 1: Request Validation
  |     - Method enforcement
  |     - Content-Type validation
  |     - Request size limits
  |
  +-- Phase 2: Attack Detection (anomaly scoring)
  |     - SQLi rules (score +5)
  |     - XSS rules (score +5)
  |     - RCE rules (score +5)
  |     - Data leakage rules (score +5)
  |     - Protocol violation rules (score +3)
  |
  +-- Phase 3: Reputation and Rate Limiting
  |     - IP reputation check
  |     - Rate limit per IP
  |     - Rate limit per session
  |
  +-- Phase 4: Scoring and Decision
  |     - If score >= threshold: BLOCK
  |     - If score >= warning: LOG + CHALLENGE
  |     - Else: ALLOW
  |
  +-- Phase 5: Response Inspection
        - Check response for data leakage
        - Add security headers
```

Write the anomaly scoring configuration:

```apacheconf
# Anomaly Scoring Configuration
# Each rule increments the anomaly score
# Blocking threshold determines when to block

# TODO: Define the scoring thresholds
SecAction "id:5000,phase:1,pass,\
  nolog,\
  setvar:tx.anomaly_score=0,\
  setvar:tx.blocking_threshold=5,\
  setvar:tx.warning_threshold=3"

# TODO: Write the scoring enforcement rule (phase 5)
# Block if anomaly_score >= blocking_threshold
# Log warning if anomaly_score >= warning_threshold
```

<details><summary>Hint</summary>

Anomaly scoring avoids false positives by requiring multiple rules to fire before blocking. A single SQL keyword might be a false positive, but a SQL keyword plus a comment sequence plus a UNION keyword is almost certainly an attack. Use `setvar:tx.anomaly_score=+5` in each detection rule. In phase 5, use `SecRule TX:ANOMALY_SCORE "@ge %{tx.blocking_threshold}" "block"`.

</details>

---

### Part B: Implement Per-Endpoint Rule Sets

Different endpoints need different security postures. Write rules that apply different policies based on the request path.

| Endpoint Category | Path Pattern | Security Level | Special Rules |
|-------------------|-------------|---------------|---------------|
| Public API | `/api/v1/*` | High | Strict input validation, JSON body parsing |
| Admin Portal | `/admin/*` | Maximum | IP allowlist, MFA verification header, strict WAF |
| Static Assets | `/static/*`, `*.js`, `*.css` | Low | Minimal rules, high rate limit |
| Health Check | `/health`, `/ready` | None | No WAF rules, no logging |
| Login/Auth | `/api/auth/*` | Maximum | Strict rate limit, credential stuffing detection |

Write the path-based rule configuration:

```apacheconf
# Health check bypass
SecRule REQUEST_URI "@beginsWith /health" \
  "id:5010,phase:1,pass,nolog,ctl:ruleEngine=Off"

# Admin portal IP allowlist
# TODO: Only allow admin access from trusted IPs

# Login endpoint rate limiting
# TODO: Limit to 5 attempts per minute per IP

# API endpoint JSON validation
# TODO: Validate Content-Type is application/json for POST/PUT

# Static asset optimization
# TODO: Skip body inspection for static assets
```

<details><summary>Hint</summary>

Use `ctl:ruleEngine=Off` to completely disable WAF for health checks. For IP allowlisting, use `@ipMatch` or `@ipMatchFromFile`. For rate limiting, use `setvar:ip.login_counter=+1,expirevar:ip.login_counter=60` and then check `ip.login_counter @gt 5`. For JSON validation, check `REQUEST_HEADERS:Content-Type @beginsWith application/json`.

</details>

---

### Part C: Implement Bot Detection

Write rules that detect and handle automated traffic (bots, scrapers, credential stuffing tools):

1. **User-Agent analysis** -- detect known bot/attack tool signatures
2. **Behavioral analysis** -- detect requests that are too fast or too regular for human users
3. **Header fingerprinting** -- detect missing or unusual headers that indicate automated tools
4. **JavaScript challenge** -- for suspicious but not clearly malicious requests

```apacheconf
# Known attack tool User-Agents
SecRule REQUEST_HEADERS:User-Agent "@pmFromFile bot-user-agents.txt" \
  "id:5020,phase:1,block,msg:'Bot: Known attack tool detected',severity:HIGH"

# Missing standard browser headers (likely automated)
# TODO: Detect requests without Accept-Language, Accept-Encoding

# Request rate anomaly
# TODO: Track requests per IP and flag if > 100 per minute

# Header order fingerprinting
# TODO: Some WAFs can detect header ordering that differs from standard browsers
```

Write the bot handling strategy:

| Bot Type | Detection Method | Response | Reasoning |
|----------|-----------------|----------|-----------|
| Known attack tool | User-Agent match | Block immediately | No legitimate reason |
| Credential stuffing | Rate + login endpoint | Block + CAPTCHA | Protect authentication |
| Web scraper | Rate + pattern | Throttle (429) | May be legitimate API user |
| Search engine bot | User-Agent + verify | Allow with monitoring | SEO benefit |
| Unknown automation | Behavioral analysis | JavaScript challenge | Differentiate humans from bots |

<details><summary>Hint</summary>

Not all bots are bad. Search engine crawlers drive traffic. API clients are legitimate automation. The key is distinguishing malicious automation from legitimate automation. Use a tiered response: block known attack tools, challenge suspicious automation, allow verified bots and API clients.

</details>

---

### Part D: Implement Data Leakage Prevention

Write response inspection rules that prevent sensitive data from being leaked in responses:

```apacheconf
# Block credit card numbers in responses (PCI DSS)
# TODO: Detect 16-digit card number patterns in response body

# Block SSN patterns in responses
# TODO: Detect 9-digit SSN patterns (XXX-XX-XXXX)

# Block API keys in responses
# TODO: Detect common API key formats (sk-*, AKIA*, etc.)

# Block internal IP addresses in responses
# TODO: Detect RFC 1918 addresses in response body

# Block stack traces in responses
# TODO: Detect Java/Python/Node.js stack trace patterns
```

<details><summary>Hint</summary>

Use `SecRule RESPONSE_BODY` with phase 4 (response inspection). For credit cards, use `@rx \b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b` to match card number formats with optional separators. For SSNs, use `@rx \b\d{3}-\d{2}-\d{4}\b`. Be careful with false positives -- a page discussing credit card formatting might legitimately contain these patterns. Consider using `sanitizeMatchedBytes` to redact rather than block.

</details>

---

### Part E: Create the AWS WAF / Terraform Configuration

Translate your rule architecture into an AWS WAF v2 Terraform configuration:

```hcl
# waf.tf - AWS WAF Web ACL for the financial application

resource "aws_wafv2_web_acl" "main" {
  name        = "finance-app-waf"
  description = "WAF for financial services application"
  scope       = "REGIONAL"

  default_action {
    allow {}
  }

  # TODO: Add rule groups
  # 1. AWS Managed Rules (SQLi, XSS, Common)
  # 2. Rate limiting rules
  # 3. Bot control rules
  # 4. Custom anomaly scoring rules
  # 5. Geo-restriction rules

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "FinanceAppWAF"
    sampled_requests_enabled   = true
  }
}

# TODO: Define rate limit rule
# TODO: Define IP reputation rule
# TODO: Define custom rule group for anomaly scoring
# TODO: Associate with ALB/CloudFront
```

Complete the Terraform configuration with:
1. AWS managed rule groups for baseline protection
2. Custom rate limiting rules
3. IP reputation-based blocking
4. Request size limits
5. Logging configuration to S3/CloudWatch

<details><summary>Hint</summary>

Use `aws_managed_rule_group` references for `AWSManagedRulesCommonRuleSet`, `AWSManagedRulesSQLiRuleSet`, and `AWSManagedRulesKnownBadInputsRuleSet`. For rate limiting, use `aws_wafv2_rule_group` with a `rate_based_statement`. For logging, use `aws_wafv2_web_acl_logging_configuration`. Associate the WAF with an ALB using `aws_wafv2_web_acl_association`.

</details>

## Success Criteria

- [ ] Anomaly scoring architecture with configurable thresholds
- [ ] Per-endpoint rule sets with appropriate security levels
- [ ] Bot detection with tiered responses (block, challenge, throttle, allow)
- [ ] Data leakage prevention rules for PCI DSS compliance
- [ ] Terraform/AWS WAF configuration that is deployable
- [ ] Health check and static asset endpoints bypass unnecessary rules
- [ ] Admin portal has additional IP allowlist protection
- [ ] Login endpoints have strict rate limiting
- [ ] Can explain the trade-offs between security, false positive rate, and latency

## What You Should Understand After This Exercise

- Anomaly scoring is superior to single-rule blocking for reducing false positives
- Different endpoints require different security postures -- one-size-fits-all rules are either too strict or too loose
- Bot detection requires balancing security with usability (API clients, search engines)
- Data leakage prevention is a PCI DSS requirement and must inspect responses, not just requests
- WAF configuration should be managed as code (Terraform) for version control and auditability
- A WAF is one layer in defense in depth -- it does not replace application-level security
