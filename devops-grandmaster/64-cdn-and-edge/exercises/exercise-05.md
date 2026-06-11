# Exercise 05: CDN with Origin Failover and Security (Integration)

## Objective

Build a production-grade CDN architecture that combines origin failover, WAF
integration, signed URLs for premium content, and DDoS protection into a cohesive
system.

## Background

You operate a digital media platform that serves both free and premium content. Free
content (articles, images) is cached aggressively at the edge. Premium content (video
courses, downloadable resources) requires authentication and must not be cached publicly.
The platform has experienced DDoS attacks and needs protection at the CDN layer.

## Instructions

### Part A -- Origin Failover Architecture

Deploy an architecture with primary and failover origins:

```
Primary Origin:  S3 bucket in us-east-1 (static assets + API proxy)
Failover Origin: S3 bucket in us-west-2 (static assets replica)
API Origin:      ALB in us-east-1 (dynamic API)

Requirements:
  - Static assets failover automatically if primary S3 returns 5xx.
  - API calls failover to a warm standby ALB in us-west-2.
  - Failover must complete within 10 seconds (configure health checks accordingly).
```

Write the CloudFront distribution configuration (Terraform or CloudFormation) with:
- Origin groups for failover.
- Health check configuration with appropriate intervals and thresholds.

### Part B -- WAF Integration

Configure AWS WAF rules attached to the CloudFront distribution:

| Rule | Priority | Action | Description |
|------|----------|--------|-------------|
| Rate Limit | 1 | Block | Block IPs exceeding 2000 requests per 5 minutes |
| SQL Injection | 2 | Block | Block common SQL injection patterns in query strings |
| Geo Restriction | 3 | Block | Block traffic from countries where you do not operate |
| Bot Control | 4 | Challenge | Challenge known bot user agents |
| IP Reputation | 5 | Block | Block IPs from AWS managed threat intelligence list |

Write the WAF rule configuration for at least two of the above rules.

### Part C -- Signed URLs for Premium Content

Implement a signed URL system for premium video content:

1. When a user with an active subscription requests a video, generate a signed URL
   with:
   - Expiration: 1 hour from generation.
   - Resource path: `/premium/videos/{course_id}/{lesson_id}.mp4`.
   - IP restriction (optional): limit to the requesting user's IP.
2. Write the signing logic in your preferred language.
3. Explain the difference between signed URLs and signed cookies, and when to use each.

### Part D -- DDoS Protection Layers

Describe and configure a defense-in-depth DDoS protection strategy:

```
Layer 1: CDN Edge (Absorb volumetric attacks)
  - Configure: ???

Layer 2: WAF (Filter application-layer attacks)
  - Configure: ???

Layer 3: Origin (Protect the backend)
  - Configure: ???

Layer 4: Monitoring and Response
  - Configure: ???
```

For each layer, specify the specific AWS services/features used and how they
complement each other.

### Part E -- End-to-End Security Headers

Create a response headers policy that implements a complete security posture:

1. `Strict-Transport-Security` with `includeSubDomains` and `preload`.
2. `Content-Security-Policy` that restricts script sources.
3. `X-Frame-Options: DENY`.
4. `X-Content-Type-Options: nosniff`.
5. `Referrer-Policy: strict-origin-when-cross-origin`.
6. `Permissions-Policy` to restrict camera, microphone, and geolocation.

Write the complete response headers policy configuration.

### Part F -- Observability

Design a monitoring dashboard that tracks:

1. Cache hit ratio (target: >90%).
2. Origin latency (p50, p95, p99).
3. Error rate by type (4xx, 5xx).
4. WAF blocked requests (by rule).
5. Bandwidth usage by content type.
6. Signed URL generation rate and failures.

Specify which CloudWatch metrics, logs, or custom metrics you would use.

## Success Criteria

- [ ] Origin failover is testable: stopping the primary origin results in automatic
      switchover within 10 seconds.
- [ ] WAF rules block SQL injection attempts (test with `curl` and a known payload).
- [ ] Signed URLs are only valid for the specified duration and resource.
- [ ] DDoS protection covers all four layers with no single point of failure.
- [ ] Security headers receive an A+ rating on securityheaders.com.
- [ ] Monitoring dashboard covers all six required metrics.

## Hints

<details>
<summary>Hint 1 -- Signed URL generation with AWS SDK</summary>
```python
import boto3
from botocore.signer import CloudFrontSigner
from datetime import datetime, timedelta
import rsa

def generate_signed_url(resource_url, key_pair_id, private_key_path, expiration_hours=1):
    with open(private_key_path, 'r') as f:
        private_key = rsa.PrivateKey.load_pkcs1(f.read())

    expire_date = datetime.utcnow() + timedelta(hours=expiration_hours)
    signer = CloudFrontSigner(key_pair_id, lambda message: rsa.sign(message, private_key, 'SHA-1'))
    return signer.generate_presigned_url(resource_url, date_less_than=expire_date)
```
</details>

<details>
<summary>Hint 2 -- Signed URLs vs Signed Cookies</summary>
Use signed URLs when you need to protect individual resources (a specific video file).
Use signed cookies when you need to protect many resources at once (all content under
`/premium/*`) -- the user gets one cookie and can access all matching resources without
individual URLs. Signed cookies are better for video players that need to request
multiple segments.
</details>

<details>
<summary>Hint 3 -- WAF rate limiting rule</summary>
```json
{
  "Name": "RateLimitRule",
  "Priority": 1,
  "Statement": {
    "RateBasedStatement": {
      "Limit": 2000,
      "AggregateKeyType": "IP"
    }
  },
  "Action": {
    "Block": {}
  },
  "VisibilityConfig": {
    "SampledRequestsEnabled": true,
    "CloudWatchMetricsEnabled": true,
    "MetricName": "RateLimitRule"
  }
}
```
</details>

<details>
<summary>Hint 4 -- CloudFront access logs for observability</summary>
Enable CloudFront standard or real-time logs. Standard logs include every request with
cache status, origin latency, bytes transferred, and HTTP status. Use Athena to query
logs or ship them to CloudWatch Logs Insights for real-time dashboards. Real-time logs
deliver within seconds but cost more.
</details>
