# Solution 05: CDN with Origin Failover and Security

## Architecture Overview

```
                        [WAF]
                          |
User -> [CloudFront CDN] -+-> [Origin Group: Static Assets]
              |                  |
              |          Primary: S3 us-east-1
              |          Failover: S3 us-west-2
              |
              +-> [Origin Group: API]
                     |
             Primary: ALB us-east-1
             Failover: ALB us-west-2
```

## Part A -- Origin Failover Architecture

### Terraform Configuration

```hcl
# --- Static Assets Origin Group ---

resource "aws_cloudfront_distribution" "media" {
  enabled         = true
  http_version    = "http2and3"
  price_class     = "PriceClass_All"
  web_acl_id      = aws_wafv2_web_acl.cdn.arn

  # Primary S3 origin
  origin {
    domain_name              = aws_s3_bucket.primary.bucket_regional_domain_name
    origin_id                = "s3-primary"
    origin_access_control_id = aws_cloudfront_origin_access_control.primary.id

    origin_shield {
      enabled              = true
      origin_shield_region = "us-east-1"
    }
  }

  # Failover S3 origin
  origin {
    domain_name              = aws_s3_bucket.failover.bucket_regional_domain_name
    origin_id                = "s3-failover"
    origin_access_control_id = aws_cloudfront_origin_access_control.failover.id

    origin_shield {
      enabled              = true
      origin_shield_region = "us-west-2"
    }
  }

  # API primary origin
  origin {
    domain_name = aws_lb.primary.dns_name
    origin_id   = "api-primary"

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
      origin_read_timeout    = 30
    }

    origin_shield {
      enabled              = true
      origin_shield_region = "us-east-1"
    }
  }

  # API failover origin
  origin {
    domain_name = aws_lb.failover.dns_name
    origin_id   = "api-failover"

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
      origin_read_timeout    = 30
    }

    origin_shield {
      enabled              = true
      origin_shield_region = "us-west-2"
    }
  }

  # Static assets origin group with failover
  origin_group {
    origin_id = "static-failover-group"

    failover_criteria {
      status_codes = [500, 502, 503, 504]
    }

    member {
      origin_id = "s3-primary"
    }

    member {
      origin_id = "s3-failover"
    }
  }

  # API origin group with failover
  origin_group {
    origin_id = "api-failover-group"

    failover_criteria {
      status_codes = [500, 502, 503, 504]
    }

    member {
      origin_id = "api-primary"
    }

    member {
      origin_id = "api-failover"
    }
  }

  # Default behavior: static assets with failover
  default_cache_behavior {
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "static-failover-group"
    viewer_protocol_policy = "redirect-to-https"
    compress               = true

    cache_policy_id = aws_cloudfront_cache_policy.static.id
  }

  # API behavior with failover
  ordered_cache_behavior {
    path_pattern           = "/api/*"
    allowed_methods        = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "api-failover-group"
    viewer_protocol_policy = "https-only"

    cache_policy_id          = "4135ea2d-6df8-44a3-9df3-4b5a84be39ad"  # CachingDisabled
    origin_request_policy_id = "216adef6-5c7f-47e4-b989-5492eafa07d3"  # AllViewer
  }

  # Premium content behavior (signed URLs required)
  ordered_cache_behavior {
    path_pattern           = "/premium/*"
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "static-failover-group"
    viewer_protocol_policy = "redirect-to-https"

    cache_policy_id = aws_cloudfront_cache_policy.premium.id

    trusted_key_groups = [aws_cloudfront_key_group.signing.id]
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    acm_certificate_arn      = aws_acm_certificate.cert.arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.2_2021"
  }
}
```

### Health Check Configuration

The failover within origin groups is automatic. CloudFront sends a request to the
primary origin, and if it receives a 5xx status code, it retries with the secondary
origin. The default timeout is 30 seconds, but you can reduce it:

```hcl
# In the origin configuration
custom_origin_config {
  origin_read_timeout    = 10  # seconds
  origin_keepalive_timeout = 5
}
```

For more sophisticated health checking, use Route 53 health checks to remove unhealthy
origins from DNS before CloudFront even tries them.

## Part B -- WAF Integration

### Rate Limiting Rule

```hcl
resource "aws_wafv2_web_acl" "cdn" {
  name  = "cdn-waf"
  scope = "CLOUDFRONT"

  default_action {
    allow {}
  }

  # Rule 1: Rate limiting
  rule {
    name     = "RateLimitRule"
    priority = 1

    action {
      block {}
    }

    statement {
      rate_based_statement {
        limit              = 2000
        aggregate_key_type = "IP"

        scope_down_statement {
          byte_match_statement {
            positional_constraint = "STARTS_WITH"
            search_string         = "/"

            field_to_match {
              uri_path {}
            }

            text_transformation {
              priority = 0
              type     = "LOWERCASE"
            }
          }
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "RateLimitRule"
      sampled_requests_enabled   = true
    }
  }

  # Rule 2: SQL Injection
  rule {
    name     = "SQLInjectionRule"
    priority = 2

    action {
      block {}
    }

    statement {
      sqli_match_statement {
        field_to_match {
          query_string {}
        }

        text_transformation {
          priority = 0
          type     = "URL_DECODE"
        }
        text_transformation {
          priority = 1
          type     = "HTML_ENTITY_DECODE"
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "SQLInjectionRule"
      sampled_requests_enabled   = true
    }
  }

  # Rule 3: AWS Managed IP Reputation List
  rule {
    name     = "AWSManagedIPReputation"
    priority = 5

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesAmazonIpReputationList"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "AWSManagedIPReputation"
      sampled_requests_enabled   = true
    }
  }

  # Rule 4: AWS Managed Bot Control
  rule {
    name     = "AWSManagedBotControl"
    priority = 4

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesBotControlRuleSet"
        vendor_name = "AWS"

        managed_rule_group_configs {
          aws_managed_rules_bot_control_rule_set {
            inspection_level = "COMMON"
          }
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "AWSManagedBotControl"
      sampled_requests_enabled   = true
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "CDNWAF"
    sampled_requests_enabled   = true
  }
}
```

### How WAF Evaluates Rules

WAF evaluates rules in priority order (lowest number first). If Rule 1 (rate limiting)
blocks the request, Rules 2-5 are not evaluated. This is important for both cost (WAF
charges per rule evaluation) and latency (fewer evaluations = faster response).

## Part C -- Signed URLs for Premium Content

### Signed URL Generation

```python
import boto3
from botocore.signer import CloudFrontSigner
from datetime import datetime, timedelta
import rsa
import hashlib

class SignedURLService:
    def __init__(self, key_pair_id, private_key_path):
        self.key_pair_id = key_pair_id
        with open(private_key_path, 'r') as f:
            self.private_key = rsa.PrivateKey.load_pkcs1(f.read())

    def _rsa_signer(self, message):
        return rsa.sign(message, self.private_key, 'SHA-1')

    def generate_signed_url(
        self,
        resource_url: str,
        expiration_hours: int = 1,
        ip_address: str = None
    ) -> str:
        """
        Generate a signed URL for premium content.

        Args:
            resource_url: Full CloudFront URL (e.g., https://cdn.example.com/premium/videos/course1/lesson1.mp4)
            expiration_hours: Hours until the URL expires
            ip_address: Optional IP address restriction (e.g., "203.0.113.1/32")

        Returns:
            Signed URL string
        """
        expire_date = datetime.utcnow() + timedelta(hours=expiration_hours)

        signer = CloudFrontSigner(self.key_pair_id, self._rsa_signer)

        policy = self._build_custom_policy(
            resource_url,
            expire_date,
            ip_address
        )

        return signer.generate_presigned_url(
            resource_url,
            policy=policy
        )

    def _build_custom_policy(self, resource_url, expire_date, ip_address=None):
        """Build a custom policy for fine-grained access control."""
        policy = {
            "Statement": [{
                "Resource": resource_url,
                "Condition": {
                    "DateLessThan": {
                        "AWS:EpochTime": int(expire_date.timestamp())
                    }
                }
            }]
        }

        if ip_address:
            policy["Statement"][0]["Condition"]["IpAddress"] = {
                "AWS:SourceIp": ip_address
            }

        return json.dumps(policy)

    def generate_signed_cookie(
        self,
        resource_path: str,
        expiration_hours: int = 1
    ) -> dict:
        """
        Generate signed cookies for accessing multiple resources under a path.

        Use this for video players that request multiple segments.
        """
        expire_date = datetime.utcnow() + timedelta(hours=expiration_hours)
        policy = self._build_custom_policy(
            f"https://cdn.example.com{resource_path}*",
            expire_date
        )

        # The policy must be base64-encoded for cookies
        import base64
        policy_b64 = base64.b64encode(policy.encode()).decode()

        # Sign the policy
        signature = rsa.sign(policy.encode(), self.private_key, 'SHA-1')
        signature_b64 = base64.b64encode(signature).decode()

        return {
            'CloudFront-Policy': policy_b64,
            'CloudFront-Signature': signature_b64,
            'CloudFront-Key-Pair-Id': self.key_pair_id
        }


# Usage
service = SignedURLService(
    key_pair_id='K2JC2R8EXAMPLE',
    private_key_path='rsa-private.pem'
)

# Single video URL
url = service.generate_signed_url(
    'https://cdn.example.com/premium/videos/course1/lesson1.mp4',
    expiration_hours=1,
    ip_address='203.0.113.1/32'
)

# Cookie for video player (access all segments)
cookies = service.generate_signed_cookie(
    '/premium/videos/course1/',
    expiration_hours=2
)
```

### Signed URLs vs Signed Cookies

| Feature | Signed URLs | Signed Cookies |
|---------|-------------|----------------|
| Scope | Single resource | All resources matching a path pattern |
| Use case | Download link for a specific file | Video player requesting multiple segments |
| Distribution | Send in email, embed in page | Set as HTTP cookie, sent automatically |
| IP restriction | Supported | Supported |
| Expiration | Per-URL | Per-cookie (applies to all matching resources) |
| Complexity | Lower (one URL per resource) | Higher (cookie management) |

**When to use signed URLs**: When you need to share a link to a specific resource
(e.g., email with a download link, embed a specific video in a page).

**When to use signed cookies**: When a single "session" should grant access to many
resources (e.g., a video player requesting 100+ segments for a single course video).

## Part D -- DDoS Protection Layers

```
Layer 1: CDN Edge (Absorb volumetric attacks)
  Service: CloudFront + AWS Shield Standard (included free)
  Configuration:
    - CloudFront absorbs DDoS traffic at 400+ edge locations globally
    - Shield Standard provides always-on detection for Layer 3/4 attacks
    - Automatic inline mitigation for common attacks
    - No configuration needed -- this is always active

Layer 2: WAF (Filter application-layer attacks)
  Service: AWS WAF v2 + AWS Shield Advanced (optional)
  Configuration:
    - Rate limiting: 2000 requests per IP per 5 minutes
    - SQL injection rules (managed rule group)
    - Cross-site scripting rules (managed rule group)
    - Bot control (challenge suspicious user agents)
    - IP reputation list (block known malicious IPs)
    - Geographic restriction (block traffic from regions you do not serve)
    - Request size constraints (block oversized requests)

Layer 3: Origin (Protect the backend)
  Services: ALB + Security Groups + Auto Scaling
  Configuration:
    - ALB only accepts traffic from CloudFront IP ranges (via security group)
    - ALB rate limiting and connection limits
    - Auto Scaling ensures origin capacity scales with traffic
    - Application-level rate limiting per user/API key
    - Database connection pooling to prevent connection exhaustion

Layer 4: Monitoring and Response
  Services: CloudWatch + Shield Advanced Response Team + SNS
  Configuration:
    - CloudWatch alarms on: request rate, error rate, origin latency, WAF blocked count
    - Shield Advanced provides 24/7 DDoS Response Team (DRT) access
    - SNS notifications for threshold breaches
    - Automated runbook execution (e.g., increase WAF rate limit, scale origins)
    - Post-incident analysis with CloudFront access logs and WAF logs
```

### Why Defense-in-Depth

Each layer handles a different class of attack:

- **Layer 1** (CDN) handles volumetric attacks by distributing traffic across hundreds
  of PoPs. A 1 Tbps attack is absorbed by the CDN's aggregate capacity (~200+ Tbps).
- **Layer 2** (WAF) handles application-layer attacks that look like legitimate traffic
  but contain malicious payloads (SQL injection, XSS, credential stuffing).
- **Layer 3** (Origin) provides the last line of defense. Even if an attacker bypasses
  the CDN and WAF, the origin's security groups and rate limits prevent overwhelm.
- **Layer 4** (Monitoring) ensures that attacks are detected and responded to quickly,
  even if they use novel techniques that bypass existing rules.

## Part E -- End-to-End Security Headers

### Response Headers Policy

```hcl
resource "aws_cloudfront_response_headers_policy" "security" {
  name    = "production-security-headers"
  comment = "Complete security posture for media platform"

  security_headers_config {
    strict_transport_security {
      access_control_max_age_sec = 31536000
      include_subdomains         = true
      preload                    = true
      override                   = true
    }

    content_type_options {
      override = true
    }

    frame_options {
      frame_option = "DENY"
      override     = true
    }

    xss_protection {
      mode_block = true
      protection = true
      override   = true
    }

    referrer_policy {
      referrer_policy = "strict-origin-when-cross-origin"
      override        = true
    }

    content_security_policy {
      content_security_policy = "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.example.com; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; media-src 'self' https://cdn.example.com; connect-src 'self' https://api.example.com; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"
      override                = true
    }
  }

  custom_headers_config {
    items {
      header   = "Permissions-Policy"
      value    = "camera=(), microphone=(), geolocation=(), payment=(), usb=(), magnetometer=()"
      override = true
    }
  }
}
```

### Header Explanations

| Header | Value | Purpose |
|--------|-------|---------|
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains; preload` | Forces HTTPS for 1 year. `preload` allows inclusion in browser HSTS preload lists. Prevents SSL stripping attacks. |
| `X-Content-Type-Options` | `nosniff` | Prevents browsers from MIME-type sniffing. Stops attackers from uploading a malicious HTML file disguised as an image. |
| `X-Frame-Options` | `DENY` | Prevents the site from being embedded in iframes. Stops clickjacking attacks. |
| `X-XSS-Protection` | `1; mode=block` | Enables the browser's built-in XSS filter (legacy, but still useful for older browsers). |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Sends full URL as referrer for same-origin requests, but only the origin for cross-origin. Prevents leaking path information to third parties. |
| `Content-Security-Policy` | (see above) | Restricts where resources can be loaded from. Prevents XSS by blocking inline scripts from untrusted sources. `frame-ancestors 'none'` is the CSP equivalent of `X-Frame-Options: DENY`. |
| `Permissions-Policy` | (see above) | Disables browser features that the site does not need. Reduces attack surface by blocking camera, microphone, geolocation, etc. |

## Part F -- Observability

### CloudWatch Metrics to Monitor

```hcl
# CloudWatch Dashboard
resource "aws_cloudfront_monitor" "dashboard" {
  # These are accessed via CloudWatch console or API
}
```

**Metrics to track:**

| Metric | Source | Target | Alert Threshold |
|--------|--------|--------|-----------------|
| Cache Hit Ratio | CloudFront `CacheHitRate` | >90% | <80% for 5 minutes |
| Origin Latency (p50) | CloudFront `OriginLatency` | <100ms | >200ms for 5 minutes |
| Origin Latency (p95) | CloudFront `OriginLatency` | <500ms | >1000ms for 5 minutes |
| Error Rate (4xx) | CloudFront `4xxErrorRate` | <1% | >5% for 5 minutes |
| Error Rate (5xx) | CloudFront `5xxErrorRate` | <0.1% | >1% for 5 minutes |
| WAF Blocked Requests | WAF `BlockedRequests` | Monitor for spikes | >10000 blocked in 1 minute |
| Bandwidth | CloudFront `BytesDownloaded` | Monitor for anomalies | >2x baseline for 10 minutes |

### CloudWatch Alarms

```hcl
resource "aws_cloudwatch_metric_alarm" "cache_hit_ratio" {
  alarm_name          = "cloudfront-low-cache-hit-ratio"
  comparison_operator = "LessThanThreshold"
  evaluation_periods  = 3
  metric_name         = "CacheHitRate"
  namespace           = "AWS/CloudFront"
  period              = 300
  statistic           = "Average"
  threshold           = 80
  alarm_description   = "Cache hit ratio dropped below 80%"
  alarm_actions       = [aws_sns_topic.alerts.arn]

  dimensions = {
    DistributionId = aws_cloudfront_distribution.media.id
  }
}

resource "aws_cloudwatch_metric_alarm" "origin_5xx" {
  alarm_name          = "cloudfront-origin-5xx-errors"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 2
  metric_name         = "5xxErrorRate"
  namespace           = "AWS/CloudFront"
  period              = 300
  statistic           = "Average"
  threshold           = 1
  alarm_description   = "Origin 5xx error rate exceeded 1%"
  alarm_actions       = [aws_sns_topic.alerts.arn]

  dimensions = {
    DistributionId = aws_cloudfront_distribution.media.id
  }
}

resource "aws_cloudwatch_metric_alarm" "waf_blocked" {
  alarm_name          = "waf-high-block-rate"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "BlockedRequests"
  namespace           = "AWS/WAFV2"
  period              = 60
  statistic           = "Sum"
  threshold           = 10000
  alarm_description   = "WAF blocked more than 10,000 requests in 1 minute"
  alarm_actions       = [aws_sns_topic.alerts.arn]

  dimensions = {
    WebACL = aws_wafv2_web_acl.cdn.name
    Rule   = "ALL"
  }
}
```

### CloudFront Access Logs

Enable standard or real-time logging:

```hcl
resource "aws_cloudfront_distribution" "media" {
  # ... other configuration ...

  logging_config {
    bucket          = aws_s3_bucket.logs.bucket_domain_name
    include_cookies = false
    prefix          = "cloudfront/"
  }
}
```

Use Athena to query logs:

```sql
-- Top 10 most requested paths
SELECT cs_uri_stem, COUNT(*) as request_count
FROM cloudfront_logs
WHERE date = '2025-01-15'
GROUP BY cs_uri_stem
ORDER BY request_count DESC
LIMIT 10;

-- Cache hit ratio by path pattern
SELECT
  cs_uri_stem,
  COUNT(*) as total_requests,
  SUM(CASE WHEN x_edge_result_type = 'Hit' THEN 1 ELSE 0 END) as cache_hits,
  ROUND(100.0 * SUM(CASE WHEN x_edge_result_type = 'Hit' THEN 1 ELSE 0 END) / COUNT(*), 2) as hit_ratio
FROM cloudfront_logs
WHERE date = '2025-01-15'
GROUP BY cs_uri_stem
HAVING COUNT(*) > 100
ORDER BY hit_ratio ASC;
```

## Common Mistakes to Avoid

1. **Not restricting S3 bucket to CloudFront**: Without the bucket policy condition
   that limits access to the CloudFront service principal, anyone can bypass the CDN
   and access S3 directly.

2. **WAF rules too permissive**: Overly broad rate limiting (e.g., 100,000 requests
   per 5 minutes) provides no protection against moderate attacks. Test your rules
   against realistic attack patterns.

3. **Signed URLs with no expiration**: Always set an expiration. A signed URL without
   expiration is equivalent to a public URL that happens to be harder to guess.

4. **Forgetting `stale-if-error` for premium content**: If the origin serving premium
   content goes down, users with valid signed URLs get errors. Add `stale-if-error`
   to allow the CDN to serve cached premium content during outages.

5. **Monitoring only CDN metrics, not origin metrics**: A high cache hit ratio can
   mask origin problems. Monitor both CDN and origin metrics to catch issues before
   cache expiry reveals them.

6. **Not testing failover**: Deploy the architecture and then manually disable the
   primary origin (e.g., change the security group to deny traffic). Verify that
   failover occurs within your target time and that no data is lost.

## Key Takeaway

A production CDN architecture requires defense-in-depth: origin failover for
availability, WAF for application-layer security, signed URLs for access control, and
DDoS protection that spans every layer from edge to origin. The monitoring dashboard
is not optional -- it is the system that tells you whether everything else is working.
Without observability, you are flying blind during incidents.
