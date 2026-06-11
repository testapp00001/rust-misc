# Solution 05: Comprehensive WAF Policy Design

## Part A: Anomaly Scoring Architecture

```apacheconf
# Anomaly Scoring Configuration
SecAction "id:5000,phase:1,pass,\
  nolog,\
  setvar:tx.anomaly_score=0,\
  setvar:tx.blocking_threshold=5,\
  setvar:tx.warning_threshold=3,\
  setvar:tx.sqli_score=5,\
  setvar:tx.xss_score=5,\
  setvar:tx.rce_score=5,\
  setvar:tx.protocol_score=3"

# Scoring enforcement in phase 5 (response)
SecRule TX:ANOMALY_SCORE "@ge %{tx.blocking_threshold}" \
  "id:5099,phase:5,block,msg:'Request blocked: Anomaly score %{TX.ANOMALY_SCORE} exceeded threshold %{tx.blocking_threshold}',severity:CRITICAL"

SecRule TX:ANOMALY_SCORE "@ge %{tx.warning_threshold}" \
  "id:5098,phase:5,pass,log,msg:'Warning: Anomaly score %{TX.ANOMALY_SCORE} exceeded warning threshold %{tx.warning_threshold}',severity:WARNING,chain"
  SecRule TX:ANOMALY_SCORE "@lt %{tx.blocking_threshold}" ""

# Example: SQLi detection rule that increments score instead of blocking
SecRule ARGS "@rx (?i)(union\s+select)" \
  "id:5001,phase:2,pass,msg:'SQLi: UNION SELECT pattern',\
   setvar:tx.anomaly_score=+%{tx.sqli_score},logdata:'%{MATCHED_VAR}'"
```

### Why This Works

Anomaly scoring solves the false positive problem by requiring multiple rules to fire before blocking:

- A single SQL keyword in a search query increments the score by 5 but does not block.
- A SQL keyword plus a comment sequence plus a UNION pattern increments the score by 15, which exceeds the threshold and blocks.
- Legitimate traffic rarely triggers more than one or two rules, keeping the score below the threshold.

The `setvar:tx.anomaly_score=+%{tx.sqli_score}` syntax adds the rule's score to the running total. Each rule contributes independently, and the final decision is made in phase 5 after all rules have been evaluated.

### Common Mistakes

- **Setting the threshold too low.** A threshold of 2-3 will cause false positives. Start with 5 and tune based on production traffic.
- **Blocking in phase 1/2 instead of phase 5.** If you block immediately on the first rule match, you lose the benefit of anomaly scoring. All rules should increment the score; the blocking decision happens in phase 5.
- **Not logging sub-threshold matches.** Even if a request is not blocked, log the score and which rules fired. This data is essential for tuning.

---

## Part B: Per-Endpoint Rule Sets

```apacheconf
# Health check bypass
SecRule REQUEST_URI "@beginsWith /health" \
  "id:5010,phase:1,pass,nolog,ctl:ruleEngine=Off"
SecRule REQUEST_URI "@beginsWith /ready" \
  "id:5011,phase:1,pass,nolog,ctl:ruleEngine=Off"

# Admin portal IP allowlist
SecRule REQUEST_URI "@beginsWith /admin" \
  "id:5012,phase:1,chain,block,msg:'Admin: Access from unauthorized IP',severity:CRITICAL"
  SecRule REMOTE_ADDR "!@ipMatch 10.0.0.0/8" \
    "chain"
    SecRule REMOTE_ADDR "!@ipMatch 192.168.1.0/24" ""

# Admin portal requires MFA verification header
SecRule REQUEST_URI "@beginsWith /admin" \
  "id:5013,phase:1,chain,block,msg:'Admin: Missing MFA verification header',severity:CRITICAL"
  SecRule REQUEST_METHOD "@rx ^(POST|PUT|PATCH|DELETE)$" \
    "chain"
    SecRule &REQUEST_HEADERS:X-MFA-Verified "@eq 0" ""

# Login endpoint rate limiting
SecRule REQUEST_URI "@beginsWith /api/auth/login" \
  "id:5014,phase:1,pass,nolog,setvar:ip.login_counter=+1,expirevar:ip.login_counter=60"
SecRule REQUEST_URI "@beginsWith /api/auth/login" \
  "id:5015,phase:1,chain,block,msg:'Auth: Login rate limit exceeded',severity:HIGH"
  SecRule IP:LOGIN_COUNTER "@gt 5" ""

# API endpoint JSON validation
SecRule REQUEST_URI "@beginsWith /api/" \
  "id:5016,phase:1,chain,block,msg:'API: Missing or invalid Content-Type',severity:MEDIUM"
  SecRule REQUEST_METHOD "@rx ^(POST|PUT|PATCH)$" \
    "chain"
    SecRule REQUEST_HEADERS:Content-Type "!@beginsWith application/json" ""

# Static asset optimization -- skip body inspection
SecRule REQUEST_URI "@rx \.(js|css|png|jpg|jpeg|gif|svg|ico|woff2?)$" \
  "id:5017,phase:1,pass,nolog,ctl:ruleRemoveTargetById=ARGS;941100,ctl:ruleRemoveTargetById=ARGS;942100"

# API versioning -- different rules for v1 vs v2
SecRule REQUEST_URI "@beginsWith /api/v1/" \
  "id:5018,phase:1,pass,nolog,setvar:tx.api_version=1"
SecRule REQUEST_URI "@beginsWith /api/v2/" \
  "id:5019,phase:1,pass,nolog,setvar:tx.api_version=2"
```

### Why This Works

Different endpoints have different security requirements:

- **Health checks** should bypass the WAF entirely. They are simple GET requests with no user input. Running WAF rules on them wastes CPU and risks false positives (e.g., a health check path containing "select").
- **Admin portal** needs maximum protection: IP allowlist, MFA verification, and strict WAF rules. An attacker who reaches the admin portal should be blocked by multiple layers.
- **Login endpoints** need strict rate limiting to prevent credential stuffing. 5 attempts per minute per IP is aggressive but appropriate for a financial application.
- **API endpoints** need Content-Type validation to prevent form-based CSRF attacks. JSON APIs should reject requests with `application/x-www-form-urlencoded` content type.
- **Static assets** do not need body inspection. Skipping WAF rules for static files reduces latency and CPU usage.

### Common Mistakes

- **Applying the same rules to all endpoints.** This causes either too many false positives (strict rules on free-text fields) or too few detections (lenient rules on structured inputs).
- **Not bypassing health checks.** Health check endpoints that trigger WAF rules cause false alerts and waste analyst time.
- **Forgetting about API versioning.** Different API versions may have different security requirements. v2 might have stricter input validation than v1.

---

## Part C: Bot Detection

```apacheconf
# Known attack tool User-Agents
SecRule REQUEST_HEADERS:User-Agent "@pmFromFile bot-user-agents.txt" \
  "id:5020,phase:1,block,msg:'Bot: Known attack tool detected',severity:CRITICAL,\
   logdata:'%{REQUEST_HEADERS.User-Agent}'"

# Missing Accept-Language header (likely automated)
SecRule REQUEST_HEADERS:User-Agent "!@rx ^$" \
  "id:5021,phase:1,chain,setvar:tx.bot_score=+1,msg:'Bot: Missing Accept-Language',severity:INFO"
  SecRule &REQUEST_HEADERS:Accept-Language "@eq 0" ""

# Missing Accept-Encoding header (likely automated)
SecRule REQUEST_HEADERS:User-Agent "!@rx ^$" \
  "id:5022,phase:1,chain,setvar:tx.bot_score=+1,msg:'Bot: Missing Accept-Encoding',severity:INFO"
  SecRule &REQUEST_HEADERS:Accept-Encoding "@eq 0" ""

# Request rate anomaly (> 100 per minute)
SecRule REQUEST_URI "!@beginsWith /health" \
  "id:5023,phase:1,pass,nolog,setvar:ip.request_counter=+1,expirevar:ip.request_counter=60"
SecRule IP:REQUEST_COUNTER "@gt 100" \
  "id:5024,phase:1,setvar:tx.bot_score=+3,msg:'Bot: High request rate',severity:WARNING"

# Suspicious User-Agent patterns (curl, wget, python-requests, etc.)
SecRule REQUEST_HEADERS:User-Agent "@rx (?i)(curl|wget|python-requests|go-http-client|java/|libwww-perl|scrapy|httpclient)" \
  "id:5025,phase:1,setvar:tx.bot_score=+2,msg:'Bot: Suspicious User-Agent',severity:INFO"

# Bot score threshold
SecRule TX:BOT_SCORE "@ge 5" \
  "id:5029,phase:1,block,msg:'Bot: Bot score exceeded threshold',severity:WARNING"
```

Bot handling strategy:

| Bot Type | Detection Method | Response | Reasoning |
|----------|-----------------|----------|-----------|
| Known attack tool (sqlmap, nikto, nmap) | User-Agent signature | Block immediately with 403 | No legitimate reason for these tools to access production. Immediate block. |
| Credential stuffing | Rate + login endpoint pattern | Block + CAPTCHA challenge | Protect authentication. CAPTCHA allows legitimate users who trigger rate limits. |
| Web scraper | Rate + pattern analysis | Throttle with 429 + Retry-After | May be a legitimate API user or business partner. Throttle, do not block. |
| Search engine bot (Googlebot, Bingbot) | User-Agent + reverse DNS verification | Allow with monitoring | SEO benefit. Verify via reverse DNS to prevent spoofing. |
| Unknown automation | Missing headers + rate | JavaScript challenge (Cloudflare Turnstile, reCAPTCHA) | Differentiate humans from bots without blocking. Humans pass the challenge; bots cannot. |

### Why This Works

Bot detection uses multiple signals to classify traffic:

- **User-Agent analysis** catches known tools but is easily spoofed.
- **Header fingerprinting** detects tools that do not send standard browser headers.
- **Rate analysis** detects automated behavior regardless of User-Agent.
- **Behavioral analysis** (request patterns, timing) provides the strongest signal.

The tiered response (block, challenge, throttle, allow) balances security with usability. Blocking all automation breaks API clients. Throttling allows legitimate use while preventing abuse.

### Common Mistakes

- **Blocking all bots.** Some bots are beneficial (search engines, monitoring). Use allowlists for known-good bots.
- **Relying on User-Agent alone.** User-Agent is trivially spoofed. Combine with rate, headers, and behavioral analysis.
- **Not verifying search engine bots.** Attackers spoof Googlebot's User-Agent. Verify via reverse DNS lookup.

---

## Part D: Data Leakage Prevention

```apacheconf
# Block credit card numbers in responses (PCI DSS)
SecRule RESPONSE_BODY "@rx \b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|6(?:011|5[0-9]{2})[0-9]{12}|(?:2131|1800|35\d{3})\d{11})\b" \
  "id:5030,phase:4,block,msg:'Data Leakage: Credit card number in response',severity:CRITICAL,\
   logdata:'Credit card pattern detected'"

# Block SSN patterns in responses
SecRule RESPONSE_BODY "@rx \b\d{3}-\d{2}-\d{4}\b" \
  "id:5031,phase:4,block,msg:'Data Leakage: SSN pattern in response',severity:CRITICAL"

# Block API keys in responses
SecRule RESPONSE_BODY "@rx (?i)\b(sk-[a-zA-Z0-9]{20,}|AKIA[0-9A-Z]{16}|AIza[0-9A-Za-z_-]{35}|ghp_[a-zA-Z0-9]{36})\b" \
  "id:5032,phase:4,block,msg:'Data Leakage: API key in response',severity:CRITICAL"

# Block internal IP addresses in responses
SecRule RESPONSE_BODY "@rx \b(10\.\d{1,3}\.\d{1,3}\.\d{1,3}|172\.(1[6-9]|2\d|3[01])\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3})\b" \
  "id:5033,phase:4,pass,log,msg:'Data Leakage: Internal IP in response',severity:WARNING"

# Block stack traces in responses
SecRule RESPONSE_BODY "@rx (?i)(at\s+\w+\.\w+\(.*:\d+\)|Traceback \(most recent call last\)|Exception in thread|FATAL:\s+|ERROR:\s+.*stack trace)" \
  "id:5034,phase:4,block,msg:'Data Leakage: Stack trace in response',severity:HIGH"

# Block database error messages in responses
SecRule RESPONSE_BODY "@rx (?i)(SQL syntax|mysql_fetch|ORA-\d{5}|PostgreSQL.*ERROR|sqlite3\.|SQLSTATE\[)" \
  "id:5035,phase:4,block,msg:'Data Leakage: Database error in response',severity:CRITICAL"
```

### Why This Works

Response inspection is a PCI DSS requirement and a critical defense against data leakage:

- **Credit card detection** uses the Luhn algorithm patterns for major card brands (Visa, Mastercard, Amex, Discover, Diners). This prevents accidental logging or display of card numbers.
- **SSN detection** catches Social Security Numbers in the `XXX-XX-XXXX` format.
- **API key detection** catches keys from major providers (Stripe `sk-`, AWS `AKIA`, Google `AIza`, GitHub `ghp_`).
- **Stack trace detection** catches programming errors that reveal internal implementation details.
- **Database error detection** catches SQL errors that reveal database structure.

### Common Mistakes

- **Not inspecting responses.** Most WAF configurations only inspect requests. Response inspection is essential for preventing data leakage.
- **False positives on documentation pages.** A page explaining credit card formatting will contain card number patterns. Exclude documentation paths from response inspection.
- **Not redacting, only blocking.** Sometimes blocking the entire response is too aggressive. Consider using `sanitizeMatchedBytes` to redact sensitive data while allowing the response through.

---

## Part E: AWS WAF Terraform Configuration

```hcl
# waf.tf - AWS WAF Web ACL for the financial application

resource "aws_wafv2_web_acl" "main" {
  name        = "finance-app-waf"
  description = "WAF for financial services application"
  scope       = "REGIONAL"

  default_action {
    allow {}
  }

  # AWS Managed Rules: Common Rule Set
  rule {
    name     = "AWSManagedRulesCommonRuleSet"
    priority = 1

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesCommonRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "CommonRuleSet"
      sampled_requests_enabled   = true
    }
  }

  # AWS Managed Rules: SQL Injection
  rule {
    name     = "AWSManagedRulesSQLiRuleSet"
    priority = 2

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesSQLiRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "SQLiRuleSet"
      sampled_requests_enabled   = true
    }
  }

  # Rate limiting: 100 requests per IP per 5 minutes
  rule {
    name     = "RateLimitPerIP"
    priority = 3

    action {
      block {
        custom_response {
          response_code = 429
          custom_response_body_key = "rate-limit-exceeded"
        }
      }
    }

    statement {
      rate_based_statement {
        limit              = 100
        aggregate_key_type = "IP"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "RateLimitPerIP"
      sampled_requests_enabled   = true
    }
  }

  # Admin portal IP restriction
  rule {
    name     = "AdminIPAllowlist"
    priority = 4

    action {
      block {
        custom_response {
          response_code = 403
          custom_response_body_key = "forbidden"
        }
      }
    }

    statement {
      and_statement {
        statement {
          byte_match_statement {
            positional_constraint = "STARTS_WITH"
            search_string         = "/admin"
            field_to_match {
              uri_path {}
            }
            text_transformation {
              priority = 0
              type     = "LOWERCASE"
            }
          }
        }
        statement {
          not_statement {
            statement {
              ip_set_reference_statement {
                arn = aws_wafv2_ip_set.admin_allowed_ips.arn
              }
            }
          }
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "AdminIPAllowlist"
      sampled_requests_enabled   = true
    }
  }

  # Geographic restriction (block sanctioned countries)
  rule {
    name     = "GeoRestriction"
    priority = 5

    action {
      block {}
    }

    statement {
      geo_match_statement {
        country_codes = ["KP", "IR", "SY", "CU"]
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "GeoRestriction"
      sampled_requests_enabled   = true
    }
  }

  # Request size limit
  rule {
    name     = "RequestSizeLimit"
    priority = 6

    action {
      block {
        custom_response {
          response_code = 413
        }
      }
    }

    statement {
      size_constraint_statement {
        field_to_match {
          body {
            oversize_handling = "MATCH"
          }
        }
        comparison_operator = "GT"
        size                = 1048576  # 1MB
        text_transformation {
          priority = 0
          type     = "NONE"
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "RequestSizeLimit"
      sampled_requests_enabled   = true
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "FinanceAppWAF"
    sampled_requests_enabled   = true
  }

  tags = {
    Environment = "production"
    Application = "finance-app"
    Compliance  = "pci-dss"
  }
}

# IP set for admin allowlist
resource "aws_wafv2_ip_set" "admin_allowed_ips" {
  name               = "admin-allowed-ips"
  description        = "IPs allowed to access admin portal"
  scope              = "REGIONAL"
  ip_address_version = "IPV4"
  addresses          = [
    "10.0.0.0/8",
    "192.168.1.0/24"
  ]
}

# Custom response bodies
resource "aws_wafv2_web_acl" "main" {
  # ... (above)

  custom_response_body {
    key          = "rate-limit-exceeded"
    content      = jsonencode({
      error   = "rate_limit_exceeded"
      message = "Too many requests. Please try again later."
    })
    content_type = "APPLICATION_JSON"
  }

  custom_response_body {
    key          = "forbidden"
    content      = jsonencode({
      error   = "forbidden"
      message = "Access denied."
    })
    content_type = "APPLICATION_JSON"
  }
}

# Logging configuration
resource "aws_wafv2_web_acl_logging_configuration" "main" {
  log_destination_configs = [aws_cloudwatch_log_group.waf_logs.arn]
  resource_arn           = aws_wafv2_web_acl.main.arn

  logging_filter {
    default_behavior = "DROP"

    filter {
      behavior = "KEEP"
      condition {
        action_condition {
          action = "BLOCK"
        }
      }
      requirement = "MEETS_ANY"
    }

    filter {
      behavior = "KEEP"
      condition {
        action_condition {
          action = "COUNT"
        }
      }
      requirement = "MEETS_ANY"
    }
  }
}

resource "aws_cloudwatch_log_group" "waf_logs" {
  name              = "aws-waf-logs-finance-app"
  retention_in_days = 90
}

# Associate with ALB
resource "aws_wafv2_web_acl_association" "main" {
  resource_arn = aws_lb.main.arn
  web_acl_arn  = aws_wafv2_web_acl.main.arn
}
```

### Why This Works

The AWS WAF configuration provides comprehensive protection:

1. **Managed rule groups** (rules 1-2): AWS maintains and updates these rules automatically. They cover the OWASP Top 10 and common attack patterns.
2. **Rate limiting** (rule 3): Prevents brute force and DDoS at the application layer. AWS WAF uses a sliding window for rate calculation.
3. **IP allowlist** (rule 4): Restricts admin portal access to known IPs. Uses a separate IP set for easy management.
4. **Geographic restriction** (rule 5): Blocks traffic from sanctioned countries. This is a PCI DSS and regulatory requirement for financial applications.
5. **Request size limit** (rule 6): Prevents large payload attacks (buffer overflow, slow POST).
6. **Logging** captures all blocked and counted requests for forensic analysis. The logging filter keeps only relevant events to control costs.

### Common Mistakes

- **Not using managed rule groups.** AWS managed rules are maintained by AWS security researchers and updated automatically. They provide baseline protection with minimal tuning.
- **Blocking on COUNT mode.** Always test new rules in COUNT mode first. Switch to BLOCK only after verifying no false positives.
- **Not logging blocked requests.** Without logs, you cannot tune rules or investigate incidents. Log at least all blocked requests.
- **Forgetting to associate the WAF with the ALB.** The WAF does nothing until it is attached to a resource.

---

## Common Mistakes to Avoid

1. **Not tuning before production.** Deploy in COUNT mode first. Analyze the logs for false positives. Tune thresholds and exceptions. Only then switch to BLOCK mode.

2. **Setting and forgetting.** WAF rules need ongoing maintenance. New attack techniques emerge, application endpoints change, and traffic patterns shift. Review rules monthly.

3. **Not correlating WAF logs with other systems.** WAF logs are most valuable when correlated with application logs, authentication logs, and SIEM systems. A blocked SQLi attempt from an IP that also tried to log in with 50 different usernames is a stronger signal than either event alone.

4. **WAF as the only security control.** A WAF is one layer in defense in depth. It does not replace parameterized queries, output encoding, authentication, or authorization.

5. **Not monitoring WAF health.** If the WAF is down or misconfigured, traffic flows through unprotected. Monitor WAF availability and rule evaluation metrics.

## Key Takeaway

A comprehensive WAF policy is not a single set of rules but a system that combines anomaly scoring for detection accuracy, per-endpoint policies for appropriate security levels, bot detection for automated traffic management, and data leakage prevention for compliance. The key to production success is tuning: start strict, analyze false positives, and adjust thresholds based on real traffic patterns. A WAF that blocks legitimate users will be disabled. A WAF that allows attacks is worse than useless. The goal is the narrow path between these extremes, and reaching it requires continuous monitoring and adjustment.
