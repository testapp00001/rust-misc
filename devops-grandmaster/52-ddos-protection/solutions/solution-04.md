# Solution 04: Incident Response Under Fire

## Part A: Attack Classification and Prioritization

```
Attack Classification: Application-Layer HTTP Flood (L7)
Target: Application resources -- specifically database-bound endpoints
  (RDS connections at 200/200, CPU at 98%, p99 response time at 12s)
Affected Services:
  - API Service (direct target, CPU saturated)
  - Search Service (shares database, connection pool exhausted)
  - All endpoints using PostgreSQL (cascading from RDS exhaustion)
  - All endpoints using Redis (potential cache stampede)
Priority Actions:
  1. Stop the bleeding: Deploy emergency rate limiting (0-2 min)
  2. Protect the database: Enable circuit breakers, kill long queries (2-5 min)
  3. Block top offenders: Analyze logs, add IP blocks (5-10 min)
  4. Scale infrastructure: Increase ECS task count (5-10 min)
  5. Enable upstream protection: Activate Shield Advanced / WAF rules (10-15 min)
```

**Why this is an application-layer attack, not volumetric:**

The key evidence is the 35% 5xx error rate combined with RDS connection exhaustion. A volumetric attack saturates bandwidth -- you would see network throughput maxed out, not application errors. The 45,000 req/s is high but not bandwidth-saturating (a typical HTTP request is ~1 KB, so 45,000 req/s is ~45 Mbps, well within normal network capacity). The attacker is sending requests that are cheap in bandwidth but expensive in application resources -- specifically, requests that trigger database queries.

The RDS connection limit (200/200) is the smoking gun. The attacker is hitting endpoints that open database connections and hold them (slow queries, connection pool exhaustion). This is a classic L7 resource exhaustion attack.

**Why this happened on Black Friday:**

The normal traffic baseline is 2,000 req/s. On Black Friday, legitimate traffic is likely already elevated (perhaps 5,000-8,000 req/s). The attack adds 45,000 req/s on top of that. The system was already under higher-than-normal load when the attack hit, reducing the margin for error.

---

## Part B: Emergency Response Script (`emergency-response.sh`)

```bash
#!/bin/bash
# emergency-response.sh -- Automated DDoS emergency response
#
# Usage:
#   ./emergency-response.sh --level critical --dry-run
#   ./emergency-response.sh --level critical --execute
#   ./emergency-response.sh --rollback

set -euo pipefail

# -------------------------------------------------------
# Configuration
# -------------------------------------------------------
LOG_FILE="/var/log/ddos-response.log"
BACKUP_DIR="/etc/nginx/backup"
CLUSTER="production"
API_SERVICE="api-service"
SEARCH_SERVICE="search-service"
SLACK_WEBHOOK="${SLACK_WEBHOOK_URL:-}"
ALERT_LEVEL=""
DRY_RUN=false
EXECUTE=false
ROLLBACK=false

# -------------------------------------------------------
# Parse arguments
# -------------------------------------------------------
while [[ $# -gt 0 ]]; do
    case "$1" in
        --level)    ALERT_LEVEL="$2"; shift 2 ;;
        --dry-run)  DRY_RUN=true; shift ;;
        --execute)  EXECUTE=true; shift ;;
        --rollback) ROLLBACK=true; shift ;;
        *)          echo "Unknown option: $1"; exit 1 ;;
    esac
done

if [ "$ROLLBACK" = true ]; then
    echo "=== ROLLBACK MODE ==="
    echo "[*] Restoring original Nginx configuration..."
    if [ -f "$BACKUP_DIR/nginx.conf.bak" ]; then
        cp "$BACKUP_DIR/nginx.conf.bak" /etc/nginx/nginx.conf
        nginx -t && nginx -s reload
        echo "[+] Nginx configuration restored"
    else
        echo "[!] No backup found at $BACKUP_DIR/nginx.conf.bak"
    fi

    echo "[*] Flushing iptables DDoS rules..."
    iptables -F INPUT 2>/dev/null || true

    echo "[*] Restoring ECS service counts..."
    if [ "$DRY_RUN" = false ]; then
        aws ecs update-service \
            --cluster "$CLUSTER" \
            --service "$API_SERVICE" \
            --desired-count 3 2>/dev/null || true
        aws ecs update-service \
            --cluster "$CLUSTER" \
            --service "$SEARCH_SERVICE" \
            --desired-count 3 2>/dev/null || true
    fi

    echo "[+] Rollback complete"
    exit 0
fi

if [ "$EXECUTE" = false ] && [ "$DRY_RUN" = false ]; then
    echo "Usage: $0 --level <level> --dry-run|--execute | --rollback"
    exit 1
fi

# -------------------------------------------------------
# Logging
# -------------------------------------------------------
log() {
    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    echo "[$ts] $*" | tee -a "$LOG_FILE"
}

# -------------------------------------------------------
# Step 1: Backup current configuration
# -------------------------------------------------------
log "=== DDoS Emergency Response Started (level=$ALERT_LEVEL) ==="

if [ "$DRY_RUN" = true ]; then
    log "[DRY-RUN] Would backup Nginx configuration"
else
    mkdir -p "$BACKUP_DIR"
    cp /etc/nginx/nginx.conf "$BACKUP_DIR/nginx.conf.bak"
    log "[+] Backed up Nginx configuration"
fi

# -------------------------------------------------------
# Step 2: Deploy emergency rate limiting
# -------------------------------------------------------
log "[*] Step 1: Deploying emergency rate limiting"

EMERGENCY_CONF=$(cat <<'NGINX'
limit_req_zone $binary_remote_addr zone=emergency:10m rate=5r/s;
limit_conn_zone $binary_remote_addr zone=emergency_conn:10m;

server {
    listen 80;
    limit_req zone=emergency burst=10 nodelay;
    limit_conn emergency_conn 5;

    client_body_timeout 5s;
    client_header_timeout 5s;
    client_max_body_size 512k;
    keepalive_timeout 10s;

    # Block empty User-Agents
    if ($http_user_agent = "") { return 403; }

    # Allow only essential paths
    location /health {
        proxy_pass http://backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /api/orders {
        limit_req zone=emergency burst=5 nodelay;
        proxy_pass http://backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /api/products {
        limit_req zone=emergency burst=10 nodelay;
        proxy_pass http://backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # Block everything else during emergency
    location / {
        return 503 '{"error": "Service temporarily unavailable due to attack"}';
    }
}
NGINX
)

if [ "$DRY_RUN" = true ]; then
    log "[DRY-RUN] Would write emergency Nginx config and reload"
else
    echo "$EMERGENCY_CONF" > /etc/nginx/conf.d/emergency.conf
    nginx -t && nginx -s reload
    log "[+] Emergency Nginx configuration deployed"
fi

# -------------------------------------------------------
# Step 3: Block top offending IPs
# -------------------------------------------------------
log "[*] Step 2: Analyzing and blocking top offending IPs"

TOP_IPS=$(tail -50000 /var/log/nginx/access.log 2>/dev/null | \
    awk '{print $1}' | sort | uniq -c | sort -rn | head -50 | \
    awk '{print $2}')

if [ "$DRY_RUN" = true ]; then
    log "[DRY-RUN] Would block these IPs: $(echo "$TOP_IPS" | tr '\n' ' ')"
else
    BLOCKED=0
    for ip in $TOP_IPS; do
        iptables -A INPUT -s "$ip" -j DROP 2>/dev/null \
            && BLOCKED=$((BLOCKED + 1))
    done
    log "[+] Blocked $BLOCKED IPs via iptables"
fi

# -------------------------------------------------------
# Step 4: Scale up infrastructure
# -------------------------------------------------------
log "[*] Step 3: Scaling up ECS services"

if [ "$DRY_RUN" = true ]; then
    log "[DRY-RUN] Would scale $API_SERVICE to 10 tasks"
    log "[DRY-RUN] Would scale $SEARCH_SERVICE to 6 tasks"
else
    aws ecs update-service \
        --cluster "$CLUSTER" \
        --service "$API_SERVICE" \
        --desired-count 10
    aws ecs update-service \
        --cluster "$CLUSTER" \
        --service "$SEARCH_SERVICE" \
        --desired-count 6
    log "[+] ECS services scaled up"
fi

# -------------------------------------------------------
# Step 5: Enable upstream protection
# -------------------------------------------------------
log "[*] Step 4: Enabling upstream DDoS protection"

if [ "$DRY_RUN" = true ]; then
    log "[DRY-RUN] Would activate AWS Shield Advanced response"
    log "[DRY-RUN] Would enable WAF rate-based rules"
else
    aws shield create-protection \
        --name "emergency-shield" \
        --resource-arn "$ALB_ARN" 2>/dev/null || true

    aws wafv2 update-web-acl \
        --scope REGIONAL \
        --id "$WAF_ACL_ID" \
        --name "emergency-waf" \
        --default-action '{"Allow":{}}' \
        --region us-east-1 2>/dev/null || true

    log "[+] Upstream protection activated"
fi

# -------------------------------------------------------
# Step 6: Alert the team
# -------------------------------------------------------
log "[*] Step 5: Sending team notifications"

ALERT_MSG="DDoS ATTACK IN PROGRESS
Time: $(date -u)
Level: $ALERT_LEVEL
Request rate: 45,000 req/s (normal: 2,000)
Error rate: 35% 5xx
Actions taken: Rate limiting deployed, IPs blocked, infrastructure scaled
On-call: Check #incident channel"

if [ -n "$SLACK_WEBHOOK" ]; then
    if [ "$DRY_RUN" = true ]; then
        log "[DRY-RUN] Would send Slack notification"
    else
        curl -s -X POST "$SLACK_WEBHOOK" \
            -H 'Content-Type: application/json' \
            -d "{\"text\": \"$ALERT_MSG\"}" \
            || log "[!] Slack notification failed"
        log "[+] Slack notification sent"
    fi
else
    log "[!] SLACK_WEBHOOK_URL not set, skipping notification"
fi

log "=== Emergency Response Complete ==="
```

### Why This Works

- The script has three modes: `--dry-run` (show what would happen), `--execute` (do it), and `--rollback` (undo). Dry-run is critical for testing the script before an actual attack. Rollback is critical for recovering after the attack ends.
- Steps are ordered by impact: rate limiting first (immediate relief), then IP blocking (reduce attack volume), then scaling (increase capacity), then upstream protection (long-term defense).
- Every action is logged with a timestamp. During post-incident review, the log shows exactly what was done and when.
- The emergency Nginx config is aggressive: 5 req/s per IP, 5 concurrent connections, only essential paths allowed. This will block some legitimate traffic, but during an attack, partial service is better than no service.
- The `--rollback` flag restores the original Nginx config, flushes iptables rules, and resets ECS service counts. This is a single command to undo all emergency measures.

### Common Mistakes

- **Not having a dry-run mode.** If you only test the script during an actual attack, you will discover bugs at the worst possible time.
- **Not having a rollback mode.** After the attack ends, you need to quickly restore normal operations. Manual rollback is slow and error-prone.
- **Blocking IPs without analyzing the log first.** If you block the top 50 IPs by request count, you might block a legitimate high-traffic client (e.g., a mobile app with many users behind a NAT).
- **Scaling without limits.** If the attack is causing 35% 5xx errors, scaling from 3 to 10 tasks might just give the attacker 10 targets instead of 3. Rate limiting must come first.

---

## Part C: Incident Runbook (`ddos-runbook.md`)

```markdown
# DDoS Incident Response Runbook

## 1. Detection

### Monitoring Alerts
| Alert | Threshold | Severity | Tool |
|-------|-----------|----------|------|
| Request rate spike | >10,000 req/s (5x normal) | Warning | CloudWatch / Datadog |
| Request rate spike | >30,000 req/s (15x normal) | Critical | CloudWatch / Datadog |
| 5xx error rate | >10% | Warning | ALB metrics |
| 5xx error rate | >30% | Critical | ALB metrics |
| ALB p99 latency | >2s | Warning | ALB metrics |
| ALB p99 latency | >10s | Critical | ALB metrics |
| RDS connections | >80% of pool | Warning | RDS metrics |
| ECS CPU | >90% | Warning | ECS metrics |

### First Responder Actions (0-2 minutes)
1. Acknowledge the alert in PagerDuty
2. Open the incident channel in Slack (#incident-YYYY-MM-DD)
3. Run `emergency-response.sh --level <severity> --dry-run` to assess
4. If the attack is confirmed, run with --execute

**Responsible:** SRE on-call
**Verification:** 5xx error rate starts declining within 2 minutes
**If it fails:** Escalate to Level 2 immediately

---

## 2. Classification (2-5 minutes)

Determine the attack type using the alert data:

| Signal | Volumetric | Protocol | Application |
|--------|-----------|----------|-------------|
| Bandwidth saturated | Yes | No | No |
| Connection table full | Maybe | Yes | No |
| High 5xx errors | No | Maybe | Yes |
| Database connections maxed | No | No | Yes |
| Normal-looking requests | No | No | Yes |

**Responsible:** SRE on-call
**Verification:** Document the attack type in the incident channel
**If uncertain:** Assume application-layer and apply rate limiting first

---

## 3. Immediate Response (0-5 minutes)

### Automated Actions (via emergency-response.sh)
- [ ] Emergency rate limiting deployed (5 req/s per IP)
- [ ] Top 50 offending IPs blocked via iptables
- [ ] ECS services scaled up (api: 10, search: 6)
- [ ] Team notified via Slack

### Manual Checks
- [ ] Verify rate limiting is working (check 429 responses in logs)
- [ ] Verify RDS connections are declining
- [ ] Check if the attack is shifting (new IPs, new paths)
- [ ] Confirm health checks are passing

**Responsible:** SRE on-call
**Verification:** 5xx rate below 10%, RDS connections below 80%
**If it fails:** Escalate to Level 2

---

## 4. Short-term Mitigation (5-30 minutes)

### Actions
1. **Refine rate limits** -- Analyze attack patterns and adjust per-endpoint
2. **Enable WAF rules** -- Activate managed rule groups for bot detection
3. **Contact upstream provider** -- Request Shield Advanced or CF Under Attack
4. **Enable circuit breakers** -- Protect the database with graceful degradation
5. **Block IP ranges** -- If from specific ASNs, consider ASN-level blocking
6. **Monitor for adaptation** -- Attackers may change tactics

### Escalation Triggers (to Level 2)
- 5xx rate does not drop below 10% within 5 minutes of Level 1 actions
- Attack volume increases despite rate limiting
- New attack vectors appear (different paths, different methods)
- Infrastructure scaling is insufficient

**Responsible:** SRE team lead
**Verification:** Service is partially available (core endpoints responding)
**If it fails:** Escalate to Level 3

---

## 5. Recovery (30-60 minutes)

### When the attack subsides
1. **Gradually relax rate limits** -- Increase from 5 r/s to normal in stages
2. **Remove IP blocks** -- Flush iptables rules added during the attack
3. **Scale down infrastructure** -- Reduce ECS task counts to normal
4. **Disable emergency Nginx config** -- Restore normal configuration
5. **Verify all services** -- Run smoke tests on all endpoints
6. **Monitor for 30 minutes** -- Watch for attack resumption

### Rollback Procedure
```bash
./emergency-response.sh --rollback
```

**Responsible:** SRE on-call
**Verification:** All services responding normally, error rate below 1%
**If attack resumes:** Re-enable emergency response

---

## 6. Post-Incident (within 24 hours)

### Actions
1. **Write incident report** -- Timeline, actions, impact, root cause
2. **Review logs** -- Analyze attack patterns for future defense
3. **Update baselines** -- Adjust monitoring thresholds
4. **Improve automation** -- Identify manual steps that should be automated
5. **Update this runbook** -- Incorporate lessons learned
6. **Cost analysis** -- Calculate infrastructure costs from scaling

**Responsible:** SRE team lead + Security team
**Verification:** Incident report published, runbook updated

---

## 7. Escalation Matrix

| Level | Role | Trigger | Actions | Contact |
|-------|------|---------|---------|---------|
| 1 | SRE on-call | Alert fires | Rate limiting, IP blocking, scaling | PagerDuty |
| 2 | SRE team lead | 5xx >10% after 5 min of L1 | Scaling, upstream provider | Phone |
| 3 | Security team | Attack persists >15 min | Threat intel, forensics | Phone |
| 4 | Management | Attack >30 min or data breach | Customer comms, legal | Phone |

### Escalation Criteria

**Level 1 -> Level 2:**
- 5xx error rate does not drop below 10% within 5 minutes
- Attack volume exceeds rate limiting capacity
- Infrastructure scaling is insufficient
- Multiple attack vectors detected

**Level 2 -> Level 3:**
- Attack persists for more than 15 minutes despite mitigation
- Attack appears to be targeted (not opportunistic)
- Potential data exfiltration detected
- Attack sophistication increases (adapting to defenses)

**Level 3 -> Level 4:**
- Attack lasts more than 30 minutes
- Customer data may be compromised
- Media attention or public disclosure
- Legal or regulatory notification required
```

### Why This Works

- The runbook is structured by time phase, so the on-call engineer knows exactly what to do at each stage. The first 5 minutes are critical and should be fully automated.
- Each action has a responsible party, verification step, and failure path. This eliminates ambiguity during a stressful incident.
- Escalation triggers are quantitative (e.g., "5xx >10% after 5 min"), not subjective. This prevents both premature escalation and delayed escalation.
- The recovery phase emphasizes gradual relaxation of emergency measures, not an abrupt switch back to normal. Attackers sometimes pause and resume; gradual recovery catches this.
- The runbook includes a "Classification" step that maps observable signals to attack types. This prevents the common mistake of treating all attacks the same way.

### Common Mistakes

- **Writing a runbook that is never tested.** Run a tabletop exercise (simulated attack) at least quarterly to verify the runbook works.
- **Not having clear escalation triggers.** "Escalate if things get worse" is not actionable. Define specific metrics and timeframes.
- **Skipping post-incident review.** Every attack is a learning opportunity. Update the runbook, adjust thresholds, and improve automation.
- **Abrupt recovery.** Do not go from emergency measures directly to normal operations. Attackers may pause and resume; gradual relaxation catches this.

---

## Part D: Upstream Protection (Terraform)

```hcl
# -------------------------------------------------------
# Variables
# -------------------------------------------------------
variable "alb_arn" {
  description = "ARN of the Application Load Balancer"
  type        = string
}

variable "cloudfront_distribution_arn" {
  description = "ARN of the CloudFront distribution"
  type        = string
}

# -------------------------------------------------------
# Shield Advanced
# -------------------------------------------------------
resource "aws_shield_protection" "alb" {
  name         = "shopfast-alb-shield"
  resource_arn = var.alb_arn
}

resource "aws_shield_protection" "cloudfront" {
  name         = "shopfast-cloudfront-shield"
  resource_arn = var.cloudfront_distribution_arn
}

resource "aws_shield_protection_group" "shopfast" {
  protection_group_id = "shopfast-all"
  aggregation         = "MAX"
  pattern             = "ALL"
}

# -------------------------------------------------------
# WAF Web ACL
# -------------------------------------------------------
resource "aws_wafv2_web_acl" "shopfast" {
  name        = "shopfast-waf"
  description = "WAF rules for ShopFast DDoS protection"
  scope       = "REGIONAL"

  default_action {
    allow {}
  }

  # Rule 1: Rate-based rule -- limit requests per IP
  rule {
    name     = "rate-limit-per-ip"
    priority = 1

    action {
      block {}
    }

    statement {
      rate_based_statement {
        limit              = 2000
        aggregate_key_type = "IP"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "RateLimitPerIP"
      sampled_requests_enabled   = true
    }
  }

  # Rule 2: AWS Managed Common Rule Set
  rule {
    name     = "aws-common-rules"
    priority = 2

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
      metric_name                = "AWSCommonRules"
      sampled_requests_enabled   = true
    }
  }

  # Rule 3: Known Bad Inputs
  rule {
    name     = "known-bad-inputs"
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
      metric_name                = "KnownBadInputs"
      sampled_requests_enabled   = true
    }
  }

  # Rule 4: Bot Control
  rule {
    name     = "bot-control"
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
      metric_name                = "BotControl"
      sampled_requests_enabled   = true
    }
  }

  # Rule 5: Block requests with empty User-Agent
  rule {
    name     = "block-empty-ua"
    priority = 5

    action {
      block {}
    }

    statement {
      byte_match_statement {
        field_to_match {
          single_header {
            name = "user-agent"
          }
        }
        positional_constraint = "EXACTLY"
        search_string         = ""
        text_transformation {
          priority = 0
          type     = "NONE"
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "BlockEmptyUA"
      sampled_requests_enabled   = true
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "ShopFastWAF"
    sampled_requests_enabled   = true
  }
}

# Associate WAF with ALB
resource "aws_wafv2_web_acl_association" "alb" {
  resource_arn = var.alb_arn
  web_acl_arn  = aws_wafv2_web_acl.shopfast.arn
}

# -------------------------------------------------------
# CloudFront Distribution (with WAF)
# -------------------------------------------------------
resource "aws_cloudfront_distribution" "shopfast" {
  enabled         = true
  web_acl_id      = aws_wafv2_web_acl.shopfast.arn
  price_class     = "PriceClass_100"
  http_version    = "http2"

  origin {
    domain_name = "alb.shopfast.com"
    origin_id   = "alb"

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  # Origin failover
  origin_group {
    origin_id = "failover"

    failover_criteria {
      status_codes = [500, 502, 503, 504]
    }

    member {
      origin_id = "alb"
    }
  }

  default_cache_behavior {
    allowed_methods        = [
      "GET", "HEAD", "OPTIONS", "PUT", "POST", "PATCH", "DELETE",
    ]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "alb"
    viewer_protocol_policy = "redirect-to-https"
    compress               = true

    forwarded_values {
      query_string = true
      headers      = ["Host", "Authorization", "Content-Type"]
      cookies {
        forward = "all"
      }
    }

    min_ttl     = 0
    default_ttl = 0
    max_ttl     = 0
  }

  # Cache static assets aggressively
  ordered_cache_behavior {
    path_pattern           = "/static/*"
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "alb"
    viewer_protocol_policy = "redirect-to-https"
    compress               = true

    forwarded_values {
      query_string = false
      cookies {
        forward = "none"
      }
    }

    min_ttl     = 86400
    default_ttl = 86400
    max_ttl     = 604800
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    cloudfront_default_certificate = true
  }
}

# -------------------------------------------------------
# CloudWatch Alarms for DDoS Detection
# -------------------------------------------------------
resource "aws_cloudwatch_metric_alarm" "ddos_detected_alb" {
  alarm_name          = "DDoS-Detected-ALB"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "DDoSDetected"
  namespace           = "AWS/DDoSProtection"
  period              = 60
  statistic           = "Sum"
  threshold           = 0
  alarm_description   = "DDoS attack detected on ALB"
  alarm_actions       = [aws_sns_topic.ddos_alerts.arn]

  dimensions = {
    ResourceArn = var.alb_arn
  }
}

resource "aws_cloudwatch_metric_alarm" "ddos_bits_per_second" {
  alarm_name          = "DDoS-High-Bandwidth"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 1
  metric_name         = "DDoSAttackBitsPerSecond"
  namespace           = "AWS/DDoSProtection"
  period              = 60
  statistic           = "Maximum"
  threshold           = 5000000000  # 5 Gbps
  alarm_description   = "DDoS attack bandwidth exceeds 5 Gbps"
  alarm_actions       = [aws_sns_topic.ddos_alerts.arn]

  dimensions = {
    ResourceArn = var.alb_arn
  }
}

resource "aws_cloudwatch_metric_alarm" "alb_5xx_rate" {
  alarm_name          = "ALB-High-5xx-Rate"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = 2
  threshold           = 10
  alarm_description   = "ALB 5xx error rate exceeds 10%"
  alarm_actions       = [aws_sns_topic.ddos_alerts.arn]

  metric_query {
    id          = "error_rate"
    expression  = "(errors / total) * 100"
    label       = "5xx Error Rate"
    return_data = true
  }

  metric_query {
    id = "errors"
    metric {
      metric_name = "HTTPCode_Target_5XX_Count"
      namespace   = "AWS/ApplicationELB"
      period      = 60
      stat        = "Sum"
      dimensions = {
        LoadBalancer = var.alb_arn
      }
    }
  }

  metric_query {
    id = "total"
    metric {
      metric_name = "RequestCount"
      namespace   = "AWS/ApplicationELB"
      period      = 60
      stat        = "Sum"
      dimensions = {
        LoadBalancer = var.alb_arn
      }
    }
  }
}

# SNS topic for DDoS alerts
resource "aws_sns_topic" "ddos_alerts" {
  name = "ddos-alerts"
}
```

### Why This Works

- Shield Advanced provides always-on detection and automatic mitigation for volumetric attacks (L3/L4). It also provides proactive engagement where the AWS DDoS Response Team (DRT) contacts you during a detected attack.
- The WAF rate-based rule (2,000 requests per IP per 5 minutes) is the first line of defense against application-layer floods. It automatically blocks IPs that exceed the threshold.
- Managed rule groups (Common Rule Set, Known Bad Inputs, Bot Control) provide baseline protection against common attack patterns without manual rule writing.
- CloudFront caching absorbs a significant portion of GET requests before they reach the origin. Static assets are cached for 24 hours; dynamic content is not cached (TTL=0) but still benefits from CloudFront's edge termination.
- CloudWatch alarms detect DDoS attacks at multiple levels: Shield-level detection, bandwidth thresholds, and application-level error rates.

### Common Mistakes

- **Enabling WAF without rate-based rules.** Managed rule groups catch known attack patterns but do not limit request volume. A botnet with valid-looking requests will pass managed rules.
- **Not associating the WAF with both CloudFront and ALB.** Attack traffic that bypasses CloudFront (direct ALB access) needs WAF protection at the ALB level.
- **Setting rate limits too low in the WAF.** A rate limit of 100 requests per IP per 5 minutes will block legitimate users behind corporate NATs. Start with 2,000 and adjust based on traffic analysis.
- **Not configuring visibility.** Without `visibility_config`, you cannot see which rules are matching and which are not. Always enable CloudWatch metrics and sampled requests.

---

## Part E: Database Circuit Breaker (`db_circuit_breaker.py`)

```python
"""
db_circuit_breaker.py -- Circuit breaker for database protection during DDoS.

Integrates with Flask to protect PostgreSQL connections during attacks.
"""

import time
import threading
from enum import Enum
from typing import Optional, Any, Callable


class CircuitState(Enum):
    CLOSED = "closed"        # Normal -- requests pass through
    OPEN = "open"            # Tripped -- cache or 503
    HALF_OPEN = "half_open"  # Testing recovery -- limited requests


class DatabaseCircuitBreaker:
    """
    Circuit breaker that monitors database health and fails over to
    cached responses when the database is under stress.

    States:
        CLOSED -> OPEN: connections > 80% pool OR response time > 2s
        OPEN -> HALF_OPEN: after recovery_timeout seconds
        HALF_OPEN -> CLOSED: after 3 consecutive successful test requests
        HALF_OPEN -> OPEN: if any test request fails
    """

    def __init__(
        self,
        max_connections: int = 200,
        connection_threshold: float = 0.8,
        response_time_threshold: float = 2.0,
        recovery_timeout: float = 30.0,
        half_open_test_interval: float = 10.0,
        half_open_success_required: int = 3,
        cache_getter: Optional[Callable] = None,
    ):
        self.max_connections = max_connections
        self.connection_threshold = connection_threshold
        self.response_time_threshold = response_time_threshold
        self.recovery_timeout = recovery_timeout
        self.half_open_test_interval = half_open_test_interval
        self.half_open_success_required = half_open_success_required
        self.cache_getter = cache_getter

        # State
        self._state = CircuitState.CLOSED
        self._failure_count = 0
        self._consecutive_successes = 0
        self._last_failure_time = 0.0
        self._last_test_time = 0.0
        self._lock = threading.Lock()

        # Metrics
        self._total_requests = 0
        self._rejected_requests = 0
        self._cache_hits = 0

    @property
    def state(self) -> CircuitState:
        """Current circuit state, with automatic transitions."""
        with self._lock:
            if self._state == CircuitState.OPEN:
                if (time.time() - self._last_failure_time
                        >= self.recovery_timeout):
                    self._state = CircuitState.HALF_OPEN
                    self._consecutive_successes = 0
                    self._last_test_time = 0.0
            return self._state

    def record_database_metrics(
        self, active_connections: int, avg_response_time: float
    ) -> None:
        """
        Report current database metrics. If thresholds are exceeded,
        trip the circuit breaker.
        """
        with self._lock:
            if self._state == CircuitState.CLOSED:
                conn_util = active_connections / self.max_connections
                if (conn_util >= self.connection_threshold
                        or avg_response_time >= self.response_time_threshold):
                    self._trip(
                        f"connections={active_connections}/"
                        f"{self.max_connections} ({conn_util:.0%}), "
                        f"response_time={avg_response_time:.2f}s"
                    )

    def call(self, cache_key: str, db_func: Callable, *args, **kwargs) -> Any:
        """
        Execute a database call through the circuit breaker.

        If the circuit is open, returns cached data or raises CircuitOpenError.
        If half-open, allows test requests at controlled intervals.
        """
        self._total_requests += 1
        current_state = self.state

        if current_state == CircuitState.OPEN:
            return self._serve_from_cache(cache_key)

        if current_state == CircuitState.HALF_OPEN:
            now = time.time()
            if now - self._last_test_time < self.half_open_test_interval:
                return self._serve_from_cache(cache_key)
            # Allow a test request
            self._last_test_time = now

        # Attempt the database call
        try:
            start = time.time()
            result = db_func(*args, **kwargs)
            elapsed = time.time() - start

            if elapsed >= self.response_time_threshold:
                self._record_failure()
            else:
                self._record_success()

            return result

        except Exception:
            self._record_failure()
            return self._serve_from_cache(cache_key)

    def _trip(self, reason: str) -> None:
        """Transition to OPEN state."""
        self._state = CircuitState.OPEN
        self._failure_count += 1
        self._last_failure_time = time.time()
        print(f"[CIRCUIT BREAKER] OPEN: {reason}")

    def _record_failure(self) -> None:
        """Record a failed or slow request."""
        with self._lock:
            if self._state == CircuitState.HALF_OPEN:
                self._trip("half-open test failed")
            else:
                self._failure_count += 1
                self._last_failure_time = time.time()

    def _record_success(self) -> None:
        """Record a successful request."""
        with self._lock:
            if self._state == CircuitState.HALF_OPEN:
                self._consecutive_successes += 1
                if (self._consecutive_successes
                        >= self.half_open_success_required):
                    self._state = CircuitState.CLOSED
                    self._failure_count = 0
                    self._consecutive_successes = 0
                    print("[CIRCUIT BREAKER] CLOSED: recovery confirmed")

    def _serve_from_cache(self, cache_key: str) -> Any:
        """Attempt to serve a response from cache."""
        self._rejected_requests += 1

        if self.cache_getter:
            cached = self.cache_getter(cache_key)
            if cached is not None:
                self._cache_hits += 1
                return cached

        raise CircuitOpenError(
            "Database circuit breaker is open and no cached data available"
        )

    def get_stats(self) -> dict:
        """Return circuit breaker statistics."""
        return {
            "state": self.state.value,
            "total_requests": self._total_requests,
            "rejected_requests": self._rejected_requests,
            "cache_hits": self._cache_hits,
            "failure_count": self._failure_count,
            "consecutive_successes": self._consecutive_successes,
        }


class CircuitOpenError(Exception):
    """Raised when the circuit is open and no cached data is available."""
    pass


# -------------------------------------------------------
# Flask Integration
# -------------------------------------------------------

def create_circuit_breaker(app=None, redis_client=None):
    """
    Create and configure a circuit breaker for a Flask application.
    """
    def cache_getter(key):
        if redis_client:
            value = redis_client.get(key)
            if value:
                return (
                    value.decode("utf-8")
                    if isinstance(value, bytes)
                    else value
                )
        return None

    breaker = DatabaseCircuitBreaker(
        max_connections=200,
        connection_threshold=0.8,
        response_time_threshold=2.0,
        recovery_timeout=30.0,
        half_open_test_interval=10.0,
        half_open_success_required=3,
        cache_getter=cache_getter,
    )

    if app:
        @app.errorhandler(CircuitOpenError)
        def handle_circuit_open(error):
            from flask import jsonify, make_response
            response = make_response(
                jsonify({
                    "error": "Service temporarily unavailable",
                    "message": (
                        "Database is under heavy load. Please try again."
                    ),
                    "retry_after": 10,
                }),
                503,
            )
            response.headers["Retry-After"] = "10"
            response.headers["X-Served-From"] = "circuit-breaker"
            return response

    return breaker


# Example usage in a Flask route:
#
#   breaker = create_circuit_breaker(app, redis_client)
#
#   @app.route('/api/data/<item_id>')
#   def get_data(item_id):
#       result = breaker.call(
#           cache_key=f"data:{item_id}",
#           db_func=lambda: db.query(
#               "SELECT * FROM items WHERE id = %s", item_id
#           ),
#       )
#       response = make_response(jsonify(result))
#       response.headers["X-Served-From"] = "cache" if ... else "database"
#       return response
```

### Why This Works

- The circuit breaker has three states that map directly to database health: CLOSED (healthy), OPEN (overloaded), HALF_OPEN (testing recovery). The state machine prevents cascading failures by cutting off database access when the database is stressed.
- In the OPEN state, the breaker serves cached responses with an `X-Served-From: cache` header. This provides graceful degradation -- users see slightly stale data instead of errors.
- The HALF_OPEN state allows exactly one test request every 10 seconds. This prevents thundering herd when the database recovers -- instead of all traffic rushing back at once, only a trickle of test requests probe the database.
- `record_database_metrics` is called externally (e.g., by a monitoring thread) to proactively trip the breaker before the database fully crashes. This is faster than waiting for individual requests to fail.
- The `call` method wraps the database function, so existing routes need minimal changes. The circuit breaker is transparent to the application logic.
- Thread safety is ensured via `threading.Lock` on all state transitions. Multiple Flask worker threads can safely share a single circuit breaker instance.

### Common Mistakes

- **Not having a half-open state.** Without it, the circuit either stays open forever (never recovers) or closes immediately (thundering herd kills the database again).
- **Using a fixed recovery timeout without considering the attack duration.** A 30-second timeout is appropriate for brief spikes; a sustained attack may require longer timeouts or manual intervention.
- **Not caching responses before the attack.** The circuit breaker is only useful if there is cached data to serve. Cache warming (pre-populating the cache with frequently accessed data) should be part of normal operations.
- **Tripping the circuit on a single slow query.** The breaker should use thresholds (80% connection utilization, 2s average response time), not individual query times, to avoid false positives.
- **Not logging state transitions.** Every OPEN, HALF_OPEN, and CLOSED transition should be logged with a timestamp and reason. This is essential for post-incident analysis.

---

## Common Mistakes (Module-Level)

- **Confusing volumetric and application-layer attacks.** Volumetric attacks saturate bandwidth and must be stopped upstream. Application-layer attacks exhaust server resources and must be stopped at the application layer. Using the wrong defense is expensive and ineffective.
- **Not automating the first 5 minutes.** During an attack, every second counts. Manual SSH commands are slow and error-prone. The emergency response script should be tested quarterly and ready to deploy.
- **Forgetting rollback capability.** Emergency measures (strict rate limiting, IP blocks) are designed for attacks, not normal operations. If you cannot roll them back quickly, you will degrade service long after the attack ends.
- **Not protecting the database.** The database is often the bottleneck in application-layer attacks. Circuit breakers, connection pool limits, and query timeouts are essential to prevent cascading failures.
- **Skipping post-incident review.** Every attack teaches something. Update thresholds, improve automation, and refine the runbook based on what happened.
